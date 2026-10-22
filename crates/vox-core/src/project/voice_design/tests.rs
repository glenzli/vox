use super::*;
use crate::infer_runtime::voice_design::tests::{fixture, spec};

fn project() -> Project {
    Project::create(
        &std::env::temp_dir().join(format!("vox-design-{}.vox", uuid::Uuid::now_v7())),
        "Synthetic design tests",
    )
    .unwrap()
}

fn generated(project: &Project, variations: u8) -> VoiceDesignOutcome {
    project
        .generate_voice_design_using(
            project.read().unwrap().version,
            spec(),
            variations,
            &SpeechSynthesisControl::default(),
            |spec, _| Ok(fixture(spec)),
            &|_, _| {},
        )
        .unwrap()
}

#[test]
fn design_batches_are_bounded_and_no_candidate_is_an_accepted_voice() {
    let project = project();
    for count in [0, 5, u8::MAX] {
        assert_eq!(
            project
                .generate_voice_design_using(
                    0,
                    spec(),
                    count,
                    &SpeechSynthesisControl::default(),
                    |_, _| panic!("invalid count dispatch"),
                    &|_, _| {},
                )
                .unwrap_err()
                .code,
            "invalid_voice_design"
        );
    }
    assert_eq!(project.read().unwrap().version, 0);
    let outcome = generated(&project, 4);
    assert_eq!(outcome.state, VoiceDesignState::Completed);
    assert_eq!(outcome.completed, 4);
    let manifest = project.read().unwrap();
    assert!(manifest.voice_library.is_empty());
    assert_eq!(manifest.voice_design_candidates.len(), 4);
    assert_eq!(manifest.voice_design_runs[0].status, "completed");
    for (index, candidate) in manifest.voice_design_candidates.iter().enumerate() {
        assert_eq!(candidate.variation, index as u8 + 1);
        assert!(candidate.accepted_reference.is_none());
        assert_eq!(candidate.generation.spec, spec());
        assert_eq!(
            candidate.contract.origin,
            AudioOriginDisclosure::SyntheticSpeech
        );
    }
}

#[test]
fn completed_cancelled_sample_survives_reopen_export_and_disclosed_echo_copy() {
    let project = project();
    let control = SpeechSynthesisControl::default();
    let mut dispatched = 0;
    let outcome = project
        .generate_voice_design_using(
            0,
            spec(),
            3,
            &control,
            |spec, control| {
                dispatched += 1;
                control.cancel();
                Ok(fixture(spec))
            },
            &|_, _| {},
        )
        .unwrap();
    assert_eq!(dispatched, 1);
    assert_eq!(outcome.state, VoiceDesignState::Cancelled);
    assert_eq!(outcome.completed, 1);
    let manifest = Project::open(&project.path).unwrap().recover().unwrap();
    assert!(manifest.voice_library.is_empty());
    assert_eq!(manifest.voice_design_runs[0].status, "cancelled");
    let candidate = &manifest.voice_design_candidates[0];
    let original = project.object(&candidate.original).unwrap();
    assert_eq!(
        std::fs::read(
            project
                .material_file("voice_design", &candidate.id)
                .unwrap()
        )
        .unwrap(),
        original
    );
    let export = project.path.with_extension("wav");
    project
        .export_material("voice_design", &candidate.id, &export)
        .unwrap();
    assert_eq!(std::fs::read(export).unwrap(), original);
    let echo = project
        .material_echo("voice_design", &candidate.id)
        .unwrap();
    let working = std::fs::read(&echo).unwrap();
    assert_ne!(working, original);
    assert_eq!(
        crate::audio::decode_pcm_s16le_wav(&working, AudioOriginDisclosure::SyntheticSpeech)
            .unwrap()
            .1,
        crate::audio::decode_pcm_s16le_wav(&original, AudioOriginDisclosure::SyntheticSpeech)
            .unwrap()
            .1
    );
    let receipt: serde_json::Value =
        serde_json::from_slice(&std::fs::read(echo.parent().unwrap().join("source.json")).unwrap())
            .unwrap();
    assert_eq!(receipt["synthetic"], true);
    assert_eq!(receipt["voice_design"]["candidate_id"], candidate.id);
    assert_eq!(
        receipt["voice_design"]["generation"]["job_id"],
        candidate.generation.job_id
    );
    assert_eq!(project.object(&candidate.original).unwrap(), original);
}

#[test]
fn failed_and_partial_failed_batches_keep_terminal_journals_and_completed_samples() {
    let project = project();
    let failure = project
        .generate_voice_design_using(
            0,
            spec(),
            2,
            &SpeechSynthesisControl::default(),
            |_, _| Err(Error::new("infer_unavailable")),
            &|_, _| {},
        )
        .unwrap();
    assert_eq!(failure.state, VoiceDesignState::Failed);
    assert_eq!(failure.manifest.voice_design_runs[0].status, "failed");
    let mut calls = 0;
    let outcome = project
        .generate_voice_design_using(
            failure.manifest.version,
            spec(),
            3,
            &SpeechSynthesisControl::default(),
            |spec, _| {
                calls += 1;
                if calls == 1 {
                    Ok(fixture(spec))
                } else {
                    Err(Error::new("model_failure"))
                }
            },
            &|_, _| {},
        )
        .unwrap();
    assert_eq!(calls, 2);
    assert_eq!(outcome.state, VoiceDesignState::PartialFailed);
    assert_eq!(outcome.completed, 1);
    assert_eq!(outcome.error.as_deref(), Some("model_failure"));
    let manifest = project.recover().unwrap();
    assert_eq!(manifest.voice_design_candidates.len(), 1);
    assert_eq!(manifest.voice_design_runs[1].completed, 1);
    assert_eq!(manifest.voice_design_runs[1].status, "partial_failed");
    assert!(manifest.voice_library.is_empty());
}

#[test]
fn named_acceptance_preserves_exact_synthetic_sample_and_reuses_existing_clone_bindings() {
    let project = project();
    let outcome = generated(&project, 1);
    let candidate = outcome.manifest.voice_design_candidates[0].clone();
    let version = outcome.manifest.version;
    assert_eq!(
        project
            .accept_voice_design(version, &candidate.id, " ")
            .unwrap_err()
            .code,
        "invalid_material"
    );
    assert_eq!(
        project
            .accept_voice_design(0, &candidate.id, "Garden narrator")
            .unwrap_err()
            .code,
        "stale_project"
    );
    assert!(project.read().unwrap().voice_library.is_empty());
    let manifest = project
        .accept_voice_design(version, &candidate.id, "Garden narrator")
        .unwrap();
    let voice = &manifest.voice_library[0];
    assert_eq!(voice.origin, "designed");
    assert_eq!(voice.reference.audio.as_ref(), Some(&candidate.original));
    assert_eq!(voice.reference.transcript.as_ref(), Some(&spec().text));
    assert!(
        voice
            .reference
            .consent_receipt_id
            .starts_with("synthetic-design-acceptance:")
    );
    assert_eq!(
        voice.design.as_ref().unwrap().authorization_basis,
        "accepted_synthetic_design"
    );
    assert!(voice.reference.is_executable());
    let voice_id = voice.id.clone();
    assert_eq!(
        manifest.voice_design_candidates[0]
            .accepted_reference
            .as_ref(),
        Some(&voice_id)
    );
    assert_eq!(
        project
            .accept_voice_design(manifest.version, &candidate.id, "Again")
            .unwrap_err()
            .code,
        "voice_design_already_accepted"
    );
    assert_eq!(
        project
            .discard_voice_design(manifest.version, &candidate.id)
            .unwrap_err()
            .code,
        "candidate_missing"
    );
    let manifest = project
        .bind_reference(manifest.version, "", &voice_id)
        .unwrap();
    let manifest = project
        .bind_reference(manifest.version, "Guide", &voice_id)
        .unwrap();
    assert_eq!(
        manifest.draft.reference_voice.as_ref(),
        Some(&manifest.voice_library[0].reference)
    );
    assert_eq!(
        manifest.draft.options.reference_roles["Guide"],
        manifest.voice_library[0].reference
    );
    let mut draft = manifest.draft;
    draft.text = "Fresh text using the saved design.".into();
    let manifest = project.save(manifest.version, draft, None).unwrap();
    let manifest = project.adopt_script(manifest.version).unwrap();
    let inputs = project.render_inputs(&manifest.draft).unwrap();
    assert_eq!(inputs.len(), 2);
    assert_eq!(
        inputs[1].bytes().unwrap(),
        project.object(&candidate.original).unwrap()
    );
    let reopened = Project::open(&project.path).unwrap().read().unwrap();
    assert_eq!(
        reopened.sources[0].document.reference_voice,
        reopened.draft.reference_voice
    );
    let echo = project.material_echo("voice", &voice_id).unwrap();
    let receipt: serde_json::Value =
        serde_json::from_slice(&std::fs::read(echo.parent().unwrap().join("source.json")).unwrap())
            .unwrap();
    assert_eq!(receipt["synthetic"], true);
    assert_eq!(receipt["voice_design"]["candidate_id"], candidate.id);
}

#[test]
fn concurrent_draft_save_does_not_destroy_a_completed_design_sample() {
    let project = project();
    let outcome = project
        .generate_voice_design_using(
            0,
            spec(),
            1,
            &SpeechSynthesisControl::default(),
            |spec, _| {
                let manifest = project.read().unwrap();
                let mut draft = manifest.draft;
                draft.text = "Typing during design generation.".into();
                project.save(manifest.version, draft, None).unwrap();
                Ok(fixture(spec))
            },
            &|_, _| {},
        )
        .unwrap();
    assert_eq!(
        outcome.manifest.draft.text,
        "Typing during design generation."
    );
    assert_eq!(outcome.completed, 1);
    assert_eq!(outcome.manifest.voice_design_candidates.len(), 1);
}

#[test]
fn progress_reports_only_journaled_completed_samples() {
    let project = project();
    let events = std::sync::Mutex::new(Vec::new());
    project
        .generate_voice_design_using(
            0,
            spec(),
            2,
            &SpeechSynthesisControl::default(),
            |spec, _| Ok(fixture(spec)),
            &|completed, total| {
                let manifest = project.read().unwrap();
                assert_eq!(manifest.voice_design_candidates.len(), completed);
                assert_eq!(
                    usize::from(manifest.voice_design_runs[0].completed),
                    completed
                );
                events.lock().unwrap().push((completed, total));
            },
        )
        .unwrap();
    assert_eq!(*events.lock().unwrap(), [(0, 2), (1, 2), (2, 2)]);
}

#[test]
fn recover_marks_unknown_active_request_without_replaying_or_losing_completed_candidate() {
    let project = project();
    let outcome = generated(&project, 2);
    let mut manifest = outcome.manifest;
    manifest.voice_design_candidates.pop();
    let run = &mut manifest.voice_design_runs[0];
    run.candidate_ids.pop();
    run.completed = 1;
    run.active_variation = Some(2);
    run.status = "running".into();
    project.write(&manifest).unwrap();
    let reopened = Project::open(&project.path).unwrap();
    let producer = OpenOptions::new()
        .read(true)
        .write(true)
        .open(project.path.join("render.lock"))
        .unwrap();
    producer.lock().unwrap();
    assert_eq!(
        reopened.recover().unwrap().voice_design_runs[0].status,
        "running"
    );
    drop(producer);
    let recovered = reopened.recover().unwrap();
    assert_eq!(recovered.voice_design_runs[0].status, "interrupted_unknown");
    assert_eq!(recovered.voice_design_runs[0].active_variation, Some(2));
    assert_eq!(recovered.voice_design_candidates.len(), 1);
    assert_eq!(reopened.recover().unwrap().version, recovered.version);
}

#[test]
fn unaccepted_discard_preserves_run_receipt_and_accepted_audio_is_rechecked() {
    let project = project();
    let outcome = generated(&project, 2);
    let first = &outcome.manifest.voice_design_candidates[0];
    let second = &outcome.manifest.voice_design_candidates[1];
    let manifest = project
        .discard_voice_design(outcome.manifest.version, &first.id)
        .unwrap();
    assert_eq!(manifest.voice_design_candidates.len(), 1);
    assert_eq!(manifest.voice_design_runs[0].candidate_ids.len(), 2);
    let path = project
        .path
        .join("objects")
        .join(second.original.digest.to_string());
    std::fs::write(path, b"corrupt artifact").unwrap();
    assert_eq!(
        project
            .accept_voice_design(manifest.version, &second.id, "Narrator")
            .unwrap_err()
            .code,
        "corrupt_object"
    );
    assert!(project.read().unwrap().voice_library.is_empty());
}
