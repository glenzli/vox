use super::*;
use crate::{AudioOriginDisclosure, AudioValueContract};
use std::{
    path::PathBuf,
    sync::{Arc, Barrier},
};
struct Fixture {
    root: PathBuf,
    project: Project,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("vox-contract-{}.vox", uuid::Uuid::now_v7()));
        let project = Project::create(&root, "Contract fixture").unwrap();
        Self { root, project }
    }
    fn source(&self) -> Manifest {
        let d = Document {
            text: "Original exact text.\r\n你好。\n".into(),
            ..Document::default()
        };
        let m = self.project.save(0, d, None).unwrap();
        self.project.adopt_script(m.version).unwrap()
    }
    fn candidate(&self, m: &Manifest) -> (Manifest, String) {
        let bytes = wav();
        let content = self.project.put_object(&bytes, "audio/wav").unwrap();
        let id = uuid::Uuid::now_v7().to_string();
        let source = m.sources.last().unwrap();
        let r = AudioRevision {
            id: id.clone(),
            source_revision: source.id.clone(),
            document_digest: source.document.digest(),
            content,
            contract: AudioValueContract::pcm_s16le_wav(
                24000,
                1,
                240,
                AudioOriginDisclosure::SyntheticSpeech,
            )
            .unwrap(),
            provenance: None,
            origin: "fixture".into(),
            parent_audio: None,
            source_project: None,
            edit: None,
            segment_assembly: None,
        };
        let m = self
            .project
            .update(m.version, |m| {
                m.candidates.push(r);
                Ok(())
            })
            .unwrap()
            .0;
        (m, id)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
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
fn stale_writes_do_not_replace_working_draft() {
    let f = Fixture::new();
    let m = f.source();
    let before = f.project.read().unwrap().draft;
    let mut d = before.clone();
    d.text = "Stale".into();
    assert_eq!(
        f.project.save(0, d, None).unwrap_err().code,
        "stale_project"
    );
    assert_eq!(f.project.read().unwrap().draft, before);
    assert!(m.version > 0);
}
#[test]
fn concurrent_same_version_has_one_winner() {
    let f = Fixture::new();
    let barrier = Arc::new(Barrier::new(3));
    let handles = (0..2)
        .map(|i| {
            let p = f.project.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                let d = Document {
                    text: format!("Draft {i}"),
                    ..Document::default()
                };
                p.save(0, d, None).is_ok()
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();
    assert_eq!(
        handles
            .into_iter()
            .filter_map(|h| h.join().ok())
            .filter(|x| *x)
            .count(),
        1
    );
}
#[test]
fn revisions_preserve_source_bytes_and_accepted_audio_after_rewrite() {
    let f = Fixture::new();
    let m = f.source();
    let (m, id) = f.candidate(&m);
    let m = f.project.accept(m.version, &id).unwrap();
    let accepted = f.project.audio_revision(&id).unwrap();
    let mut d = m.draft.clone();
    d.text = "Rewritten".into();
    let m = f.project.save(m.version, d, None).unwrap();
    let m = f.project.adopt_script(m.version).unwrap();
    assert_eq!(
        m.sources[0].document.text,
        "Original exact text.\r\n你好。\n"
    );
    assert_eq!(m.audio.len(), 1);
    assert_eq!(f.project.object(&accepted.content).unwrap(), wav());
    assert_eq!(
        Project::open(&f.root).unwrap().read().unwrap().audio_head,
        Some(id)
    );
}
#[test]
fn old_candidate_cannot_be_accepted_after_source_changes() {
    let f = Fixture::new();
    let m = f.source();
    let (m, id) = f.candidate(&m);
    let mut d = m.draft.clone();
    d.text = "New source".into();
    let m = f.project.save(m.version, d, None).unwrap();
    let m = f.project.adopt_script(m.version).unwrap();
    assert_eq!(
        f.project.accept(m.version, &id).unwrap_err().code,
        "stale_source"
    );
    assert!(f.project.read().unwrap().audio.is_empty());
}
#[test]
fn tampered_audio_is_rejected_at_consumption_and_acceptance() {
    let f = Fixture::new();
    let m = f.source();
    let (m, id) = f.candidate(&m);
    let r = f.project.audio_revision(&id).unwrap();
    std::fs::write(
        f.root.join("objects").join(r.content.digest.to_string()),
        b"changed",
    )
    .unwrap();
    assert_eq!(
        f.project.accept(m.version, &id).unwrap_err().code,
        "corrupt_object"
    );
    assert_eq!(f.project.playback(&id).unwrap_err().code, "corrupt_object");
}
#[test]
fn runtime_candidate_requires_vox_identity_receipt() {
    let f = Fixture::new();
    let m = f.source();
    let (m, id) = f.candidate(&m);
    let m = f
        .project
        .update(m.version, |m| {
            m.candidates[0].origin = "synthesis".into();
            Ok(())
        })
        .unwrap()
        .0;
    assert_eq!(
        f.project.accept(m.version, &id).unwrap_err().code,
        "invalid_provenance"
    );
}
#[test]
fn echo_round_trip_is_pinned_candidate_and_preserves_original() {
    let f = Fixture::new();
    let m = f.source();
    let (m, id) = f.candidate(&m);
    let m = f.project.accept(m.version, &id).unwrap();
    let dest = f.root.join("echo-work");
    let (m, ticket) = f.project.begin_echo(m.version, &id, &dest).unwrap();
    let source = std::fs::read(dest.join("source.wav")).unwrap();
    assert!(String::from_utf8_lossy(&source).contains("Echo source disclosure:"));
    assert_eq!(
        crate::audio::decode_pcm_s16le_wav(&source, AudioOriginDisclosure::SyntheticSpeech)
            .unwrap()
            .1,
        crate::audio::decode_pcm_s16le_wav(&wav(), AudioOriginDisclosure::SyntheticSpeech)
            .unwrap()
            .1
    );
    let m = f
        .project
        .review_echo(
            m.version,
            &dest.join("handoff.json"),
            &dest.join("source.wav"),
            false,
        )
        .unwrap();
    assert_eq!(m.audio.len(), 1);
    assert_eq!(m.candidates.len(), 1);
    assert_eq!(m.candidates[0].parent_audio.as_deref(), Some(id.as_str()));
    assert_eq!(m.candidates[0].edit.as_ref().unwrap().ticket_id, ticket.id);
    let m = f.project.accept(m.version, &m.candidates[0].id).unwrap();
    assert_eq!(m.audio.len(), 2);
    assert_eq!(f.project.object(&m.audio[0].content).unwrap(), wav());
}
#[test]
fn another_project_cannot_return_a_ticket() {
    let f = Fixture::new();
    let m = f.source();
    let (m, id) = f.candidate(&m);
    let m = f.project.accept(m.version, &id).unwrap();
    let dest = f.root.join("echo-work");
    f.project.begin_echo(m.version, &id, &dest).unwrap();
    let other = Fixture::new();
    assert_eq!(
        other
            .project
            .review_echo(
                0,
                &dest.join("handoff.json"),
                &dest.join("source.wav"),
                false
            )
            .unwrap_err()
            .code,
        "invalid_handoff"
    );
}
#[test]
fn interrupted_render_is_observed_without_replaying_it() {
    let f = Fixture::new();
    let m = f.source();
    f.project
        .update(m.version, |m| {
            m.runs.push(RenderRun {
                id: "fixture-run".into(),
                source_revision: m.source_head.clone().unwrap(),
                status: "running".into(),
                error: None,
            });
            Ok(())
        })
        .unwrap();
    let m = f.project.recover().unwrap();
    assert_eq!(m.runs[0].status, "interrupted_unknown");
    assert!(m.candidates.is_empty());
}
#[test]
fn executor_initialization_failure_finishes_render_run() {
    let f = Fixture::new();
    let before = f.source();
    let error = f
        .project
        .render(
            before.version,
            "invalid endpoint",
            crate::SpeechSynthesisControl::default(),
            Arc::new(|_, _| {}),
        )
        .unwrap_err();
    assert_eq!(error.code, "credential_unavailable");
    let failed = f.project.read().unwrap();
    assert_eq!(failed.runs.len(), 1);
    assert_eq!(failed.runs[0].status, "failed");
    assert_eq!(failed.runs[0].error.as_deref(), Some(error.code.as_str()));
    assert_eq!(failed.draft, before.draft);
    assert_eq!(failed.source_head, before.source_head);
    assert!(failed.candidates.is_empty());
    assert!(failed.audio.is_empty());
    // Recovery must retain the observed failure rather than relabel it as process loss.
    assert_eq!(f.project.recover().unwrap().version, failed.version);
}
#[test]
fn export_never_overwrites_an_existing_file() {
    let f = Fixture::new();
    let m = f.source();
    let (_, id) = f.candidate(&m);
    let out = f.root.join("export.wav");
    f.project.export(&id, &out).unwrap();
    assert_eq!(std::fs::read(&out).unwrap(), wav());
    assert_eq!(
        f.project.export(&id, &out).unwrap_err().code,
        "destination_exists"
    );
}
