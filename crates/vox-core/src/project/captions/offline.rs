//! Offline subtitle authoring and a deliberately plain, lossless import dialect.
use super::*;

pub const MAX_SUBTITLE_IMPORT_BYTES: usize = 1024 * 1024;
const MAX_SUBTITLE_SNAPSHOT_BYTES: u64 = 2 * 1024 * 1024;
const TRACK_MEDIA_TYPE: &str = "application/vnd.vox.subtitle-track+json";
const REVISION_MEDIA_TYPE: &str = "application/vnd.vox.subtitle-revision+json";
const EDIT_MEDIA_TYPE: &str = "application/vnd.vox.subtitle-edit+json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubtitleImportFormat {
    Srt,
    Vtt,
}

impl SubtitleImportFormat {
    fn parse(format: &str) -> Result<Self> {
        match format {
            "srt" => Ok(Self::Srt),
            "vtt" => Ok(Self::Vtt),
            _ => Err(Error::new("invalid_subtitle_format")),
        }
    }

    fn media_type(self) -> &'static str {
        match self {
            Self::Srt => "application/x-subrip",
            Self::Vtt => "text/vtt",
        }
    }
}

/// The complete base remains independently verifiable after discarding a candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubtitleParent {
    pub revision_id: Option<String>,
    pub track: ContentRef,
    pub revision: Option<ContentRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SubtitleProvenance {
    ManualRevision {
        parent: SubtitleParent,
        edit: ContentRef,
    },
    ImportedSubtitles {
        original: ContentRef,
        format: SubtitleImportFormat,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubtitleEdit {
    schema: String,
    parent: SubtitleParent,
    cues: Vec<SubtitleCue>,
    language: String,
}

impl Project {
    /// Text, timing, splits and merges are one full-array candidate revision.
    /// `None` pins the current selection or measured track; `Some` pins history.
    pub fn edit_subtitles(
        &self,
        expected: u64,
        audio_id: &str,
        base_candidate_id: Option<&str>,
        cues: Vec<SubtitleCue>,
    ) -> Result<(Manifest, String)> {
        self.update(expected, |manifest| {
            check_capacity(manifest)?;
            let audio = accepted_audio(manifest, audio_id)?.clone();
            self.checked_audio(&audio)?;
            validate_edit_cues(&cues, Some(duration_ms(&audio)))?;
            let base_id = base_candidate_id.or_else(|| {
                manifest
                    .subtitle_selections
                    .get(audio_id)
                    .map(String::as_str)
            });
            let (track, revision) = if let Some(id) = base_id {
                let revision = manifest
                    .subtitle_candidates
                    .iter()
                    .chain(&manifest.subtitles)
                    .find(|revision| revision.id == id)
                    .ok_or_else(|| Error::new("subtitle_missing"))?;
                if revision.audio_revision != audio_id {
                    return Err(Error::new("stale_audio"));
                }
                (self.checked_subtitle(manifest, revision)?, Some(revision))
            } else {
                (self.measured_subtitle_track(manifest, &audio)?, None)
            };
            let parent = SubtitleParent {
                revision_id: track.revision_id.clone(),
                track: self.put_subtitle_snapshot(&track, TRACK_MEDIA_TYPE)?,
                revision: revision
                    .map(|revision| self.put_subtitle_snapshot(revision, REVISION_MEDIA_TYPE))
                    .transpose()?,
            };
            let language = track.language.unwrap_or_default();
            let edit = SubtitleEdit {
                schema: "vox.subtitle-edit/1".into(),
                parent: parent.clone(),
                cues: cues.clone(),
                language: language.clone(),
            };
            let edit = self.put_subtitle_snapshot(&edit, EDIT_MEDIA_TYPE)?;
            let revision = offline_revision(
                &audio,
                cues,
                language,
                SubtitleProvenance::ManualRevision { parent, edit },
            );
            // Includes the newly added ancestor edge, so the depth limit cannot
            // create an unreadable candidate at the boundary.
            self.checked_subtitle(manifest, &revision)?;
            let id = revision.id.clone();
            manifest.subtitle_candidates.push(revision);
            validate_records(manifest)?;
            Ok(id)
        })
    }

    /// Import exact UTF-8 bytes locally. Parsing never dispatches Runtime work.
    pub fn import_subtitles(
        &self,
        expected: u64,
        audio_id: &str,
        bytes: &[u8],
        format: &str,
        language: Option<&str>,
    ) -> Result<(Manifest, String)> {
        self.update(expected, |manifest| {
            check_capacity(manifest)?;
            let format = SubtitleImportFormat::parse(format)?;
            validate_language(language.unwrap_or_default())?;
            let audio = accepted_audio(manifest, audio_id)?.clone();
            self.checked_audio(&audio)?;
            let cues = parse_import(bytes, format, Some(duration_ms(&audio)))?;
            let original = self.put_object(bytes, format.media_type())?;
            let revision = offline_revision(
                &audio,
                cues,
                language.unwrap_or_default().into(),
                SubtitleProvenance::ImportedSubtitles { original, format },
            );
            let id = revision.id.clone();
            manifest.subtitle_candidates.push(revision);
            validate_records(manifest)?;
            Ok(id)
        })
    }

    fn put_subtitle_snapshot(
        &self,
        value: &impl Serialize,
        media_type: &str,
    ) -> Result<ContentRef> {
        let bytes = serde_json::to_vec(value)?;
        if bytes.len() as u64 > MAX_SUBTITLE_SNAPSHOT_BYTES {
            return Err(Error::new("subtitle_snapshot_limit"));
        }
        self.put_object(&bytes, media_type)
    }

    pub(super) fn check_offline_subtitle(
        &self,
        manifest: &Manifest,
        revision: &SubtitleRevision,
        provenance: &SubtitleProvenance,
        audio: &AudioRevision,
        audio_bytes: &[u8],
        depth: usize,
    ) -> Result<()> {
        match provenance {
            SubtitleProvenance::ImportedSubtitles { original, format } => {
                let original = self.object(original)?;
                if parse_import(&original, *format, Some(duration_ms(audio)))? != revision.cues {
                    return Err(Error::new("invalid_subtitles"));
                }
            }
            SubtitleProvenance::ManualRevision { parent, edit } => {
                let edit: SubtitleEdit = serde_json::from_slice(&self.object(edit)?)?;
                if edit.schema != "vox.subtitle-edit/1"
                    || edit.parent != *parent
                    || edit.cues != revision.cues
                    || edit.language != revision.language
                {
                    return Err(Error::new("invalid_subtitles"));
                }
                let track: SubtitleTrack = serde_json::from_slice(&self.object(&parent.track)?)?;
                if track.audio_revision != revision.audio_revision
                    || track.audio_digest != revision.audio_digest
                    || track.source_revision != revision.source_revision
                    || track.revision_id != parent.revision_id
                    || track.language.clone().unwrap_or_default() != revision.language
                    || parent.revision_id.as_ref() == Some(&revision.id)
                {
                    return Err(Error::new("invalid_subtitles"));
                }
                let expected = if let Some(snapshot) = &parent.revision {
                    let base: SubtitleRevision = serde_json::from_slice(&self.object(snapshot)?)?;
                    if Some(&base.id) != parent.revision_id.as_ref() {
                        return Err(Error::new("invalid_subtitles"));
                    }
                    self.checked_subtitle_depth(manifest, &base, audio, audio_bytes, depth + 1)?
                } else {
                    if parent.revision_id.is_some() {
                        return Err(Error::new("invalid_subtitles"));
                    }
                    self.measured_subtitle_track(manifest, audio)?
                };
                if track != expected {
                    return Err(Error::new("invalid_subtitles"));
                }
            }
        }
        Ok(())
    }
}

fn check_capacity(manifest: &Manifest) -> Result<()> {
    if manifest.subtitle_candidates.len() + manifest.subtitles.len() >= 128 {
        return Err(Error::new("project_limit"));
    }
    Ok(())
}

fn duration_ms(audio: &AudioRevision) -> u64 {
    frame_milliseconds(audio.contract.frame_count, audio.contract.sample_rate_hz)
}

fn offline_revision(
    audio: &AudioRevision,
    cues: Vec<SubtitleCue>,
    language: String,
    provenance: SubtitleProvenance,
) -> SubtitleRevision {
    SubtitleRevision {
        id: uuid::Uuid::now_v7().to_string(),
        run_id: None,
        audio_revision: audio.id.clone(),
        audio_digest: audio.content.digest,
        source_revision: audio.source_revision.clone(),
        document_digest: audio.document_digest.clone(),
        transcript: cue_transcript(&cues),
        language,
        cues,
        alignment: None,
        generation: None,
        provenance: Some(provenance),
    }
}

pub(super) fn validate_offline_record(revision: &SubtitleRevision) -> Result<()> {
    if revision.run_id.is_some()
        || revision.alignment.is_some()
        || revision.generation.is_some()
        || revision.transcript != cue_transcript(&revision.cues)
    {
        return Err(Error::new("invalid_subtitles"));
    }
    validate_language(&revision.language)?;
    validate_edit_cues(&revision.cues, None)?;
    match revision.provenance.as_ref() {
        Some(SubtitleProvenance::ManualRevision { parent, edit }) => {
            if !valid_snapshot(&parent.track, TRACK_MEDIA_TYPE)
                || !valid_snapshot(edit, EDIT_MEDIA_TYPE)
                || parent
                    .revision_id
                    .as_ref()
                    .is_some_and(|id| !valid_id(id) || *id == revision.id)
                || parent.revision_id.is_some() != parent.revision.is_some()
                || parent
                    .revision
                    .as_ref()
                    .is_some_and(|object| !valid_snapshot(object, REVISION_MEDIA_TYPE))
            {
                return Err(Error::new("invalid_subtitles"));
            }
        }
        Some(SubtitleProvenance::ImportedSubtitles { original, format }) => {
            if original.media_type != format.media_type()
                || original.byte_length == 0
                || original.byte_length > MAX_SUBTITLE_IMPORT_BYTES as u64
            {
                return Err(Error::new("invalid_subtitles"));
            }
        }
        None => return Err(Error::new("invalid_subtitles")),
    }
    Ok(())
}

fn valid_snapshot(content: &ContentRef, media_type: &str) -> bool {
    content.media_type == media_type
        && content.byte_length > 0
        && content.byte_length <= MAX_SUBTITLE_SNAPSHOT_BYTES
}

fn validate_language(language: &str) -> Result<()> {
    if !language.is_empty()
        && (language == "auto"
            || !crate::infer_runtime::speech::SPEECH_LANGUAGES.contains(&language))
    {
        return Err(Error::new("invalid_subtitle_language"));
    }
    Ok(())
}

fn cue_transcript(cues: &[SubtitleCue]) -> String {
    cues.iter()
        .map(|cue| cue.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn validate_edit_cues(cues: &[SubtitleCue], duration_ms: Option<u64>) -> Result<()> {
    if cues.len() > 8192 {
        return Err(Error::new("invalid_subtitles"));
    }
    validate_cues(cues, duration_ms)?;
    let text_bytes = cues
        .iter()
        .try_fold(0usize, |total, cue| total.checked_add(cue.text.len() + 1));
    if text_bytes.is_none_or(|size| size > MAX_ALIGNMENT_TRANSCRIPT_BYTES + 1)
        || cues.iter().any(|cue| {
            cue.text.contains('\r')
                || cue.text.contains("-->")
                || cue.text.split('\n').any(|line| line.trim().is_empty())
        })
    {
        return Err(Error::new("invalid_subtitles"));
    }
    Ok(())
}

/// Supported dialect: plain indexed SRT; plain WebVTT with optional cue
/// identifiers; canonical timestamps and six named WebVTT text entities. Styling,
/// cue settings, metadata blocks and markup are rejected instead of discarded.
fn parse_import(
    bytes: &[u8],
    format: SubtitleImportFormat,
    duration_ms: Option<u64>,
) -> Result<Vec<SubtitleCue>> {
    if bytes.is_empty() || bytes.len() > MAX_SUBTITLE_IMPORT_BYTES {
        return Err(Error::new("invalid_subtitle_import"));
    }
    let input = std::str::from_utf8(bytes).map_err(|_| Error::new("invalid_subtitle_import"))?;
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);
    let normalized = input.replace("\r\n", "\n");
    if normalized.contains('\r')
        || normalized
            .chars()
            .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\t'))
    {
        return Err(Error::new("invalid_subtitle_import"));
    }
    let mut lines = normalized.split('\n').peekable();
    if format == SubtitleImportFormat::Vtt
        && (lines.next() != Some("WEBVTT") || lines.next() != Some(""))
    {
        return Err(Error::new("unsupported_subtitle_features"));
    }
    let mut cues = Vec::new();
    while let Some(line) = lines.next() {
        if line.is_empty() {
            continue;
        }
        if cues.len() >= 8192 {
            return Err(Error::new("invalid_subtitle_import"));
        }
        let timing = match format {
            SubtitleImportFormat::Srt => {
                if line.len() > 32 || !line.bytes().all(|ch| ch.is_ascii_digit()) {
                    return Err(Error::new("invalid_subtitle_import"));
                }
                lines
                    .next()
                    .ok_or_else(|| Error::new("invalid_subtitle_import"))?
            }
            SubtitleImportFormat::Vtt if line.contains(" --> ") => line,
            SubtitleImportFormat::Vtt => {
                if line.len() > 512
                    || line.contains("-->")
                    || ["NOTE", "STYLE", "REGION"].iter().any(|reserved| {
                        line == *reserved
                            || line
                                .strip_prefix(reserved)
                                .is_some_and(|suffix| suffix.starts_with([' ', '\t']))
                    })
                {
                    return Err(Error::new("unsupported_subtitle_features"));
                }
                // Cue IDs are labels, not ordering or styling instructions.
                // Original labels stay in the retained file; times govern order.
                lines
                    .next()
                    .ok_or_else(|| Error::new("invalid_subtitle_import"))?
            }
        };
        let (start, end) = timing
            .split_once(" --> ")
            .ok_or_else(|| Error::new("invalid_subtitle_import"))?;
        if end.contains(char::is_whitespace) {
            return Err(Error::new("unsupported_subtitle_features"));
        }
        let start_ms = parse_timestamp(start, format)?;
        let end_ms = parse_timestamp(end, format)?;
        let mut text = String::new();
        while let Some(line) = lines.peek() {
            if line.is_empty() {
                break;
            }
            let line = lines.next().expect("peeked line");
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(line);
        }
        let text = decode_text(&text)?;
        cues.push(SubtitleCue {
            start_ms,
            end_ms,
            text,
        });
    }
    validate_edit_cues(&cues, duration_ms)?;
    Ok(cues)
}

fn parse_timestamp(value: &str, format: SubtitleImportFormat) -> Result<u64> {
    let invalid = || Error::new("invalid_subtitle_import");
    let separator = if format == SubtitleImportFormat::Srt {
        ','
    } else {
        '.'
    };
    let (whole, millis) = value.split_once(separator).ok_or_else(invalid)?;
    if millis.len() != 3 || !millis.bytes().all(|ch| ch.is_ascii_digit()) {
        return Err(invalid());
    }
    let fields = whole.split(':').collect::<Vec<_>>();
    let (hours, minutes, seconds) = match fields.as_slice() {
        [hours, minutes, seconds] if hours.len() >= 2 => (*hours, *minutes, *seconds),
        [minutes, seconds] if format == SubtitleImportFormat::Vtt => ("00", *minutes, *seconds),
        _ => return Err(invalid()),
    };
    if minutes.len() != 2
        || seconds.len() != 2
        || ![hours, minutes, seconds]
            .iter()
            .all(|part| part.bytes().all(|ch| ch.is_ascii_digit()))
    {
        return Err(invalid());
    }
    let hours: u64 = hours.parse().map_err(|_| invalid())?;
    let minutes: u64 = minutes.parse().map_err(|_| invalid())?;
    let seconds: u64 = seconds.parse().map_err(|_| invalid())?;
    let millis: u64 = millis.parse().map_err(|_| invalid())?;
    if minutes >= 60 || seconds >= 60 {
        return Err(invalid());
    }
    hours
        .checked_mul(3_600_000)
        .and_then(|value| value.checked_add(minutes * 60_000 + seconds * 1000 + millis))
        .ok_or_else(invalid)
}

fn decode_text(value: &str) -> Result<String> {
    if value.contains('<') || value.contains('>') || value.contains("{\\") {
        return Err(Error::new("unsupported_subtitle_features"));
    }
    let mut remaining = value;
    let mut output = String::with_capacity(value.len());
    while let Some(index) = remaining.find('&') {
        output.push_str(&remaining[..index]);
        remaining = &remaining[index..];
        let entity = [
            ("&amp;", "&"),
            ("&lt;", "<"),
            ("&gt;", ">"),
            ("&nbsp;", "\u{a0}"),
            ("&lrm;", "\u{200e}"),
            ("&rlm;", "\u{200f}"),
        ]
        .into_iter()
        .find(|(entity, _)| remaining.starts_with(entity));
        if let Some((entity, text)) = entity {
            output.push_str(text);
            remaining = &remaining[entity.len()..];
        } else {
            if remaining[1..]
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '#')
                && remaining[1..].find(';').is_some_and(|index| {
                    remaining[1..=index]
                        .chars()
                        .all(|ch| ch.is_ascii_alphanumeric() || ch == '#')
                })
            {
                return Err(Error::new("unsupported_subtitle_features"));
            }
            output.push('&');
            remaining = &remaining[1..];
        }
    }
    output.push_str(remaining);
    Ok(output)
}

#[cfg(test)]
mod tests;
