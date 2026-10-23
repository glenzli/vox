//! Exact-transcript forced alignment through the official local Runtime contract.
use super::{
    job_provenance::{JobPolicyProfile, parse_job_snapshot, valid_job_id},
    sdk::{InferRuntimeSdk, SdkAdapterError, execution_failure},
};
use crate::{
    AudioOriginDisclosure, ContentDigest, ExternalExecutionProvenance, SpeechSynthesisControl,
    audio::parse_pcm_s16le_wav,
    project::{Error, Result},
};
use infer_runtime_client::{AlignmentResponse, JobSnapshot};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs::OpenOptions, io::Write, path::PathBuf, time::Duration};
use unicode_categories::UnicodeCategories;

pub const ALIGNMENT_INTENT: &str = "audio.align";
pub const ALIGNMENT_DEPLOYMENT: &str = "mlx_qwen3_forced_aligner_0_6b";
pub const MAX_ALIGNMENT_RESPONSE_BYTES: usize = 1024 * 1024;
pub const MAX_ALIGNMENT_TRANSCRIPT_BYTES: usize = 65_536;
const MAX_ALIGNMENT_ITEMS: usize = 8192;
const MAX_ALIGNMENT_AUDIO_BYTES: usize = 128 * 1024 * 1024;
const MAX_ALIGNMENT_DURATION_SECONDS: f64 = 1800.0;

/// The caller explicitly supplies and confirms the words actually present in
/// the selected audio. A source-derived suggestion is not an editing receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AlignmentSpec {
    pub transcript: String,
    pub language: String,
}

impl AlignmentSpec {
    pub fn validate(&self) -> Result<()> {
        if self.transcript.len() > MAX_ALIGNMENT_TRANSCRIPT_BYTES
            || !self
                .transcript
                .chars()
                .any(|ch| ch.is_letter() || ch.is_number())
            || self.transcript.chars().any(invalid_control)
            || self.language == "auto"
            || !super::speech::SPEECH_LANGUAGES.contains(&self.language.as_str())
        {
            return Err(Error::new("invalid_alignment_spec"));
        }
        Ok(())
    }

    fn metadata() -> BTreeMap<String, String> {
        BTreeMap::from([
            ("infer.capability_floor".into(), "foundational".into()),
            ("infer.policy".into(), "local-first".into()),
            ("infer.priority".into(), "interactive".into()),
            ("infer.latency".into(), "interactive".into()),
            ("infer.placement".into(), "local_only".into()),
            ("infer.prefer".into(), "local".into()),
            ("infer.offline_required".into(), "true".into()),
            ("infer.fallback".into(), "none".into()),
            ("infer.max_cost_usd".into(), "0".into()),
            ("infer.deployment_ids".into(), ALIGNMENT_DEPLOYMENT.into()),
        ])
    }
}

/// Core Jobs are payload-free. The hashes bind the exact consumed input and
/// retained canonical JSON, while the Job proves the Runtime identity/policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AlignmentReceipt {
    pub spec: AlignmentSpec,
    pub audio_digest: ContentDigest,
    pub job_id: String,
    pub job: JobSnapshot,
    pub provenance: ExternalExecutionProvenance,
    pub output_digest: ContentDigest,
}

impl AlignmentReceipt {
    pub(crate) fn validate_metadata(&self) -> Result<()> {
        self.spec.validate()?;
        if !valid_job_id(&self.job_id) || serde_json::to_vec(&self.job)?.len() > 65_536 {
            return Err(Error::new("invalid_provenance"));
        }
        let checked = parse_job_snapshot(
            &self.job_id,
            ALIGNMENT_INTENT,
            JobPolicyProfile::Alignment,
            self.job.clone(),
        )
        .map_err(|_| Error::new("invalid_provenance"))?;
        if !self.provenance.is_bounded() || self.provenance != checked {
            return Err(Error::new("invalid_provenance"));
        }
        Ok(())
    }

    pub(crate) fn validate(&self, response: &[u8], audio: &[u8]) -> Result<AlignmentResponse> {
        self.validate_metadata()?;
        let duration = audio_duration(audio)?;
        if response.len() > MAX_ALIGNMENT_RESPONSE_BYTES
            || self.audio_digest != ContentDigest::from_bytes(audio)
            || self.output_digest != ContentDigest::from_bytes(response)
        {
            return Err(Error::new("invalid_alignment"));
        }
        let response: AlignmentResponse =
            serde_json::from_slice(response).map_err(|_| Error::new("invalid_alignment"))?;
        if response.extra.get("id").and_then(serde_json::Value::as_str)
            != Some(self.job_id.as_str())
        {
            return Err(Error::new("invalid_alignment"));
        }
        aligned_tokens(&response, &self.spec, duration)?;
        Ok(response)
    }
}

pub fn generate(
    endpoint: &str,
    audio: &[u8],
    spec: AlignmentSpec,
    control: &SpeechSynthesisControl,
) -> Result<(Vec<u8>, AlignmentReceipt)> {
    spec.validate()?;
    audio_duration(audio)?;
    if control.is_cancelled() {
        return Err(Error::new("alignment_cancelled"));
    }
    let sdk = super::official_sdk(endpoint, super::credential_path()).map_err(map_error)?;
    generate_using(&sdk, audio, spec, control)
}

fn generate_using(
    sdk: &dyn InferRuntimeSdk,
    audio: &[u8],
    spec: AlignmentSpec,
    control: &SpeechSynthesisControl,
) -> Result<(Vec<u8>, AlignmentReceipt)> {
    spec.validate()?;
    let duration = audio_duration(audio)?;
    if control.is_cancelled() {
        return Err(Error::new("alignment_cancelled"));
    }
    let input = AlignmentInput::new(audio)?;
    // A stop during this bounded unary call is observed when it returns. It
    // cannot assert provider interruption; a validated result remains reviewable.
    if control.is_cancelled() {
        return Err(Error::new("alignment_cancelled"));
    }
    let response = sdk
        .align_file(
            &input.path.join("source.wav"),
            &spec.transcript,
            &spec.language,
            &AlignmentSpec::metadata(),
            Duration::from_secs(600),
        )
        .map_err(map_error)?;
    aligned_tokens(&response, &spec, duration)?;
    let job_id = response
        .extra
        .get("id")
        .and_then(serde_json::Value::as_str)
        .filter(|id| valid_job_id(id))
        .ok_or_else(|| Error::new("infer_invalid_response"))?
        .to_owned();
    let job = sdk.job(&job_id).map_err(map_error)?;
    let provenance = parse_job_snapshot(
        &job_id,
        ALIGNMENT_INTENT,
        JobPolicyProfile::Alignment,
        job.clone(),
    )
    .map_err(|error| Error::new(error.code()))?;
    let bytes = serde_json::to_vec(&response)?;
    if bytes.len() > MAX_ALIGNMENT_RESPONSE_BYTES {
        return Err(Error::new("infer_invalid_response"));
    }
    let receipt = AlignmentReceipt {
        spec,
        audio_digest: ContentDigest::from_bytes(audio),
        job_id,
        job,
        provenance,
        output_digest: ContentDigest::from_bytes(&bytes),
    };
    receipt.validate(&bytes, audio)?;
    Ok((bytes, receipt))
}

fn map_error(error: SdkAdapterError) -> Error {
    Error::new(execution_failure(error).0)
}

pub(crate) fn audio_duration(audio: &[u8]) -> Result<f64> {
    if audio.len() > MAX_ALIGNMENT_AUDIO_BYTES {
        return Err(Error::new("alignment_audio_too_large"));
    }
    let contract = parse_pcm_s16le_wav(audio, AudioOriginDisclosure::ImportedUnverified)?;
    let duration = contract.frame_count as f64 / f64::from(contract.sample_rate_hz);
    if duration <= 0.0 || duration > MAX_ALIGNMENT_DURATION_SECONDS {
        return Err(Error::new("alignment_audio_too_large"));
    }
    Ok(duration)
}

#[derive(Debug)]
pub(crate) struct AlignedToken {
    pub start_ms: u64,
    pub end_ms: u64,
    pub start_byte: usize,
    pub end_byte: usize,
}

/// Match every provider token to the confirmed transcript in order. The
/// provider's documented L/N/apostrophe projection removes punctuation and
/// combining marks; ranges restore those exact original characters for captions.
pub(crate) fn aligned_tokens(
    response: &AlignmentResponse,
    spec: &AlignmentSpec,
    duration: f64,
) -> Result<Vec<AlignedToken>> {
    spec.validate()?;
    if response.text != spec.transcript
        || response.language != spec.language
        || response
            .extra
            .get("model")
            .and_then(serde_json::Value::as_str)
            != Some(ALIGNMENT_INTENT)
        || response
            .extra
            .get("id")
            .and_then(serde_json::Value::as_str)
            .is_none_or(|id| !valid_job_id(id))
        || response.items.is_empty()
        || response.items.len() > MAX_ALIGNMENT_ITEMS
        || serde_json::to_vec(response)?.len() > MAX_ALIGNMENT_RESPONSE_BYTES
    {
        return Err(Error::new("invalid_alignment"));
    }
    let characters = spec
        .transcript
        .char_indices()
        .filter(|(_, ch)| kept_character(*ch))
        .collect::<Vec<_>>();
    let mut consumed = 0;
    let mut previous_end = 0.0;
    let mut result = Vec::with_capacity(response.items.len());
    for item in &response.items {
        if item.text.is_empty()
            || item.text.len() > 4096
            || item.text.chars().any(invalid_control)
            || !item.start.is_finite()
            || !item.end.is_finite()
            || item.start < previous_end
            || item.end < item.start
            || item.end > duration
        {
            return Err(Error::new("invalid_alignment"));
        }
        let token = item
            .text
            .chars()
            .filter(|ch| kept_character(*ch))
            .collect::<Vec<_>>();
        let next = consumed + token.len();
        if token.is_empty()
            || characters
                .get(consumed..next)
                .is_none_or(|slice| !slice.iter().map(|(_, ch)| *ch).eq(token.iter().copied()))
        {
            return Err(Error::new("alignment_transcript_mismatch"));
        }
        let start_byte = if consumed == 0 {
            0
        } else {
            characters[consumed].0
        };
        let end_byte = characters
            .get(next)
            .map_or(spec.transcript.len(), |(offset, _)| *offset);
        result.push(AlignedToken {
            start_ms: (item.start * 1000.0).round() as u64,
            end_ms: (item.end * 1000.0).round() as u64,
            start_byte,
            end_byte,
        });
        previous_end = item.end;
        consumed = next;
    }
    if consumed != characters.len() || result.iter().all(|token| token.start_ms == token.end_ms) {
        return Err(Error::new("alignment_transcript_mismatch"));
    }
    Ok(result)
}

fn kept_character(ch: char) -> bool {
    ch == '\'' || ch.is_letter() || ch.is_number()
}

fn invalid_control(ch: char) -> bool {
    ch.is_control() && !matches!(ch, '\r' | '\n' | '\t')
}

struct AlignmentInput {
    path: PathBuf,
}
impl AlignmentInput {
    fn new(bytes: &[u8]) -> Result<Self> {
        let input = Self {
            path: std::env::temp_dir().join(format!("vox-align-{}", uuid::Uuid::now_v7())),
        };
        crate::project::store::private_directory(&input.path)?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(input.path.join("source.wav"))?;
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(input)
    }
}
impl Drop for AlignmentInput {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::audio::SpeechWaveAssembly;
    use infer_runtime_client::{AlignmentItem, AudioBytesResponse, SpeechRequest};
    use serde_json::json;
    use std::{path::Path, sync::Mutex};

    pub(crate) fn wav(frames: usize) -> Vec<u8> {
        let mut wave = SpeechWaveAssembly::default();
        wave.push_pcm(24_000, 1, &vec![0; frames * 2]).unwrap();
        wave.finish().unwrap()
    }

    pub(crate) fn spec() -> AlignmentSpec {
        AlignmentSpec {
            transcript: "  Hello, 中文 café e\u{301}!\n".into(),
            language: "English".into(),
        }
    }

    pub(crate) fn speech_receipt() -> (String, ExternalExecutionProvenance) {
        let job = super::super::job_provenance::tests::speech_job();
        let provenance = parse_job_snapshot(
            &job.id,
            "speech.synthesize",
            JobPolicyProfile::Speech,
            job.clone(),
        )
        .unwrap();
        (job.id, provenance)
    }

    pub(crate) fn fixture(audio: &[u8], spec: AlignmentSpec) -> (Vec<u8>, AlignmentReceipt) {
        let mut job = super::super::job_provenance::tests::speech_job();
        job.id = format!("vox_align_{}", uuid::Uuid::now_v7());
        job.intent = ALIGNMENT_INTENT.into();
        job.capability_contract = Some("infer.audio.alignment@20260811.1".into());
        job.deployment = ALIGNMENT_DEPLOYMENT.into();
        job.constraints["capability_floor"] = json!("foundational");
        job.constraints["named_route"]["ordered_ids"] = json!([ALIGNMENT_DEPLOYMENT]);
        job.routing.capability_floor = "foundational".into();
        job.routing.named_route.as_mut().unwrap().ordered_ids = vec![ALIGNMENT_DEPLOYMENT.into()];
        job.routing.candidates[0].deployment = ALIGNMENT_DEPLOYMENT.into();
        job.attempts[0].deployment = ALIGNMENT_DEPLOYMENT.into();
        let provenance = parse_job_snapshot(
            &job.id,
            ALIGNMENT_INTENT,
            JobPolicyProfile::Alignment,
            job.clone(),
        )
        .unwrap();
        let response = AlignmentResponse {
            text: spec.transcript.clone(),
            language: spec.language.clone(),
            items: vec![AlignmentItem {
                text: spec
                    .transcript
                    .chars()
                    .filter(|ch| kept_character(*ch))
                    .collect(),
                start: 0.01,
                end: 0.5,
            }],
            extra: BTreeMap::from([
                ("id".into(), json!(job.id)),
                ("model".into(), json!(ALIGNMENT_INTENT)),
            ]),
        };
        let bytes = serde_json::to_vec(&response).unwrap();
        let receipt = AlignmentReceipt {
            spec,
            audio_digest: ContentDigest::from_bytes(audio),
            job_id: job.id.clone(),
            job,
            provenance,
            output_digest: ContentDigest::from_bytes(&bytes),
        };
        receipt.validate(&bytes, audio).unwrap();
        (bytes, receipt)
    }

    struct AlignmentSdk {
        response: AlignmentResponse,
        job: JobSnapshot,
        seen: Mutex<Option<PathBuf>>,
        expected_audio: Vec<u8>,
        expected_spec: AlignmentSpec,
    }
    impl InferRuntimeSdk for AlignmentSdk {
        fn synthesize_speech(
            &self,
            _: &SpeechRequest,
            _: Duration,
        ) -> std::result::Result<AudioBytesResponse, SdkAdapterError> {
            panic!("alignment never synthesizes speech")
        }
        fn job(&self, id: &str) -> std::result::Result<JobSnapshot, SdkAdapterError> {
            assert_eq!(id, self.job.id);
            Ok(self.job.clone())
        }
        fn align_file(
            &self,
            path: &Path,
            transcript: &str,
            language: &str,
            metadata: &BTreeMap<String, String>,
            timeout: Duration,
        ) -> std::result::Result<AlignmentResponse, SdkAdapterError> {
            assert_eq!(std::fs::read(path).unwrap(), self.expected_audio);
            assert_eq!(transcript, self.expected_spec.transcript);
            assert_eq!(language, self.expected_spec.language);
            assert_eq!(metadata, &AlignmentSpec::metadata());
            assert_eq!(timeout, Duration::from_secs(600));
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                assert_eq!(
                    std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
                    0o600
                );
                assert_eq!(
                    std::fs::metadata(path.parent().unwrap())
                        .unwrap()
                        .permissions()
                        .mode()
                        & 0o777,
                    0o700
                );
            }
            *self.seen.lock().unwrap() = Some(path.into());
            Ok(self.response.clone())
        }
    }

    #[test]
    fn exact_audio_transcript_policy_and_private_temporary_input_use_official_boundary() {
        let audio = wav(48_000);
        let spec = spec();
        let (response, receipt) = fixture(&audio, spec.clone());
        let sdk = AlignmentSdk {
            response: serde_json::from_slice(&response).unwrap(),
            job: receipt.job,
            seen: Mutex::new(None),
            expected_audio: audio.clone(),
            expected_spec: spec.clone(),
        };
        let (bytes, receipt) = generate_using(
            &sdk,
            &audio,
            spec.clone(),
            &SpeechSynthesisControl::default(),
        )
        .unwrap();
        assert_eq!(bytes, response);
        assert_eq!(receipt.spec, spec);
        assert_eq!(receipt.audio_digest, ContentDigest::from_bytes(&audio));
        assert!(!sdk.seen.lock().unwrap().as_ref().unwrap().exists());
    }

    #[test]
    fn explicit_language_and_nonempty_bounded_transcript_are_required_before_discovery() {
        let audio = wav(48_000);
        let control = SpeechSynthesisControl::default();
        for spec in [
            AlignmentSpec {
                transcript: " ".into(),
                ..spec()
            },
            AlignmentSpec {
                transcript: "!!!".into(),
                ..spec()
            },
            AlignmentSpec {
                transcript: "x".repeat(65_537),
                ..spec()
            },
            AlignmentSpec {
                transcript: "bad\0".into(),
                ..spec()
            },
            AlignmentSpec {
                language: "auto".into(),
                ..spec()
            },
            AlignmentSpec {
                language: "zh".into(),
                ..spec()
            },
        ] {
            assert_eq!(
                generate("invalid endpoint", &audio, spec, &control)
                    .unwrap_err()
                    .code,
                "invalid_alignment_spec"
            );
        }
        control.cancel();
        assert_eq!(
            generate("invalid endpoint", &audio, spec(), &control)
                .unwrap_err()
                .code,
            "alignment_cancelled"
        );
    }

    #[test]
    fn timestamps_and_response_bounds_fail_closed() {
        let audio = wav(48_000);
        let spec = spec();
        let (bytes, _) = fixture(&audio, spec.clone());
        let valid: AlignmentResponse = serde_json::from_slice(&bytes).unwrap();
        for (start, end) in [
            (f64::NAN, 0.5),
            (0.0, f64::INFINITY),
            (-0.1, 0.5),
            (0.6, 0.5),
            (0.0, 2.001),
            (0.0, 0.0),
        ] {
            let mut response = valid.clone();
            response.items[0].start = start;
            response.items[0].end = end;
            assert!(aligned_tokens(&response, &spec, 2.0).is_err());
        }
        let mut response = valid.clone();
        response.extra.insert(
            "future".into(),
            json!("x".repeat(MAX_ALIGNMENT_RESPONSE_BYTES)),
        );
        assert_eq!(
            aligned_tokens(&response, &spec, 2.0).unwrap_err().code,
            "invalid_alignment"
        );
        let mut response = valid;
        response.items = vec![response.items[0].clone(); MAX_ALIGNMENT_ITEMS + 1];
        assert!(aligned_tokens(&response, &spec, 2.0).is_err());
    }

    #[test]
    fn complete_unicode_token_order_restores_punctuation_without_inventing_words() {
        let audio = wav(48_000);
        let spec = AlignmentSpec {
            transcript: "“你好”， café e\u{301}!".into(),
            language: "Chinese".into(),
        };
        let (bytes, _) = fixture(&audio, spec.clone());
        let mut response: AlignmentResponse = serde_json::from_slice(&bytes).unwrap();
        response.items = vec![
            AlignmentItem {
                text: "你".into(),
                start: 0.0,
                end: 0.1,
            },
            AlignmentItem {
                text: "好".into(),
                start: 0.1,
                end: 0.2,
            },
            AlignmentItem {
                text: "café".into(),
                start: 0.3,
                end: 0.7,
            },
            AlignmentItem {
                text: "e".into(),
                start: 0.7,
                end: 0.9,
            },
        ];
        let tokens = aligned_tokens(&response, &spec, 2.0).unwrap();
        assert_eq!(
            tokens
                .iter()
                .map(|token| &spec.transcript[token.start_byte..token.end_byte])
                .collect::<String>(),
            spec.transcript
        );
        let mut missing = response.clone();
        missing.items.pop();
        assert_eq!(
            aligned_tokens(&missing, &spec, 2.0).unwrap_err().code,
            "alignment_transcript_mismatch"
        );
        let mut substituted = response.clone();
        substituted.items[2].text = "cafe".into();
        assert!(aligned_tokens(&substituted, &spec, 2.0).is_err());
        response.items[2].start = 0.1;
        assert!(aligned_tokens(&response, &spec, 2.0).is_err());
    }

    #[test]
    fn receipt_rechecks_app_route_job_input_and_consumed_output_hashes() {
        let audio = wav(48_000);
        let (bytes, receipt) = fixture(&audio, spec());
        for field in [
            "app_id",
            "intent",
            "deployment",
            "physical_model",
            "job_id",
            "fallback",
        ] {
            let mut invalid = receipt.clone();
            match field {
                "app_id" => invalid.job.app_id = "shape".into(),
                "intent" => invalid.job.intent = "speech.synthesize".into(),
                "deployment" => invalid.job.deployment = "other".into(),
                "physical_model" => invalid.provenance.physical_model = "other".into(),
                "fallback" => invalid.job.constraints["fallback"] = json!("allow"),
                _ => invalid.job_id = "other".into(),
            }
            assert_eq!(
                invalid.validate(&bytes, &audio).unwrap_err().code,
                "invalid_provenance"
            );
        }
        assert!(receipt.validate(&bytes, &wav(24_000)).is_err());
        assert!(receipt.validate(b"{}", &audio).is_err());
        let mut changed: AlignmentResponse = serde_json::from_slice(&bytes).unwrap();
        changed.text = "Different".into();
        let changed = serde_json::to_vec(&changed).unwrap();
        let mut invalid = receipt;
        invalid.output_digest = ContentDigest::from_bytes(&changed);
        assert!(invalid.validate(&changed, &audio).is_err());
    }
}
