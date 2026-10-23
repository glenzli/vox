//! Immutable, revision-pinned collection export. Editing and mixing remain Echo's job.
mod pcm;
#[cfg(test)]
mod tests;

use super::{
    Chapter,
    export::{publish_export, require_current_settings, require_current_source},
    store::{check_version, selected_chapters, settings_digest},
};
use crate::{
    ContentDigest, ContentRef,
    audio::decode_pcm_s16le_wav,
    project::{
        AudioRevision, Error, Manifest, Project, Result, SourceRevision, SubtitleCue,
        SubtitleSource, SubtitleTrack,
    },
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, io::Write, path::Path};

pub const MAX_COMPILATION_BYTES: u64 = 512 * 1024 * 1024;
pub const MAX_COMPILATION_INPUT_BYTES: u64 = 512 * 1024 * 1024;
pub const MAX_COMPILATION_SECONDS: u64 = 24 * 60 * 60;
pub const MAX_CHAPTER_GAP_MS: u64 = 600_000;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChapterCompilationAudioMode {
    #[default]
    Preserve,
    NormalizeMono24000,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChapterCompilationSubtitles {
    #[default]
    Required,
    AudioOnly,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChapterCompilationOptions {
    #[serde(default)]
    pub gap_ms: u64,
    #[serde(default)]
    pub audio_mode: ChapterCompilationAudioMode,
    #[serde(default)]
    pub subtitles: ChapterCompilationSubtitles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterCompilationOutput {
    pub sample_rate_hz: u32,
    pub channels: u16,
    pub frames: u64,
    pub gap_frames: u64,
    pub duration_ms: u64,
    pub byte_length: u64,
    pub subtitle_cues: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterCompilationPreview {
    pub chapter_id: String,
    pub title: String,
    pub project_id: String,
    pub project_version: Option<u64>,
    pub issues: Vec<String>,
    pub source_revision: Option<String>,
    pub audio_revision: Option<String>,
    pub sample_rate_hz: Option<u32>,
    pub channels: Option<u16>,
    pub frames: Option<u64>,
    pub output_frames: Option<u64>,
    pub start_frame: Option<u64>,
    pub end_frame: Option<u64>,
    pub subtitle_source: Option<SubtitleSource>,
    pub alignment_revision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterCompilationPreflight {
    pub schema: String,
    pub project_id: String,
    pub project_version: u64,
    pub title: String,
    pub fingerprint: String,
    pub options: ChapterCompilationOptions,
    pub ready: bool,
    pub issues: Vec<String>,
    pub chapters: Vec<ChapterCompilationPreview>,
    pub output: Option<ChapterCompilationOutput>,
}

/// Every output chapter pins the complete immutable source and audio record.
/// `conversion` describes export-only processing, never a replacement project take.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterCompilationMarker {
    pub chapter_id: String,
    pub title: String,
    pub project_id: String,
    pub project_version: u64,
    pub source: SourceRevision,
    pub audio: AudioRevision,
    pub source_file: String,
    pub start_frame: u64,
    pub end_frame: u64,
    pub start_ms: u64,
    pub end_ms: u64,
    pub conversion: Option<ChapterPcmConversion>,
    pub subtitle_track: Option<SubtitleTrack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterPcmConversion {
    pub method: String,
    pub input_sample_rate_hz: u32,
    pub input_channels: u16,
    pub input_frames: u64,
    pub output_sample_rate_hz: u32,
    pub output_channels: u16,
    pub output_frames: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterCompilationManifest {
    pub schema: String,
    pub project_id: String,
    pub project_version: u64,
    pub title: String,
    pub fingerprint: String,
    pub shared_settings_digest: String,
    pub options: ChapterCompilationOptions,
    pub output: ChapterCompilationOutput,
    pub wav: String,
    pub audio: ContentRef,
    pub srt: Option<String>,
    pub vtt: Option<String>,
    pub chapter_markers: String,
    pub ffmetadata: String,
    pub chapters: Vec<ChapterCompilationMarker>,
}

struct PreparedChapter {
    project: Project,
    manifest: Manifest,
    source: Option<SourceRevision>,
    audio: Option<AudioRevision>,
    subtitles: Option<SubtitleTrack>,
    preview: ChapterCompilationPreview,
}

impl Project {
    /// Inspect every selected chapter without changing acceptance or project state.
    /// An explicit selection is ordered exactly as supplied; an empty selection
    /// uses the active collection order. Issues never silently omit a chapter.
    pub fn chapter_compilation_preflight(
        &self,
        expected: u64,
        ids: &[String],
        options: &ChapterCompilationOptions,
    ) -> Result<ChapterCompilationPreflight> {
        let _parent_lock = self.lock()?;
        let manifest = self.read()?;
        check_version(&manifest, expected)?;
        let chapters = compilation_selection(&manifest, ids)?;
        let (projects, _child_locks) = self.compilation_children(&manifest, &chapters)?;
        let (preflight, _) = prepare(&manifest, &chapters, &projects, options)?;
        Ok(preflight)
    }

    /// Export exactly a reviewed preflight. Parent and child locks keep all pins
    /// stable until no-clobber publication; immutable input bytes are revalidated.
    pub fn export_chapter_compilation(
        &self,
        expected: u64,
        ids: &[String],
        options: &ChapterCompilationOptions,
        expected_fingerprint: &str,
        destination: &Path,
    ) -> Result<ChapterCompilationManifest> {
        let _parent_lock = self.lock()?;
        let parent = self.read()?;
        check_version(&parent, expected)?;
        let chapters = compilation_selection(&parent, ids)?;
        let (projects, _child_locks) = self.compilation_children(&parent, &chapters)?;
        let (preflight, prepared) = prepare(&parent, &chapters, &projects, options)?;
        if expected_fingerprint != preflight.fingerprint {
            return Err(Error::new("compilation_preflight_changed"));
        }
        if !preflight.ready {
            return Err(Error::new("compilation_not_ready"));
        }
        let output = preflight
            .output
            .ok_or_else(|| Error::new("compilation_not_ready"))?;
        let mut markers = Vec::with_capacity(prepared.len());
        let mut cues = Vec::new();
        for (index, chapter) in prepared.iter().enumerate() {
            let source = chapter
                .source
                .clone()
                .ok_or_else(|| Error::new("stale_source"))?;
            let audio = chapter
                .audio
                .clone()
                .ok_or_else(|| Error::new("chapter_audio_unaccepted"))?;
            let start = chapter
                .preview
                .start_frame
                .ok_or_else(|| Error::new("compilation_not_ready"))?;
            let end = chapter
                .preview
                .end_frame
                .ok_or_else(|| Error::new("compilation_not_ready"))?;
            if let Some(track) = &chapter.subtitles {
                cues.extend(offset_cues(track, start, end, output.sample_rate_hz)?);
            }
            let conversion = (audio.contract.sample_rate_hz != output.sample_rate_hz
                || audio.contract.channels != output.channels)
                .then(|| ChapterPcmConversion {
                    method: if audio.contract.sample_rate_hz == output.sample_rate_hz {
                        "pcm16_arithmetic_mean_downmix_round_nearest".into()
                    } else {
                        "pcm16_mean_downmix_65_tap_windowed_sinc_round_nearest".into()
                    },
                    input_sample_rate_hz: audio.contract.sample_rate_hz,
                    input_channels: audio.contract.channels,
                    input_frames: audio.contract.frame_count,
                    output_sample_rate_hz: output.sample_rate_hz,
                    output_channels: output.channels,
                    output_frames: end - start,
                });
            markers.push(ChapterCompilationMarker {
                chapter_id: chapter.preview.chapter_id.clone(),
                title: chapter.preview.title.clone(),
                project_id: chapter.manifest.id.clone(),
                project_version: chapter.manifest.version,
                source,
                audio,
                source_file: format!(
                    "sources/{:03}-{}.txt",
                    index + 1,
                    chapter.preview.chapter_id
                ),
                start_frame: start,
                end_frame: end,
                start_ms: frame_ms(start, output.sample_rate_hz),
                end_ms: frame_ms(end, output.sample_rate_hz),
                conversion,
                subtitle_track: chapter.subtitles.clone(),
            });
        }
        let mut result = None;
        publish_export(self, destination, |staging| {
            fs::create_dir(staging.join("sources"))?;
            let content = pcm::write_compilation(staging, &prepared, &output)?;
            let captions = options.subtitles == ChapterCompilationSubtitles::Required;
            let manifest = ChapterCompilationManifest {
                schema: "vox.chapter-compilation/1".into(),
                project_id: parent.id.clone(),
                project_version: parent.version,
                title: parent.title.clone(),
                fingerprint: preflight.fingerprint.clone(),
                shared_settings_digest: settings_digest(&parent.draft),
                options: options.clone(),
                output: output.clone(),
                wav: "compilation.wav".into(),
                audio: content,
                srt: captions.then(|| "compilation.srt".into()),
                vtt: captions.then(|| "compilation.vtt".into()),
                chapter_markers: "chapters.json".into(),
                ffmetadata: "chapters.ffmetadata".into(),
                chapters: markers.clone(),
            };
            for marker in &manifest.chapters {
                write_new(
                    &staging.join(&marker.source_file),
                    marker.source.document.text.as_bytes(),
                )?;
            }
            let chapter_json = serde_json::json!({
                "schema": "vox.chapter-markers/1",
                "sample_rate_hz": output.sample_rate_hz,
                "frames": output.frames,
                "chapters": manifest.chapters.iter().map(|chapter| serde_json::json!({
                    "chapter_id": chapter.chapter_id,
                    "title": chapter.title,
                    "start_frame": chapter.start_frame,
                    "end_frame": chapter.end_frame,
                    "start_ms": chapter.start_ms,
                    "end_ms": chapter.end_ms,
                    "source_revision": chapter.source.id,
                    "audio_revision": chapter.audio.id,
                })).collect::<Vec<_>>()
            });
            write_new(
                &staging.join("chapters.json"),
                &serde_json::to_vec_pretty(&chapter_json)?,
            )?;
            write_new(
                &staging.join("chapters.ffmetadata"),
                ffmetadata(&manifest).as_bytes(),
            )?;
            if captions {
                // Reuse the canonical literal-caption serializer. Source and
                // alignment pins remain per chapter in the export manifest.
                let first = prepared[0]
                    .subtitles
                    .as_ref()
                    .ok_or_else(|| Error::new("subtitles_unavailable"))?;
                let merged = SubtitleTrack {
                    cues: cues.clone(),
                    ..first.clone()
                };
                write_new(
                    &staging.join("compilation.srt"),
                    merged.serialize("srt")?.as_bytes(),
                )?;
                write_new(
                    &staging.join("compilation.vtt"),
                    merged.serialize("vtt")?.as_bytes(),
                )?;
            }
            write_new(
                &staging.join("manifest.json"),
                &serde_json::to_vec_pretty(&manifest)?,
            )?;
            fs::File::open(staging.join("sources"))?.sync_all()?;
            result = Some(manifest);
            Ok(())
        })?;
        result.ok_or_else(|| Error::new("file_error"))
    }

    fn compilation_children(
        &self,
        parent: &Manifest,
        chapters: &[Chapter],
    ) -> Result<(BTreeMap<String, Project>, Vec<fs::File>)> {
        let mut children = BTreeMap::new();
        for chapter in chapters {
            children.insert(
                chapter.id.clone(),
                self.chapter_project_for(parent, &chapter.id)?,
            );
        }
        let mut locks = Vec::with_capacity(children.len());
        // Deterministic lock order permits multiple read-only collections to use
        // this same contract without depending on a user's chapter order.
        for child in children.values() {
            locks.push(child.lock()?);
        }
        Ok((children, locks))
    }
}

fn compilation_selection(parent: &Manifest, ids: &[String]) -> Result<Vec<Chapter>> {
    let mut chapters = selected_chapters(parent, ids)?;
    if !ids.is_empty() {
        let order = ids
            .iter()
            .enumerate()
            .map(|(index, id)| (id, index))
            .collect::<BTreeMap<_, _>>();
        chapters.sort_by_key(|chapter| order[&chapter.id]);
    }
    Ok(chapters)
}

fn prepare(
    parent: &Manifest,
    chapters: &[Chapter],
    projects: &BTreeMap<String, Project>,
    options: &ChapterCompilationOptions,
) -> Result<(ChapterCompilationPreflight, Vec<PreparedChapter>)> {
    if options.gap_ms > MAX_CHAPTER_GAP_MS {
        return Err(Error::new("invalid_compilation_options"));
    }
    let mut preflight = ChapterCompilationPreflight {
        schema: "vox.chapter-compilation-preflight/1".into(),
        project_id: parent.id.clone(),
        project_version: parent.version,
        title: parent.title.clone(),
        fingerprint: String::new(),
        options: options.clone(),
        ready: false,
        issues: Vec::new(),
        chapters: Vec::with_capacity(chapters.len()),
        output: None,
    };
    let mut prepared = Vec::with_capacity(chapters.len());
    let mut input_bytes = 0_u64;
    for chapter in chapters {
        let project = projects[&chapter.id].clone();
        let manifest = project.read()?;
        let mut preview = ChapterCompilationPreview {
            chapter_id: chapter.id.clone(),
            title: chapter.title.clone(),
            project_id: chapter.project_id.clone(),
            project_version: Some(manifest.version),
            issues: Vec::new(),
            source_revision: None,
            audio_revision: None,
            sample_rate_hz: None,
            channels: None,
            frames: None,
            output_frames: None,
            start_frame: None,
            end_frame: None,
            subtitle_source: None,
            alignment_revision: None,
        };
        if let Err(error) = require_current_settings(parent, &manifest) {
            push_issue(&mut preview.issues, &error.code);
        }
        if let Err(error) = require_current_source(&manifest) {
            push_issue(&mut preview.issues, &error.code);
        }
        let source = manifest
            .sources
            .iter()
            .find(|source| Some(&source.id) == manifest.source_head.as_ref())
            .cloned();
        let audio = manifest
            .audio
            .iter()
            .find(|audio| Some(&audio.id) == manifest.audio_head.as_ref())
            .cloned();
        preview.source_revision = source.as_ref().map(|source| source.id.clone());
        let mut subtitles = None;
        if let Some(audio) = &audio {
            preview.audio_revision = Some(audio.id.clone());
            preview.sample_rate_hz = Some(audio.contract.sample_rate_hz);
            preview.channels = Some(audio.contract.channels);
            preview.frames = Some(audio.contract.frame_count);
            if source.as_ref().is_none_or(|source| {
                audio.source_revision != source.id
                    || audio.document_digest != source.document.digest()
            }) {
                push_issue(&mut preview.issues, "stale_source");
            }
            input_bytes = input_bytes.saturating_add(audio.content.byte_length);
            if input_bytes > MAX_COMPILATION_INPUT_BYTES {
                push_issue(&mut preview.issues, "compilation_input_too_large");
            } else {
                match project.object(&audio.content).and_then(|bytes| {
                    let (contract, _) = decode_pcm_s16le_wav(&bytes, audio.contract.origin)?;
                    if contract != audio.contract {
                        return Err(Error::new("corrupt_audio"));
                    }
                    if audio.segment_assembly.is_some() {
                        project.validate_segment_assembly(&manifest, audio)?;
                    }
                    Ok(())
                }) {
                    Ok(()) => {}
                    Err(error) => push_issue(&mut preview.issues, &error.code),
                }
            }
            if options.audio_mode == ChapterCompilationAudioMode::NormalizeMono24000
                && audio.contract.channels > 2
            {
                push_issue(&mut preview.issues, "compilation_conversion_channels");
            }
            if options.subtitles == ChapterCompilationSubtitles::Required
                && preview.issues.is_empty()
            {
                match project.subtitle_track(&audio.id) {
                    Ok(track) => {
                        preview.subtitle_source = Some(track.source);
                        preview.alignment_revision = track.alignment_revision.clone();
                        subtitles = Some(track);
                    }
                    Err(error) => push_issue(&mut preview.issues, &error.code),
                }
            }
        } else {
            push_issue(&mut preview.issues, "chapter_audio_unaccepted");
        }
        prepared.push(PreparedChapter {
            project,
            manifest,
            source,
            audio,
            subtitles,
            preview,
        });
    }
    if input_bytes > MAX_COMPILATION_INPUT_BYTES {
        push_issue(&mut preflight.issues, "compilation_input_too_large");
    }
    let format = if options.audio_mode == ChapterCompilationAudioMode::NormalizeMono24000 {
        Some((24_000, 1))
    } else {
        prepared.iter().find_map(|chapter| {
            chapter
                .audio
                .as_ref()
                .map(|audio| (audio.contract.sample_rate_hz, audio.contract.channels))
        })
    };
    if let Some((rate, channels)) = format {
        let gap_frames = rounded_ratio(options.gap_ms, u64::from(rate), 1000)?;
        let mut frames = 0_u64;
        let mut known = true;
        let mut count = 0;
        for (index, chapter) in prepared.iter_mut().enumerate() {
            let Some(audio) = &chapter.audio else {
                known = false;
                continue;
            };
            if !(8_000..=384_000).contains(&audio.contract.sample_rate_hz)
                || !(1..=32).contains(&audio.contract.channels)
            {
                push_issue(&mut chapter.preview.issues, "corrupt_audio");
                known = false;
                continue;
            }
            if options.audio_mode == ChapterCompilationAudioMode::Preserve
                && (audio.contract.sample_rate_hz, audio.contract.channels) != (rate, channels)
            {
                push_issue(&mut chapter.preview.issues, "compilation_format_mismatch");
                push_issue(&mut preflight.issues, "compilation_format_mismatch");
                known = false;
                continue;
            }
            let length = rounded_ratio(
                audio.contract.frame_count,
                u64::from(rate),
                u64::from(audio.contract.sample_rate_hz),
            )?;
            if length == 0 {
                push_issue(&mut chapter.preview.issues, "compilation_empty_audio");
            }
            if index > 0 {
                frames = frames
                    .checked_add(gap_frames)
                    .ok_or_else(|| Error::new("compilation_too_large"))?;
            }
            chapter.preview.output_frames = Some(length);
            chapter.preview.start_frame = Some(frames);
            frames = frames
                .checked_add(length)
                .ok_or_else(|| Error::new("compilation_too_large"))?;
            chapter.preview.end_frame = Some(frames);
            if let Some(track) = &chapter.subtitles {
                if let Err(error) = offset_cues(track, frames - length, frames, rate) {
                    push_issue(&mut chapter.preview.issues, &error.code);
                }
                count += track.cues.len();
            }
        }
        if known {
            let byte_length = frames
                .checked_mul(u64::from(channels) * 2)
                .and_then(|pcm| pcm.checked_add(44))
                .ok_or_else(|| Error::new("compilation_too_large"))?;
            if byte_length > MAX_COMPILATION_BYTES {
                push_issue(&mut preflight.issues, "compilation_too_large");
            }
            if frames > u64::from(rate) * MAX_COMPILATION_SECONDS {
                push_issue(&mut preflight.issues, "compilation_too_long");
            }
            preflight.output = Some(ChapterCompilationOutput {
                sample_rate_hz: rate,
                channels,
                frames,
                gap_frames,
                duration_ms: frame_ms(frames, rate),
                byte_length,
                subtitle_cues: count,
            });
        }
    }
    preflight.chapters = prepared
        .iter()
        .map(|chapter| chapter.preview.clone())
        .collect();
    preflight.ready = preflight.issues.is_empty()
        && preflight.output.is_some()
        && preflight
            .chapters
            .iter()
            .all(|chapter| chapter.issues.is_empty());
    // Include immutable records and the complete accepted caption selection. A
    // child may change independently without advancing the collection version.
    let pins = prepared
        .iter()
        .map(|chapter| {
            (
                &chapter.manifest.id,
                chapter.manifest.version,
                &chapter.source,
                &chapter.audio,
                &chapter.subtitles,
            )
        })
        .collect::<Vec<_>>();
    preflight.fingerprint = ContentDigest::from_bytes(&serde_json::to_vec(&(
        &preflight,
        settings_digest(&parent.draft),
        pins,
    ))?)
    .to_string();
    Ok((preflight, prepared))
}

fn push_issue(issues: &mut Vec<String>, issue: &str) {
    if !issues.iter().any(|existing| existing == issue) {
        issues.push(issue.into());
    }
}

fn rounded_ratio(value: u64, numerator: u64, denominator: u64) -> Result<u64> {
    if denominator == 0 {
        return Err(Error::new("corrupt_audio"));
    }
    value
        .checked_mul(numerator)
        .and_then(|value| value.checked_add(denominator / 2))
        .map(|value| value / denominator)
        .ok_or_else(|| Error::new("compilation_too_large"))
}

fn frame_ms(frames: u64, rate: u32) -> u64 {
    (frames * 1000 + u64::from(rate) / 2) / u64::from(rate)
}

fn offset_cues(track: &SubtitleTrack, start: u64, end: u64, rate: u32) -> Result<Vec<SubtitleCue>> {
    let offset_ms = frame_ms(start, rate);
    let end_ms = frame_ms(end, rate);
    track
        .cues
        .iter()
        .map(|cue| {
            let start_ms = offset_ms
                .checked_add(cue.start_ms)
                .ok_or_else(|| Error::new("compilation_too_large"))?
                .min(end_ms);
            let end_ms = offset_ms
                .checked_add(cue.end_ms)
                .ok_or_else(|| Error::new("compilation_too_large"))?
                .min(end_ms);
            if start_ms >= end_ms {
                return Err(Error::new("compilation_subtitle_resolution"));
            }
            Ok(SubtitleCue {
                start_ms,
                end_ms,
                text: cue.text.clone(),
            })
        })
        .collect()
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn ffmetadata(manifest: &ChapterCompilationManifest) -> String {
    fn escape(value: &str) -> String {
        value
            .replace('\\', "\\\\")
            .replace('=', "\\=")
            .replace(';', "\\;")
            .replace('#', "\\#")
            .replace('\n', "\\\n")
            .replace('\r', "")
    }
    let mut text = format!(";FFMETADATA1\ntitle={}\n", escape(&manifest.title));
    for chapter in &manifest.chapters {
        text.push_str(&format!(
            "[CHAPTER]\nTIMEBASE=1/{}\nSTART={}\nEND={}\ntitle={}\n",
            manifest.output.sample_rate_hz,
            chapter.start_frame,
            chapter.end_frame,
            escape(&chapter.title)
        ));
    }
    text
}
