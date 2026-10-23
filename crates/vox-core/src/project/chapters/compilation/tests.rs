use super::super::tests::{Fixture, output, wav};
use super::*;
use crate::{
    AudioOriginDisclosure,
    audio::{SpeechWaveAssembly, decode_pcm_s16le_wav},
    project::ScriptFormat,
};

fn audio_only() -> ChapterCompilationOptions {
    ChapterCompilationOptions {
        subtitles: ChapterCompilationSubtitles::AudioOnly,
        ..Default::default()
    }
}

fn preflight(
    f: &Fixture,
    ids: &[String],
    options: &ChapterCompilationOptions,
) -> ChapterCompilationPreflight {
    f.project
        .chapter_compilation_preflight(f.project.read().unwrap().version, ids, options)
        .unwrap()
}

fn export(
    f: &Fixture,
    ids: &[String],
    options: &ChapterCompilationOptions,
    name: &str,
) -> ChapterCompilationManifest {
    let preview = preflight(f, ids, options);
    assert!(preview.ready, "{preview:?}");
    f.project
        .export_chapter_compilation(
            preview.project_version,
            ids,
            options,
            &preview.fingerprint,
            &f.base.join(name),
        )
        .unwrap()
}

fn pcm_wav(rate: u32, channels: u16, frames: usize, samples: &[i16]) -> Vec<u8> {
    let mut wave = SpeechWaveAssembly::default();
    let pcm = (0..frames)
        .flat_map(|_| samples.iter().flat_map(|sample| sample.to_le_bytes()))
        .collect::<Vec<_>>();
    wave.push_pcm(rate, channels, &pcm).unwrap();
    wave.finish().unwrap()
}

fn echo_replace(f: &Fixture, chapter_id: &str, bytes: &[u8]) -> String {
    let state = f.project.chapter_state(chapter_id).unwrap();
    let echo = f.base.join(format!("echo-{}", uuid::Uuid::now_v7()));
    let (parent, _) = f
        .project
        .begin_chapter_echo(
            f.project.read().unwrap().version,
            chapter_id,
            state.project.audio_head.as_ref().unwrap(),
            &echo,
        )
        .unwrap();
    let returned = f
        .base
        .join(format!("returned-{}.wav", uuid::Uuid::now_v7()));
    fs::write(&returned, bytes).unwrap();
    let parent = f
        .project
        .review_chapter_echo(
            parent.version,
            chapter_id,
            &echo.join("handoff.json"),
            &returned,
            false,
        )
        .unwrap();
    let state = f.project.chapter_state(chapter_id).unwrap();
    let audio = state.project.candidates.last().unwrap().id.clone();
    f.project
        .accept_chapter(parent.version, chapter_id, &audio)
        .unwrap();
    audio
}

#[test]
fn compilation_preserves_pcm_source_bytes_order_and_gaps_with_measured_captions() {
    let f = Fixture::new();
    let one = f.add("One;=#", "First.\r\n", ScriptFormat::Plain);
    let two = f.add("第二章", "Second.", ScriptFormat::Plain);
    f.batch(&[], |id, request| {
        Ok(output(request, if id == one { 1 } else { 2 }))
    });
    f.accept(&one);
    f.accept(&two);
    let options = ChapterCompilationOptions {
        gap_ms: 125,
        ..Default::default()
    };
    let child_before = fs::read(
        f.project
            .chapter_project(&one)
            .unwrap()
            .path
            .join("project.json"),
    )
    .unwrap();
    let parent_before = fs::read(f.project.path.join("project.json")).unwrap();
    let ids = [two.clone(), one.clone()];
    let manifest = export(&f, &ids, &options, "complete");
    assert_eq!(
        manifest
            .chapters
            .iter()
            .map(|chapter| &chapter.chapter_id)
            .collect::<Vec<_>>(),
        vec![&two, &one]
    );
    assert_eq!(manifest.output.frames, 3480);
    assert_eq!(manifest.output.gap_frames, 3000);
    assert_eq!(
        (
            manifest.chapters[0].start_frame,
            manifest.chapters[0].end_frame
        ),
        (0, 240)
    );
    assert_eq!(
        (
            manifest.chapters[1].start_frame,
            manifest.chapters[1].end_frame
        ),
        (3240, 3480)
    );
    let bytes = fs::read(f.base.join("complete/compilation.wav")).unwrap();
    let (contract, pcm) =
        decode_pcm_s16le_wav(&bytes, AudioOriginDisclosure::ImportedUnverified).unwrap();
    assert_eq!(contract.frame_count, 3480);
    assert_eq!(
        pcm[..480],
        decode_pcm_s16le_wav(&wav(2), AudioOriginDisclosure::ImportedUnverified)
            .unwrap()
            .1[..]
    );
    assert!(pcm[480..6480].iter().all(|byte| *byte == 0));
    assert_eq!(manifest.audio.digest, ContentDigest::from_bytes(&bytes));
    assert_eq!(manifest.audio.byte_length, bytes.len() as u64);
    assert_eq!(
        fs::read(
            f.base
                .join("complete")
                .join(&manifest.chapters[1].source_file)
        )
        .unwrap(),
        b"First.\r\n"
    );
    let captions = fs::read_to_string(f.base.join("complete/compilation.srt")).unwrap();
    assert!(captions.contains("00:00:00,135 --> 00:00:00,145\nFirst."));
    assert!(
        fs::read_to_string(f.base.join("complete/compilation.vtt"))
            .unwrap()
            .starts_with("WEBVTT")
    );
    let metadata = fs::read_to_string(f.base.join("complete/chapters.ffmetadata")).unwrap();
    assert!(metadata.contains("TIMEBASE=1/24000\nSTART=3240\nEND=3480"));
    assert!(metadata.contains("title=One\\;\\=\\#"));
    let from_disk: ChapterCompilationManifest =
        serde_json::from_slice(&fs::read(f.base.join("complete/manifest.json")).unwrap()).unwrap();
    assert_eq!(from_disk.fingerprint, manifest.fingerprint);
    assert_eq!(
        fs::read(f.project.path.join("project.json")).unwrap(),
        parent_before
    );
    assert_eq!(
        fs::read(
            f.project
                .chapter_project(&one)
                .unwrap()
                .path
                .join("project.json")
        )
        .unwrap(),
        child_before
    );
}

#[test]
fn compilation_requires_explicit_current_acceptance_and_reports_all_selected_chapters() {
    let f = Fixture::new();
    let one = f.add("One", "First.", ScriptFormat::Plain);
    let two = f.add("Two", "Second.", ScriptFormat::Plain);
    f.batch(&[], |_, request| Ok(output(request, 1)));
    let preview = preflight(&f, &[], &audio_only());
    assert!(!preview.ready);
    assert_eq!(preview.chapters.len(), 2);
    assert!(
        preview
            .chapters
            .iter()
            .all(|chapter| chapter.issues.contains(&"chapter_audio_unaccepted".into()))
    );
    f.accept(&one);
    f.accept(&two);
    let child = f.project.chapter_project(&one).unwrap();
    let current = child.read().unwrap();
    let mut document = current.draft;
    document.text = "Changed only in the child.".into();
    child.save(current.version, document, None).unwrap();
    let preview = preflight(&f, &[], &audio_only());
    assert!(!preview.ready);
    assert!(preview.chapters[0].issues.contains(&"stale_source".into()));
    assert!(preview.chapters[1].issues.is_empty());
    let parent = f.project.read().unwrap();
    let mut document = parent.draft;
    document.voice = crate::SPEECH_PRESETS[1].alias.into();
    f.project.save(parent.version, document, None).unwrap();
    let preview = preflight(&f, &[], &audio_only());
    assert!(
        preview
            .chapters
            .iter()
            .all(|chapter| chapter.issues.contains(&"chapter_settings_changed".into()))
    );
    assert_eq!(
        f.project
            .export_chapter_compilation(
                preview.project_version,
                &[],
                &audio_only(),
                &preview.fingerprint,
                &f.base.join("invalid")
            )
            .unwrap_err()
            .code,
        "compilation_not_ready"
    );
    assert!(!f.base.join("invalid").exists());
}

#[test]
fn compilation_selection_respects_explicit_order_collection_reorder_and_archiving() {
    let f = Fixture::new();
    assert_eq!(
        f.project
            .chapter_compilation_preflight(0, &[], &audio_only())
            .unwrap_err()
            .code,
        "chapter_missing"
    );
    let one = f.add("One", "First.", ScriptFormat::Plain);
    let two = f.add("Two", "Second.", ScriptFormat::Plain);
    f.batch(&[], |_, request| Ok(output(request, 1)));
    f.accept(&one);
    f.accept(&two);
    let m = f.project.read().unwrap();
    assert_eq!(
        f.project
            .chapter_compilation_preflight(m.version, &[one.clone(), one.clone()], &audio_only())
            .unwrap_err()
            .code,
        "invalid_chapter_selection"
    );
    let m = f
        .project
        .reorder_chapters(m.version, &[two.clone(), one.clone()])
        .unwrap();
    assert_eq!(
        preflight(&f, &[], &audio_only()).chapters[0].chapter_id,
        two
    );
    assert_eq!(
        preflight(&f, &[one.clone(), two.clone()], &audio_only()).chapters[0].chapter_id,
        one
    );
    f.project.archive_chapter(m.version, &two, true).unwrap();
    assert_eq!(preflight(&f, &[], &audio_only()).chapters.len(), 1);
    assert_eq!(
        f.project
            .chapter_compilation_preflight(f.project.read().unwrap().version, &[two], &audio_only())
            .unwrap_err()
            .code,
        "chapter_missing"
    );
}

#[test]
fn compilation_snapshot_fences_independent_child_changes_and_no_clobber_paths() {
    let f = Fixture::new();
    let one = f.add("One", "First.", ScriptFormat::Plain);
    f.batch(&[], |_, request| Ok(output(request, 1)));
    f.accept(&one);
    let options = ChapterCompilationOptions::default();
    let preview = preflight(&f, &[], &options);
    let child = f.project.chapter_project(&one).unwrap();
    let current = child.read().unwrap();
    child
        .save(
            current.version,
            current.draft,
            Some("Independent child title".into()),
        )
        .unwrap();
    assert_eq!(f.project.read().unwrap().version, preview.project_version);
    assert_eq!(
        f.project
            .export_chapter_compilation(
                preview.project_version,
                &[],
                &options,
                &preview.fingerprint,
                &f.base.join("old-preview")
            )
            .unwrap_err()
            .code,
        "compilation_preflight_changed"
    );
    assert!(!f.base.join("old-preview").exists());
    let manifest = export(&f, &[], &options, "new-preview");
    let before = fs::read(f.base.join("new-preview/manifest.json")).unwrap();
    assert_eq!(
        f.project
            .export_chapter_compilation(
                preview.project_version,
                &[],
                &options,
                &manifest.fingerprint,
                &f.base.join("new-preview")
            )
            .unwrap_err()
            .code,
        "destination_exists"
    );
    assert_eq!(
        fs::read(f.base.join("new-preview/manifest.json")).unwrap(),
        before
    );
    assert_eq!(
        f.project
            .export_chapter_compilation(
                preview.project_version,
                &[],
                &options,
                &manifest.fingerprint,
                &f.project.path.join("generated")
            )
            .unwrap_err()
            .code,
        "invalid_destination"
    );
    assert!(!f.project.path.join("generated").exists());
    #[cfg(unix)]
    {
        let alias = f.base.join("project-alias");
        std::os::unix::fs::symlink(&f.project.path, &alias).unwrap();
        assert_eq!(
            f.project
                .export_chapter_compilation(
                    preview.project_version,
                    &[],
                    &options,
                    &manifest.fingerprint,
                    &alias.join("generated")
                )
                .unwrap_err()
                .code,
            "invalid_destination"
        );
        let dangling = f.base.join("dangling");
        std::os::unix::fs::symlink(f.base.join("absent"), &dangling).unwrap();
        assert_eq!(
            f.project
                .export_chapter_compilation(
                    preview.project_version,
                    &[],
                    &options,
                    &manifest.fingerprint,
                    &dangling
                )
                .unwrap_err()
                .code,
            "destination_exists"
        );
    }
}

#[test]
fn compilation_normalization_is_explicit_preserves_originals_and_omits_stale_captions() {
    let f = Fixture::new();
    let one = f.add("Stereo edit", "First.", ScriptFormat::Plain);
    let two = f.add("Mono", "Second.", ScriptFormat::Plain);
    f.batch(&[], |_, request| Ok(output(request, 4)));
    f.accept(&one);
    f.accept(&two);
    let original = pcm_wav(24_000, 2, 241, &[100, -99]);
    let edited = echo_replace(&f, &one, &original);
    let keep = preflight(&f, &[], &audio_only());
    assert!(!keep.ready);
    assert!(keep.issues.contains(&"compilation_format_mismatch".into()));
    let options = ChapterCompilationOptions {
        audio_mode: ChapterCompilationAudioMode::NormalizeMono24000,
        ..Default::default()
    };
    let required = preflight(&f, &[], &options);
    assert!(!required.ready);
    assert!(
        required.chapters[0]
            .issues
            .contains(&"subtitles_unavailable".into())
    );
    let options = ChapterCompilationOptions {
        subtitles: ChapterCompilationSubtitles::AudioOnly,
        ..options
    };
    let manifest = export(&f, &[], &options, "normalized");
    assert_eq!(manifest.chapters[0].audio.id, edited);
    assert_eq!(
        manifest.chapters[0].audio.content.digest,
        ContentDigest::from_bytes(&original)
    );
    assert!(
        manifest.chapters[0]
            .conversion
            .as_ref()
            .unwrap()
            .method
            .contains("downmix")
    );
    assert!(manifest.chapters[1].conversion.is_none());
    assert!(manifest.srt.is_none() && manifest.vtt.is_none());
    assert!(!f.base.join("normalized/compilation.srt").exists());
    let bytes = fs::read(f.base.join("normalized/compilation.wav")).unwrap();
    let (contract, pcm) =
        decode_pcm_s16le_wav(&bytes, AudioOriginDisclosure::ImportedUnverified).unwrap();
    assert_eq!(
        (
            contract.sample_rate_hz,
            contract.channels,
            contract.frame_count
        ),
        (24_000, 1, 481)
    );
    assert!(
        pcm[..482]
            .chunks_exact(2)
            .all(|sample| i16::from_le_bytes([sample[0], sample[1]]) == 1)
    );
    assert_eq!(
        f.project
            .chapter_project(&one)
            .unwrap()
            .object(&manifest.chapters[0].audio.content)
            .unwrap(),
        original
    );
}

#[test]
fn compilation_resamples_rates_and_rounds_cumulative_markers_once() {
    let f = Fixture::new();
    let one = f.add("44.1k", "First.", ScriptFormat::Plain);
    let two = f.add("24k", "Second.", ScriptFormat::Plain);
    f.batch(&[], |_, request| Ok(output(request, 5)));
    f.accept(&one);
    f.accept(&two);
    echo_replace(&f, &one, &pcm_wav(44_100, 2, 443, &[300, 500]));
    echo_replace(&f, &two, &pcm_wav(24_000, 1, 241, &[500]));
    let options = ChapterCompilationOptions {
        gap_ms: 1,
        audio_mode: ChapterCompilationAudioMode::NormalizeMono24000,
        subtitles: ChapterCompilationSubtitles::AudioOnly,
    };
    let manifest = export(&f, &[], &options, "resampled");
    assert_eq!(manifest.chapters[0].end_frame, 241);
    assert_eq!(manifest.chapters[1].start_frame, 265);
    assert_eq!(manifest.chapters[1].end_frame, 506);
    assert_eq!(manifest.chapters[1].end_ms, 21);
    let bytes = fs::read(f.base.join("resampled/compilation.wav")).unwrap();
    let (_, pcm) = decode_pcm_s16le_wav(&bytes, AudioOriginDisclosure::ImportedUnverified).unwrap();
    assert!(
        pcm[..482]
            .chunks_exact(2)
            .all(|sample| i16::from_le_bytes([sample[0], sample[1]]) == 400)
    );
    assert!(pcm[482..530].iter().all(|byte| *byte == 0));
    assert!(
        manifest.chapters[0]
            .conversion
            .as_ref()
            .unwrap()
            .method
            .contains("65_tap")
    );
    let first = export(&f, &[], &options, "repeat");
    assert_eq!(manifest.audio.digest, first.audio.digest);
}

#[test]
fn compilation_rejects_limits_and_cleans_failed_staging() {
    let f = Fixture::new();
    let one = f.add("One", "First.", ScriptFormat::Plain);
    f.batch(&[], |_, request| Ok(output(request, 1)));
    f.accept(&one);
    let options = ChapterCompilationOptions {
        gap_ms: MAX_CHAPTER_GAP_MS + 1,
        ..audio_only()
    };
    assert_eq!(
        f.project
            .chapter_compilation_preflight(f.project.read().unwrap().version, &[], &options)
            .unwrap_err()
            .code,
        "invalid_compilation_options"
    );
    let destination = f.base.join("write-failed");
    assert_eq!(
        publish_export(&f.project, &destination, |staging| {
            fs::write(staging.join("partial.wav"), b"incomplete")?;
            Err(Error::new("injected_write_failure"))
        })
        .unwrap_err()
        .code,
        "injected_write_failure"
    );
    assert!(!destination.exists());
    assert!(fs::read_dir(&f.base).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".vox-chapter-export-")
    }));
    let second = f.add("Second", "Second.", ScriptFormat::Plain);
    f.batch(std::slice::from_ref(&second), |_, request| {
        Ok(output(request, 2))
    });
    f.accept(&second);
    // Valid high-rate stereo input plus a ten-minute gap exceeds the bounded
    // output without allocating or writing that output during preflight.
    echo_replace(&f, &one, &pcm_wav(384_000, 2, 3840, &[1, 1]));
    echo_replace(&f, &second, &pcm_wav(384_000, 2, 3840, &[1, 1]));
    let options = ChapterCompilationOptions {
        gap_ms: MAX_CHAPTER_GAP_MS,
        ..audio_only()
    };
    let preview = preflight(&f, &[], &options);
    assert!(!preview.ready);
    assert!(preview.issues.contains(&"compilation_too_large".into()));
}

#[test]
fn accepted_alignment_on_echo_audio_drives_both_exports_and_is_revision_pinned() {
    let f = Fixture::new();
    let one = f.add("Edited chapter", "First.", ScriptFormat::Plain);
    f.batch(&[], |_, request| Ok(output(request, 1)));
    f.accept(&one);
    let bytes = pcm_wav(24_000, 1, 14_400, &[77]);
    let edited = echo_replace(&f, &one, &bytes);
    let options = ChapterCompilationOptions::default();
    assert!(!preflight(&f, &[], &options).ready);
    let before = preflight(&f, &[], &audio_only());
    let child = f.project.chapter_project(&one).unwrap();
    let accepted = crate::project::captions::tests::accept_alignment_fixture(
        &child,
        &edited,
        crate::infer_runtime::captions::AlignmentSpec {
            transcript: "First.".into(),
            language: "English".into(),
        },
    );
    assert_eq!(f.project.read().unwrap().version, before.project_version);
    let after = preflight(&f, &[], &options);
    assert!(after.ready);
    assert_eq!(
        after.chapters[0].subtitle_source,
        Some(SubtitleSource::ForcedAlignment)
    );
    assert_eq!(
        after.chapters[0].alignment_revision.as_ref(),
        Some(&accepted.subtitles[0].id)
    );
    assert_eq!(
        f.project
            .export_chapter_compilation(
                before.project_version,
                &[],
                &audio_only(),
                &before.fingerprint,
                &f.base.join("before-alignment")
            )
            .unwrap_err()
            .code,
        "compilation_preflight_changed"
    );
    let package = f
        .project
        .export_chapters(before.project_version, &[], &f.base.join("package"))
        .unwrap();
    assert_eq!(
        package.chapters[0].subtitle_source,
        Some(SubtitleSource::ForcedAlignment)
    );
    assert_eq!(
        package.chapters[0].alignment_revision.as_ref(),
        Some(&accepted.subtitles[0].id)
    );
    let manifest = export(&f, &[], &options, "aligned");
    let track = manifest.chapters[0].subtitle_track.as_ref().unwrap();
    assert_eq!(track.audio_revision, edited);
    assert_eq!(track.source, SubtitleSource::ForcedAlignment);
    assert_eq!(
        track.alignment_revision.as_ref(),
        Some(&accepted.subtitles[0].id)
    );
    assert_eq!(
        fs::read(f.base.join("aligned/compilation.srt")).unwrap(),
        fs::read(
            f.base
                .join("package")
                .join(package.chapters[0].srt.as_ref().unwrap())
        )
        .unwrap()
    );
}

#[test]
fn fractional_chapter_frames_accumulate_before_millisecond_rounding() {
    let f = Fixture::new();
    let ids = (0..3)
        .map(|index| f.add(&format!("Chapter {index}"), "Text.", ScriptFormat::Plain))
        .collect::<Vec<_>>();
    f.batch(&[], |_, request| Ok(output(request, 1)));
    for id in &ids {
        f.accept(id);
        echo_replace(&f, id, &pcm_wav(24_000, 1, 250, &[9]));
    }
    let options = ChapterCompilationOptions {
        gap_ms: 1,
        ..audio_only()
    };
    let manifest = export(&f, &[], &options, "fractional");
    assert_eq!(manifest.output.frames, 798);
    assert_eq!(manifest.output.duration_ms, 33);
    assert_eq!(manifest.chapters[2].start_frame, 548);
    assert_eq!(manifest.chapters[2].start_ms, 23);
    assert_eq!(manifest.chapters[2].end_frame, 798);
    // Summing individually rounded 10ms chapters would start the third at22ms.
    assert_ne!(manifest.chapters[2].start_ms, 22);
}
