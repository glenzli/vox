use super::*;
use crate::audio::SpeechWaveAssembly;
use infer_runtime_client::{PreparedSoundPrompt, SOUND_PROMPT_RULES_REVISION};
use serde_json::json;

fn project() -> Project {
    Project::create(
        &std::env::temp_dir().join(format!("vox-sound-batch-{}.vox", uuid::Uuid::now_v7())),
        "Sound work",
    )
    .unwrap()
}
fn spec() -> SoundSpec {
    SoundSpec {
        prompt: "  Sparse piano and rain.  ".into(),
        seconds: 1,
        seed: u32::MAX,
        music: false,
    }
}
fn fixture(spec: SoundSpec) -> (Vec<u8>, SoundReceipt) {
    let mut wave = SpeechWaveAssembly::default();
    wave.push_pcm(44_100, 2, &vec![0; usize::from(spec.seconds) * 44_100 * 4])
        .unwrap();
    let bytes = wave.finish().unwrap();
    let deployment = if spec.music {
        "stable_audio_3_sm_music_mlx"
    } else {
        "stable_audio_3_sm_sfx_mlx"
    };
    let provenance = serde_json::from_value(json!({
        "contract_revision":"infer.consumer-core@20260823.1",
        "capability_contract":"infer.audio.sound-generation@20260926.2",
        "app_id":"vox", "intent":"audio.generate_sound", "provider":"fixture",
        "deployment":deployment, "model_profile":"fixture", "model_build":"fixture",
        "physical_model":"fixture", "placement":"local", "capability_level":"foundational",
        "evaluation_status":"provisional", "resource_class":"standard", "policy":"local-first",
        "priority":"interactive", "requested_policy":"local-first", "requested_priority":"interactive",
        "requested_provider_access_class":null, "requested_placement":"local_only",
        "requested_preference":"local", "offline_required":true, "requested_latency":"balanced",
        "fallback":"none", "requested_deadline_ms":null, "max_cost_microusd":0,
        "capability_floor":"foundational", "routing_candidates":[{
            "provider":"fixture", "deployment":deployment, "status":"eligible", "rank":1, "reason_codes":[]
        }], "attempts":[{
            "number":1, "provider":"fixture", "deployment":deployment, "outcome":"succeeded",
            "trigger":"initial", "error_kind":null
        }]
    })).unwrap();
    let receipt = SoundReceipt {
        prepared: PreparedSoundPrompt {
            original_prompt: spec.prompt.clone(),
            effective_prompt: spec.prompt.clone(),
            rules_revision: SOUND_PROMPT_RULES_REVISION.into(),
            text_job: None,
            preparation_elapsed_ms: 0,
        },
        spec,
        provenance,
        output_digest: crate::ContentDigest::from_bytes(&bytes),
    };
    receipt.validate(&bytes).unwrap();
    (bytes, receipt)
}

#[test]
fn categories_constrain_only_short_instrumental_music_and_seeds_stay_distinct() {
    let plan = SoundBatchPlan::new("Piano", spec(), SoundCategory::ShortMusic, 4).unwrap();
    assert!(plan.spec.music);
    assert!(
        plan.spec
            .prompt
            .ends_with("Instrumental music only. No vocals, singing or speech.")
    );
    assert_eq!(plan.authored_prompt, spec().prompt);
    assert_eq!(
        (0..4).map(|i| plan.variant(i).seed).collect::<Vec<_>>(),
        [u32::MAX, 0, 1, 2]
    );
    let ambience = SoundBatchPlan::new("Rain", spec(), SoundCategory::Ambience, 1).unwrap();
    assert!(!ambience.spec.music);
    assert_eq!(ambience.spec.prompt, spec().prompt);
    let mut multiline = spec();
    multiline.prompt = "Rain.\nDistant wind.\tNo speech.".into();
    let multiline = SoundBatchPlan::new("Rain", multiline, SoundCategory::Ambience, 1).unwrap();
    assert_eq!(
        multiline.authored_prompt,
        "Rain.\nDistant wind.\tNo speech."
    );
    assert_eq!(multiline.spec.prompt, "Rain. Distant wind. No speech.");
    assert!(SoundBatchPlan::new("Piano", spec(), SoundCategory::ShortMusic, 5).is_err());
}

#[test]
fn cancelling_after_a_completed_request_preserves_original_candidate_and_metadata() {
    let p = project();
    let control = SpeechSynthesisControl::default();
    let plan = SoundBatchPlan::new("Piano", spec(), SoundCategory::ShortMusic, 3).unwrap();
    let outcome = p
        .generate_sound_batch_using(0, plan, &control, |spec, control| {
            let completed = fixture(spec);
            control.cancel();
            Ok(completed)
        })
        .unwrap();
    assert_eq!(outcome.state, SoundBatchState::Cancelled);
    assert_eq!(outcome.completed, 1);
    assert_eq!(outcome.requested, 3);
    let reopened = Project::open(&p.path).unwrap().read().unwrap();
    let sound = &reopened.sound_library[0];
    assert!(sound.content.is_none());
    assert_eq!(
        sound.metadata.as_ref().unwrap().category,
        SoundCategory::ShortMusic
    );
    assert_eq!(
        sound.metadata.as_ref().unwrap().authored_prompt,
        spec().prompt
    );
    let original = p.object(&sound.original).unwrap();
    let export = p
        .path
        .parent()
        .unwrap()
        .join(format!("vox-sound-export-{}.wav", uuid::Uuid::now_v7()));
    p.export_material("sound", &sound.id, &export).unwrap();
    assert_eq!(std::fs::read(export).unwrap(), original);
    let accepted = p.accept_sound(reopened.version, &sound.id).unwrap();
    assert_eq!(
        p.object(&accepted.sound_library[0].original).unwrap(),
        original
    );
    assert!(accepted.sound_library[0].content.is_some());
}

#[test]
fn a_later_failure_keeps_completed_variants_and_reports_partial_state() {
    let p = project();
    let plan = SoundBatchPlan::new("Rain", spec(), SoundCategory::Ambience, 3).unwrap();
    let mut calls = 0;
    let outcome = p
        .generate_sound_batch_using(0, plan, &SpeechSynthesisControl::default(), |spec, _| {
            calls += 1;
            if calls == 2 {
                return Err(Error::new("infer_unavailable"));
            }
            Ok(fixture(spec))
        })
        .unwrap();
    assert_eq!(outcome.state, SoundBatchState::PartialFailed);
    assert_eq!(outcome.completed, 1);
    assert_eq!(outcome.error.as_deref(), Some("infer_unavailable"));
    assert_eq!(
        Project::open(&p.path)
            .unwrap()
            .read()
            .unwrap()
            .sound_library
            .len(),
        1
    );
}

#[test]
fn legacy_sound_without_metadata_round_trips_and_cancel_before_dispatch_writes_nothing() {
    let p = project();
    let m = p
        .add_sound(
            0,
            "Bell",
            &crate::materials::builtin_wav("bell").unwrap(),
            "builtin",
            None,
        )
        .unwrap();
    let value = serde_json::to_value(&m.sound_library[0]).unwrap();
    assert!(value.get("metadata").is_none());
    let legacy: super::super::SoundMaterial = serde_json::from_value(value).unwrap();
    assert!(legacy.metadata.is_none());
    let control = SpeechSynthesisControl::default();
    control.cancel();
    let plan = SoundBatchPlan::new("Rain", spec(), SoundCategory::Ambience, 2).unwrap();
    let outcome = p
        .generate_sound_batch_using(m.version, plan, &control, |_, _| {
            panic!("cancelled dispatch")
        })
        .unwrap();
    assert_eq!(outcome.completed, 0);
    assert_eq!(outcome.state, SoundBatchState::Cancelled);
    assert_eq!(outcome.manifest.version, m.version);
}
