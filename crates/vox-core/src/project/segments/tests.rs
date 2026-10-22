use crate::{
    AudioOriginDisclosure, ContentDigest, ExecutionOutput, ExecutionRequest,
    ExternalExecutionProvenance, PronunciationRule, SpeechSegmentProvenance,
    SpeechSynthesisControl, SpeechSynthesisOperation,
    audio::{SpeechWaveAssembly, decode_pcm_s16le_wav, parse_pcm_s16le_wav},
    project::{Document, Manifest, Project, ScriptFormat},
    speech_script::SpeechCueAction,
};
use std::sync::Arc;

struct Fixture {
    project: Project,
}
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("vox-segments-{}.vox", uuid::Uuid::now_v7()));
        Self {
            project: Project::create(&path, "Segment contracts").unwrap(),
        }
    }
    fn source(&self, document: Document) -> Manifest {
        let m = self
            .project
            .save(self.project.read().unwrap().version, document, None)
            .unwrap();
        self.project.adopt_script(m.version).unwrap()
    }
    fn generate(&self, index: usize, seed: i16) -> Manifest {
        let plan = self.project.segment_plan().unwrap();
        let definition = &plan.segments[index];
        let m = self.project.read().unwrap();
        self.project
            .render_segment_using(
                m.version,
                &definition.key,
                SpeechSynthesisControl::default(),
                Arc::new(|_, _| {}),
                |request| {
                    assert_eq!(
                        request.inputs[0].bytes().unwrap(),
                        definition.text.as_bytes()
                    );
                    assert_eq!(
                        request.inputs.len(),
                        1,
                        "other speech and cues never reach this request"
                    );
                    Ok(output(request, seed))
                },
            )
            .unwrap()
    }
    fn generate_accept(&self, index: usize, seed: i16) -> Manifest {
        let m = self.generate(index, seed);
        self.project
            .accept_segment(m.version, &m.segment_candidates.last().unwrap().id)
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.project.path);
    }
}

fn wav(seed: i16) -> Vec<u8> {
    let mut wave = SpeechWaveAssembly::default();
    let pcm = (0..240)
        .flat_map(|_| seed.to_le_bytes())
        .collect::<Vec<_>>();
    wave.push_pcm(24_000, 1, &pcm).unwrap();
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
fn output(request: &ExecutionRequest, seed: i16) -> ExecutionOutput {
    let bytes = wav(seed);
    let text = std::str::from_utf8(request.inputs[0].bytes().unwrap()).unwrap();
    let operation: SpeechSynthesisOperation = serde_json::from_slice(&request.instruction).unwrap();
    let spoken = crate::pronunciation::apply(text, &operation.pronunciation_rules).unwrap();
    let contract = parse_pcm_s16le_wav(&bytes, AudioOriginDisclosure::SyntheticSpeech).unwrap();
    let job_id = format!("resp_fixture_{}", uuid::Uuid::now_v7());
    let mut p = provenance();
    if spoken != text {
        p.speech_segments.push(SpeechSegmentProvenance {
            job_id: job_id.clone(),
            input_start: 0,
            input_end: text.len() as u32,
            input_digest: ContentDigest::from_bytes(text.as_bytes()),
            spoken_digest: Some(ContentDigest::from_bytes(spoken.as_bytes())),
            output_digest: ContentDigest::from_bytes(&bytes),
            frames: contract.frame_count,
            runtime: Box::new(provenance()),
        });
    }
    ExecutionOutput {
        bytes,
        media_type: "audio/wav".into(),
        executor_job_id: Some(job_id),
        external_provenance: Some(p),
        content_contract: None,
    }
}
fn production(text: &str) -> Document {
    Document {
        text: text.into(),
        format: ScriptFormat::ProductionScript,
        ..Document::default()
    }
}
fn pronunciation(text: &str, spoken: &str) -> PronunciationRule {
    PronunciationRule {
        text: text.into(),
        spoken: spoken.into(),
        language: None,
        role: None,
    }
}
fn pcm(bytes: &[u8]) -> Vec<u8> {
    decode_pcm_s16le_wav(bytes, AudioOriginDisclosure::SyntheticSpeech)
        .unwrap()
        .1
        .to_vec()
}

#[test]
fn candidates_persist_reopen_and_only_explicit_accept_selects() {
    let f = Fixture::new();
    let original = "First exact text.\r\n[pause: 0.25s]\r\nSecond exact text.\r\n";
    f.source(production(original));
    let plan = f.project.segment_plan().unwrap();
    assert!(plan.adopted && plan.ready && !plan.all_accepted);
    assert_eq!(plan.segments.len(), 2);
    let m = f.generate(1, 1234);
    assert_eq!(m.segment_candidates.len(), 1);
    assert!(
        m.accepted_segments.is_empty() && m.segment_selections.is_empty() && m.audio.is_empty()
    );
    assert_eq!(m.draft.text.as_bytes(), original.as_bytes());
    let reopened = Project::open(&f.project.path).unwrap();
    let plan = reopened.segment_plan().unwrap();
    assert_eq!(plan.segments[1].candidates.len(), 1);
    assert!(plan.segments[0].candidates.is_empty());
    let id = &m.segment_candidates[0].id;
    assert_eq!(
        std::fs::read(reopened.segment_playback(id).unwrap()).unwrap(),
        wav(1234)
    );
    let m = reopened.accept_segment(m.version, id).unwrap();
    assert_eq!(m.segment_selections.len(), 1);
    assert_eq!(reopened.read().unwrap().sources[0].document.text, original);
    assert!(m.version > 0);
}

#[test]
fn unchanged_selection_reuses_across_revisions_and_old_audio_is_preserved() {
    let f = Fixture::new();
    f.source(production("First.\n[pause: 0.25s]\nSecond."));
    f.generate_accept(0, 101);
    let m = f.generate_accept(1, 202);
    let plan = f.project.segment_plan().unwrap();
    let second_key = plan.segments[1].key.clone();
    let second_id = plan.segments[1].accepted_id.clone().unwrap();
    let m = f.project.assemble_segments(m.version).unwrap();
    assert!(m.audio.is_empty(), "assembly is still a candidate");
    let audio_id = m.candidates[0].id.clone();
    let original_bytes = f.project.object(&m.candidates[0].content).unwrap();
    let m = f.project.accept(m.version, &audio_id).unwrap();
    let mut changed = m.draft;
    changed.text = "Rewritten first.\n[pause: 0.5s]\nSecond.".into();
    let m = f.source(changed);
    let plan = f.project.segment_plan().unwrap();
    assert_eq!(plan.segments[1].key, second_key);
    assert_eq!(plan.segments[1].accepted_id.as_ref(), Some(&second_id));
    assert!(plan.segments[0].accepted_id.is_none());
    assert!(!plan.can_assemble);
    assert_eq!(
        f.project.assemble_segments(m.version).unwrap_err().code,
        "segment_selection_missing"
    );
    let m = f.generate_accept(0, 303);
    let m = f.project.assemble_segments(m.version).unwrap();
    let latest = &m.candidates[0];
    assert_eq!(latest.contract.frame_count, 240 + 12_000 + 240);
    let latest_pcm = pcm(&f.project.object(&latest.content).unwrap());
    assert_eq!(&latest_pcm[latest_pcm.len() - 480..], pcm(&wav(202)));
    assert_eq!(
        f.project.object(&m.audio[0].content).unwrap(),
        original_bytes
    );
    assert_eq!(f.project.audio_revision(&audio_id).unwrap().id, audio_id);
    assert_eq!(
        Project::open(&f.project.path)
            .unwrap()
            .read()
            .unwrap()
            .audio_head
            .as_ref(),
        Some(&audio_id)
    );
}

#[test]
fn accepted_alternatives_can_be_reselected_across_exact_source_revisions() {
    let f = Fixture::new();
    f.source(production("Keep.\n[pause: 0.1s]\nChange."));
    let m = f.generate_accept(0, 1);
    let first = m.accepted_segments[0].id.clone();
    let m = f.generate_accept(0, 2);
    let second = m.accepted_segments[1].id.clone();
    assert_ne!(first, second);
    let mut d = m.draft;
    d.text = "Keep.\n[pause: 0.1s]\nChanged.".into();
    let m = f.source(d);
    let m = f.project.accept_segment(m.version, &first).unwrap();
    let plan = f.project.segment_plan().unwrap();
    assert_eq!(plan.segments[0].accepted_id.as_ref(), Some(&first));
    assert_eq!(
        plan.segments[0]
            .candidates
            .iter()
            .filter(|c| c.accepted)
            .count(),
        2
    );
    assert_eq!(m.accepted_segments.len(), 2);
}

#[test]
fn candidates_require_current_adopted_source_and_version() {
    let f = Fixture::new();
    let m = f.source(production("One.\n[pause: 0.1s]\nTwo."));
    let key = f.project.segment_plan().unwrap().segments[0].key.clone();
    assert_eq!(
        f.project
            .render_segment(
                m.version - 1,
                &key,
                "invalid",
                SpeechSynthesisControl::default(),
                Arc::new(|_, _| {})
            )
            .unwrap_err()
            .code,
        "stale_project"
    );
    let m = f.generate(0, 5);
    let id = m.segment_candidates[0].id.clone();
    let mut d = m.draft;
    d.text = "One.\n[pause: 0.1s]\nChanged two.".into();
    let m = f.project.save(m.version, d.clone(), None).unwrap();
    let plan = f.project.segment_plan().unwrap();
    assert!(!plan.adopted && plan.ready);
    assert_eq!(plan.segments[0].key, key);
    assert_eq!(
        f.project.accept_segment(m.version, &id).unwrap_err().code,
        "unadopted_script"
    );
    let m = f.project.adopt_script(m.version).unwrap();
    assert_eq!(
        f.project.accept_segment(m.version, &id).unwrap_err().code,
        "stale_source"
    );
    assert!(m.segment_selections.is_empty());
}

#[test]
fn changed_settings_reference_and_pronunciation_invalidate_exact_segments() {
    let f = Fixture::new();
    let mut d = production("PCP first.\n[pause: 0.1s]\nSecond.");
    let m = f.source(d.clone());
    let baseline = f.project.segment_plan().unwrap();
    d.pronunciation_rules
        .push(pronunciation("PCP", "pee see pee"));
    let m = f.project.save(m.version, d.clone(), None).unwrap();
    let plan = f.project.segment_plan().unwrap();
    assert_eq!(plan.segments[0].text, "PCP first.");
    assert_eq!(plan.segments[0].spoken_text, "pee see pee first.");
    assert_ne!(plan.segments[0].key, baseline.segments[0].key);
    assert_eq!(
        plan.segments[1].key, baseline.segments[1].key,
        "unrelated rule must not invalidate other speech"
    );
    assert_ne!(d.digest(), baseline.document_digest);
    d.speed_milli = 1150;
    f.project.save(m.version, d, None).unwrap();
    let plan = f.project.segment_plan().unwrap();
    assert_ne!(plan.segments[1].key, baseline.segments[1].key);
}

#[test]
fn repeats_replay_selected_pcm_cues_and_pauses_without_other_generation() {
    let f = Fixture::new();
    let cue = f.project.put_object(&wav(321), "audio/wav").unwrap();
    let mut d = production(
        "[cue: turn]\n[repeat: 3; gap: 0.125s; cue: turn]\nHello.\n[pause: 0.01s]\n[end-repeat]",
    );
    d.options
        .cues
        .insert("turn".into(), SpeechCueAction::Audio { content: cue });
    f.source(d);
    assert_eq!(f.project.segment_plan().unwrap().segments.len(), 1);
    let m = f.generate_accept(0, 123);
    let m = f.project.assemble_segments(m.version).unwrap();
    assert_eq!(m.runs.len(), 1, "repeat and assembly submit no jobs");
    let audio = &m.candidates[0];
    let bytes = f.project.object(&audio.content).unwrap();
    let speech = pcm(&wav(123));
    let cue = pcm(&wav(321));
    let original = [speech.as_slice(), &[0; 480]].concat();
    let between = [cue.as_slice(), &vec![0; 6000]].concat();
    let expected = [
        original.as_slice(),
        between.as_slice(),
        original.as_slice(),
        between.as_slice(),
        original.as_slice(),
    ]
    .concat();
    assert_eq!(pcm(&bytes), expected);
    let a = audio.segment_assembly.as_ref().unwrap();
    assert_eq!(a.timings.len(), 3);
    assert_eq!(a.timings[0].start_frame, 0);
    assert_eq!(a.timings[1].start_frame, 3720);
    assert_eq!(a.timings[2].start_frame, 7440);
    let srt = f.project.path.join("captions.srt");
    f.project.export_subtitles(&audio.id, &srt, "srt").unwrap();
    let srt_text = std::fs::read_to_string(&srt).unwrap();
    assert!(srt_text.contains("00:00:00,155 --> 00:00:00,165\nHello."));
    assert_eq!(
        f.project
            .export_subtitles(&audio.id, &srt, "srt")
            .unwrap_err()
            .code,
        "destination_exists"
    );
    let vtt = f.project.path.join("captions.vtt");
    f.project.export_subtitles(&audio.id, &vtt, "vtt").unwrap();
    assert!(
        std::fs::read_to_string(vtt)
            .unwrap()
            .starts_with("WEBVTT\n\n1\n00:00:00.000 --> 00:00:00.010")
    );
    f.project.accept(m.version, &audio.id).unwrap();
}

#[test]
fn cancellation_and_concurrent_source_edits_suppress_late_segments() {
    let f = Fixture::new();
    let m = f.source(production("Chosen."));
    let key = f.project.segment_plan().unwrap().segments[0].key.clone();
    let control = SpeechSynthesisControl::default();
    let cancel = control.clone();
    let error = f
        .project
        .render_segment_using(m.version, &key, control, Arc::new(|_, _| {}), |request| {
            cancel.cancel();
            Ok(output(request, 1))
        })
        .unwrap_err();
    assert_eq!(error.code, "speech_cancelled");
    let m = f.project.read().unwrap();
    assert_eq!(m.runs[0].status, "cancelled");
    assert!(m.segment_candidates.is_empty());
    let error = f
        .project
        .render_segment_using(
            m.version,
            &key,
            SpeechSynthesisControl::default(),
            Arc::new(|_, _| {}),
            |request| {
                let current = f.project.read().unwrap();
                let mut d = current.draft;
                d.text = "Edited during generation.".into();
                f.project.save(current.version, d, None).unwrap();
                Ok(output(request, 1))
            },
        )
        .unwrap_err();
    assert_eq!(error.code, "stale_project");
    let m = f.project.read().unwrap();
    assert_eq!(m.runs[1].status, "failed");
    assert_eq!(m.draft.text, "Edited during generation.");
    assert!(m.segment_candidates.is_empty());
}

#[test]
fn segment_executor_initialization_failure_finishes_journal() {
    let f = Fixture::new();
    let m = f.source(production("Chosen."));
    let key = f.project.segment_plan().unwrap().segments[0].key.clone();
    let error = f
        .project
        .render_segment(
            m.version,
            &key,
            "invalid endpoint",
            SpeechSynthesisControl::default(),
            Arc::new(|_, _| {}),
        )
        .unwrap_err();
    assert_eq!(error.code, "credential_unavailable");
    let m = f.project.read().unwrap();
    assert_eq!(m.runs[0].status, "failed");
    assert_eq!(f.project.recover().unwrap().version, m.version);
}

#[test]
fn corrupt_segment_and_assembly_metadata_are_rejected_at_consumption() {
    let f = Fixture::new();
    f.source(production("Chosen."));
    let m = f.generate(0, 1);
    let r = &m.segment_candidates[0];
    let path = f
        .project
        .path
        .join("objects")
        .join(r.content.digest.to_string());
    std::fs::write(&path, b"tampered").unwrap();
    assert_eq!(
        f.project.accept_segment(m.version, &r.id).unwrap_err().code,
        "corrupt_object"
    );
    std::fs::write(path, wav(1)).unwrap();
    let m = f.project.accept_segment(m.version, &r.id).unwrap();
    let m = f.project.assemble_segments(m.version).unwrap();
    let id = m.candidates[0].id.clone();
    let m = f
        .project
        .update(m.version, |m| {
            m.candidates[0].segment_assembly.as_mut().unwrap().timings[0].end_frame += 1;
            Ok(())
        })
        .unwrap()
        .0;
    assert_eq!(
        f.project.accept(m.version, &id).unwrap_err().code,
        "invalid_segment_assembly"
    );
}

#[test]
fn legacy_empty_fields_preserve_document_and_operation_bytes() {
    let legacy = format!(
        "{{\"text\":\"Exact.\\r\\n\",\"format\":\"plain\",\"voice\":\"{}\",\"language\":\"auto\",\"speed_milli\":1000,\"options\":{{\"revision\":\"20260922.2\",\"roles\":{{}},\"cues\":{{}}}}}}",
        crate::SPEECH_PRESETS[0].alias
    );
    let d: Document = serde_json::from_str(&legacy).unwrap();
    assert_eq!(serde_json::to_string(&d).unwrap(), legacy);
    assert_eq!(
        d.digest(),
        blake3::hash(legacy.as_bytes()).to_hex().to_string()
    );
    assert!(
        !serde_json::to_string(&d.operation().unwrap())
            .unwrap()
            .contains("pronunciation_rules")
    );
    let f = Fixture::new();
    let m = f.source(d);
    let bytes = std::fs::read(f.project.path.join("project.json")).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    assert!(
        !text.contains("segment_candidates")
            && !text.contains("accepted_segments")
            && !text.contains("segment_selections")
    );
    assert_eq!(
        Project::open(&f.project.path)
            .unwrap()
            .read()
            .unwrap()
            .draft
            .digest(),
        m.draft.digest()
    );
}

#[test]
fn invalid_or_incomplete_source_still_has_readable_unready_plan() {
    let f = Fixture::new();
    f.source(production("[cue: unresolved]\n[audio: unresolved]\nHello."));
    let plan = f.project.segment_plan().unwrap();
    assert!(!plan.ready && !plan.can_assemble && plan.segments.is_empty());
    let mut d = Document {
        text: "Exact".into(),
        ..Document::default()
    };
    d.pronunciation_rules.push(pronunciation("", "no"));
    assert_eq!(d.operation().unwrap_err().code, "invalid_pronunciation");
}

#[test]
fn narration_source_bytes_and_duplicate_choices_survive_exact_assembly() {
    let f = Fixture::new();
    let text = format!(
        "{}\r\n{}",
        "Narration exact. ".repeat(20),
        "后面的内容。".repeat(60)
    );
    f.source(Document {
        text: text.clone(),
        ..Document::default()
    });
    let plan = f.project.segment_plan().unwrap();
    assert!(plan.segments.len() > 1);
    assert_eq!(
        plan.segments
            .iter()
            .map(|s| s.text.as_str())
            .collect::<String>(),
        text
    );
    for index in 0..plan.segments.len() {
        f.generate_accept(index, index as i16 + 1);
    }
    let m = f.project.read().unwrap();
    let m = f.project.assemble_segments(m.version).unwrap();
    assert!(
        m.candidates[0]
            .provenance
            .as_ref()
            .unwrap()
            .speech_script
            .is_none()
    );
    assert_eq!(
        m.candidates[0].contract.frame_count,
        240 * plan.segments.len() as u64
    );
    assert_eq!(m.draft.text, text);
    let m = f.project.accept(m.version, &m.candidates[0].id).unwrap();
    assert_eq!(m.audio.len(), 1);

    let other = Fixture::new();
    other.source(production("Same.\n[pause: 0.1s]\nSame."));
    let plan = other.project.segment_plan().unwrap();
    assert_eq!(plan.segments[0].text, plan.segments[1].text);
    assert_ne!(plan.segments[0].key, plan.segments[1].key);
    other.generate_accept(1, 2);
    let plan = other.project.segment_plan().unwrap();
    assert!(plan.segments[0].accepted_id.is_none());
    assert!(plan.segments[1].accepted_id.is_some());
    other.generate_accept(0, 1);
    other.source(production("Different.\n[pause: 0.1s]\nSame."));
    let plan = other.project.segment_plan().unwrap();
    assert!(
        plan.segments[1].accepted_id.is_none(),
        "changing duplicate count must not shift the other occurrence onto the removed take"
    );
}

#[test]
fn reference_identity_change_invalidates_only_the_selected_role() {
    let f = Fixture::new();
    let mut d = production(
        "[role: A]\n[role: B]\n[speaker: A]\nFirst.\n[pause: 0.1s]\n[speaker: B]\nSecond.",
    );
    let crate::SpeechVoiceSelection::Preset(preset) = d.operation().unwrap().voice else {
        panic!()
    };
    d.options.roles.insert("A".into(), preset.clone());
    d.options.roles.insert("B".into(), preset);
    let m = f.source(d);
    let original = f.project.segment_plan().unwrap();
    let path = f.project.path.join("reference.wav");
    std::fs::write(&path, crate::materials::builtin_wav("bell").unwrap()).unwrap();
    let m = f
        .project
        .import_reference(
            m.version,
            "Synthetic fixture",
            "Reference exact text",
            &path,
            true,
            false,
        )
        .unwrap();
    f.project
        .bind_reference(m.version, "A", &m.voice_library[0].id)
        .unwrap();
    let changed = f.project.segment_plan().unwrap();
    assert!(changed.ready && !changed.adopted);
    assert_ne!(changed.segments[0].key, original.segments[0].key);
    assert_eq!(changed.segments[1].key, original.segments[1].key);
}
