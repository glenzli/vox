use super::{Error, Result};
use crate::{
    AudioValueContract, ContentRef, ExternalExecutionProvenance,
    INFER_SPEECH_VOICE_CATALOG_REVISION, PresetVoiceAlias, PresetVoiceSelection,
    SpeechSynthesisOperation, SpeechVoiceSelection,
    speech_script::{SpeechScriptOptions, parse_speech_script_for_review, reading_preview},
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice_catalog_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivery: Option<crate::speech_script::SpeechDelivery>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_voice: Option<crate::AuthorizedVoiceReference>,
    pub language: String,
    pub speed_milli: u16,
    pub options: SpeechScriptOptions,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pronunciation_rules: Vec<crate::PronunciationRule>,
}
impl Default for Document {
    fn default() -> Self {
        Self {
            text: String::new(),
            format: ScriptFormat::Plain,
            voice: crate::SPEECH_PRESETS[0].alias.into(),
            voice_catalog_revision: None,
            delivery: None,
            reference_voice: None,
            language: "auto".into(),
            speed_milli: 1000,
            options: SpeechScriptOptions::default(),
            pronunciation_rules: Vec::new(),
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
            self.voice_catalog_revision
                .as_deref()
                .unwrap_or(INFER_SPEECH_VOICE_CATALOG_REVISION),
        )
        .map_err(|_| Error::new("invalid_voice"))?;
        let mut op = SpeechSynthesisOperation::new(
            self.language.clone(),
            self.reference_voice.clone().map_or(
                SpeechVoiceSelection::Preset(voice),
                SpeechVoiceSelection::AuthorizedReference,
            ),
            self.speed_milli,
            true,
        )
        .map_err(|_| Error::new("invalid_settings"))?;
        op.delivery = self.delivery;
        op.pronunciation_rules = self.pronunciation_rules.clone();
        op.validate()
            .map_err(|_| Error::new("invalid_pronunciation"))?;
        if self.format == ScriptFormat::ProductionScript {
            op.script = Some(self.options.clone());
        }
        if (self.reference_voice.is_some()
            || (self.format == ScriptFormat::ProductionScript
                && !self.options.reference_roles.is_empty()))
            && self.speed_milli != 1000
        {
            return Err(Error::new("clone_controls_unsupported"));
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
        let (mut plan, note_lines) = parse_speech_script_for_review(&self.text);
        for event in &plan.events {
            if let crate::speech_script::SpeechScriptEventKind::Speech { role, delivery, .. } =
                &event.kind
            {
                let reference = self.options.reference_roles.contains_key(role)
                    || (!self.options.roles.contains_key(role) && self.reference_voice.is_some());
                if reference && *delivery != crate::speech_script::SpeechDelivery::Neutral {
                    plan.issues.push(crate::speech_script::SpeechScriptIssue {
                        line: event.line,
                        code: "clone_controls_unsupported".into(),
                        text: String::new(),
                    });
                }
            }
        }
        serde_json::json!({"ready":plan.ready(&self.options)&&self.operation().is_ok(),"roles":plan.roles.keys().collect::<Vec<_>>(),"cues":plan.cues.keys().collect::<Vec<_>>(),"issues":plan.issues,"readiness":plan.readiness(&self.options),"spoken_passages":plan.readiness(&self.options).spoken_passages,"events":plan.events,"reading":reading_preview(&self.text,&plan,&note_lines),"pause_ms":plan.pause_millis(),"shared_voices":plan.shared_voices(&self.options)})
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segment_assembly: Option<super::SegmentAssembly>,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub segment_candidates: Vec<super::SegmentRevision>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accepted_segments: Vec<super::SegmentRevision>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub segment_selections: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub handoffs: Vec<crate::handoff::Ticket>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub voice_library: Vec<super::materials::ReferenceVoice>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sound_library: Vec<super::materials::SoundMaterial>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub voice_design_candidates: Vec<super::VoiceDesignCandidate>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub voice_design_runs: Vec<super::VoiceDesignRun>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chapters: Vec<super::Chapter>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chapter_batches: Vec<super::ChapterBatchRun>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chapter_owner: Option<super::ChapterOwner>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape_import: Option<super::shape_transfer::ImportReceipt>,
}
