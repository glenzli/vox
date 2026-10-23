//! Audio-pinned subtitle candidates, explicit acceptance and shared export.
use super::{AudioRevision, Error, Manifest, Project, Result, ScriptFormat};
use crate::{
    ContentDigest, ContentRef, SpeechSynthesisControl,
    assembly::ScriptActionKind,
    audio::parse_pcm_s16le_wav,
    infer_runtime::captions::{
        AlignmentReceipt, AlignmentSpec, MAX_ALIGNMENT_RESPONSE_BYTES,
        MAX_ALIGNMENT_TRANSCRIPT_BYTES, aligned_tokens, audio_duration,
    },
};
use infer_runtime_client::AlignmentResponse;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs::OpenOptions, io::Write, path::Path};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubtitleCue {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubtitleSource {
    MeasuredSegments,
    ForcedAlignment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleTrack {
    pub audio_revision: String,
    pub audio_digest: ContentDigest,
    pub source_revision: String,
    pub source: SubtitleSource,
    pub cues: Vec<SubtitleCue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alignment_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

impl SubtitleTrack {
    /// Both chapter and whole-take export use the same millisecond serializer.
    pub fn serialize(&self, format: &str) -> Result<String> {
        if !matches!(format, "srt" | "vtt") {
            return Err(Error::new("invalid_subtitle_format"));
        }
        validate_cues(&self.cues, None)?;
        let mut output = if format == "vtt" {
            "WEBVTT\n\n".into()
        } else {
            String::new()
        };
        for (index, cue) in self.cues.iter().enumerate() {
            let text = cue
                .text
                .lines()
                .filter(|line| !line.trim().is_empty())
                .collect::<Vec<_>>()
                .join("\n");
            // Preserve literal text instead of interpreting authored WebVTT markup.
            let text = if format == "vtt" {
                text.replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
            } else {
                text
            };
            output.push_str(&format!(
                "{}\n{} --> {}\n{}\n\n",
                index + 1,
                timestamp(cue.start_ms, format),
                timestamp(cue.end_ms, format),
                text,
            ));
        }
        Ok(output)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubtitleRevision {
    pub id: String,
    pub run_id: String,
    pub audio_revision: String,
    pub audio_digest: ContentDigest,
    pub source_revision: String,
    pub document_digest: String,
    pub transcript: String,
    pub language: String,
    pub cues: Vec<SubtitleCue>,
    pub alignment: ContentRef,
    pub generation: AlignmentReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubtitleRun {
    pub id: String,
    pub audio_revision: String,
    pub audio_digest: ContentDigest,
    pub source_revision: String,
    pub document_digest: String,
    pub spec: AlignmentSpec,
    pub status: String,
    /// Retained after explicit discard as a historical result identity.
    pub candidate_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubtitleState {
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Serialize)]
pub struct SubtitleOutcome {
    pub manifest: Manifest,
    pub run_id: String,
    pub candidate_id: Option<String>,
    pub state: SubtitleState,
    pub error: Option<String>,
}

pub(super) fn recover_records(manifest: &mut Manifest) -> bool {
    let mut changed = false;
    for run in &mut manifest.subtitle_runs {
        if run.status == "running" {
            run.status = "interrupted_unknown".into();
            changed = true;
        }
    }
    changed
}

impl Project {
    pub fn align_subtitles(
        &self,
        expected: u64,
        audio_id: &str,
        endpoint: &str,
        spec: AlignmentSpec,
        control: &SpeechSynthesisControl,
    ) -> Result<SubtitleOutcome> {
        self.align_subtitles_using(expected, audio_id, spec, control, |audio, spec, control| {
            crate::infer_runtime::captions::generate(endpoint, audio, spec, control)
        })
    }

    fn align_subtitles_using(
        &self,
        expected: u64,
        audio_id: &str,
        spec: AlignmentSpec,
        control: &SpeechSynthesisControl,
        generate: impl FnOnce(
            &[u8],
            AlignmentSpec,
            &SpeechSynthesisControl,
        ) -> Result<(Vec<u8>, AlignmentReceipt)>,
    ) -> Result<SubtitleOutcome> {
        spec.validate()?;
        let producer = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.path.join("render.lock"))?;
        producer.try_lock().map_err(|_| Error::new("render_busy"))?;
        let snapshot = self.read()?;
        if snapshot.version != expected {
            return Err(Error::new("stale_project"));
        }
        let audio = accepted_audio(&snapshot, audio_id)?.clone();
        let bytes = self.checked_audio(&audio)?;
        audio_duration(&bytes)?;
        let run_id = uuid::Uuid::now_v7().to_string();
        let (manifest, _) = self.update(expected, |manifest| {
            if manifest.subtitle_runs.len() >= 256
                || manifest.subtitle_candidates.len() + manifest.subtitles.len() >= 128
            {
                return Err(Error::new("project_limit"));
            }
            recover_records(manifest);
            manifest.subtitle_runs.push(SubtitleRun {
                id: run_id.clone(),
                audio_revision: audio.id.clone(),
                audio_digest: audio.content.digest,
                source_revision: audio.source_revision.clone(),
                document_digest: audio.document_digest.clone(),
                spec: spec.clone(),
                status: "running".into(),
                candidate_id: None,
                error: None,
            });
            validate_records(manifest)
        })?;
        let mut outcome = SubtitleOutcome {
            manifest,
            run_id: run_id.clone(),
            candidate_id: None,
            state: SubtitleState::Completed,
            error: None,
        };
        if control.is_cancelled() {
            outcome.state = SubtitleState::Cancelled;
            outcome.manifest = self.finish_subtitle_run(&run_id, None, "cancelled", None)?;
            return Ok(outcome);
        }
        let result =
            generate(&bytes, spec.clone(), control).and_then(|(response_bytes, generation)| {
                if generation.spec != spec || generation.audio_digest != audio.content.digest {
                    return Err(Error::new("invalid_provenance"));
                }
                let response = generation.validate(&response_bytes, &bytes)?;
                let cues = cues_for_alignment(&response, &spec, audio_duration(&bytes)?)?;
                let revision = SubtitleRevision {
                    id: uuid::Uuid::now_v7().to_string(),
                    run_id: run_id.clone(),
                    audio_revision: audio.id.clone(),
                    audio_digest: audio.content.digest,
                    source_revision: audio.source_revision.clone(),
                    document_digest: audio.document_digest.clone(),
                    transcript: spec.transcript,
                    language: spec.language,
                    cues,
                    alignment: self.put_object(&response_bytes, "application/json")?,
                    generation,
                };
                let candidate_id = revision.id.clone();
                let status = if control.is_cancelled() {
                    "cancelled"
                } else {
                    "completed"
                };
                let manifest = self.finish_subtitle_run(&run_id, Some(revision), status, None)?;
                Ok((manifest, candidate_id))
            });
        match result {
            Ok((manifest, candidate_id)) => {
                outcome.manifest = manifest;
                outcome.candidate_id = Some(candidate_id);
                if control.is_cancelled() {
                    outcome.state = SubtitleState::Cancelled;
                }
            }
            Err(error) => {
                outcome.state = if control.is_cancelled() {
                    SubtitleState::Cancelled
                } else {
                    SubtitleState::Failed
                };
                let status = if outcome.state == SubtitleState::Cancelled {
                    "cancelled"
                } else {
                    "failed"
                };
                outcome.manifest =
                    self.finish_subtitle_run(&run_id, None, status, Some(error.code.clone()))?;
                outcome.error = Some(error.code);
            }
        }
        Ok(outcome)
    }

    fn finish_subtitle_run(
        &self,
        run_id: &str,
        revision: Option<SubtitleRevision>,
        status: &str,
        error: Option<String>,
    ) -> Result<Manifest> {
        let _lock = self.lock()?;
        let mut manifest = self.read()?;
        let index = manifest
            .subtitle_runs
            .iter()
            .position(|run| run.id == run_id && run.status == "running")
            .ok_or_else(|| Error::new("invalid_subtitle_run"))?;
        // The exact accepted audio/source pins, rather than mutable draft/head,
        // fence completion. Concurrent local edits remain independently saved.
        let run = &manifest.subtitle_runs[index];
        let audio = accepted_audio(&manifest, &run.audio_revision)?;
        if audio.content.digest != run.audio_digest
            || audio.source_revision != run.source_revision
            || audio.document_digest != run.document_digest
        {
            return Err(Error::new("stale_audio"));
        }
        if revision.is_some() {
            self.checked_audio(audio)?;
        }
        if let Some(revision) = revision {
            if manifest.subtitle_candidates.len() + manifest.subtitles.len() >= 128 {
                return Err(Error::new("project_limit"));
            }
            manifest.subtitle_runs[index].candidate_id = Some(revision.id.clone());
            manifest.subtitle_candidates.push(revision);
        }
        manifest.subtitle_runs[index].status = status.into();
        manifest.subtitle_runs[index].error = error;
        validate_records(&manifest)?;
        manifest.version = manifest
            .version
            .checked_add(1)
            .ok_or_else(|| Error::new("project_limit"))?;
        self.write(&manifest)?;
        Ok(manifest)
    }

    pub fn accept_subtitles(&self, expected: u64, id: &str) -> Result<Manifest> {
        self.update(expected, |manifest| {
            let revision = manifest
                .subtitle_candidates
                .iter()
                .chain(&manifest.subtitles)
                .find(|revision| revision.id == id)
                .cloned()
                .ok_or_else(|| Error::new("subtitle_missing"))?;
            self.checked_subtitle(manifest, &revision)?;
            if let Some(index) = manifest
                .subtitle_candidates
                .iter()
                .position(|revision| revision.id == id)
            {
                manifest
                    .subtitles
                    .push(manifest.subtitle_candidates.remove(index));
            }
            manifest
                .subtitle_selections
                .insert(revision.audio_revision.clone(), revision.id);
            validate_records(manifest)
        })
        .map(|result| result.0)
    }

    pub fn discard_subtitles(&self, expected: u64, id: &str) -> Result<Manifest> {
        self.update(expected, |manifest| {
            let index = manifest
                .subtitle_candidates
                .iter()
                .position(|revision| revision.id == id)
                .ok_or_else(|| Error::new("subtitle_missing"))?;
            manifest.subtitle_candidates.remove(index);
            validate_records(manifest)
        })
        .map(|result| result.0)
    }

    /// Explicitly restore measured assembly captions for the exact audio.
    pub fn use_measured_subtitles(&self, expected: u64, audio_id: &str) -> Result<Manifest> {
        self.update(expected, |manifest| {
            let audio = audio_record(manifest, audio_id)?;
            self.measured_subtitle_track(manifest, audio)?;
            manifest.subtitle_selections.remove(audio_id);
            Ok(())
        })
        .map(|result| result.0)
    }

    pub fn subtitle_track(&self, audio_id: &str) -> Result<SubtitleTrack> {
        let manifest = self.read()?;
        let audio = audio_record(&manifest, audio_id)?;
        if let Some(id) = manifest.subtitle_selections.get(audio_id) {
            let revision = manifest
                .subtitles
                .iter()
                .find(|revision| revision.id == *id)
                .ok_or_else(|| Error::new("invalid_subtitles"))?;
            return self.checked_subtitle(&manifest, revision);
        }
        self.measured_subtitle_track(&manifest, audio)
    }

    pub fn subtitle_candidate_track(&self, id: &str) -> Result<SubtitleTrack> {
        let manifest = self.read()?;
        let revision = manifest
            .subtitle_candidates
            .iter()
            .chain(&manifest.subtitles)
            .find(|revision| revision.id == id)
            .ok_or_else(|| Error::new("subtitle_missing"))?;
        self.checked_subtitle(&manifest, revision)
    }

    /// Editable suggestion from immutable speech content, including scoped
    /// pronunciations and replay. Echo edits still require user confirmation.
    pub fn subtitle_transcript(&self, audio_id: &str) -> Result<String> {
        let manifest = self.read()?;
        let audio = audio_record(&manifest, audio_id)?;
        if let Some(id) = manifest.subtitle_selections.get(audio_id) {
            let revision = manifest
                .subtitles
                .iter()
                .find(|revision| revision.id == *id)
                .ok_or_else(|| Error::new("invalid_subtitles"))?;
            self.checked_subtitle(&manifest, revision)?;
            return Ok(revision.transcript.clone());
        }
        let source = manifest
            .sources
            .iter()
            .find(|source| source.id == audio.source_revision)
            .ok_or_else(|| Error::new("source_missing"))?;
        if source.document.digest() != audio.document_digest {
            return Err(Error::new("stale_source"));
        }
        let compiled = self.compile_segments(&source.document)?;
        let mut pieces: Vec<Option<String>> = Vec::with_capacity(compiled.actions.len());
        for action in compiled.actions {
            let piece = match action.kind {
                ScriptActionKind::Speech { text, operation } => Some(
                    crate::pronunciation::apply(&text, &operation.pronunciation_rules)
                        .ok_or_else(|| Error::new("invalid_pronunciation"))?,
                ),
                ScriptActionKind::Replay { piece } => pieces
                    .get(piece as usize)
                    .cloned()
                    .ok_or_else(|| Error::new("invalid_speech_script"))?,
                ScriptActionKind::Local { .. } => None,
            };
            pieces.push(piece);
        }
        let separator = if source.document.format == ScriptFormat::Plain {
            ""
        } else {
            "\n"
        };
        let transcript = pieces
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(separator);
        if transcript.len() > MAX_ALIGNMENT_TRANSCRIPT_BYTES {
            return Err(Error::new("invalid_alignment_spec"));
        }
        Ok(transcript)
    }

    pub fn export_subtitles(&self, audio_id: &str, destination: &Path, format: &str) -> Result<()> {
        let text = self.subtitle_track(audio_id)?.serialize(format)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)
            .map_err(|_| Error::new("destination_exists"))?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }

    fn checked_audio(&self, audio: &AudioRevision) -> Result<Vec<u8>> {
        let bytes = self.object(&audio.content)?;
        if parse_pcm_s16le_wav(&bytes, audio.contract.origin)? != audio.contract {
            return Err(Error::new("corrupt_audio"));
        }
        Ok(bytes)
    }

    fn checked_subtitle(
        &self,
        manifest: &Manifest,
        revision: &SubtitleRevision,
    ) -> Result<SubtitleTrack> {
        let audio = accepted_audio(manifest, &revision.audio_revision)?;
        let bytes = self.checked_audio(audio)?;
        let response = self.object(&revision.alignment)?;
        let parsed = revision.generation.validate(&response, &bytes)?;
        if cues_for_alignment(&parsed, &revision.generation.spec, audio_duration(&bytes)?)?
            != revision.cues
        {
            return Err(Error::new("invalid_subtitles"));
        }
        Ok(SubtitleTrack {
            audio_revision: audio.id.clone(),
            audio_digest: audio.content.digest,
            source_revision: audio.source_revision.clone(),
            source: SubtitleSource::ForcedAlignment,
            cues: revision.cues.clone(),
            alignment_revision: Some(revision.id.clone()),
            language: Some(revision.language.clone()),
        })
    }

    fn measured_subtitle_track(
        &self,
        manifest: &Manifest,
        audio: &AudioRevision,
    ) -> Result<SubtitleTrack> {
        let assembly = audio
            .segment_assembly
            .as_ref()
            .ok_or_else(|| Error::new("subtitles_unavailable"))?;
        self.validate_segment_assembly(manifest, audio)?;
        let cues = assembly
            .timings
            .iter()
            .map(|timing| SubtitleCue {
                start_ms: frame_milliseconds(timing.start_frame, audio.contract.sample_rate_hz),
                end_ms: frame_milliseconds(timing.end_frame, audio.contract.sample_rate_hz),
                text: timing.text.clone(),
            })
            .collect::<Vec<_>>();
        validate_cues(
            &cues,
            Some(frame_milliseconds(
                audio.contract.frame_count,
                audio.contract.sample_rate_hz,
            )),
        )?;
        Ok(SubtitleTrack {
            audio_revision: audio.id.clone(),
            audio_digest: audio.content.digest,
            source_revision: audio.source_revision.clone(),
            source: SubtitleSource::MeasuredSegments,
            cues,
            alignment_revision: None,
            language: None,
        })
    }
}

fn accepted_audio<'a>(manifest: &'a Manifest, id: &str) -> Result<&'a AudioRevision> {
    manifest
        .audio
        .iter()
        .find(|audio| audio.id == id)
        .ok_or_else(|| Error::new("accepted_audio_missing"))
}
fn audio_record<'a>(manifest: &'a Manifest, id: &str) -> Result<&'a AudioRevision> {
    manifest
        .audio
        .iter()
        .chain(&manifest.candidates)
        .find(|audio| audio.id == id)
        .ok_or_else(|| Error::new("audio_missing"))
}

pub(crate) fn validate_records(manifest: &Manifest) -> Result<()> {
    if manifest.subtitle_candidates.len() + manifest.subtitles.len() > 128
        || manifest.subtitle_runs.len() > 256
        || manifest.subtitle_selections.len() > 1024
    {
        return Err(Error::new("invalid_subtitles"));
    }
    let mut runs = BTreeSet::new();
    let mut result_ids = BTreeSet::new();
    for run in &manifest.subtitle_runs {
        run.spec.validate()?;
        let audio = accepted_audio(manifest, &run.audio_revision)?;
        if !valid_id(&run.id)
            || !runs.insert(&run.id)
            || run.audio_digest != audio.content.digest
            || run.source_revision != audio.source_revision
            || run.document_digest != audio.document_digest
            || !matches!(
                run.status.as_str(),
                "running" | "completed" | "cancelled" | "failed" | "interrupted_unknown"
            )
            || (run.status == "completed" && run.candidate_id.is_none())
            || (matches!(
                run.status.as_str(),
                "running" | "failed" | "interrupted_unknown"
            ) && run.candidate_id.is_some())
            || run
                .candidate_id
                .as_ref()
                .is_some_and(|id| !valid_id(id) || !result_ids.insert(id))
            || run
                .error
                .as_ref()
                .is_some_and(|error| error.is_empty() || error.len() > 256 || !error.is_ascii())
        {
            return Err(Error::new("invalid_subtitles"));
        }
        let source = manifest
            .sources
            .iter()
            .find(|source| source.id == run.source_revision)
            .ok_or_else(|| Error::new("source_missing"))?;
        if source.document.digest() != run.document_digest {
            return Err(Error::new("stale_source"));
        }
    }
    let mut ids = BTreeSet::new();
    for revision in manifest
        .subtitle_candidates
        .iter()
        .chain(&manifest.subtitles)
    {
        revision.generation.validate_metadata()?;
        let audio = accepted_audio(manifest, &revision.audio_revision)?;
        let run = manifest
            .subtitle_runs
            .iter()
            .find(|run| run.id == revision.run_id)
            .ok_or_else(|| Error::new("invalid_subtitles"))?;
        if !valid_id(&revision.id)
            || !ids.insert(&revision.id)
            || run.candidate_id.as_ref() != Some(&revision.id)
            || !matches!(run.status.as_str(), "completed" | "cancelled")
            || run.audio_revision != revision.audio_revision
            || run.audio_digest != revision.audio_digest
            || run.source_revision != revision.source_revision
            || run.document_digest != revision.document_digest
            || revision.audio_digest != audio.content.digest
            || revision.generation.audio_digest != revision.audio_digest
            || revision.generation.spec != run.spec
            || revision.transcript != run.spec.transcript
            || revision.language != run.spec.language
            || revision.alignment.media_type != "application/json"
            || revision.alignment.byte_length > MAX_ALIGNMENT_RESPONSE_BYTES as u64
            || revision.alignment.digest != revision.generation.output_digest
            || revision.cues.len() > 8192
        {
            return Err(Error::new("invalid_subtitles"));
        }
        validate_cues(
            &revision.cues,
            Some(frame_milliseconds(
                audio.contract.frame_count,
                audio.contract.sample_rate_hz,
            )),
        )?;
    }
    for (audio_id, revision_id) in &manifest.subtitle_selections {
        if !manifest
            .subtitles
            .iter()
            .any(|revision| revision.id == *revision_id && revision.audio_revision == *audio_id)
        {
            return Err(Error::new("invalid_subtitles"));
        }
    }
    Ok(())
}

fn valid_id(id: &str) -> bool {
    uuid::Uuid::parse_str(id).is_ok()
}

fn validate_cues(cues: &[SubtitleCue], duration_ms: Option<u64>) -> Result<()> {
    if cues.is_empty() {
        return Err(Error::new("subtitles_unavailable"));
    }
    let mut previous_end = 0;
    for cue in cues {
        if cue.start_ms < previous_end
            || cue.end_ms <= cue.start_ms
            || duration_ms.is_some_and(|duration| cue.end_ms > duration)
            || cue.text.trim().is_empty()
            || cue.text.len() > MAX_ALIGNMENT_TRANSCRIPT_BYTES
            || cue
                .text
                .chars()
                .any(|ch| ch.is_control() && !matches!(ch, '\r' | '\n' | '\t'))
        {
            return Err(Error::new("invalid_subtitles"));
        }
        previous_end = cue.end_ms;
    }
    Ok(())
}

fn cues_for_alignment(
    response: &AlignmentResponse,
    spec: &AlignmentSpec,
    duration: f64,
) -> Result<Vec<SubtitleCue>> {
    let tokens = aligned_tokens(response, spec, duration)?;
    let mut cues: Vec<SubtitleCue> = Vec::new();
    let mut first = 0;
    for (index, token) in tokens.iter().enumerate() {
        let text = &spec.transcript[tokens[first].start_byte..token.end_byte];
        let last = index + 1 == tokens.len();
        let sentence_end = text
            .trim_end()
            .ends_with(['.', '!', '?', ';', '。', '！', '？', '；'])
            || text.ends_with(['\n', '\r']);
        let boundary = last
            || sentence_end
            || text.chars().count() >= 64
            || token.end_ms.saturating_sub(tokens[first].start_ms) >= 6000;
        if boundary && token.end_ms > tokens[first].start_ms {
            cues.push(SubtitleCue {
                start_ms: tokens[first].start_ms,
                end_ms: token.end_ms,
                text: text.trim().to_owned(),
            });
            first = index + 1;
        }
    }
    // A zero-duration trailing token is retained with the preceding cue instead
    // of silently disappearing or creating an invalid zero-length caption.
    if first < tokens.len() {
        let final_cue = cues
            .last_mut()
            .ok_or_else(|| Error::new("invalid_subtitles"))?;
        let prior_start = tokens
            .iter()
            .find(|token| token.start_ms == final_cue.start_ms)
            .map(|token| token.start_byte)
            .ok_or_else(|| Error::new("invalid_subtitles"))?;
        final_cue.text = spec.transcript[prior_start..].trim().into();
        final_cue.end_ms = tokens
            .last()
            .ok_or_else(|| Error::new("invalid_subtitles"))?
            .end_ms;
    }
    validate_cues(&cues, Some((duration * 1000.0).round() as u64))?;
    Ok(cues)
}

fn frame_milliseconds(frames: u64, sample_rate: u32) -> u64 {
    let milliseconds = (u128::from(frames) * 1000 + u128::from(sample_rate) / 2)
        .checked_div(u128::from(sample_rate))
        .unwrap_or(0);
    u64::try_from(milliseconds).unwrap_or(u64::MAX)
}
fn timestamp(milliseconds: u64, format: &str) -> String {
    format!(
        "{:02}:{:02}:{:02}{}{:03}",
        milliseconds / 3_600_000,
        milliseconds / 60_000 % 60,
        milliseconds / 1000 % 60,
        if format == "srt" { ',' } else { '.' },
        milliseconds % 1000
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        AudioOriginDisclosure, ExecutionOutput,
        infer_runtime::captions::tests::{fixture, spec, speech_receipt, wav},
        project::Document,
    };
    use std::sync::Arc;

    pub(crate) fn accept_alignment_fixture(
        project: &Project,
        audio_id: &str,
        spec: AlignmentSpec,
    ) -> Manifest {
        let outcome = project
            .align_subtitles_using(
                project.read().unwrap().version,
                audio_id,
                spec,
                &SpeechSynthesisControl::default(),
                |bytes, spec, _| Ok(fixture(bytes, spec)),
            )
            .unwrap();
        project
            .accept_subtitles(
                outcome.manifest.version,
                outcome.candidate_id.as_deref().unwrap(),
            )
            .unwrap()
    }

    struct Fixture {
        project: Project,
        audio_id: String,
    }
    impl Fixture {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("vox-captions-{}.vox", uuid::Uuid::now_v7()));
            let project = Project::create(&path, "Subtitle contracts").unwrap();
            let manifest = project
                .save(
                    0,
                    Document {
                        text: "Original words, before an Echo cut.".into(),
                        ..Document::default()
                    },
                    None,
                )
                .unwrap();
            let manifest = project.adopt_script(manifest.version).unwrap();
            let audio_id = uuid::Uuid::now_v7().to_string();
            let bytes = wav(48_000);
            let content = project.put_object(&bytes, "audio/wav").unwrap();
            project
                .update(manifest.version, |manifest| {
                    let source = manifest.sources.last().unwrap();
                    manifest.audio.push(AudioRevision {
                        id: audio_id.clone(),
                        source_revision: source.id.clone(),
                        document_digest: source.document.digest(),
                        content,
                        contract: parse_pcm_s16le_wav(
                            &bytes,
                            AudioOriginDisclosure::ImportedUnverified,
                        )?,
                        provenance: None,
                        origin: "echo_edit".into(),
                        parent_audio: None,
                        source_project: None,
                        edit: None,
                        segment_assembly: None,
                    });
                    manifest.audio_head = Some(audio_id.clone());
                    Ok(())
                })
                .unwrap();
            Self { project, audio_id }
        }
        fn align(&self, control: &SpeechSynthesisControl) -> SubtitleOutcome {
            let version = self.project.read().unwrap().version;
            self.project
                .align_subtitles_using(
                    version,
                    &self.audio_id,
                    spec(),
                    control,
                    |bytes, spec, _| Ok(fixture(bytes, spec)),
                )
                .unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.project.path);
        }
    }

    #[test]
    fn edited_audio_has_no_stale_fallback_and_requires_explicit_reopenable_acceptance() {
        let f = Fixture::new();
        assert_eq!(
            f.project.subtitle_track(&f.audio_id).unwrap_err().code,
            "subtitles_unavailable"
        );
        let outcome = f.align(&SpeechSynthesisControl::default());
        assert_eq!(outcome.state, SubtitleState::Completed);
        let id = outcome.candidate_id.unwrap();
        assert!(outcome.manifest.subtitles.is_empty());
        assert!(outcome.manifest.subtitle_selections.is_empty());
        assert_eq!(
            f.project.subtitle_track(&f.audio_id).unwrap_err().code,
            "subtitles_unavailable"
        );
        let candidate = f.project.subtitle_candidate_track(&id).unwrap();
        assert_eq!(candidate.source, SubtitleSource::ForcedAlignment);
        assert_eq!(candidate.cues[0].text, "Hello, 中文 café e\u{301}!");
        let accepted = f
            .project
            .accept_subtitles(outcome.manifest.version, &id)
            .unwrap();
        assert!(accepted.subtitle_candidates.is_empty());
        assert_eq!(accepted.subtitle_selections[&f.audio_id], id);
        let reopened = Project::open(&f.project.path).unwrap();
        let track = reopened.subtitle_track(&f.audio_id).unwrap();
        assert_eq!(track.alignment_revision.as_ref(), Some(&id));
        assert_eq!(track.audio_revision, f.audio_id);
        assert_eq!(
            reopened.subtitle_transcript(&f.audio_id).unwrap(),
            spec().transcript
        );
        let path = std::env::temp_dir().join(format!("vox-caption-{}.srt", uuid::Uuid::now_v7()));
        reopened
            .export_subtitles(&f.audio_id, &path, "srt")
            .unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("00:00:00,010 --> 00:00:00,500\nHello, 中文 café"));
        assert_eq!(
            reopened
                .export_subtitles(&f.audio_id, &path, "srt")
                .unwrap_err()
                .code,
            "destination_exists"
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn cancellation_before_dispatch_skips_model_and_inflight_result_remains_reviewable() {
        let f = Fixture::new();
        let control = SpeechSynthesisControl::default();
        control.cancel();
        let outcome = f
            .project
            .align_subtitles_using(
                f.project.read().unwrap().version,
                &f.audio_id,
                spec(),
                &control,
                |_, _, _| panic!("cancelled request must not dispatch"),
            )
            .unwrap();
        assert_eq!(outcome.state, SubtitleState::Cancelled);
        assert!(outcome.candidate_id.is_none());
        assert_eq!(outcome.manifest.subtitle_runs[0].status, "cancelled");
        let control = SpeechSynthesisControl::default();
        let outcome = f
            .project
            .align_subtitles_using(
                outcome.manifest.version,
                &f.audio_id,
                spec(),
                &control,
                |bytes, spec, control| {
                    control.cancel();
                    Ok(fixture(bytes, spec))
                },
            )
            .unwrap();
        assert_eq!(outcome.state, SubtitleState::Cancelled);
        let id = outcome.candidate_id.unwrap();
        assert_eq!(outcome.manifest.subtitle_runs[1].status, "cancelled");
        assert!(outcome.manifest.subtitle_selections.is_empty());
        assert!(f.project.subtitle_candidate_track(&id).is_ok());
        f.project
            .accept_subtitles(outcome.manifest.version, &id)
            .unwrap();
    }

    #[test]
    fn generation_is_fenced_by_version_and_pins_while_concurrent_draft_saves_survive() {
        let f = Fixture::new();
        let version = f.project.read().unwrap().version;
        assert_eq!(
            f.project
                .align_subtitles_using(
                    version - 1,
                    &f.audio_id,
                    spec(),
                    &SpeechSynthesisControl::default(),
                    |_, _, _| panic!("stale version never dispatches")
                )
                .unwrap_err()
                .code,
            "stale_project"
        );
        let outcome = f
            .project
            .align_subtitles_using(
                version,
                &f.audio_id,
                spec(),
                &SpeechSynthesisControl::default(),
                |bytes, spec, _| {
                    let manifest = f.project.read().unwrap();
                    let mut draft = manifest.draft;
                    draft.text = "New draft saved during alignment.".into();
                    f.project.save(manifest.version, draft, None).unwrap();
                    Ok(fixture(bytes, spec))
                },
            )
            .unwrap();
        assert_eq!(outcome.state, SubtitleState::Completed);
        assert_eq!(
            outcome.manifest.draft.text,
            "New draft saved during alignment."
        );
        let id = outcome.candidate_id.unwrap();
        assert_eq!(
            f.project.accept_subtitles(version, &id).unwrap_err().code,
            "stale_project"
        );
        f.project
            .accept_subtitles(outcome.manifest.version, &id)
            .unwrap();
    }

    #[test]
    fn runtime_failures_and_lost_producer_runs_are_durable_without_automatic_retry() {
        let f = Fixture::new();
        let outcome = f
            .project
            .align_subtitles_using(
                f.project.read().unwrap().version,
                &f.audio_id,
                spec(),
                &SpeechSynthesisControl::default(),
                |_, _, _| Err(Error::new("infer_unavailable")),
            )
            .unwrap();
        assert_eq!(outcome.state, SubtitleState::Failed);
        assert_eq!(outcome.error.as_deref(), Some("infer_unavailable"));
        assert_eq!(outcome.manifest.subtitle_runs[0].status, "failed");
        assert_eq!(
            f.project.recover().unwrap().version,
            outcome.manifest.version
        );
        let id = uuid::Uuid::now_v7().to_string();
        let (manifest, _) = f
            .project
            .update(outcome.manifest.version, |manifest| {
                let audio = accepted_audio(manifest, &f.audio_id)?;
                manifest.subtitle_runs.push(SubtitleRun {
                    id: id.clone(),
                    audio_revision: audio.id.clone(),
                    audio_digest: audio.content.digest,
                    source_revision: audio.source_revision.clone(),
                    document_digest: audio.document_digest.clone(),
                    spec: spec(),
                    status: "running".into(),
                    candidate_id: None,
                    error: None,
                });
                Ok(())
            })
            .unwrap();
        let producer = OpenOptions::new()
            .read(true)
            .write(true)
            .open(f.project.path.join("render.lock"))
            .unwrap();
        producer.lock().unwrap();
        assert_eq!(f.project.recover().unwrap().version, manifest.version);
        assert_eq!(
            f.project
                .align_subtitles_using(
                    manifest.version,
                    &f.audio_id,
                    spec(),
                    &SpeechSynthesisControl::default(),
                    |_, _, _| panic!("overlap never dispatches")
                )
                .unwrap_err()
                .code,
            "render_busy"
        );
        drop(producer);
        let recovered = f.project.recover().unwrap();
        assert_eq!(
            recovered.subtitle_runs.last().unwrap().status,
            "interrupted_unknown"
        );
        assert!(recovered.subtitle_candidates.is_empty());
        assert_eq!(f.project.recover().unwrap().version, recovered.version);
    }

    #[test]
    fn tampered_response_audio_or_cue_metadata_cannot_be_accepted_or_exported() {
        let f = Fixture::new();
        let outcome = f.align(&SpeechSynthesisControl::default());
        let id = outcome.candidate_id.unwrap();
        let revision = &outcome.manifest.subtitle_candidates[0];
        let path = f
            .project
            .path
            .join("objects")
            .join(revision.alignment.digest.to_string());
        let original = std::fs::read(&path).unwrap();
        std::fs::write(&path, b"{}").unwrap();
        assert_eq!(
            f.project
                .accept_subtitles(outcome.manifest.version, &id)
                .unwrap_err()
                .code,
            "corrupt_object"
        );
        std::fs::write(&path, original).unwrap();
        let (manifest, _) = f
            .project
            .update(outcome.manifest.version, |manifest| {
                manifest.subtitle_candidates[0].cues[0].end_ms += 1;
                Ok(())
            })
            .unwrap();
        assert_eq!(
            f.project
                .accept_subtitles(manifest.version, &id)
                .unwrap_err()
                .code,
            "invalid_subtitles"
        );
        let manifest = f.project.discard_subtitles(manifest.version, &id).unwrap();
        assert!(manifest.subtitle_candidates.is_empty());
        assert_eq!(manifest.subtitle_runs[0].candidate_id.as_ref(), Some(&id));
        assert_eq!(
            f.project.subtitle_candidate_track(&id).unwrap_err().code,
            "subtitle_missing"
        );
    }

    #[test]
    fn source_suggestion_applies_pronunciation_and_repeats_without_current_draft_text() {
        let f = Fixture::new();
        let manifest = f.project.read().unwrap();
        let mut document = manifest.draft;
        document.format = ScriptFormat::ProductionScript;
        document.text = "[repeat: 2]\nDr. Li.\n[end-repeat]".into();
        document.pronunciation_rules.push(crate::PronunciationRule {
            text: "Dr.".into(),
            spoken: "Doctor".into(),
            language: None,
            role: None,
        });
        let manifest = f.project.save(manifest.version, document, None).unwrap();
        let manifest = f.project.adopt_script(manifest.version).unwrap();
        let (manifest, _) = f
            .project
            .update(manifest.version, |manifest| {
                let source = manifest.sources.last().unwrap();
                let audio = manifest
                    .audio
                    .iter_mut()
                    .find(|audio| audio.id == f.audio_id)
                    .unwrap();
                audio.source_revision = source.id.clone();
                audio.document_digest = source.document.digest();
                Ok(())
            })
            .unwrap();
        let mut draft = manifest.draft;
        draft.text = "Different unadopted text.".into();
        f.project.save(manifest.version, draft, None).unwrap();
        assert_eq!(
            f.project.subtitle_transcript(&f.audio_id).unwrap(),
            "Doctor Li.\nDoctor Li."
        );
    }

    #[test]
    fn measured_caption_fallback_stays_with_original_assembly_after_echo_return() {
        let f = Fixture::new();
        let manifest = f.project.read().unwrap();
        let key = f.project.segment_plan().unwrap().segments[0].key.clone();
        let manifest = f
            .project
            .render_segment_using(
                manifest.version,
                &key,
                SpeechSynthesisControl::default(),
                Arc::new(|_, _| {}),
                |_| {
                    let (job_id, provenance) = speech_receipt();
                    Ok(ExecutionOutput {
                        bytes: wav(48_000),
                        media_type: "audio/wav".into(),
                        executor_job_id: Some(job_id),
                        external_provenance: Some(provenance),
                        content_contract: None,
                    })
                },
            )
            .unwrap();
        let manifest = f
            .project
            .accept_segment(manifest.version, &manifest.segment_candidates[0].id)
            .unwrap();
        let manifest = f.project.assemble_segments(manifest.version).unwrap();
        let original_id = manifest.candidates[0].id.clone();
        f.project.accept(manifest.version, &original_id).unwrap();
        assert_eq!(
            f.project.subtitle_track(&original_id).unwrap().source,
            SubtitleSource::MeasuredSegments
        );
        assert_eq!(
            f.project.subtitle_track(&f.audio_id).unwrap_err().code,
            "subtitles_unavailable"
        );
        let manifest = accept_alignment_fixture(&f.project, &original_id, spec());
        let id = manifest.subtitle_selections[&original_id].clone();
        assert_eq!(
            f.project.subtitle_track(&original_id).unwrap().source,
            SubtitleSource::ForcedAlignment
        );
        let manifest = f
            .project
            .use_measured_subtitles(manifest.version, &original_id)
            .unwrap();
        assert_eq!(
            f.project.subtitle_track(&original_id).unwrap().source,
            SubtitleSource::MeasuredSegments
        );
        assert_eq!(
            manifest.subtitles.len(),
            1,
            "restoring measured captions retains accepted history"
        );
        f.project.accept_subtitles(manifest.version, &id).unwrap();
        assert_eq!(
            f.project.subtitle_track(&original_id).unwrap().source,
            SubtitleSource::ForcedAlignment
        );
    }

    #[test]
    fn serializer_keeps_literal_unicode_and_rejects_invalid_timeline_and_format() {
        let track = SubtitleTrack {
            audio_revision: "test".into(),
            audio_digest: ContentDigest::from_bytes(b"wave"),
            source_revision: "source".into(),
            source: SubtitleSource::ForcedAlignment,
            cues: vec![SubtitleCue {
                start_ms: 3_599_999,
                end_ms: 3_600_001,
                text: "中文 <tag> & café".into(),
            }],
            alignment_revision: None,
            language: None,
        };
        assert!(
            track
                .serialize("vtt")
                .unwrap()
                .contains("00:59:59.999 --> 01:00:00.001\n中文 &lt;tag&gt; &amp; café")
        );
        assert_eq!(
            track.serialize("txt").unwrap_err().code,
            "invalid_subtitle_format"
        );
        let mut invalid = track;
        invalid.cues[0].end_ms = invalid.cues[0].start_ms;
        assert_eq!(
            invalid.serialize("srt").unwrap_err().code,
            "invalid_subtitles"
        );
    }
}
