use super::*;
use crate::{audio::SpeechWaveAssembly, infer_runtime::test_support::FakeSdk};
use infer_runtime_client::AudioBytesResponse;
use serde_json::json;

pub(crate) fn spec() -> VoiceDesignSpec {
    VoiceDesignSpec {
        description: "  A calm, warm adult narrator. 清晰、自然。\n".into(),
        text: "  The garden is quiet. 花园很安静。\n".into(),
        language: "auto".into(),
    }
}

pub(crate) fn fixture(spec: VoiceDesignSpec) -> (Vec<u8>, VoiceDesignReceipt) {
    let mut wave = SpeechWaveAssembly::default();
    wave.push_pcm(24_000, 1, &vec![0; 48_000]).unwrap();
    let bytes = wave.finish().unwrap();
    let mut job = super::super::job_provenance::tests::speech_job();
    job.id = format!("vox_design_{}", uuid::Uuid::now_v7());
    job.intent = VOICE_DESIGN_INTENT.into();
    job.deployment = VOICE_DESIGN_DEPLOYMENT.into();
    job.capability_level = "foundational".into();
    job.constraints["capability_floor"] = json!("foundational");
    job.constraints["named_route"]["ordered_ids"] = json!([VOICE_DESIGN_DEPLOYMENT]);
    job.routing.capability_floor = "foundational".into();
    job.routing.named_route.as_mut().unwrap().ordered_ids = vec![VOICE_DESIGN_DEPLOYMENT.into()];
    job.routing.candidates[0].deployment = VOICE_DESIGN_DEPLOYMENT.into();
    job.attempts[0].deployment = VOICE_DESIGN_DEPLOYMENT.into();
    let provenance = parse_job_snapshot(
        &job.id,
        VOICE_DESIGN_INTENT,
        JobPolicyProfile::VoiceDesign,
        job.clone(),
    )
    .unwrap();
    let receipt = VoiceDesignReceipt {
        spec,
        job_id: job.id.clone(),
        job,
        provenance,
        output_digest: ContentDigest::from_bytes(&bytes),
    };
    receipt.validate(&bytes).unwrap();
    (bytes, receipt)
}

#[test]
fn official_speech_request_keeps_exact_description_sample_language_and_local_policy() {
    let spec = spec();
    let (bytes, receipt) = fixture(spec.clone());
    let sdk = FakeSdk::new()
        .speech(AudioBytesResponse {
            bytes: bytes.clone(),
            content_type: "audio/wav; charset=binary".into(),
            job_id: receipt.job_id.clone(),
            logical_model: VOICE_DESIGN_INTENT.into(),
        })
        .job(receipt.job);
    let (_, generated) =
        generate_using(&sdk, spec.clone(), &SpeechSynthesisControl::default()).unwrap();
    assert_eq!(generated.spec, spec);
    assert_eq!(generated.output_digest, ContentDigest::from_bytes(&bytes));
    let seen = sdk.seen_speeches.lock().unwrap();
    let request = &seen[0];
    assert_eq!(request.model, VOICE_DESIGN_INTENT);
    assert_eq!(request.input, spec.text);
    assert_eq!(request.instructions.as_ref(), Some(&spec.description));
    assert_eq!(request.language.as_ref(), Some(&spec.language));
    assert_eq!(request.voice, None);
    assert_eq!(request.speed, 1.0);
    assert_eq!(request.response_format, SpeechFormat::Wav);
    assert_eq!(request.execution_mode, ExecutionMode::Unary);
    assert_eq!(
        request.metadata["infer.deployment_ids"],
        VOICE_DESIGN_DEPLOYMENT
    );
    assert_eq!(request.metadata["infer.offline_required"], "true");
    assert_eq!(request.metadata["infer.fallback"], "none");
    assert_eq!(request.metadata["infer.max_cost_usd"], "0");
}

#[test]
fn cancellation_before_dispatch_never_discovers_or_loads_a_credential() {
    let control = SpeechSynthesisControl::default();
    control.cancel();
    assert_eq!(
        generate("invalid endpoint", spec(), &control)
            .unwrap_err()
            .code,
        "generation_cancelled"
    );
}

#[test]
fn design_requires_description_sample_and_supported_explicit_language() {
    let valid = spec();
    for invalid in [
        VoiceDesignSpec {
            description: " ".into(),
            ..valid.clone()
        },
        VoiceDesignSpec {
            description: "x".repeat(2001),
            ..valid.clone()
        },
        VoiceDesignSpec {
            text: " ".into(),
            ..valid.clone()
        },
        VoiceDesignSpec {
            text: "x".repeat(16_385),
            ..valid.clone()
        },
        VoiceDesignSpec {
            text: "words\0".into(),
            ..valid.clone()
        },
        VoiceDesignSpec {
            language: "invented-language".into(),
            ..valid
        },
    ] {
        assert_eq!(invalid.validate().unwrap_err().code, "invalid_voice_design");
    }
}

#[test]
fn receipt_rechecks_full_job_identity_policy_and_physical_evidence() {
    let (bytes, receipt) = fixture(spec());
    for field in ["app_id", "intent", "deployment", "physical_model", "job_id"] {
        let mut invalid = receipt.clone();
        match field {
            "app_id" => invalid.job.app_id = "shape".into(),
            "intent" => invalid.job.intent = "speech.synthesize".into(),
            "deployment" => invalid.job.deployment = "remote-model".into(),
            "physical_model" => invalid.provenance.physical_model = "other-model".into(),
            _ => invalid.job_id = "unknown-job".into(),
        }
        assert_eq!(
            invalid.validate(&bytes).unwrap_err().code,
            "invalid_provenance"
        );
    }
    for (field, value) in [
        ("fallback", json!("allow")),
        ("offline_required", json!(false)),
        ("max_cost_usd", json!(1.0)),
        ("placement", json!("remote_only")),
    ] {
        let mut invalid = receipt.clone();
        invalid.job.constraints[field] = value;
        assert_eq!(
            invalid.validate(&bytes).unwrap_err().code,
            "invalid_provenance"
        );
    }
    let mut invalid = receipt.clone();
    invalid.job.attempts[0].trigger = "fallback".into();
    assert!(invalid.validate(&bytes).is_err());
    let mut invalid = receipt;
    invalid.output_digest = ContentDigest::from_bytes(b"different artifact");
    assert_eq!(
        invalid.validate(&bytes).unwrap_err().code,
        "invalid_voice_design_audio"
    );
}

#[test]
fn unrelated_audio_or_route_response_never_becomes_a_design_receipt() {
    let (bytes, receipt) = fixture(spec());
    let sdk = FakeSdk::new().speech(AudioBytesResponse {
        bytes,
        content_type: "audio/wav".into(),
        job_id: receipt.job_id,
        logical_model: "speech.synthesize".into(),
    });
    assert_eq!(
        generate_using(&sdk, spec(), &SpeechSynthesisControl::default())
            .unwrap_err()
            .code,
        "infer_invalid_response"
    );
}
