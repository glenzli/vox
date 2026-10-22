//! Described synthetic voices use the existing official unary speech contract.
use super::{
    job_provenance::{JobPolicyProfile, parse_job_snapshot, valid_job_id},
    sdk::{InferRuntimeSdk, SdkAdapterError, execution_failure},
};
use crate::{
    AudioOriginDisclosure, ContentDigest, ExternalExecutionProvenance, SpeechSynthesisControl,
    audio::parse_pcm_s16le_wav,
    project::{Error, Result},
};
use infer_runtime_client::{ExecutionMode, JobSnapshot, SpeechFormat, SpeechRequest};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Duration};

pub const VOICE_DESIGN_INTENT: &str = "speech.design_voice";
pub const VOICE_DESIGN_DEPLOYMENT: &str = "mlx_qwen3_tts_voice_design_1_7b";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoiceDesignSpec {
    /// Exact authored description submitted as Runtime instructions.
    pub description: String,
    /// Exact spoken sample and the transcript of an accepted synthetic reference.
    pub text: String,
    pub language: String,
}

impl VoiceDesignSpec {
    pub fn validate(&self) -> Result<()> {
        let valid_text = |text: &str, max| {
            !text.trim().is_empty()
                && text.len() <= max
                && !text
                    .chars()
                    .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
        };
        if !valid_text(&self.description, 2000)
            || !valid_text(&self.text, 16_384)
            || !super::speech::SPEECH_LANGUAGES.contains(&self.language.as_str())
        {
            return Err(Error::new("invalid_voice_design"));
        }
        Ok(())
    }

    fn request(&self) -> SpeechRequest {
        SpeechRequest {
            model: VOICE_DESIGN_INTENT.into(),
            input: self.text.clone(),
            voice: None,
            instructions: Some(self.description.clone()),
            language: Some(self.language.clone()),
            speed: 1.0,
            response_format: SpeechFormat::Wav,
            execution_mode: ExecutionMode::Unary,
            metadata: BTreeMap::from([
                ("infer.capability_floor".into(), "foundational".into()),
                ("infer.policy".into(), "local-first".into()),
                ("infer.priority".into(), "interactive".into()),
                ("infer.latency".into(), "interactive".into()),
                ("infer.placement".into(), "local_only".into()),
                ("infer.prefer".into(), "local".into()),
                ("infer.offline_required".into(), "true".into()),
                ("infer.fallback".into(), "none".into()),
                ("infer.max_cost_usd".into(), "0".into()),
                (
                    "infer.deployment_ids".into(),
                    VOICE_DESIGN_DEPLOYMENT.into(),
                ),
            ]),
        }
    }
}

/// A payload-free Runtime Job plus its independently checked Vox policy and exact artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoiceDesignReceipt {
    pub spec: VoiceDesignSpec,
    pub job_id: String,
    pub job: JobSnapshot,
    pub provenance: ExternalExecutionProvenance,
    pub output_digest: ContentDigest,
}

impl VoiceDesignReceipt {
    pub(crate) fn validate_metadata(&self) -> Result<()> {
        self.spec.validate()?;
        if !valid_job_id(&self.job_id) || serde_json::to_vec(&self.job)?.len() > 65_536 {
            return Err(Error::new("invalid_provenance"));
        }
        let checked = parse_job_snapshot(
            &self.job_id,
            VOICE_DESIGN_INTENT,
            JobPolicyProfile::VoiceDesign,
            self.job.clone(),
        )
        .map_err(|_| Error::new("invalid_provenance"))?;
        if !self.provenance.is_bounded() || checked != self.provenance {
            return Err(Error::new("invalid_provenance"));
        }
        Ok(())
    }

    pub fn validate(&self, bytes: &[u8]) -> Result<()> {
        self.validate_metadata()?;
        let contract = parse_pcm_s16le_wav(bytes, AudioOriginDisclosure::SyntheticSpeech)?;
        if self.output_digest != ContentDigest::from_bytes(bytes)
            || bytes.len() > 25 * 1024 * 1024
            || contract.sample_rate_hz != 24_000
            || contract.channels != 1
            || !(500..=120_000).contains(&contract.duration_millis())
        {
            return Err(Error::new("invalid_voice_design_audio"));
        }
        Ok(())
    }
}

pub fn generate(
    endpoint: &str,
    spec: VoiceDesignSpec,
    control: &SpeechSynthesisControl,
) -> Result<(Vec<u8>, VoiceDesignReceipt)> {
    spec.validate()?;
    if control.is_cancelled() {
        return Err(Error::new("generation_cancelled"));
    }
    let sdk = super::official_sdk(endpoint, super::credential_path()).map_err(map_error)?;
    generate_using(&sdk, spec, control)
}

fn generate_using(
    sdk: &dyn InferRuntimeSdk,
    spec: VoiceDesignSpec,
    control: &SpeechSynthesisControl,
) -> Result<(Vec<u8>, VoiceDesignReceipt)> {
    spec.validate()?;
    if control.is_cancelled() {
        return Err(Error::new("generation_cancelled"));
    }
    let response = sdk
        .synthesize_speech(&spec.request(), Duration::from_secs(600))
        .map_err(map_error)?;
    if response.content_type.split(';').next().map(str::trim) != Some("audio/wav")
        || response.logical_model != VOICE_DESIGN_INTENT
        || !valid_job_id(&response.job_id)
    {
        return Err(Error::new("infer_invalid_response"));
    }
    let job = sdk.job(&response.job_id).map_err(map_error)?;
    let provenance = parse_job_snapshot(
        &response.job_id,
        VOICE_DESIGN_INTENT,
        JobPolicyProfile::VoiceDesign,
        job.clone(),
    )
    .map_err(|error| Error::new(error.code()))?;
    let receipt = VoiceDesignReceipt {
        spec,
        job_id: response.job_id,
        job,
        provenance,
        output_digest: ContentDigest::from_bytes(&response.bytes),
    };
    receipt.validate(&response.bytes)?;
    // Cancellation after completion stops the next variation, retaining this result for review.
    Ok((response.bytes, receipt))
}

fn map_error(error: SdkAdapterError) -> Error {
    Error::new(execution_failure(error).0)
}

#[cfg(test)]
pub(crate) mod tests;
