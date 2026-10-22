use super::super::{Error, Manifest, Result};
use crate::{
    AudioValueContract, ContentRef, ExternalExecutionProvenance, SpeechSynthesisOperation,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentRevision {
    pub id: String,
    pub source_revision: String,
    pub document_digest: String,
    pub key: String,
    pub text: String,
    pub spoken_text: String,
    pub operation: SpeechSynthesisOperation,
    pub content: ContentRef,
    pub contract: AudioValueContract,
    pub provenance: ExternalExecutionProvenance,
    pub job_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentPlan {
    pub source_revision: Option<String>,
    pub document_digest: String,
    pub adopted: bool,
    pub ready: bool,
    pub all_accepted: bool,
    pub can_assemble: bool,
    pub issues: serde_json::Value,
    pub segments: Vec<SegmentPreview>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentPreview {
    pub key: String,
    pub index: usize,
    pub line: u32,
    pub role: Option<String>,
    pub language: String,
    pub text: String,
    pub spoken_text: String,
    pub candidates: Vec<SegmentCandidatePreview>,
    pub accepted_id: Option<String>,
    pub duration_millis: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentCandidatePreview {
    pub id: String,
    pub duration_millis: u64,
    pub accepted: bool,
    pub source_revision: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentAssembly {
    pub schema: String,
    pub document_digest: String,
    pub selections: Vec<SegmentSelection>,
    pub timings: Vec<SegmentTiming>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentSelection {
    pub key: String,
    pub segment_id: String,
}
/// Speech timings are measured from exact assembled PCM frames, including each
/// repeat occurrence. These are segment captions, without word alignment claims.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentTiming {
    pub segment_id: String,
    pub key: String,
    pub text: String,
    pub start_frame: u64,
    pub end_frame: u64,
}

pub(crate) fn validate_records(m: &Manifest) -> Result<()> {
    if m.segment_candidates.len() + m.accepted_segments.len() > 4096
        || m.segment_selections.len() > 4096
    {
        return Err(Error::new("invalid_project"));
    }
    let mut ids = BTreeSet::new();
    for r in m.segment_candidates.iter().chain(&m.accepted_segments) {
        if !ids.insert(&r.id)
            || uuid::Uuid::parse_str(&r.id).is_err()
            || r.key.parse::<crate::ContentDigest>().is_err()
            || r.document_digest.parse::<crate::ContentDigest>().is_err()
            || r.text.trim().is_empty()
            || r.text.len() > 65_536
            || r.spoken_text.trim().is_empty()
            || r.spoken_text.len() > 16_384
            || r.operation.script.is_some()
            || r.operation.validate().is_err()
            || r.content.media_type != "audio/wav"
            || r.content.byte_length > 128 * 1024 * 1024
            || r.contract.sample_rate_hz != 24_000
            || r.contract.channels != 1
            || r.contract.frame_count == 0
            || r.contract.origin != crate::AudioOriginDisclosure::SyntheticSpeech
            || r.job_id.is_empty()
            || r.job_id.len() > 256
            || !r.job_id.is_ascii()
            || r.provenance.app_id != "vox"
            || !r.provenance.is_bounded()
            || !m
                .sources
                .iter()
                .any(|s| s.id == r.source_revision && s.document.digest() == r.document_digest)
        {
            return Err(Error::new("invalid_segment"));
        }
    }
    if m.segment_selections.iter().any(|(key, id)| {
        !m.accepted_segments
            .iter()
            .any(|r| r.key == *key && r.id == *id)
    }) {
        return Err(Error::new("invalid_segment"));
    }
    Ok(())
}
