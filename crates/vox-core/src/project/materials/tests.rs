use super::*;
fn project() -> Project {
    let path = std::env::temp_dir().join(format!("vox-material-{}.vox", uuid::Uuid::now_v7()));
    Project::create(&path, "Materials").unwrap()
}
fn reference_file(p: &Project) -> std::path::PathBuf {
    let f = p
        .path
        .parent()
        .unwrap()
        .join(format!("ref-{}.wav", uuid::Uuid::now_v7()));
    std::fs::write(&f, crate::materials::builtin_wav("bell").unwrap()).unwrap();
    f
}
#[test]
fn references_require_consent_and_exact_text_and_reopen_as_immutable_bindings() {
    let p = project();
    let f = reference_file(&p);
    assert_eq!(
        p.import_reference(0, "Voice", "Words", &f, false, false)
            .unwrap_err()
            .code,
        "voice_consent_required"
    );
    assert_eq!(
        p.import_reference(0, "Voice", " ", &f, true, false)
            .unwrap_err()
            .code,
        "reference_text_required"
    );
    let m = p
        .import_reference(0, "Voice", "  Exact words. 中文。\n", &f, true, false)
        .unwrap();
    let id = m.voice_library[0].id.clone();
    let m = p.bind_reference(m.version, "", &id).unwrap();
    let reference = m.draft.reference_voice.clone().unwrap();
    assert_eq!(
        reference.transcript.as_deref(),
        Some("  Exact words. 中文。\n")
    );
    let mut draft = m.draft;
    draft.text = "New speech".into();
    let m = p.save(m.version, draft, None).unwrap();
    let m = p.adopt_script(m.version).unwrap();
    let inputs = p.render_inputs(&m.draft).unwrap();
    assert_eq!(inputs.len(), 2);
    let m = p
        .import_reference(m.version, "Another", "Changed words", &f, true, false)
        .unwrap();
    let new_id = m.voice_library[1].id.clone();
    let m = p.bind_reference(m.version, "", &new_id).unwrap();
    assert_eq!(
        m.sources[0].document.reference_voice.as_ref(),
        Some(&reference)
    );
    assert_ne!(m.draft.reference_voice, Some(reference));
    assert_eq!(
        Project::open(&p.path)
            .unwrap()
            .read()
            .unwrap()
            .voice_library
            .len(),
        2
    );
}
#[test]
fn foreign_reference_cannot_be_injected_through_save() {
    let a = project();
    let b = project();
    let f = reference_file(&a);
    let m = a
        .import_reference(0, "Voice", "Words", &f, true, false)
        .unwrap();
    let mut d = b.read().unwrap().draft;
    d.reference_voice = Some(m.voice_library[0].reference.clone());
    assert_eq!(b.save(0, d, None).unwrap_err().code, "reference_missing");
    assert_eq!(b.read().unwrap().version, 0);
}
#[test]
fn sound_candidates_require_adoption_and_keep_original_bytes_after_binding() {
    let p = project();
    let original = crate::materials::builtin_wav("bell").unwrap();
    let m = p.add_sound(0, "Bell", &original, "builtin", None).unwrap();
    let id = m.sound_library[0].id.clone();
    assert_eq!(
        p.bind_sound(m.version, "turn", &id).unwrap_err().code,
        "material_not_accepted"
    );
    let m = p.accept_sound(m.version, &id).unwrap();
    let m = p.bind_sound(m.version, "turn", &id).unwrap();
    assert_eq!(p.object(&m.sound_library[0].original).unwrap(), original);
    assert!(m.draft.options.cues.contains_key("turn"));
    assert_eq!(
        p.discard_sound(m.version, &id).unwrap_err().code,
        "candidate_missing"
    );
    assert_eq!(p.accept_sound(0, &id).unwrap_err().code, "stale_project");
    let reopened = Project::open(&p.path).unwrap();
    assert!(reopened.read().unwrap().sound_library[0].content.is_some());
}
#[test]
fn legacy_document_json_and_digest_are_unchanged_when_new_fields_are_unused() {
    let p = project();
    let d = p.read().unwrap().draft;
    let serialized = serde_json::to_string(&d).unwrap();
    assert!(!serialized.contains("reference"));
    let again: super::super::Document = serde_json::from_str(&serialized).unwrap();
    assert_eq!(again.digest(), d.digest());
}
#[test]
fn reference_roles_resolve_readiness_and_report_unsupported_delivery() {
    let p = project();
    let f = reference_file(&p);
    let m = p
        .import_reference(0, "Voice", "Words", &f, true, false)
        .unwrap();
    let id = m.voice_library[0].id.clone();
    let m = p.bind_reference(m.version, "Guide", &id).unwrap();
    let mut draft = m.draft;
    draft.format = super::super::ScriptFormat::ProductionScript;
    draft.text = "[role: Guide]\n[speaker: Guide]\nHello.".into();
    assert_eq!(draft.analysis()["ready"], true);
    draft.text = "[role: Guide]\n[delivery: warm]\n[speaker: Guide]\nHello.".into();
    assert_eq!(draft.analysis()["ready"], false);
    draft.speed_milli = 1200;
    assert_eq!(
        draft.operation().unwrap_err().code,
        "clone_controls_unsupported"
    );
}

#[test]
fn switching_to_plain_keeps_unused_role_and_cue_assets_out_of_execution_inputs() {
    let p = project();
    let f = reference_file(&p);
    let m = p
        .import_reference(0, "Role voice", "Words", &f, true, false)
        .unwrap();
    let m = p
        .bind_reference(m.version, "Guide", &m.voice_library[0].id)
        .unwrap();
    let mut d = m.draft;
    d.text = "Plain narration".into();
    d.options.cues.insert(
        "turn".into(),
        crate::speech_script::SpeechCueAction::Audio {
            content: d.options.reference_roles["Guide"].audio.clone().unwrap(),
        },
    );
    d.speed_milli = 1200;
    assert!(d.operation().is_ok());
    assert_eq!(p.render_inputs(&d).unwrap().len(), 1);
}

#[cfg(unix)]
#[test]
fn new_project_reference_and_preview_are_private_to_the_owner() {
    use std::os::unix::fs::PermissionsExt;
    let p = project();
    let f = reference_file(&p);
    let m = p
        .import_reference(0, "Private", "Words", &f, true, false)
        .unwrap();
    let mode =
        |path: &std::path::Path| std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode(&p.path), 0o700);
    assert_eq!(mode(&p.path.join("project.json")), 0o600);
    let content = m.voice_library[0].reference.audio.as_ref().unwrap();
    assert_eq!(
        mode(&p.path.join("objects").join(content.digest.to_string())),
        0o600
    );
    let file = p.material_file("voice", &m.voice_library[0].id).unwrap();
    assert_eq!(mode(file.parent().unwrap()), 0o700);
}
