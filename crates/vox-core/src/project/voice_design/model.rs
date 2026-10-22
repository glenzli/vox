use super::super::{Error, Manifest, Result};
use crate::{
    AudioOriginDisclosure, AudioValueContract, ContentRef,
    infer_runtime::voice_design::{VoiceDesignReceipt, VoiceDesignSpec},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoiceDesignCandidate {
    pub id: String,
    pub run_id: String,
    pub variation: u8,
    pub variations: u8,
    pub original: ContentRef,
    pub contract: AudioValueContract,
    pub generation: VoiceDesignReceipt,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_reference: Option<String>,
}

/// A synthetic-source acceptance record. It makes no claim about a real person's consent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptedVoiceDesign {
    pub candidate_id: String,
    pub run_id: String,
    pub variation: u8,
    pub variations: u8,
    pub authorization_basis: String,
    pub generation: VoiceDesignReceipt,
}

impl AcceptedVoiceDesign {
    pub(super) fn from_candidate(candidate: &VoiceDesignCandidate) -> Self {
        Self {
            candidate_id: candidate.id.clone(),
            run_id: candidate.run_id.clone(),
            variation: candidate.variation,
            variations: candidate.variations,
            authorization_basis: "accepted_synthetic_design".into(),
            generation: candidate.generation.clone(),
        }
    }

    pub(crate) fn validate(&self, voice: &super::super::materials::ReferenceVoice) -> Result<()> {
        self.generation.validate_metadata()?;
        if self.authorization_basis != "accepted_synthetic_design"
            || !valid_uuid(&self.candidate_id)
            || !valid_uuid(&self.run_id)
            || !(1..=4).contains(&self.variations)
            || !(1..=self.variations).contains(&self.variation)
            || voice.origin != "designed"
            || voice.contract.origin != AudioOriginDisclosure::SyntheticSpeech
            || voice.reference.consent_receipt_id
                != format!("synthetic-design-acceptance:{}", self.candidate_id)
            || voice.reference.transcript.as_ref() != Some(&self.generation.spec.text)
            || voice.reference.audio.as_ref().is_none_or(|audio| {
                audio.digest != self.generation.output_digest || audio.media_type != "audio/wav"
            })
        {
            return Err(Error::new("invalid_voice_design"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoiceDesignRun {
    pub id: String,
    pub spec: VoiceDesignSpec,
    pub requested: u8,
    pub completed: u8,
    /// Includes the identities of explicitly discarded candidates as historical receipts.
    pub candidate_ids: Vec<String>,
    pub status: String,
    pub error: Option<String>,
    /// A crash while set leaves an unknown request outcome. It is never replayed automatically.
    pub active_variation: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceDesignState {
    Completed,
    Cancelled,
    PartialFailed,
    Failed,
}

#[derive(Debug, Serialize)]
pub struct VoiceDesignOutcome {
    pub manifest: Manifest,
    pub run_id: String,
    pub requested: u8,
    pub completed: u8,
    pub state: VoiceDesignState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub(super) fn valid_uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok()
}

pub(crate) fn validate_records(manifest: &Manifest) -> Result<()> {
    if manifest.voice_design_candidates.len() > 128
        || manifest.voice_design_runs.len() > 256
        || manifest
            .voice_design_runs
            .iter()
            .map(|run| &run.id)
            .collect::<BTreeSet<_>>()
            .len()
            != manifest.voice_design_runs.len()
        || manifest
            .voice_design_candidates
            .iter()
            .map(|candidate| &candidate.id)
            .collect::<BTreeSet<_>>()
            .len()
            != manifest.voice_design_candidates.len()
    {
        return Err(Error::new("invalid_voice_design"));
    }
    for run in &manifest.voice_design_runs {
        run.spec.validate()?;
        if !valid_uuid(&run.id)
            || !(1..=4).contains(&run.requested)
            || run.completed > run.requested
            || run.candidate_ids.len() != usize::from(run.completed)
            || run.candidate_ids.iter().any(|id| !valid_uuid(id))
            || run.candidate_ids.iter().collect::<BTreeSet<_>>().len() != run.candidate_ids.len()
            || !matches!(
                run.status.as_str(),
                "running"
                    | "completed"
                    | "cancelled"
                    | "partial_failed"
                    | "failed"
                    | "interrupted_unknown"
            )
            || (run.status == "completed" && run.completed != run.requested)
            || run
                .error
                .as_ref()
                .is_some_and(|error| error.len() > 256 || !error.is_ascii())
            || run.active_variation.is_some_and(|variation| {
                !matches!(run.status.as_str(), "running" | "interrupted_unknown")
                    || variation != run.completed + 1
                    || variation > run.requested
            })
        {
            return Err(Error::new("invalid_voice_design"));
        }
    }
    for candidate in &manifest.voice_design_candidates {
        candidate.generation.validate_metadata()?;
        let run = manifest
            .voice_design_runs
            .iter()
            .find(|run| run.id == candidate.run_id)
            .ok_or_else(|| Error::new("invalid_voice_design"))?;
        if !valid_uuid(&candidate.id)
            || candidate.variations != run.requested
            || !(1..=run.completed).contains(&candidate.variation)
            || run.candidate_ids.get(usize::from(candidate.variation - 1)) != Some(&candidate.id)
            || candidate.generation.spec != run.spec
            || candidate.original.media_type != "audio/wav"
            || candidate.original.digest != candidate.generation.output_digest
            || candidate.original.byte_length > 25 * 1024 * 1024
            || candidate.contract.origin != AudioOriginDisclosure::SyntheticSpeech
            || candidate.contract.sample_rate_hz != 24_000
            || candidate.contract.channels != 1
            || !(500..=120_000).contains(&candidate.contract.duration_millis())
        {
            return Err(Error::new("invalid_voice_design"));
        }
        if let Some(id) = &candidate.accepted_reference {
            let voice = manifest
                .voice_library
                .iter()
                .find(|voice| &voice.id == id)
                .ok_or_else(|| Error::new("invalid_voice_design"))?;
            let design = voice
                .design
                .as_ref()
                .ok_or_else(|| Error::new("invalid_voice_design"))?;
            design.validate(voice)?;
            if design.candidate_id != candidate.id
                || design.run_id != candidate.run_id
                || design.variation != candidate.variation
                || design.variations != candidate.variations
                || voice.reference.audio.as_ref() != Some(&candidate.original)
                || voice.contract != candidate.contract
                || serde_json::to_value(&design.generation)?
                    != serde_json::to_value(&candidate.generation)?
            {
                return Err(Error::new("invalid_voice_design"));
            }
        }
    }
    for voice in &manifest.voice_library {
        if let Some(design) = &voice.design
            && !manifest.voice_design_candidates.iter().any(|candidate| {
                candidate.id == design.candidate_id
                    && candidate.accepted_reference.as_ref() == Some(&voice.id)
            })
        {
            return Err(Error::new("invalid_voice_design"));
        }
    }
    Ok(())
}
