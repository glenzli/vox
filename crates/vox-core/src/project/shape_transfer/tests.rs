use super::*;
use crate::{
    AudioOriginDisclosure, AudioValueContract, ContentRef, speech_script::SpeechCueAction,
};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture {
    root: PathBuf,
    transfer: PathBuf,
    destination: PathBuf,
    value: Value,
    audio: Vec<u8>,
}
impl Fixture {
    fn new() -> Self {
        let root =
            std::env::temp_dir().join(format!("vox-shape-contract-{}", uuid::Uuid::now_v7()));
        let transfer = root.join("transfer");
        fs::create_dir_all(transfer.join("materials")).unwrap();
        let destination = root.join("Imported.vox");
        let text_artifact = uuid::Uuid::now_v7().to_string();
        let latest = text_material(&transfer, &text_artifact, "Latest accepted.\r\n", false);
        let historical = text_material(
            &transfer,
            &text_artifact,
            "[production: dialogue]\n[role: Guide]\n[cue: bell]\n[speaker: Guide]\nOld exact source.\n[sound: bell]\n",
            true,
        );
        let audio = wav();
        let cue = payload(&transfer, &audio, "audio/wav");
        let contract = AudioValueContract::pcm_s16le_wav(
            24000,
            1,
            240,
            AudioOriginDisclosure::SyntheticSpeech,
        )
        .unwrap();
        let mut operation = Document::default().operation().unwrap();
        let mut options = crate::speech_script::SpeechScriptOptions::default();
        let crate::SpeechVoiceSelection::Preset(voice) = &operation.voice else {
            unreachable!()
        };
        options.roles.insert("Guide".into(), voice.clone());
        options.cues.insert(
            "bell".into(),
            SpeechCueAction::Audio {
                content: serde_json::from_value(cue["content"].clone()).unwrap(),
            },
        );
        operation.script = Some(options);
        operation.delivery = Some(crate::speech_script::SpeechDelivery::Warm);
        let audio_artifact = uuid::Uuid::now_v7().to_string();
        let audio_revision = uuid::Uuid::now_v7().to_string();
        let transformation = uuid::Uuid::now_v7().to_string();
        let audio_material = json!({"source":{"artifact_id":audio_artifact,"revision_id":audio_revision},"revision":{"id":audio_revision,"artifact_id":audio_artifact,"parents":[],"content":cue["content"],"content_contract":{"kind":"audio_clip","contract":contract},"transformation_id":transformation,"created_at_unix_ms":1},"content":cue["content"],"contract":{"kind":"audio_clip","contract":contract},"relative_path":cue["relative_path"]});
        let draft_text = "Unaccepted writing.\r\n保留草稿。\n";
        let draft = payload(
            &transfer,
            draft_text.as_bytes(),
            "text/plain; charset=utf-8",
        );
        let settings = json!({"schema":"shape.operator-draft.text-authoring@20260922.1","json":json!({"text":draft_text,"profile":"plain"}).to_string()});
        let speech_settings = json!({"schema":"shape.operator-draft.audio-speech@20260922.1","json":json!({"preset_alias":Document::default().voice,"language":"auto","speed_milli":1000,"synthetic_disclosure_required":true,"script":operation.script}).to_string()});
        let head = latest["source"]["revision_id"].clone();
        let value = json!({"schema":"vox.shape-transfer/1","request_id":"offline-contract","origin":{"producer":"shape","project_id":uuid::Uuid::now_v7().to_string(),"text_artifact_id":text_artifact,"observed_text_head":head},"accepted_text":latest,
            "authoring_draft":{"status":"unaccepted","target_artifact_id":text_artifact,"expected_head":head,"graph_digest":"a".repeat(64),"graph":{"context_artifact_id":text_artifact,"expected_revision_id":head,"operators":[{"id":"draft.author","configuration":settings}]},"format":{"format":"plain"},"settings":settings,"text":{"content":draft["content"],"contract":{"kind":"text_document","contract":{"format":"plain"}},"relative_path":draft["relative_path"]}},
            "speech_draft":{"status":"unaccepted","target_artifact_id":audio_artifact,"expected_head":audio_revision,"graph_digest":"b".repeat(64),"draft_id":"draft.speech","input":historical["source"],"input_text":historical,"graph":{"context_artifact_id":audio_artifact,"expected_revision_id":audio_revision,"operators":[{"id":"draft.speech","configuration":speech_settings}]},"settings":speech_settings,"operation":operation},
            "cue_materials":[{"content":cue["content"],"contract":{"kind":"audio_clip","contract":contract},"relative_path":cue["relative_path"]}],
            "accepted_audio":{"status":"accepted_external_origin","material":audio_material,"input_text":historical,"receipt":{"transformation_id":transformation,"outcome":{"status":"succeeded"},"executor":"original_shape_fixture"},"transformation":{"id":transformation,"target_artifact_id":audio_artifact,"inputs":[historical["source"]["revision_id"]]},"operation":operation}});
        let f = Self {
            root,
            transfer,
            destination,
            value,
            audio,
        };
        f.write();
        f
    }
    fn write(&self) {
        fs::write(
            self.transfer.join("manifest.json"),
            serde_json::to_vec_pretty(&self.value).unwrap(),
        )
        .unwrap();
    }
    fn import(&self) -> Result<Project> {
        Project::import_shape_transfer(
            &self.transfer,
            &self.destination,
            &SpeechSynthesisControl::default(),
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn payload(root: &Path, bytes: &[u8], media_type: &str) -> Value {
    let content = ContentRef::new(
        ContentDigest::from_bytes(bytes),
        media_type,
        bytes.len() as u64,
    )
    .unwrap();
    let path = format!("materials/{}", content.digest);
    fs::write(root.join(&path), bytes).unwrap();
    json!({"content":content,"relative_path":path})
}
fn text_material(root: &Path, artifact: &str, text: &str, script: bool) -> Value {
    let data = payload(root, text.as_bytes(), "text/plain; charset=utf-8");
    let id = uuid::Uuid::now_v7().to_string();
    let contract = json!({"kind":"text_document","contract":if script { json!({"format":"speech_script","revision":crate::speech_script::SPEECH_SCRIPT_REVISION}) } else { json!({"format":"plain"}) }});
    json!({"source":{"artifact_id":artifact,"revision_id":id},"revision":{"id":id,"artifact_id":artifact,"parents":[],"content":data["content"],"content_contract":contract,"transformation_id":uuid::Uuid::now_v7().to_string(),"created_at_unix_ms":1},"content":data["content"],"contract":contract,"relative_path":data["relative_path"]})
}
fn wav() -> Vec<u8> {
    let mut v = b"RIFF".to_vec();
    v.extend(516_u32.to_le_bytes());
    v.extend(b"WAVEfmt ");
    v.extend(16_u32.to_le_bytes());
    v.extend(1_u16.to_le_bytes());
    v.extend(1_u16.to_le_bytes());
    v.extend(24000_u32.to_le_bytes());
    v.extend(48000_u32.to_le_bytes());
    v.extend(2_u16.to_le_bytes());
    v.extend(16_u16.to_le_bytes());
    v.extend(b"data");
    v.extend(480_u32.to_le_bytes());
    for i in 0_i16..240 {
        v.extend((i * 100).to_le_bytes());
    }
    v
}
#[test]
fn import_preserves_exact_history_drafts_settings_and_receipt() {
    let f = Fixture::new();
    let p = f.import().unwrap();
    let m = p.read().unwrap();
    assert_eq!(m.version, 0);
    assert_eq!(m.sources.len(), 2);
    assert_eq!(m.draft.text, "Unaccepted writing.\r\n保留草稿。\n");
    assert_eq!(
        m.sources
            .iter()
            .find(|s| Some(&s.id) == m.source_head.as_ref())
            .unwrap()
            .document
            .text,
        "Latest accepted.\r\n"
    );
    let audio = &m.audio[0];
    let original = m
        .sources
        .iter()
        .find(|s| s.id == audio.source_revision)
        .unwrap();
    assert!(original.document.text.contains("Old exact source."));
    assert_ne!(Some(&original.id), m.source_head.as_ref());
    assert_eq!(audio.origin, "shape_import");
    assert!(m.candidates.is_empty());
    assert_eq!(p.object(&audio.content).unwrap(), f.audio);
    let receipt = m.shape_import.as_ref().unwrap();
    assert_eq!(receipt.drafts.len(), 2);
    let speech = &receipt
        .drafts
        .iter()
        .find(|d| d.kind == "speech")
        .unwrap()
        .document;
    assert_eq!(
        speech.delivery,
        Some(crate::speech_script::SpeechDelivery::Warm)
    );
    assert_eq!(
        serde_json::to_value(speech.operation().unwrap()).unwrap(),
        f.value["speech_draft"]["operation"]
    );
    assert_eq!(p.shape_transfer_receipt().unwrap(), f.value);
    let restored = p.restore_shape_draft(m.version, "speech").unwrap();
    assert_eq!(restored.draft, *speech);
    assert_eq!(restored.audio_head, m.audio_head);
    assert_eq!(restored.source_head, m.source_head);
    assert_eq!(
        Project::open(&f.destination).unwrap().read().unwrap().draft,
        *speech
    );
}
#[test]
fn replay_preserves_later_user_changes_and_cas() {
    let f = Fixture::new();
    let p = f.import().unwrap();
    let m = p.read().unwrap();
    let draft = Document {
        text: "User changed this after import.".into(),
        ..Document::default()
    };
    let saved = p.save(m.version, draft.clone(), None).unwrap();
    let before = fs::read(p.path.join("project.json")).unwrap();
    f.import().unwrap();
    assert_eq!(before, fs::read(p.path.join("project.json")).unwrap());
    assert_eq!(p.read().unwrap().draft, draft);
    assert_eq!(
        p.restore_shape_draft(0, "speech").unwrap_err().code,
        "stale_project"
    );
    assert_eq!(p.read().unwrap().version, saved.version);
}
#[test]
fn draft_only_never_creates_accepted_sources_or_audio() {
    let mut f = Fixture::new();
    f.value["accepted_text"] = Value::Null;
    f.value["origin"]["observed_text_head"] = Value::Null;
    f.value["authoring_draft"]["expected_head"] = Value::Null;
    f.value["authoring_draft"]["graph"]["expected_revision_id"] = Value::Null;
    f.value["speech_draft"] = Value::Null;
    f.value["accepted_audio"] = Value::Null;
    f.value["cue_materials"] = json!([]);
    f.write();
    let m = f.import().unwrap().read().unwrap();
    assert!(m.sources.is_empty());
    assert!(m.source_head.is_none());
    assert!(m.audio.is_empty());
    assert_eq!(m.draft.text, "Unaccepted writing.\r\n保留草稿。\n");
}
#[test]
fn unbound_audio_retains_external_receipt_without_claiming_current_source() {
    let mut f = Fixture::new();
    f.value["accepted_audio"]["input_text"] = Value::Null;
    f.write();
    let m = f.import().unwrap().read().unwrap();
    assert_eq!(m.audio[0].origin, "shape_import_unbound");
    assert!(m.audio[0].source_revision.starts_with("external:"));
    assert!(
        m.shape_import
            .unwrap()
            .issues
            .contains(&"accepted_audio_source_unverified".into())
    );
}
#[test]
fn schema_pin_hash_and_path_failures_publish_no_project() {
    for kind in ["schema", "pin", "hash", "path", "unknown", "audio_contract"] {
        let mut f = Fixture::new();
        match kind {
            "schema" => f.value["schema"] = "unexpected/2".into(),
            "pin" => {
                f.value["accepted_text"]["source"]["revision_id"] =
                    uuid::Uuid::now_v7().to_string().into()
            }
            "hash" => {
                let file = f
                    .transfer
                    .join(f.value["accepted_text"]["relative_path"].as_str().unwrap());
                fs::write(file, "changed").unwrap();
            }
            "path" => f.value["accepted_text"]["relative_path"] = "../escape.txt".into(),
            "unknown" => f.value["unrecognized"] = true.into(),
            "audio_contract" => {
                f.value["cue_materials"][0]["contract"]["contract"]["frame_count"] = 1.into()
            }
            _ => unreachable!(),
        }
        f.write();
        assert!(f.import().is_err(), "{kind}");
        assert!(!f.destination.exists(), "{kind}");
        assert!(!fs::read_dir(&f.root).unwrap().any(|e| {
            e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".vox-shape-")
        }));
    }
}
#[cfg(unix)]
#[test]
fn symlinked_material_cannot_escape_the_package() {
    let f = Fixture::new();
    let file = f
        .transfer
        .join(f.value["accepted_text"]["relative_path"].as_str().unwrap());
    let external = f.root.join("outside.txt");
    fs::rename(&file, &external).unwrap();
    std::os::unix::fs::symlink(&external, &file).unwrap();
    assert_eq!(f.import().unwrap_err().code, "shape_transfer_path");
    assert!(!f.destination.exists());
}
#[test]
fn cancellation_before_publication_cleans_staging_and_preserves_transfer() {
    let f = Fixture::new();
    let calls = AtomicUsize::new(0);
    let result = Project::import_shape_transfer_checked(&f.transfer, &f.destination, None, &|| {
        calls.fetch_add(1, Ordering::SeqCst) > 8
    });
    assert_eq!(result.unwrap_err().code, "shape_transfer_cancelled");
    assert!(!f.destination.exists());
    assert!(f.transfer.join("manifest.json").exists());
    assert!(!fs::read_dir(&f.root).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".vox-shape-")
    }));
}
#[test]
fn conflicting_transfer_never_overwrites_an_existing_project() {
    let mut f = Fixture::new();
    let p = f.import().unwrap();
    let before = fs::read(p.path.join("project.json")).unwrap();
    f.value["request_id"] = "another-transfer".into();
    f.write();
    assert_eq!(f.import().unwrap_err().code, "shape_transfer_conflict");
    assert_eq!(before, fs::read(p.path.join("project.json")).unwrap());
}

#[test]
fn expected_manifest_hash_fences_selection_before_project_creation() {
    let f = Fixture::new();
    let control = SpeechSynthesisControl::default();
    assert_eq!(
        Project::import_shape_transfer_expected(
            &f.transfer,
            &f.destination,
            Some(&"0".repeat(64)),
            &control
        )
        .unwrap_err()
        .code,
        "shape_transfer_hash"
    );
    assert!(!f.destination.exists());
    let hash =
        ContentDigest::from_bytes(&fs::read(f.transfer.join("manifest.json")).unwrap()).to_string();
    Project::import_shape_transfer_expected(&f.transfer, &f.destination, Some(&hash), &control)
        .unwrap();
}
#[test]
fn unsupported_catalog_is_preserved_and_cannot_be_adopted_or_rendered() {
    let mut f = Fixture::new();
    f.value["speech_draft"]["operation"]["voice"]["selection"]["catalog_revision"] =
        "historical-unknown-catalog".into();
    f.write();
    let p = f.import().unwrap();
    let m = p.read().unwrap();
    let restored = p.restore_shape_draft(m.version, "speech").unwrap();
    assert_eq!(
        restored.draft.voice_catalog_revision.as_deref(),
        Some("historical-unknown-catalog")
    );
    assert_eq!(restored.analysis()["ready"], false);
    assert_eq!(
        p.adopt_script(restored.version).unwrap_err().code,
        "shape_settings_incomplete"
    );
    assert_eq!(p.shape_transfer_receipt().unwrap(), f.value);
}

#[test]
fn unconfigured_legacy_speech_is_preserved_without_selecting_a_default_voice() {
    let mut f = Fixture::new();
    f.value["speech_draft"]["input_text"] = f.value["accepted_text"].clone();
    f.value["speech_draft"]["input"] = f.value["accepted_text"]["source"].clone();
    f.value["speech_draft"]["operation"] = Value::Null;
    f.value["speech_draft"]["settings"] = Value::Null;
    f.value["speech_draft"]["graph"]["operators"][0]["configuration"] = Value::Null;
    f.value["speech_draft"]["issue"] = "speech_settings_unconfigured".into();
    f.write();
    let p = f.import().unwrap();
    let m = p.read().unwrap();
    assert_eq!(m.draft.voice, "");
    assert_eq!(m.analysis()["ready"], false);
    let restored = p.restore_shape_draft(m.version, "speech").unwrap();
    assert_eq!(restored.draft.voice, "");
    assert_eq!(
        p.adopt_script(restored.version).unwrap_err().code,
        "shape_settings_incomplete"
    );
    assert_eq!(p.shape_transfer_receipt().unwrap(), f.value);
    let mut configured = restored.draft;
    configured.voice = Document::default().voice;
    let configured = p.save(restored.version, configured, None).unwrap();
    assert_eq!(configured.analysis()["ready"], true);
    p.adopt_script(configured.version).unwrap();
}

#[test]
fn imported_project_reopens_without_transfer_or_shape_files() {
    let f = Fixture::new();
    let p = f.import().unwrap();
    fs::remove_dir_all(&f.transfer).unwrap();
    let reopened = Project::open(&f.destination).unwrap();
    assert_eq!(
        reopened
            .object(&reopened.read().unwrap().audio[0].content)
            .unwrap(),
        f.audio
    );
    assert_eq!(p.shape_transfer_receipt().unwrap(), f.value);
}
#[test]
fn oversized_material_declaration_and_unknown_nested_audio_fields_are_rejected() {
    let mut f = Fixture::new();
    f.value["accepted_text"]["content"]["byte_length"] = (4_u64 * 1024 * 1024 + 1).into();
    f.value["accepted_text"]["revision"]["content"] = f.value["accepted_text"]["content"].clone();
    f.write();
    assert!(f.import().is_err());
    assert!(!f.destination.exists());
    let mut f = Fixture::new();
    f.value["cue_materials"][0]["contract"]["contract"]["unexpected"] = true.into();
    f.write();
    assert!(f.import().is_err());
    assert!(!f.destination.exists());
}

#[test]
fn speech_mode_follows_saved_operation_even_when_input_text_contract_is_plain() {
    let mut f = Fixture::new();
    f.value["speech_draft"]["input_text"] = f.value["accepted_text"].clone();
    f.value["speech_draft"]["input"] = f.value["accepted_text"]["source"].clone();
    f.write();
    let p = f.import().unwrap();
    let m = p.read().unwrap();
    let restored = p.restore_shape_draft(m.version, "speech").unwrap();
    assert_eq!(restored.draft.format, ScriptFormat::ProductionScript);
    assert_eq!(
        serde_json::to_value(restored.draft.operation().unwrap()).unwrap(),
        f.value["speech_draft"]["operation"]
    );
    assert_eq!(restored.draft.options.roles.len(), 1);
    assert!(restored.draft.options.cues.contains_key("bell"));
    assert_eq!(
        p.shape_transfer_receipt().unwrap()["accepted_text"]["contract"]["contract"]["format"],
        "plain"
    );
}
