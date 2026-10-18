//! Shape's narrow file transfer schema. Shape graphs and receipts remain opaque evidence.
use super::super::{Document, Error, Result};
use crate::{AudioValueContract, ContentRef, SpeechSynthesisOperation};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Transfer {
    pub schema: String,
    pub request_id: String,
    pub origin: Origin,
    pub accepted_text: Option<Material>,
    pub authoring_draft: Option<AuthoringDraft>,
    pub speech_draft: Option<SpeechDraft>,
    pub cue_materials: Vec<DraftMaterial>,
    pub accepted_audio: Option<AcceptedAudio>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Origin {
    pub producer: String,
    pub project_id: String,
    pub text_artifact_id: String,
    pub observed_text_head: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourcePin {
    pub artifact_id: String,
    pub revision_id: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Material {
    pub source: SourcePin,
    pub revision: Revision,
    pub content: ContentRef,
    pub contract: Option<Contract>,
    pub relative_path: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Revision {
    pub id: String,
    pub artifact_id: String,
    pub parents: Vec<String>,
    pub content: ContentRef,
    pub content_contract: Option<Contract>,
    pub transformation_id: String,
    pub created_at_unix_ms: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(
    tag = "kind",
    content = "contract",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub(super) enum Contract {
    TextDocument(TextFormat),
    AudioClip(AudioValueContract),
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum TextFormat {
    Plain,
    SpeechScript { revision: String },
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DraftMaterial {
    pub content: ContentRef,
    pub contract: Contract,
    pub relative_path: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthoringDraft {
    pub status: String,
    pub target_artifact_id: String,
    pub expected_head: Option<String>,
    pub graph_digest: String,
    pub graph: Value,
    pub format: TextFormat,
    pub settings: Value,
    pub text: DraftMaterial,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SpeechDraft {
    pub status: String,
    pub target_artifact_id: String,
    pub expected_head: Option<String>,
    pub graph_digest: String,
    pub draft_id: String,
    pub input: Option<SourcePin>,
    pub input_text: Option<Material>,
    pub graph: Value,
    pub settings: Value,
    pub operation: Option<SpeechSynthesisOperation>,
    pub issue: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AcceptedAudio {
    pub status: String,
    pub material: Material,
    pub input_text: Option<Material>,
    pub receipt: Option<Value>,
    pub transformation: Value,
    pub operation: Option<SpeechSynthesisOperation>,
}

/// Independent import evidence; the original transfer is an immutable exact-byte object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportReceipt {
    pub schema: String,
    pub request_id: String,
    pub transfer_digest: String,
    pub producer_project_id: String,
    pub original_manifest: ContentRef,
    pub materials: Vec<ContentRef>,
    pub drafts: Vec<PreservedDraft>,
    pub unsupported_documents: Vec<String>,
    pub issues: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreservedDraft {
    pub kind: String,
    pub status: String,
    pub source: Option<SourcePin>,
    pub document: Document,
}
pub(super) fn invalid() -> Error {
    Error::new("invalid_shape_transfer")
}
pub(super) fn uuid(value: &str) -> Result<()> {
    uuid::Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| invalid())
}
pub(super) fn pin(pin: &SourcePin) -> Result<()> {
    uuid(&pin.artifact_id)?;
    uuid(&pin.revision_id)
}
pub(super) fn keys(value: &Value, allowed: &[&str]) -> Result<()> {
    let object = value.as_object().ok_or_else(invalid)?;
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(invalid());
    }
    Ok(())
}
pub(super) fn content_keys(value: &Value) -> Result<()> {
    keys(value, &["digest", "media_type", "byte_length"])
}
pub(super) fn operation_keys(value: &Value) -> Result<()> {
    keys(
        value,
        &[
            "delivery",
            "script",
            "language",
            "voice",
            "speed_milli",
            "synthetic_disclosure_required",
        ],
    )?;
    keys(&value["voice"], &["kind", "selection"])?;
    if value["voice"]["kind"] == "preset" {
        keys(&value["voice"]["selection"], &["alias", "catalog_revision"])?;
    }
    Ok(())
}

pub(super) fn contract_keys(value: &Value) -> Result<()> {
    if value.is_null() {
        return Ok(());
    }
    keys(value, &["kind", "contract"])?;
    match value["kind"].as_str() {
        Some("text_document") => keys(&value["contract"], &["format", "revision"]),
        Some("audio_clip") => keys(
            &value["contract"],
            &[
                "contract_revision",
                "container",
                "sample_format",
                "sample_rate_hz",
                "channels",
                "frame_count",
                "origin",
            ],
        ),
        _ => Err(invalid()),
    }
}
