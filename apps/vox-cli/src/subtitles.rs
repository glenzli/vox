//! Audio-pinned subtitle review and asynchronous alignment adapter.
use super::*;

pub(super) fn execute(v: &Value) -> Result<Value> {
    let p = project(v)?;
    match field(v, "action")? {
        "accept_subtitles" => Ok(snapshot(
            &p,
            p.accept_subtitles(version(v)?, field(v, "id")?)?,
        )),
        "discard_subtitles" => Ok(snapshot(
            &p,
            p.discard_subtitles(version(v)?, field(v, "id")?)?,
        )),
        "subtitle_preview" => {
            let audio_id = field(v, "audio_id")?;
            let m = p.read()?;
            let audio = m
                .audio
                .iter()
                .chain(&m.candidates)
                .find(|audio| audio.id == audio_id)
                .ok_or_else(|| Error::new("audio_not_found"))?;
            let source = m
                .sources
                .iter()
                .find(|source| source.id == audio.source_revision)
                .ok_or_else(|| Error::new("source_not_found"))?;
            let language = source.document.language.clone();
            let accepted = m.audio.iter().any(|audio| audio.id == audio_id);
            let candidates = m
                .subtitle_candidates
                .iter()
                .filter(|candidate| candidate.audio_revision == audio_id)
                .collect::<Vec<_>>();
            let runs = m
                .subtitle_runs
                .iter()
                .filter(|run| run.audio_revision == audio_id)
                .collect::<Vec<_>>();
            let track = if let Some(id) = v["candidate_id"].as_str() {
                let candidate = p.subtitle_candidate_track(id)?;
                if candidate.audio_revision != audio_id {
                    return Err(Error::new("subtitle_audio_mismatch"));
                }
                Ok(candidate)
            } else {
                p.subtitle_track(audio_id)
            };
            let (track, error) = match track {
                Ok(track) => (json!(track), Value::Null),
                Err(error) => (Value::Null, json!(error.code)),
            };
            let mut result = snapshot(&p, m.clone());
            result["audio_id"] = json!(audio_id);
            result["track"] = track;
            result["track_error"] = error;
            match p.subtitle_transcript(audio_id) {
                Ok(transcript) => {
                    result["transcript"] = json!(transcript);
                    result["transcript_error"] = Value::Null;
                }
                Err(error) => {
                    // An unavailable alignment suggestion must not hide an
                    // existing valid measured or accepted subtitle track.
                    result["transcript"] = json!("");
                    result["transcript_error"] = json!(error.code);
                }
            }
            result["language"] = json!(language);
            result["accepted_audio"] = json!(accepted);
            result["candidates"] = json!(candidates);
            result["runs"] = json!(runs);
            result["candidate_id"] = v["candidate_id"].clone();
            Ok(result)
        }
        _ => Err(Error::new("unknown_action")),
    }
}

pub(super) fn align(v: &Value, control: SpeechSynthesisControl) -> Result<Value> {
    let p = project(v)?;
    let spec = serde_json::from_value(v["spec"].clone())?;
    let outcome = p.align_subtitles(
        version(v)?,
        field(v, "audio_id")?,
        v["endpoint"].as_str().unwrap_or(""),
        spec,
        &control,
    )?;
    let summary = json!({"run_id":outcome.run_id,"candidate_id":outcome.candidate_id,
        "state":outcome.state,"error":outcome.error});
    let mut result = snapshot(&p, outcome.manifest);
    result["subtitle_outcome"] = summary;
    Ok(result)
}
