use super::*;
use crate::project::captions::tests::Fixture;

fn cue(start_ms: u64, end_ms: u64, text: &str) -> SubtitleCue {
    SubtitleCue {
        start_ms,
        end_ms,
        text: text.into(),
    }
}

fn srt() -> &'static [u8] {
    b"1\n00:00:00,000 --> 00:00:01,000\nFirst.\n\n2\n00:00:01,000 --> 00:00:02,000\nSecond.\n\n"
}

#[test]
fn import_retains_exact_bytes_unicode_and_requires_explicit_acceptance_after_reopen() {
    let f = Fixture::new();
    let before = f.project.read().unwrap();
    let bytes = "\u{feff}WEBVTT\r\n\r\n1\r\n00:00.000 --> 00:01.000\r\n  中文 café e\u{301} &lt;literal&gt; &amp;  \r\nsecond line\r\n\r\n";
    let (candidate, id) = f
        .project
        .import_subtitles(before.version, &f.audio_id, bytes.as_bytes(), "vtt", None)
        .unwrap();
    assert_eq!(candidate.audio_head, before.audio_head);
    assert_eq!(candidate.source_head, before.source_head);
    assert_eq!(candidate.subtitle_runs.len(), 0);
    assert!(candidate.subtitle_selections.is_empty());
    let revision = &candidate.subtitle_candidates[0];
    assert_eq!(revision.source(), SubtitleSource::ImportedSubtitles);
    assert!(
        revision.generation.is_none() && revision.alignment.is_none() && revision.run_id.is_none()
    );
    let Some(SubtitleProvenance::ImportedSubtitles { original, format }) = &revision.provenance
    else {
        panic!("import receipt")
    };
    assert_eq!(*format, SubtitleImportFormat::Vtt);
    assert_eq!(f.project.object(original).unwrap(), bytes.as_bytes());
    assert_eq!(
        f.project.subtitle_track(&f.audio_id).unwrap_err().code,
        "subtitles_unavailable"
    );
    let reopened = Project::open(&f.project.path).unwrap();
    let track = reopened.subtitle_candidate_track(&id).unwrap();
    assert_eq!(
        track.cues[0].text,
        "  中文 café e\u{301} <literal> &  \nsecond line"
    );
    assert_eq!(track.revision_id.as_deref(), Some(id.as_str()));
    assert!(track.alignment_revision.is_none() && track.language.is_none());
    for format in ["srt", "vtt"] {
        let encoded = track.serialize(format).unwrap();
        let parsed = parse_import(
            encoded.as_bytes(),
            SubtitleImportFormat::parse(format).unwrap(),
            Some(2000),
        )
        .unwrap();
        assert_eq!(parsed, track.cues);
    }
    let selected = reopened.accept_subtitles(candidate.version, &id).unwrap();
    assert_eq!(selected.subtitle_selections[&f.audio_id], id);
    assert_eq!(reopened.subtitle_track(&f.audio_id).unwrap(), track);
    assert_eq!(
        reopened.subtitle_transcript(&f.audio_id).unwrap(),
        revision.transcript
    );
}

#[test]
fn manual_text_time_split_and_merge_pin_the_entire_discardable_alignment_parent() {
    let f = Fixture::new();
    let aligned = f.align(&SpeechSynthesisControl::default());
    let alignment_id = aligned.candidate_id.unwrap();
    let original = aligned.manifest.subtitle_candidates[0].clone();
    let cues = vec![
        cue(0, 750, "Revised 中文 <tag> & café"),
        cue(750, 2000, "Split and re-timed."),
    ];
    let (candidate, id) = f
        .project
        .edit_subtitles(
            aligned.manifest.version,
            &f.audio_id,
            Some(&alignment_id),
            cues.clone(),
        )
        .unwrap();
    assert!(candidate.subtitle_selections.is_empty());
    assert_eq!(
        candidate.subtitle_runs.len(),
        1,
        "manual editing cannot journal a model run"
    );
    let revision = candidate.subtitle_candidates.last().unwrap();
    assert_eq!(revision.source(), SubtitleSource::ManualRevision);
    let Some(SubtitleProvenance::ManualRevision { parent, .. }) = &revision.provenance else {
        panic!("manual receipt")
    };
    assert_eq!(parent.revision_id.as_ref(), Some(&alignment_id));
    let saved: SubtitleRevision =
        serde_json::from_slice(&f.project.object(parent.revision.as_ref().unwrap()).unwrap())
            .unwrap();
    assert_eq!(
        serde_json::to_value(&saved).unwrap(),
        serde_json::to_value(&original).unwrap()
    );
    let manifest = f
        .project
        .discard_subtitles(candidate.version, &alignment_id)
        .unwrap();
    assert_eq!(
        f.project
            .subtitle_candidate_track(&alignment_id)
            .unwrap_err()
            .code,
        "subtitle_missing"
    );
    let reopened = Project::open(&f.project.path).unwrap();
    let track = reopened.subtitle_candidate_track(&id).unwrap();
    assert_eq!(track.source, SubtitleSource::ManualRevision);
    assert_eq!(track.cues, cues);
    assert!(track.alignment_revision.is_none());
    let selected = reopened.accept_subtitles(manifest.version, &id).unwrap();
    let (merged, merged_id) = reopened
        .edit_subtitles(
            selected.version,
            &f.audio_id,
            None,
            vec![cue(125, 1875, "Merged literal text.")],
        )
        .unwrap();
    assert_eq!(
        reopened
            .subtitle_track(&f.audio_id)
            .unwrap()
            .revision_id
            .as_ref(),
        Some(&id)
    );
    let selected = reopened
        .accept_subtitles(merged.version, &merged_id)
        .unwrap();
    assert_eq!(reopened.subtitle_track(&f.audio_id).unwrap().cues.len(), 1);
    reopened.accept_subtitles(selected.version, &id).unwrap();
    assert_eq!(
        reopened.subtitle_track(&f.audio_id).unwrap().cues,
        cues,
        "explicit history restoration"
    );
}

#[test]
fn import_and_edit_reject_stale_versions_unaccepted_audio_and_foreign_parents() {
    let f = Fixture::new();
    let before = f.project.read().unwrap();
    let (manifest, id) = f
        .project
        .import_subtitles(before.version, &f.audio_id, srt(), "srt", Some("English"))
        .unwrap();
    assert_eq!(
        f.project
            .import_subtitles(before.version, &f.audio_id, srt(), "srt", None)
            .unwrap_err()
            .code,
        "stale_project"
    );
    assert_eq!(
        f.project
            .edit_subtitles(before.version, &f.audio_id, Some(&id), vec![cue(0, 1, "x")])
            .unwrap_err()
            .code,
        "stale_project"
    );
    assert_eq!(
        f.project
            .import_subtitles(manifest.version, "missing", srt(), "srt", None)
            .unwrap_err()
            .code,
        "accepted_audio_missing"
    );
    assert_eq!(
        f.project
            .edit_subtitles(
                manifest.version,
                &f.audio_id,
                Some("missing"),
                vec![cue(0, 1, "x")]
            )
            .unwrap_err()
            .code,
        "subtitle_missing"
    );
    let (manifest, foreign) = f
        .project
        .update(manifest.version, |manifest| {
            let mut audio = manifest.audio[0].clone();
            audio.id = uuid::Uuid::now_v7().to_string();
            let id = audio.id.clone();
            manifest.candidates.push(audio);
            Ok(id)
        })
        .unwrap();
    assert_eq!(
        f.project
            .import_subtitles(manifest.version, &foreign, srt(), "srt", None)
            .unwrap_err()
            .code,
        "accepted_audio_missing"
    );
    let manifest = f.project.accept(manifest.version, &foreign).unwrap();
    assert_eq!(
        f.project
            .edit_subtitles(manifest.version, &foreign, Some(&id), vec![cue(0, 1, "x")])
            .unwrap_err()
            .code,
        "stale_audio"
    );
    let (manifest, _) = f
        .project
        .import_subtitles(manifest.version, &foreign, srt(), "srt", None)
        .unwrap();
    assert_eq!(
        f.project
            .subtitle_candidate_track(&id)
            .unwrap()
            .audio_revision,
        f.audio_id
    );
    assert_eq!(manifest.audio_head.as_ref(), Some(&foreign));
}

#[test]
fn imported_and_manual_content_is_rechecked_from_immutable_objects() {
    let f = Fixture::new();
    let (manifest, imported_id) = f
        .project
        .import_subtitles(
            f.project.read().unwrap().version,
            &f.audio_id,
            srt(),
            "srt",
            None,
        )
        .unwrap();
    let (manifest, manual_id) = f
        .project
        .edit_subtitles(
            manifest.version,
            &f.audio_id,
            Some(&imported_id),
            vec![cue(0, 2000, "Correction.")],
        )
        .unwrap();
    let revision = manifest.subtitle_candidates.last().unwrap();
    let Some(SubtitleProvenance::ManualRevision { edit, .. }) = &revision.provenance else {
        panic!("edit receipt")
    };
    let path = f.project.path.join("objects").join(edit.digest.to_string());
    let bytes = std::fs::read(&path).unwrap();
    std::fs::write(&path, b"{}").unwrap();
    assert_eq!(
        f.project
            .accept_subtitles(manifest.version, &manual_id)
            .unwrap_err()
            .code,
        "corrupt_object"
    );
    std::fs::write(&path, bytes).unwrap();
    let (mut tampered, _) = f
        .project
        .update(manifest.version, |manifest| {
            manifest.subtitle_candidates.last_mut().unwrap().cues[0].text = "Altered".into();
            manifest.subtitle_candidates.last_mut().unwrap().transcript = "Altered".into();
            Ok(())
        })
        .unwrap();
    assert_eq!(
        f.project
            .accept_subtitles(tampered.version, &manual_id)
            .unwrap_err()
            .code,
        "invalid_subtitles"
    );
    // Metadata alone also cannot relabel manual data as a model result.
    tampered.subtitle_candidates.last_mut().unwrap().generation = f
        .align(&SpeechSynthesisControl::default())
        .manifest
        .subtitle_candidates
        .last()
        .unwrap()
        .generation
        .clone();
    assert_eq!(
        validate_records(&tampered).unwrap_err().code,
        "invalid_subtitles"
    );
}

#[test]
fn cue_validation_covers_duration_order_overlap_controls_and_total_text_bounds() {
    let invalid = vec![
        vec![],
        vec![cue(0, 0, "empty duration")],
        vec![cue(1, 0, "reverse")],
        vec![cue(0, 2001, "beyond audio")],
        vec![cue(1000, 2000, "first"), cue(0, 1000, "out of order")],
        vec![cue(0, 1001, "first"), cue(1000, 2000, "overlap")],
        vec![cue(0, 1, "\0")],
        vec![cue(0, 1, "a\r\nb")],
        vec![cue(0, 1, "a\n\nb")],
        vec![cue(0, 1, "a\n \nb")],
        vec![cue(0, 1, "a --> b")],
        vec![cue(0, 1, &"a".repeat(65_537))],
        vec![
            cue(0, 1, &"a".repeat(32_768)),
            cue(1, 2, &"a".repeat(32_768)),
        ],
        (0..8193).map(|n| cue(n, n + 1, "x")).collect(),
    ];
    for cues in invalid {
        assert!(validate_edit_cues(&cues, Some(2000)).is_err(), "{cues:?}");
    }
    assert!(
        validate_edit_cues(
            &[
                cue(0, 1000, "literal <b> & 中文"),
                cue(1000, 2000, "next\nline")
            ],
            Some(2000)
        )
        .is_ok()
    );
    assert!(validate_edit_cues(&[cue(0, 2000, &"a".repeat(65_536))], Some(2000)).is_ok());
}

#[test]
fn import_rejects_unsupported_dialects_bad_timestamps_and_encoding() {
    let invalid: Vec<(&[u8], SubtitleImportFormat)> = vec![
        (b"", SubtitleImportFormat::Srt),
        (b"\xff", SubtitleImportFormat::Srt),
        (
            b"1\n00:00:00,000 --> 00:00:01,000\n<b>styled</b>\n",
            SubtitleImportFormat::Srt,
        ),
        (
            b"1\n00:00:00,000 --> 00:00:01,000\n{\\an8}position\n",
            SubtitleImportFormat::Srt,
        ),
        (
            b"1\n00:00:00,000 --> 00:00:01,000\n&#65;\n",
            SubtitleImportFormat::Srt,
        ),
        (
            b"WEBVTT note\n\n00:00.000 --> 00:01.000\nx\n",
            SubtitleImportFormat::Vtt,
        ),
        (
            b"WEBVTT\n\nSTYLE\n::cue {color:red}\n",
            SubtitleImportFormat::Vtt,
        ),
        (b"WEBVTT\n\nREGION\nid:one\n", SubtitleImportFormat::Vtt),
        (b"WEBVTT\n\nNOTE omitted\n", SubtitleImportFormat::Vtt),
        (
            b"WEBVTT\n\n00:00.000 --> 00:01.000 align:start\nx\n",
            SubtitleImportFormat::Vtt,
        ),
        (
            b"WEBVTT\n\n00:00.000 --> 00:01.000\n<c.red>red</c>\n",
            SubtitleImportFormat::Vtt,
        ),
        (
            b"1\n00:00:60,000 --> 00:01:01,000\nx\n",
            SubtitleImportFormat::Srt,
        ),
        (
            b"1\n00:60:00,000 --> 01:00:01,000\nx\n",
            SubtitleImportFormat::Srt,
        ),
        (
            b"1\n0:00:00,000 --> 00:00:01,000\nx\n",
            SubtitleImportFormat::Srt,
        ),
        (
            b"1\n00:00:00,00 --> 00:00:01,000\nx\n",
            SubtitleImportFormat::Srt,
        ),
        (
            b"1\n00:00:00.000 --> 00:00:01.000\nx\n",
            SubtitleImportFormat::Srt,
        ),
        (
            b"1\n00:00:00,000 --> 00:00:02,001\nx\n",
            SubtitleImportFormat::Srt,
        ),
        (
            b"1\n00:00:01,000 --> 00:00:01,000\nx\n",
            SubtitleImportFormat::Srt,
        ),
        (
            b"1\r00:00:00,000 --> 00:00:01,000\rx",
            SubtitleImportFormat::Srt,
        ),
        (
            b"1\n00:00:00,000 --> 00:00:01,000\nx\0\n",
            SubtitleImportFormat::Srt,
        ),
    ];
    for (bytes, format) in invalid {
        assert!(
            parse_import(bytes, format, Some(2000)).is_err(),
            "{bytes:?}"
        );
    }
    assert!(
        parse_import(
            &vec![b' '; MAX_SUBTITLE_IMPORT_BYTES + 1],
            SubtitleImportFormat::Srt,
            None
        )
        .is_err()
    );
    assert!(parse_timestamp("18446744073709551615:00:00,000", SubtitleImportFormat::Srt).is_err());
    assert_eq!(
        parse_timestamp("99:59:59.999", SubtitleImportFormat::Vtt).unwrap(),
        359_999_999
    );
}

#[test]
fn legacy_alignment_serialization_and_track_decoding_remain_compatible() {
    let f = Fixture::new();
    let outcome = f.align(&SpeechSynthesisControl::default());
    let revision = &outcome.manifest.subtitle_candidates[0];
    let legacy = serde_json::to_value(revision).unwrap();
    assert!(legacy.get("provenance").is_none());
    assert!(
        legacy["run_id"].is_string()
            && legacy["generation"].is_object()
            && legacy["alignment"].is_object()
    );
    let decoded: SubtitleRevision = serde_json::from_value(legacy.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), legacy);
    let track = f.project.subtitle_candidate_track(&revision.id).unwrap();
    let mut legacy_track = serde_json::to_value(&track).unwrap();
    legacy_track.as_object_mut().unwrap().remove("revision_id");
    let decoded: SubtitleTrack = serde_json::from_value(legacy_track).unwrap();
    assert!(decoded.revision_id.is_none());
    assert_eq!(decoded.alignment_revision, track.alignment_revision);
    assert_eq!(decoded.cues, track.cues);
    let mut legacy_manifest = serde_json::to_value(&outcome.manifest).unwrap();
    for key in [
        "subtitle_candidates",
        "subtitles",
        "subtitle_selections",
        "subtitle_runs",
    ] {
        legacy_manifest.as_object_mut().unwrap().remove(key);
    }
    let old: Manifest = serde_json::from_value(legacy_manifest).unwrap();
    assert!(old.subtitle_candidates.is_empty());
    validate_records(&old).unwrap();
}

#[test]
fn cue_labels_are_retained_in_original_bytes_and_never_override_timeline_order() {
    let srt =
        b"42\n00:00:00,000 --> 00:00:01,000\nFirst.\n\n7\n00:00:01,000 --> 00:00:02,000\nSecond.\n";
    let expected = vec![cue(0, 1000, "First."), cue(1000, 2000, "Second.")];
    assert_eq!(
        parse_import(srt, SubtitleImportFormat::Srt, Some(2000)).unwrap(),
        expected
    );
    let vtt = "WEBVTT\n\nopening-中文\n00:00.000 --> 00:01.000\nFirst.\n\n42\n00:01.000 --> 00:02.000\nSecond.\n";
    assert_eq!(
        parse_import(vtt.as_bytes(), SubtitleImportFormat::Vtt, Some(2000)).unwrap(),
        expected
    );
    let backward = b"42\n00:00:01,000 --> 00:00:02,000\nFirst.\n\n43\n00:00:00,000 --> 00:00:01,000\nSecond.\n";
    assert_eq!(
        parse_import(backward, SubtitleImportFormat::Srt, Some(2000))
            .unwrap_err()
            .code,
        "invalid_subtitles"
    );
}

#[test]
fn parent_depth_and_snapshot_bounds_fail_without_losing_existing_candidate() {
    let f = Fixture::new();
    let (manifest, imported) = f
        .project
        .import_subtitles(
            f.project.read().unwrap().version,
            &f.audio_id,
            srt(),
            "srt",
            None,
        )
        .unwrap();
    let (manifest, manual) = f
        .project
        .edit_subtitles(
            manifest.version,
            &f.audio_id,
            Some(&imported),
            vec![cue(0, 2000, "Correction.")],
        )
        .unwrap();
    let revision = manifest
        .subtitle_candidates
        .iter()
        .find(|revision| revision.id == manual)
        .unwrap();
    let audio = accepted_audio(&manifest, &f.audio_id).unwrap();
    let bytes = f.project.checked_audio(audio).unwrap();
    assert_eq!(
        f.project
            .checked_subtitle_depth(&manifest, revision, audio, &bytes, 127)
            .unwrap_err()
            .code,
        "invalid_subtitles"
    );
    assert!(
        f.project
            .checked_subtitle_depth(&manifest, revision, audio, &bytes, 126)
            .is_ok()
    );
    assert_eq!(
        f.project
            .put_subtitle_snapshot(
                &"x".repeat(MAX_SUBTITLE_SNAPSHOT_BYTES as usize),
                TRACK_MEDIA_TYPE
            )
            .unwrap_err()
            .code,
        "subtitle_snapshot_limit"
    );
    let mut oversized = revision.clone();
    let Some(SubtitleProvenance::ManualRevision { parent, .. }) = &mut oversized.provenance else {
        panic!("manual provenance")
    };
    parent.track.byte_length = MAX_SUBTITLE_SNAPSHOT_BYTES + 1;
    assert_eq!(
        validate_offline_record(&oversized).unwrap_err().code,
        "invalid_subtitles"
    );
    assert_eq!(f.project.read().unwrap().version, manifest.version);
    assert!(f.project.subtitle_candidate_track(&manual).is_ok());
}

#[test]
fn merged_chapter_serializer_can_exceed_one_revisions_cue_limit() {
    let track = SubtitleTrack {
        audio_revision: "compiled audio".into(),
        audio_digest: ContentDigest::from_bytes(b"compiled"),
        source_revision: "per chapter".into(),
        source: SubtitleSource::ManualRevision,
        cues: (0..8193)
            .map(|index| cue(index * 2, index * 2 + 1, "Literal."))
            .collect(),
        revision_id: None,
        alignment_revision: None,
        language: None,
    };
    assert!(track.serialize("srt").unwrap().contains("8193\n"));
    assert_eq!(
        validate_edit_cues(&track.cues, None).unwrap_err().code,
        "invalid_subtitles"
    );
}
