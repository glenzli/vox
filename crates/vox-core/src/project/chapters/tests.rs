use super::*;
use crate::{
    AudioOriginDisclosure, ContentDigest, ExecutionOutput, ExecutionRequest,
    ExternalExecutionProvenance, PronunciationRule, SpeechSegmentProvenance,
    SpeechSynthesisControl, SpeechSynthesisOperation,
    audio::{SpeechWaveAssembly, parse_pcm_s16le_wav},
    project::{Error, Manifest, Project, RenderRun, ScriptFormat},
    speech_script::SpeechCueAction,
};
use std::{fs, path::PathBuf, sync::Arc};

pub(super) struct Fixture {
    pub(super) base: PathBuf,
    pub(super) project: Project,
}
impl Fixture {
    pub(super) fn new() -> Self {
        let base = std::env::temp_dir().join(format!("vox-chapters-{}", uuid::Uuid::now_v7()));
        fs::create_dir(&base).unwrap();
        let project = Project::create(&base.join("Book.vox"), "Ordered chapters").unwrap();
        Self { base, project }
    }
    pub(super) fn add(&self, title: &str, text: &str, format: ScriptFormat) -> String {
        let m = self
            .project
            .add_chapter(self.project.read().unwrap().version, title, text, format)
            .unwrap();
        m.chapters.last().unwrap().id.clone()
    }
    pub(super) fn batch(
        &self,
        ids: &[String],
        execute: impl FnMut(&str, &ExecutionRequest) -> crate::project::Result<ExecutionOutput>,
    ) -> ChapterBatchOutcome {
        self.project
            .render_chapter_batch_using(
                self.project.read().unwrap().version,
                ids,
                false,
                SpeechSynthesisControl::default(),
                Arc::new(|_| {}),
                execute,
            )
            .unwrap()
    }
    pub(super) fn accept(&self, id: &str) -> Manifest {
        let state = self.project.chapter_state(id).unwrap();
        self.project
            .accept_chapter(
                self.project.read().unwrap().version,
                id,
                &state.project.candidates.last().unwrap().id,
            )
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

pub(super) fn wav(seed: i16) -> Vec<u8> {
    let mut wave = SpeechWaveAssembly::default();
    wave.push_pcm(
        24_000,
        1,
        &(0..240)
            .flat_map(|_| seed.to_le_bytes())
            .collect::<Vec<_>>(),
    )
    .unwrap();
    wave.finish().unwrap()
}
fn provenance() -> ExternalExecutionProvenance {
    serde_json::from_value(serde_json::json!({
        "contract_revision":crate::infer_runtime::INFER_RUNTIME_CONTRACT_VERSION,
        "capability_contract":"infer.audio.speech@20260811.1","app_id":"vox",
        "intent":"speech.synthesize","provider":"fixture-local",
        "deployment":"mlx_qwen3_tts_custom_voice_1_7b","model_profile":"fixture",
        "model_build":"fixture","physical_model":"fixture","placement":"local",
        "capability_level":"capable","evaluation_status":"provisional","resource_class":"standard",
        "policy":"local-first","priority":"interactive","requested_policy":"local-first",
        "requested_priority":"interactive","requested_provider_access_class":null,
        "requested_placement":"local_only","requested_preference":"local","offline_required":true,
        "requested_latency":"interactive","fallback":"none","requested_deadline_ms":null,
        "max_cost_microusd":0,"capability_floor":"capable",
        "routing_candidates":[{"provider":"fixture-local","deployment":"mlx_qwen3_tts_custom_voice_1_7b","status":"eligible","rank":0,"reason_codes":[]}],
        "attempts":[{"number":1,"provider":"fixture-local","deployment":"mlx_qwen3_tts_custom_voice_1_7b","outcome":"succeeded","trigger":"initial","error_kind":null}]
    })).unwrap()
}
pub(super) fn output(request: &ExecutionRequest, seed: i16) -> ExecutionOutput {
    let bytes = wav(seed);
    let text = std::str::from_utf8(request.inputs[0].bytes().unwrap()).unwrap();
    let operation: SpeechSynthesisOperation = serde_json::from_slice(&request.instruction).unwrap();
    let spoken = crate::pronunciation::apply(text, &operation.pronunciation_rules).unwrap();
    let contract = parse_pcm_s16le_wav(&bytes, AudioOriginDisclosure::SyntheticSpeech).unwrap();
    let job_id = format!("resp_fixture_{}", uuid::Uuid::now_v7());
    let mut leaf = provenance();
    if matches!(
        operation.voice,
        crate::SpeechVoiceSelection::AuthorizedReference(_)
    ) {
        let mut metadata = serde_json::to_value(&leaf).unwrap();
        metadata["intent"] = serde_json::json!("speech.clone_voice");
        metadata["capability_contract"] = serde_json::json!("infer.audio.voice-clone@20260811.1");
        metadata["deployment"] = serde_json::json!("mlx_qwen3_tts_base_1_7b");
        metadata["routing_candidates"][0]["deployment"] = metadata["deployment"].clone();
        metadata["attempts"][0]["deployment"] = metadata["deployment"].clone();
        leaf = serde_json::from_value(metadata).unwrap();
    }
    let mut receipt = leaf.clone();
    if spoken != text {
        receipt.speech_segments.push(SpeechSegmentProvenance {
            job_id: job_id.clone(),
            input_start: 0,
            input_end: text.len() as u32,
            input_digest: ContentDigest::from_bytes(text.as_bytes()),
            spoken_digest: Some(ContentDigest::from_bytes(spoken.as_bytes())),
            output_digest: ContentDigest::from_bytes(&bytes),
            frames: contract.frame_count,
            runtime: Box::new(leaf),
        });
    }
    ExecutionOutput {
        bytes,
        media_type: "audio/wav".into(),
        executor_job_id: Some(job_id),
        external_provenance: Some(receipt),
        content_contract: None,
    }
}

#[test]
fn chapter_crud_is_ordered_owned_archivable_and_durable() {
    let f = Fixture::new();
    let first = f.add("One", "原文\r\n", ScriptFormat::Plain);
    let second = f.add("Two", "Second", ScriptFormat::Plain);
    let m = f.project.read().unwrap();
    assert_eq!(
        f.project
            .reorder_chapters(m.version, &[second.clone(), second.clone()])
            .unwrap_err()
            .code,
        "invalid_chapter_order"
    );
    let m = f
        .project
        .reorder_chapters(m.version, &[second.clone(), first.clone()])
        .unwrap();
    let m = f
        .project
        .rename_chapter(m.version, &first, "第一章")
        .unwrap();
    let child_path = f.project.chapter_project(&first).unwrap().path;
    let child_before = fs::read(child_path.join("project.json")).unwrap();
    let m = f.project.archive_chapter(m.version, &first, true).unwrap();
    assert!(m.chapters[1].archived);
    assert_eq!(
        fs::read(child_path.join("project.json")).unwrap(),
        child_before
    );
    let m = f.project.archive_chapter(m.version, &first, false).unwrap();
    let reopened = Project::open(&f.project.path).unwrap();
    assert_eq!(reopened.read().unwrap().chapters[0].id, second);
    let state = reopened.chapter_state(&first).unwrap();
    assert_eq!(state.chapter.title, "第一章");
    assert_eq!(state.project.draft.text.as_bytes(), "原文\r\n".as_bytes());
    assert_eq!(
        state.project.chapter_owner.as_ref().unwrap().project_id,
        m.id
    );
    assert_eq!(
        reopened
            .add_chapter(m.version - 1, "Stale", "", ScriptFormat::Plain)
            .unwrap_err()
            .code,
        "stale_project"
    );
    assert_eq!(
        reopened
            .chapter_project("../../foreign.vox")
            .unwrap_err()
            .code,
        "chapter_missing"
    );
    assert_eq!(
        reopened
            .chapter_project(&first)
            .unwrap()
            .add_chapter(state.project.version, "Nested", "No", ScriptFormat::Plain)
            .unwrap_err()
            .code,
        "nested_chapter_project"
    );
}

#[test]
fn imports_retain_literal_utf8_markdown_bytes_and_origin_ranges() {
    let f = Fixture::new();
    let text = format!(
        "# 第一章\r\n{}\n\n# Second\n{}\n",
        "中文 without markup stripping.\n".repeat(140),
        "Tail.\n".repeat(230)
    );
    let source = f.base.join("book.md");
    fs::write(&source, &text).unwrap();
    let m = f.project.import_chapters(0, &source, 1024).unwrap();
    assert!(m.chapters.len() > 2);
    let mut rejoined = String::new();
    let mut end = 0;
    for chapter in &m.chapters {
        let state = f.project.chapter_state(&chapter.id).unwrap();
        let import = chapter.import.as_ref().unwrap();
        assert_eq!(import.start_byte, end);
        end = import.end_byte;
        assert_eq!(f.project.object(&import.original).unwrap(), text.as_bytes());
        assert_eq!(
            state.project.draft.text.as_bytes(),
            &text.as_bytes()[import.start_byte as usize..import.end_byte as usize]
        );
        assert!(state.project.draft.text.len() <= 1024);
        rejoined.push_str(&state.project.draft.text);
    }
    assert_eq!(rejoined, text);
    assert_eq!(m.chapters[0].title, "第一章");
    let invalid = f.base.join("invalid.txt");
    fs::write(&invalid, [0xff, 0xfe]).unwrap();
    assert_eq!(
        f.project
            .import_chapters(m.version, &invalid, 1024)
            .unwrap_err()
            .code,
        "invalid_utf8"
    );
    assert_eq!(f.project.read().unwrap().version, m.version);
}

#[test]
fn markdown_import_splits_real_repeated_headings_and_preserves_preamble_fences_and_ranges() {
    let f = Fixture::new();
    let text = "Preamble stays here.\r\n\r\n# 第一章\r\nFirst.\r\n```md\r\n# Fenced heading\r\n## Not a chapter\r\n```\r\n\r\n  # 第二章 ###\r\nSecond.\r\n~~~\r\n# Another fenced heading\r\n~~~\r\n";
    let path = f.base.join("short.md");
    fs::write(&path, text).unwrap();
    let m = f.project.import_chapters(0, &path, 65_536).unwrap();
    assert_eq!(m.chapters.len(), 2);
    assert_eq!(m.chapters[0].title, "第一章");
    assert_eq!(m.chapters[1].title, "第二章");
    let first = f
        .project
        .chapter_state(&m.chapters[0].id)
        .unwrap()
        .project
        .draft
        .text;
    let second = f
        .project
        .chapter_state(&m.chapters[1].id)
        .unwrap()
        .project
        .draft
        .text;
    assert!(first.starts_with("Preamble stays here.\r\n"));
    assert!(second.starts_with("  # 第二章 ###\r\n"));
    assert_eq!(format!("{first}{second}"), text);
    assert_eq!(
        m.chapters[0].import.as_ref().unwrap().end_byte,
        first.len() as u64
    );
    assert_eq!(
        m.chapters[1].import.as_ref().unwrap().start_byte,
        first.len() as u64
    );
    let text =
        "# Book\n## First section\nOne.\n```\n## False section\n```\n## Second section\nTwo.\n";
    fs::write(&path, text).unwrap();
    let m = f.project.import_chapters(m.version, &path, 65_536).unwrap();
    assert_eq!(m.chapters.len(), 4);
    assert_eq!(m.chapters[2].title, "First section");
    assert_eq!(m.chapters[3].title, "Second section");
    let text_path = f.base.join("headings.txt");
    fs::write(&text_path, "# One\nFirst.\n# Two\nSecond.").unwrap();
    let m = f
        .project
        .import_chapters(m.version, &text_path, 65_536)
        .unwrap();
    assert_eq!(
        m.chapters.len(),
        5,
        "TXT literal hash lines are not a chapter structure"
    );
}

#[test]
fn batches_make_review_candidates_and_reruns_skip_completed_work() {
    let f = Fixture::new();
    let first = f.add(
        "One",
        "First.\n[pause: 0.25s]\nSecond.",
        ScriptFormat::ProductionScript,
    );
    let second = f.add("Two", "Third.", ScriptFormat::Plain);
    let mut requests = 0;
    let outcome = f.batch(&[], |_, request| {
        requests += 1;
        Ok(output(request, requests))
    });
    assert_eq!(requests, 3);
    assert_eq!(outcome.batch.status, "completed");
    for id in [&first, &second] {
        let state = f.project.chapter_state(id).unwrap();
        assert!(state.project.audio.is_empty() && state.project.audio_head.is_none());
        assert!(
            state.project.accepted_segments.is_empty()
                && state.project.segment_selections.is_empty()
        );
        assert_eq!(state.project.candidates.len(), 1);
        assert_eq!(state.project.sources.len(), 1);
    }
    let second_run = f.batch(&[], |_, _| {
        panic!("complete exact work must not redispatch")
    });
    assert!(
        second_run
            .batch
            .chapters
            .iter()
            .all(|item| item.status == "skipped")
    );
    assert_eq!(
        f.project
            .chapter_summaries_for(&second_run.manifest)
            .unwrap()[0]
            .status,
        "review"
    );
    let m = f.accept(&first);
    assert_eq!(
        f.project.chapter_state(&first).unwrap().project.audio.len(),
        1
    );
    assert_eq!(
        f.project.chapter_summaries_for(&m).unwrap()[0].status,
        "accepted"
    );
}

#[test]
fn chapter_edit_regenerates_only_changed_segments_and_keeps_old_accepted_audio() {
    let f = Fixture::new();
    let id = f.add(
        "One",
        "Keep.\n[pause: 0.1s]\nChange.",
        ScriptFormat::ProductionScript,
    );
    f.batch(&[], |_, request| Ok(output(request, 1)));
    let m = f.accept(&id);
    let before = f.project.chapter_state(&id).unwrap();
    let old = before.project.audio[0].clone();
    let mut document = before.project.draft;
    document.text = "Keep.\n[pause: 0.3s]\nChanged.".into();
    f.project.save_chapter(m.version, &id, document).unwrap();
    let mut requests = 0;
    let result = f.batch(&[], |_, request| {
        requests += 1;
        assert_eq!(request.inputs[0].bytes().unwrap(), b"Changed.");
        Ok(output(request, 2))
    });
    assert_eq!(requests, 1);
    assert_eq!(result.batch.chapters[0].reused_segments, 1);
    let after = f.project.chapter_state(&id).unwrap();
    assert_eq!(after.project.sources.len(), 2);
    assert_eq!(after.project.audio_head.as_ref(), Some(&old.id));
    let child = f.project.chapter_project(&id).unwrap();
    assert_eq!(
        child.object(&old.content).unwrap(),
        child.object(&before.project.audio[0].content).unwrap()
    );
    assert_eq!(after.project.candidates.len(), 1);
}

#[test]
fn cancellation_saves_finished_job_and_stops_before_next_dispatch() {
    let f = Fixture::new();
    let id = f.add(
        "One",
        "One.\n[pause: 0.1s]\nTwo.",
        ScriptFormat::ProductionScript,
    );
    f.add("Two", "Three.", ScriptFormat::Plain);
    let control = SpeechSynthesisControl::default();
    let cancel = control.clone();
    let result = f
        .project
        .render_chapter_batch_using(
            f.project.read().unwrap().version,
            &[],
            false,
            control,
            Arc::new(|_| {}),
            |_, request| {
                let out = output(request, 1);
                cancel.cancel();
                Ok(out)
            },
        )
        .unwrap();
    assert_eq!(result.batch.status, "cancelled");
    assert_eq!(result.batch.chapters[0].completed_segments, 1);
    let state = f.project.chapter_state(&id).unwrap();
    assert_eq!(state.project.segment_candidates.len(), 1);
    assert!(state.project.candidates.is_empty());
    assert!(
        state
            .project
            .runs
            .iter()
            .all(|run| run.status == "completed")
    );
    let mut requests = 0;
    let resumed = f.batch(&[], |_, request| {
        requests += 1;
        Ok(output(request, 2))
    });
    assert_eq!(resumed.batch.status, "completed");
    assert_eq!(requests, 2);
    assert_eq!(resumed.batch.chapters[0].reused_segments, 1);
}

#[test]
fn later_failure_retains_completed_jobs_and_other_chapters_resume_without_replay() {
    let f = Fixture::new();
    let first = f.add(
        "One",
        "One.\n[pause: 0.1s]\nTwo.",
        ScriptFormat::ProductionScript,
    );
    let second = f.add("Two", "Three.", ScriptFormat::Plain);
    let result = f.batch(&[], |chapter, request| {
        if chapter == first && request.inputs[0].bytes().unwrap() == b"Two." {
            Err(Error::new("provider_failed"))
        } else {
            Ok(output(request, 1))
        }
    });
    assert_eq!(result.batch.status, "failed");
    assert_eq!(result.batch.chapters[0].completed_segments, 1);
    assert_eq!(result.batch.chapters[1].status, "completed");
    assert_eq!(
        f.project
            .chapter_state(&first)
            .unwrap()
            .project
            .segment_candidates
            .len(),
        1
    );
    assert_eq!(
        f.project
            .chapter_state(&second)
            .unwrap()
            .project
            .candidates
            .len(),
        1
    );
    let mut requests = 0;
    let result = f.batch(&[], |_, request| {
        requests += 1;
        Ok(output(request, 2))
    });
    assert_eq!(requests, 1);
    assert_eq!(result.batch.chapters[0].reused_segments, 1);
    assert_eq!(result.batch.chapters[1].status, "skipped");
}

#[test]
fn interrupted_runs_require_explicit_retry_and_reopening_never_dispatches() {
    let f = Fixture::new();
    let id = f.add("One", "Unknown.", ScriptFormat::Plain);
    let child = f.project.chapter_project(&id).unwrap();
    let current = child.adopt_script(child.read().unwrap().version).unwrap();
    child
        .update(current.version, |m| {
            m.runs.push(RenderRun {
                id: uuid::Uuid::now_v7().to_string(),
                source_revision: m.source_head.clone().unwrap(),
                status: "running".into(),
                error: None,
            });
            Ok(())
        })
        .unwrap();
    let reopened = Project::open(&f.project.path).unwrap();
    let error = reopened
        .render_chapter_batch_using(
            reopened.read().unwrap().version,
            &[],
            false,
            SpeechSynthesisControl::default(),
            Arc::new(|_| {}),
            |_, _| panic!("unknown request must never redispatch"),
        )
        .unwrap_err();
    assert_eq!(error.code, "chapter_interrupted_unknown");
    assert_eq!(child.read().unwrap().runs[0].status, "interrupted_unknown");
    let result = reopened
        .render_chapter_batch_using(
            reopened.read().unwrap().version,
            &[],
            true,
            SpeechSynthesisControl::default(),
            Arc::new(|_| {}),
            |_, request| Ok(output(request, 1)),
        )
        .unwrap();
    assert_eq!(result.batch.status, "completed");
    assert_eq!(
        child.read().unwrap().runs[0].status,
        "interrupted_retry_authorized"
    );
    f.batch(&[], |_, _| {
        panic!("resolved complete work must stay complete")
    });
}

#[test]
fn parent_settings_change_stops_later_dispatch_and_preserves_pinned_output() {
    let f = Fixture::new();
    let id = f.add(
        "One",
        "One.\n[pause: 0.1s]\nTwo.",
        ScriptFormat::ProductionScript,
    );
    let mut requests = 0;
    let result = f.batch(&[], |_, request| {
        requests += 1;
        let m = f.project.read().unwrap();
        let mut settings = m.draft;
        settings.speed_milli = 1100;
        f.project.save(m.version, settings, None).unwrap();
        Ok(output(request, 1))
    });
    assert_eq!(requests, 1);
    assert_eq!(result.batch.status, "failed");
    assert_eq!(
        result.batch.chapters[0].error.as_deref(),
        Some("chapter_settings_changed")
    );
    let state = f.project.chapter_state(&id).unwrap();
    assert!(!state.shared_settings_current);
    assert_eq!(state.project.segment_candidates.len(), 1);
    assert_eq!(
        state.project.segment_candidates[0].operation.speed_milli,
        1000
    );
    let mut requests = 0;
    f.batch(&[], |_, request| {
        requests += 1;
        Ok(output(request, 2))
    });
    assert_eq!(
        requests, 2,
        "new global settings invalidate both old segment keys"
    );
}

#[test]
fn shared_pronunciation_and_cues_keep_source_bytes_and_exact_materials() {
    let f = Fixture::new();
    let mut m = f.project.read().unwrap();
    let cue = f.project.put_object(&wav(99), "audio/wav").unwrap();
    m.draft.options.cues.insert(
        "turn".into(),
        SpeechCueAction::Audio {
            content: cue.clone(),
        },
    );
    m.draft.pronunciation_rules.push(PronunciationRule {
        text: "Vox".into(),
        spoken: "Voice".into(),
        language: None,
        role: None,
    });
    f.project.save(m.version, m.draft, None).unwrap();
    let script = "[cue: turn]\n[repeat: 2; gap: 0.125s; cue: turn]\nVox.\n[end-repeat]";
    let id = f.add("One", script, ScriptFormat::ProductionScript);
    let child = f.project.chapter_project(&id).unwrap();
    assert_eq!(child.object(&cue).unwrap(), wav(99));
    let result = f.batch(&[], |_, request| {
        assert_eq!(
            request.inputs.len(),
            1,
            "local cue bytes never become synthesis inputs"
        );
        Ok(output(request, 1))
    });
    assert_eq!(result.batch.chapters[0].total_segments, 1);
    let state = f.project.chapter_state(&id).unwrap();
    assert_eq!(state.project.draft.text.as_bytes(), script.as_bytes());
    assert_eq!(state.project.segment_candidates[0].spoken_text, "Voice.");
    assert_eq!(
        state.project.candidates[0]
            .segment_assembly
            .as_ref()
            .unwrap()
            .timings
            .len(),
        2
    );
    let mut changed = state.project.draft;
    changed.pronunciation_rules[0].spoken = "Voks".into();
    child.save(state.project.version, changed, None).unwrap();
    let m = f
        .project
        .publish_chapter_settings(f.project.read().unwrap().version, &id)
        .unwrap();
    assert_eq!(m.draft.pronunciation_rules[0].spoken, "Voks");
    assert!(
        f.project
            .chapter_state(&id)
            .unwrap()
            .shared_settings_current
    );
}

#[test]
fn export_requires_current_acceptance_and_has_portable_measured_caption_files() {
    let f = Fixture::new();
    let one = f.add(
        "One",
        "First.\n[pause: 0.25s]\nSecond.",
        ScriptFormat::ProductionScript,
    );
    let two = f.add("Two", "第三章。", ScriptFormat::Plain);
    f.batch(&[], |_, request| Ok(output(request, 1)));
    let target = f.base.join("export");
    assert_eq!(
        f.project
            .export_chapters(f.project.read().unwrap().version, &[], &target)
            .unwrap_err()
            .code,
        "chapter_audio_unaccepted"
    );
    assert!(!target.exists());
    f.accept(&one);
    f.accept(&two);
    let m = f.project.read().unwrap();
    let exported = f.project.export_chapters(m.version, &[], &target).unwrap();
    assert_eq!(
        exported
            .chapters
            .iter()
            .map(|chapter| chapter.chapter_id.as_str())
            .collect::<Vec<_>>(),
        vec![one.as_str(), two.as_str()]
    );
    assert_eq!(exported.chapters[0].frames, 6480);
    let caption =
        fs::read_to_string(target.join(exported.chapters[0].srt.as_ref().unwrap())).unwrap();
    assert!(caption.contains("00:00:00,260 --> 00:00:00,270\nSecond."));
    assert!(
        fs::read_to_string(target.join(exported.chapters[1].vtt.as_ref().unwrap()))
            .unwrap()
            .starts_with("WEBVTT")
    );
    let from_disk: ChapterExportManifest =
        serde_json::from_slice(&fs::read(target.join("manifest.json")).unwrap()).unwrap();
    for chapter in &from_disk.chapters {
        assert!(!PathBuf::from(&chapter.wav).is_absolute());
        assert_eq!(
            ContentDigest::from_bytes(&fs::read(target.join(&chapter.wav)).unwrap()),
            chapter.audio.digest
        );
    }
    assert_eq!(
        f.project
            .export_chapters(m.version, &[], &target)
            .unwrap_err()
            .code,
        "destination_exists"
    );
}

#[test]
fn echo_is_pinned_to_one_chapter_and_return_requires_separate_acceptance() {
    let f = Fixture::new();
    let one = f.add("One", "First.", ScriptFormat::Plain);
    let two = f.add("Two", "Second.", ScriptFormat::Plain);
    f.batch(&[], |_, request| Ok(output(request, 1)));
    f.accept(&one);
    f.accept(&two);
    let state = f.project.chapter_state(&one).unwrap();
    let old_audio = state.project.audio_head.unwrap();
    let handoff = f.base.join("echo");
    let m = f.project.read().unwrap();
    let (m, ticket) = f
        .project
        .begin_chapter_echo(m.version, &one, &old_audio, &handoff)
        .unwrap();
    assert_eq!(ticket.project_id, state.project.id);
    let returned = f.base.join("returned.wav");
    fs::write(&returned, wav(8)).unwrap();
    assert_eq!(
        f.project
            .review_chapter_echo(
                m.version,
                &two,
                &handoff.join("handoff.json"),
                &returned,
                false
            )
            .unwrap_err()
            .code,
        "invalid_handoff"
    );
    let m = f
        .project
        .review_chapter_echo(
            m.version,
            &one,
            &handoff.join("handoff.json"),
            &returned,
            false,
        )
        .unwrap();
    let state = f.project.chapter_state(&one).unwrap();
    assert_eq!(
        state.project.audio_head.as_deref(),
        Some(old_audio.as_str())
    );
    assert_eq!(state.project.candidates[0].origin, "echo_file_return");
    let edited = &state.project.candidates[0].id;
    let m = f.project.accept_chapter(m.version, &one, edited).unwrap();
    let export = f
        .project
        .export_chapters(
            m.version,
            std::slice::from_ref(&one),
            &f.base.join("edited-export"),
        )
        .unwrap();
    assert!(export.chapters[0].srt.is_none() && export.chapters[0].vtt.is_none());
    assert_eq!(
        export.chapters[0].subtitles_unavailable_reason.as_deref(),
        Some("accepted_audio_has_no_measured_segment_timings")
    );
    assert_eq!(
        fs::read(f.base.join("edited-export").join(&export.chapters[0].wav)).unwrap(),
        wav(8)
    );
    assert_eq!(
        export.chapters[0].edit.as_ref().unwrap().ticket_id,
        ticket.id
    );
    assert_eq!(export.chapters[0].parent_audio.as_ref(), Some(&old_audio));
    let result = f.batch(&[], |_, _| {
        panic!("accepted Echo edit is complete current chapter work")
    });
    assert_eq!(result.batch.chapters[0].audio_id.as_ref(), Some(edited));
}

#[test]
fn malformed_or_symlinked_child_cannot_import_an_arbitrary_project() {
    let f = Fixture::new();
    let id = f.add("One", "Owned.", ScriptFormat::Plain);
    let child = f.project.chapter_project(&id).unwrap();
    let current = child.read().unwrap();
    child
        .update(current.version, |m| {
            m.chapter_owner.as_mut().unwrap().project_id = uuid::Uuid::now_v7().to_string();
            Ok(())
        })
        .unwrap();
    assert_eq!(
        f.project.chapter_state(&id).unwrap_err().code,
        "invalid_chapter_owner"
    );
    #[cfg(unix)]
    {
        let foreign = Project::create(&f.base.join("Foreign.vox"), "Foreign").unwrap();
        let original = child.path.with_extension("backup");
        fs::rename(&child.path, original).unwrap();
        std::os::unix::fs::symlink(&foreign.path, &child.path).unwrap();
        assert_eq!(
            f.project.chapter_project(&id).unwrap_err().code,
            "invalid_chapter_owner"
        );
    }
}

#[test]
fn parent_batch_recovery_requires_retry_even_when_current_job_completed_before_crash() {
    let f = Fixture::new();
    f.add("One", "First.", ScriptFormat::Plain);
    f.add("Two", "Second.", ScriptFormat::Plain);
    let result = f.batch(&[], |_, request| Ok(output(request, 1)));
    f.project
        .update(result.manifest.version, |m| {
            let batch = m.chapter_batches.last_mut().unwrap();
            batch.status = "running".into();
            batch.chapters[0].status = "running".into();
            batch.chapters[1].status = "pending".into();
            Ok(())
        })
        .unwrap();
    let m = f.project.recover_chapter_batches().unwrap();
    assert_eq!(m.chapter_batches[0].status, "interrupted_unknown");
    assert_eq!(
        m.chapter_batches[0].chapters[0].status,
        "interrupted_unknown"
    );
    assert_eq!(m.chapter_batches[0].chapters[1].status, "cancelled");
    assert_eq!(
        f.project
            .render_chapter_batch_using(
                m.version,
                &[],
                false,
                SpeechSynthesisControl::default(),
                Arc::new(|_| {}),
                |_, _| panic!("unknown batch must wait for explicit retry")
            )
            .unwrap_err()
            .code,
        "chapter_interrupted_unknown"
    );
    let result = f
        .project
        .render_chapter_batch_using(
            m.version,
            &[],
            true,
            SpeechSynthesisControl::default(),
            Arc::new(|_| {}),
            |_, _| panic!("completed bytes survive the crash boundary"),
        )
        .unwrap();
    assert!(
        result
            .batch
            .chapters
            .iter()
            .all(|item| item.status == "skipped")
    );
    assert_eq!(
        result.manifest.chapter_batches[0].error.as_deref(),
        Some("retry_authorized")
    );
}

#[test]
fn shared_cast_change_invalidates_only_the_changed_role() {
    let f = Fixture::new();
    let m = f.project.read().unwrap();
    let mut document = m.draft;
    let crate::SpeechVoiceSelection::Preset(preset) = document.operation().unwrap().voice else {
        panic!()
    };
    document.options.roles.insert("A".into(), preset.clone());
    document.options.roles.insert("B".into(), preset);
    f.project.save(m.version, document, None).unwrap();
    let id = f.add(
        "Two roles",
        "[role: A]\n[role: B]\n[speaker: A]\nFirst.\n[pause: 0.1s]\n[speaker: B]\nSecond.",
        ScriptFormat::ProductionScript,
    );
    f.batch(&[], |_, request| Ok(output(request, 1)));
    let m = f.project.read().unwrap();
    let mut document = m.draft;
    let mut other = document.clone();
    other.voice = crate::SPEECH_PRESETS[1].alias.into();
    let crate::SpeechVoiceSelection::Preset(preset) = other.operation().unwrap().voice else {
        panic!()
    };
    document.options.roles.insert("A".into(), preset);
    let m = f.project.save(m.version, document, None).unwrap();
    assert!(
        !f.project
            .chapter_state(&id)
            .unwrap()
            .shared_settings_current
    );
    let mut count = 0;
    let result = f.batch(&[], |_, request| {
        count += 1;
        assert_eq!(request.inputs[0].bytes().unwrap(), b"First.");
        Ok(output(request, 2))
    });
    assert_eq!(count, 1);
    assert_eq!(result.batch.chapters[0].reused_segments, 1);
    assert!(
        f.project
            .chapter_state(&id)
            .unwrap()
            .shared_settings_current
    );
    assert!(result.manifest.version > m.version);
}

#[test]
fn a_chapter_source_edit_blocks_old_acceptance_echo_and_export_before_readoption() {
    let f = Fixture::new();
    let id = f.add("One", "First.", ScriptFormat::Plain);
    f.batch(&[], |_, request| Ok(output(request, 1)));
    let state = f.project.chapter_state(&id).unwrap();
    let original_id = state.project.candidates[0].id.clone();
    let child = f.project.chapter_project(&id).unwrap();
    let mut document = state.project.draft;
    document.text = "Changed but not adopted.".into();
    child.save(state.project.version, document, None).unwrap();
    let m = f.project.read().unwrap();
    assert_eq!(
        f.project
            .accept_chapter(m.version, &id, &original_id)
            .unwrap_err()
            .code,
        "stale_source"
    );
    f.batch(&[], |_, request| Ok(output(request, 2)));
    f.accept(&id);
    let state = f.project.chapter_state(&id).unwrap();
    let handoff = f.base.join("echo-stale");
    let (m, _) = f
        .project
        .begin_chapter_echo(
            f.project.read().unwrap().version,
            &id,
            state.project.audio_head.as_ref().unwrap(),
            &handoff,
        )
        .unwrap();
    let child_manifest = child.read().unwrap();
    let mut document = child_manifest.draft;
    document.text = "Changed once again.".into();
    child.save(child_manifest.version, document, None).unwrap();
    let returned = f.base.join("stale-return.wav");
    fs::write(&returned, wav(7)).unwrap();
    assert_eq!(
        f.project
            .review_chapter_echo(
                m.version,
                &id,
                &handoff.join("handoff.json"),
                &returned,
                false
            )
            .unwrap_err()
            .code,
        "stale_source"
    );
    assert_eq!(
        f.project
            .export_chapters(m.version, &[], &f.base.join("stale-export"))
            .unwrap_err()
            .code,
        "stale_source"
    );
}

#[test]
fn shared_reference_bytes_and_design_provenance_roundtrip_between_parent_and_chapter() {
    use crate::{
        infer_runtime::voice_design::tests::{fixture, spec},
        project::{VoiceDesignCandidate, VoiceDesignRun},
    };
    let f = Fixture::new();
    let (bytes, generation) = fixture(spec());
    let run_id = uuid::Uuid::now_v7().to_string();
    let candidate_id = uuid::Uuid::now_v7().to_string();
    let original = f.project.put_object(&bytes, "audio/wav").unwrap();
    let m = f
        .project
        .update(0, |m| {
            m.voice_design_runs.push(VoiceDesignRun {
                id: run_id.clone(),
                spec: generation.spec.clone(),
                requested: 1,
                completed: 1,
                candidate_ids: vec![candidate_id.clone()],
                status: "completed".into(),
                error: None,
                active_variation: None,
            });
            m.voice_design_candidates.push(VoiceDesignCandidate {
                id: candidate_id.clone(),
                run_id,
                variation: 1,
                variations: 1,
                original: original.clone(),
                contract: parse_pcm_s16le_wav(&bytes, AudioOriginDisclosure::SyntheticSpeech)
                    .unwrap(),
                generation,
                accepted_reference: None,
            });
            Ok(())
        })
        .unwrap()
        .0;
    let m = f
        .project
        .accept_voice_design(m.version, &candidate_id, "Designed narrator")
        .unwrap();
    let reference_id = m.voice_library[0].id.clone();
    f.project
        .bind_reference(m.version, "", &reference_id)
        .unwrap();
    let id = f.add("One", "Cloned speech.", ScriptFormat::Plain);
    let child = f.project.chapter_project(&id).unwrap();
    assert_eq!(child.object(&original).unwrap(), bytes);
    let state = f.project.chapter_state(&id).unwrap();
    assert_eq!(state.project.voice_design_candidates.len(), 1);
    assert_eq!(state.project.voice_design_runs.len(), 1);
    assert_eq!(state.project.voice_library[0].id, reference_id);
    f.batch(&[], |_, request| {
        assert_eq!(request.inputs.len(), 2);
        assert_eq!(request.inputs[1].bytes().unwrap(), bytes);
        Ok(output(request, 1))
    });
    let m = f
        .project
        .publish_chapter_settings(f.project.read().unwrap().version, &id)
        .unwrap();
    assert_eq!(m.voice_design_candidates.len(), 1);
    assert_eq!(m.voice_design_runs.len(), 1);
    Project::open(&f.project.path).unwrap();
    Project::open(&child.path).unwrap();
}

#[test]
fn an_explicit_segment_selection_reassembles_without_reusing_an_older_whole_candidate() {
    let f = Fixture::new();
    let id = f.add("One", "One.", ScriptFormat::Plain);
    f.batch(&[], |_, request| Ok(output(request, 1)));
    let child = f.project.chapter_project(&id).unwrap();
    let state = f.project.chapter_state(&id).unwrap();
    let original_audio = state.project.candidates[0].id.clone();
    let m = child
        .render_segment_using(
            state.project.version,
            &state.segment_plan.segments[0].key,
            SpeechSynthesisControl::default(),
            Arc::new(|_, _| {}),
            |request| Ok(output(request, 2)),
        )
        .unwrap();
    let chosen = m.segment_candidates.last().unwrap().id.clone();
    child.accept_segment(m.version, &chosen).unwrap();
    let result = f.batch(&[], |_, _| {
        panic!("the selected immutable segment is sufficient")
    });
    let item = &result.batch.chapters[0];
    assert_eq!(item.status, "completed");
    assert_eq!(item.reused_segments, 1);
    assert_ne!(item.audio_id.as_ref(), Some(&original_audio));
    let state = f.project.chapter_state(&id).unwrap();
    assert!(state.project.audio.is_empty());
    let assembled = state.project.candidates.last().unwrap();
    assert_eq!(
        assembled.segment_assembly.as_ref().unwrap().selections[0].segment_id,
        chosen
    );
}
