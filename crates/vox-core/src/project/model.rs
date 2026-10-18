use super::{Error, Result};
use crate::{
    AudioValueContract, ContentRef, ExternalExecutionProvenance,
    INFER_SPEECH_VOICE_CATALOG_REVISION, PresetVoiceAlias, PresetVoiceSelection,
    SpeechSynthesisOperation, SpeechVoiceSelection,
    speech_script::{SpeechScriptOptions, parse_speech_script_for_review},
};
use serde::{Deserialize, Serialize};

pub const PROJECT_SCHEMA: &str = "vox.project/1";
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptFormat {
    Plain,
    ProductionScript,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub text: String,
    pub format: ScriptFormat,
    pub voice: String,
    pub language: String,
    pub speed_milli: u16,
    pub options: SpeechScriptOptions,
}
impl Default for Document {
    fn default() -> Self {
        Self {
            text: String::new(),
            format: ScriptFormat::Plain,
            voice: crate::SPEECH_PRESETS[0].alias.into(),
            language: "auto".into(),
            speed_milli: 1000,
            options: SpeechScriptOptions::default(),
        }
    }
}
impl Document {
    pub fn operation(&self) -> Result<SpeechSynthesisOperation> {
        if self.text.len() > 64 * 1024 {
            return Err(Error::new("script_too_large"));
        }
        let voice = PresetVoiceSelection::new(
            PresetVoiceAlias::new(self.voice.clone()).map_err(|_| Error::new("invalid_voice"))?,
            INFER_SPEECH_VOICE_CATALOG_REVISION,
        )
        .map_err(|_| Error::new("invalid_voice"))?;
        let mut op = SpeechSynthesisOperation::new(
            self.language.clone(),
            SpeechVoiceSelection::Preset(voice),
            self.speed_milli,
            true,
        )
        .map_err(|_| Error::new("invalid_settings"))?;
        if self.format == ScriptFormat::ProductionScript {
            op.script = Some(self.options.clone());
        }
        if !crate::supported_speech_operation(&op) || !self.options.is_valid() {
            return Err(Error::new("invalid_settings"));
        }
        Ok(op)
    }
    pub fn digest(&self) -> String {
        blake3::hash(&serde_json::to_vec(self).expect("document serializes"))
            .to_hex()
            .to_string()
    }
    pub fn analysis(&self) -> serde_json::Value {
        if self.format == ScriptFormat::Plain {
            return serde_json::json!({"ready": !self.text.trim().is_empty() && self.operation().is_ok(),"roles":[],"cues":[],"issues":[],"spoken_passages":usize::from(!self.text.trim().is_empty())});
        }
        let (plan, _spoken_lines) = parse_speech_script_for_review(&self.text);
        serde_json::json!({"ready":plan.ready(&self.options)&&self.operation().is_ok(),"roles":plan.roles.keys().collect::<Vec<_>>(),"cues":plan.cues.keys().collect::<Vec<_>>(),"issues":plan.issues,"readiness":plan.readiness(&self.options),"spoken_passages":plan.readiness(&self.options).spoken_passages,"events":plan.events,"pause_ms":plan.pause_millis(),"shared_voices":plan.shared_voices(&self.options)})
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRevision {
    pub id: String,
    pub document: Document,
    pub source_digest: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioRevision {
    pub id: String,
    pub source_revision: String,
    pub document_digest: String,
    pub content: ContentRef,
    pub contract: AudioValueContract,
    pub provenance: Option<ExternalExecutionProvenance>,
    pub origin: String,
    pub parent_audio: Option<String>,
    pub source_project: Option<String>,
    #[serde(default)]
    pub edit: Option<crate::handoff::EditReceipt>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderRun {
    pub id: String,
    pub source_revision: String,
    pub status: String,
    pub error: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub schema: String,
    pub id: String,
    pub version: u64,
    pub title: String,
    pub draft: Document,
    pub sources: Vec<SourceRevision>,
    pub source_head: Option<String>,
    pub candidates: Vec<AudioRevision>,
    pub audio: Vec<AudioRevision>,
    pub audio_head: Option<String>,
    pub runs: Vec<RenderRun>,
    #[serde(default)]
    pub handoffs: Vec<crate::handoff::Ticket>,
}
