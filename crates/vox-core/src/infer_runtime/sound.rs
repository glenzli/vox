//! Official SDK sound generation with exact Vox policy and retained prompt/job evidence.
use super::job_provenance::{JobPolicyProfile, parse_job_snapshot};
use crate::{
    AudioOriginDisclosure, ContentDigest, ExternalExecutionProvenance,
    audio::parse_pcm_s16le_wav,
    project::{Error, Result},
};
use infer_runtime_client::{Client, PreparedSoundPrompt, SoundGenerationRequest, SoundModelChoice};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Duration};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SoundSpec {
    pub prompt: String,
    pub seconds: u8,
    pub seed: u32,
    #[serde(default)]
    pub music: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundReceipt {
    pub spec: SoundSpec,
    pub prepared: PreparedSoundPrompt,
    pub provenance: ExternalExecutionProvenance,
    pub output_digest: ContentDigest,
}
impl SoundSpec {
    fn deployment(&self) -> &'static str {
        if self.music {
            "stable_audio_3_sm_music_mlx"
        } else {
            "stable_audio_3_sm_sfx_mlx"
        }
    }
}
impl SoundReceipt {
    pub fn validate(&self, bytes: &[u8]) -> Result<()> {
        self.prepared
            .validate_for_generation(&self.spec.prompt, "vox")
            .map_err(|_| Error::new("prompt_preparation_failed"))?;
        let c = parse_pcm_s16le_wav(bytes, AudioOriginDisclosure::SyntheticSound)?;
        let p = &self.provenance;
        if ContentDigest::from_bytes(bytes) != self.output_digest
            || c.sample_rate_hz != 44_100
            || c.channels != 2
            || c.frame_count != u64::from(self.spec.seconds) * 44_100
            || !p.is_bounded()
            || p.app_id != "vox"
            || p.intent != "audio.generate_sound"
            || p.deployment != self.spec.deployment()
            || p.capability_contract.as_deref() != Some("infer.audio.sound-generation@20260926.2")
            || p.placement != "local"
            || !p.offline_required
            || p.fallback != "none"
            || p.max_cost_microusd != 0
        {
            return Err(Error::new("invalid_provenance"));
        }
        Ok(())
    }
}
pub fn generate(
    endpoint: &str,
    spec: SoundSpec,
    control: &crate::SpeechSynthesisControl,
) -> Result<(Vec<u8>, SoundReceipt)> {
    if spec.prompt.trim().is_empty()
        || spec.prompt.len() > 2000
        || !(1..=30).contains(&spec.seconds)
    {
        return Err(Error::new("invalid_sound_request"));
    }
    if control.is_cancelled() {
        return Err(Error::new("generation_cancelled"));
    }
    let client = Client::with_discovery(
        super::resolver(endpoint).map_err(|_| Error::new("infer_invalid_endpoint"))?,
    )
    .credential_file(super::credential_path())
    .build()
    .map_err(|_| Error::new("credential_unavailable"))?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| Error::new("infer_unavailable"))?;
    runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(600), async {
            let prepared = client
                .prepare_sound_prompt(&spec.prompt)
                .await
                .map_err(map_error)?;
            prepared
                .validate_for_generation(&spec.prompt, "vox")
                .map_err(|_| Error::new("prompt_preparation_failed"))?;
            if control.is_cancelled() {
                return Err(Error::new("generation_cancelled"));
            }
            let request = SoundGenerationRequest {
                model: "audio.generate_sound".into(),
                model_choice: Some(if spec.music {
                    SoundModelChoice::SmallMusic
                } else {
                    SoundModelChoice::SmallSfx
                }),
                prompt: prepared.effective_prompt.clone(),
                duration_seconds: spec.seconds,
                seed: Some(spec.seed),
                metadata: BTreeMap::from([
                    ("infer.capability_floor".into(), "foundational".into()),
                    ("infer.policy".into(), "local-first".into()),
                    ("infer.priority".into(), "interactive".into()),
                    ("infer.latency".into(), "balanced".into()),
                    ("infer.placement".into(), "local_only".into()),
                    ("infer.prefer".into(), "local".into()),
                    ("infer.offline_required".into(), "true".into()),
                    ("infer.fallback".into(), "none".into()),
                    ("infer.max_cost_usd".into(), "0".into()),
                ]),
            };
            let response = client
                .generate_sound_effect(&request)
                .await
                .map_err(map_error)?;
            let job = client.job(&response.job_id).await.map_err(map_error)?;
            if response.logical_model != "audio.generate_sound"
                || response.seed != spec.seed
                || response.duration_seconds != spec.seconds
                || response.deployment != job.deployment
                || response.physical_model != job.physical_model
                || response.model_build != job.model_build
                || response.provider != job.provider
            {
                return Err(Error::new("infer_invalid_response"));
            }
            let provenance = parse_job_snapshot(
                &response.job_id,
                "audio.generate_sound",
                JobPolicyProfile::Sound(spec.deployment()),
                job,
            )
            .map_err(|e| Error::new(e.code()))?;
            let receipt = SoundReceipt {
                spec,
                prepared,
                provenance,
                output_digest: ContentDigest::from_bytes(&response.wav),
            };
            receipt.validate(&response.wav)?;
            if control.is_cancelled() {
                return Err(Error::new("generation_cancelled"));
            }
            Ok((response.wav, receipt))
        })
        .await
        .map_err(|_| Error::new("infer_unavailable"))?
    })
}
fn map_error(error: infer_runtime_client::Error) -> Error {
    let (code, _) = super::sdk::execution_failure(super::sdk::SdkAdapterError::Sdk(error));
    Error::new(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_before_dispatch_does_not_contact_runtime() {
        let control = crate::SpeechSynthesisControl::default();
        control.cancel();
        let spec = SoundSpec {
            prompt: "A bell".into(),
            seconds: 2,
            seed: 42,
            music: false,
        };
        assert_eq!(
            generate("invalid endpoint", spec, &control)
                .unwrap_err()
                .code,
            "generation_cancelled"
        );
    }
}
