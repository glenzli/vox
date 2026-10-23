use crate::{
    ContentRef,
    project::{Error, Manifest, Result, SegmentPlan},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_CHAPTERS: usize = 256;
pub const MAX_CHAPTER_BYTES: usize = 65_536;
pub const MAX_IMPORT_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChapterOwner {
    pub project_id: String,
    pub chapter_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChapterImport {
    pub original_name: String,
    pub original: ContentRef,
    pub start_byte: u64,
    pub end_byte: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    pub project_id: String,
    #[serde(default)]
    pub archived: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import: Option<ChapterImport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterState {
    pub chapter: Chapter,
    pub project: Manifest,
    pub analysis: serde_json::Value,
    pub segment_plan: SegmentPlan,
    pub shared_settings_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterSummary {
    pub id: String,
    pub title: String,
    pub project_id: String,
    pub archived: bool,
    pub version: u64,
    pub status: String,
    pub ready: bool,
    pub shared_settings_current: bool,
    pub source_revision: Option<String>,
    pub document_digest: String,
    pub total_segments: usize,
    pub completed_segments: usize,
    pub candidate_ids: Vec<String>,
    pub audio_id: Option<String>,
    pub interrupted_unknown: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterBatchRun {
    pub id: String,
    pub status: String,
    pub settings_digest: String,
    pub chapters: Vec<ChapterBatchItem>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterBatchItem {
    pub chapter_id: String,
    pub source_revision: Option<String>,
    pub document_digest: Option<String>,
    pub status: String,
    pub total_segments: usize,
    pub completed_segments: usize,
    pub reused_segments: usize,
    pub audio_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterBatchProgress {
    pub batch_id: String,
    pub chapter_id: String,
    pub chapter_index: usize,
    pub total_chapters: usize,
    pub completed_segments: usize,
    pub total_segments: usize,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterBatchOutcome {
    pub manifest: Manifest,
    pub batch: ChapterBatchRun,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterExportManifest {
    pub schema: String,
    pub project_id: String,
    pub title: String,
    pub chapters: Vec<ChapterExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterExport {
    pub chapter_id: String,
    pub title: String,
    pub project_id: String,
    pub source_revision: String,
    pub document_digest: String,
    pub audio_revision: String,
    pub audio: ContentRef,
    pub document: crate::project::Document,
    pub origin: String,
    pub provenance: Option<crate::ExternalExecutionProvenance>,
    pub edit: Option<crate::handoff::EditReceipt>,
    pub parent_audio: Option<String>,
    pub wav: String,
    pub srt: Option<String>,
    pub vtt: Option<String>,
    pub subtitles_unavailable_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle_source: Option<crate::project::SubtitleSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alignment_revision: Option<String>,
    pub sample_rate_hz: u32,
    pub frames: u64,
}

pub(super) fn valid_title(title: &str) -> bool {
    !title.trim().is_empty() && title.len() <= 512 && !title.chars().any(char::is_control)
}

fn valid_uuid(id: &str) -> bool {
    uuid::Uuid::parse_str(id).is_ok()
        && id.len() == 36
        && id.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-')
}

pub(crate) fn validate_records(m: &Manifest) -> Result<()> {
    if m.chapters.len() > MAX_CHAPTERS
        || m.chapter_batches.len() > 1024
        || (m.chapter_owner.is_some() && (!m.chapters.is_empty() || !m.chapter_batches.is_empty()))
        || m.chapter_owner.as_ref().is_some_and(|owner| {
            !valid_uuid(&owner.project_id)
                || !valid_uuid(&owner.chapter_id)
                || owner.project_id == m.id
        })
    {
        return Err(Error::new("invalid_chapters"));
    }
    let mut ids = BTreeSet::new();
    let mut project_ids = BTreeSet::new();
    for chapter in &m.chapters {
        if !valid_uuid(&chapter.id)
            || !valid_uuid(&chapter.project_id)
            || chapter.project_id == m.id
            || !ids.insert(chapter.id.as_str())
            || !project_ids.insert(chapter.project_id.as_str())
            || !valid_title(&chapter.title)
            || chapter.import.as_ref().is_some_and(|import| {
                import.original_name.is_empty()
                    || import.original_name.len() > 512
                    || import.original_name.chars().any(char::is_control)
                    || import.original.byte_length > MAX_IMPORT_BYTES
                    || import.original.media_type != "text/plain; charset=utf-8"
                    || import.start_byte >= import.end_byte
                    || import.end_byte > import.original.byte_length
                    || import.end_byte - import.start_byte > MAX_CHAPTER_BYTES as u64
            })
        {
            return Err(Error::new("invalid_chapters"));
        }
    }
    let mut batches = BTreeSet::new();
    for batch in &m.chapter_batches {
        let mut items = BTreeSet::new();
        if !valid_uuid(&batch.id)
            || !batches.insert(batch.id.as_str())
            || batch
                .settings_digest
                .parse::<crate::ContentDigest>()
                .is_err()
            || batch.chapters.is_empty()
            || batch.chapters.len() > MAX_CHAPTERS
            || !matches!(
                batch.status.as_str(),
                "running" | "completed" | "failed" | "cancelled" | "interrupted_unknown"
            )
            || batch.chapters.iter().any(|item| {
                !ids.contains(item.chapter_id.as_str())
                    || !items.insert(item.chapter_id.as_str())
                    || item.total_segments > 512
                    || item.completed_segments > item.total_segments
                    || item.reused_segments > item.completed_segments
                    || !matches!(
                        item.status.as_str(),
                        "pending"
                            | "running"
                            | "completed"
                            | "skipped"
                            | "failed"
                            | "cancelled"
                            | "interrupted_unknown"
                    )
                    || item
                        .source_revision
                        .as_deref()
                        .is_some_and(|id| !valid_uuid(id))
                    || item
                        .document_digest
                        .as_ref()
                        .is_some_and(|digest| digest.parse::<crate::ContentDigest>().is_err())
                    || item.audio_id.as_deref().is_some_and(|id| !valid_uuid(id))
                    || item.error.as_ref().is_some_and(|error| error.len() > 256)
            })
            || batch.error.as_ref().is_some_and(|error| error.len() > 256)
        {
            return Err(Error::new("invalid_chapter_batch"));
        }
    }
    Ok(())
}
