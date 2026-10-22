use super::{
    Chapter, ChapterBatchItem, ChapterBatchOutcome, ChapterBatchProgress, ChapterBatchRun,
    store::{check_version, selected_chapters, settings_digest, shared_document},
};
use crate::{
    ExecutionOutput, ExecutionRequest, Executor, InferRuntimeSpeechExecutor,
    SpeechSynthesisControl,
    project::{Error, Manifest, Project, Result, SegmentSelection},
};
use std::{fs::OpenOptions, sync::Arc};

/// Called by Project::recover only while its render and manifest locks are held.
/// Reopening never dispatches work whose completion is unknown.
pub(crate) fn recover_records(m: &mut Manifest) -> bool {
    let mut changed = false;
    for batch in &mut m.chapter_batches {
        if batch.status == "running" {
            batch.status = "interrupted_unknown".into();
            batch.error = Some("process_interrupted".into());
            for item in &mut batch.chapters {
                if item.status == "running" {
                    item.status = "interrupted_unknown".into();
                    item.error = Some("process_interrupted".into());
                } else if item.status == "pending" {
                    item.status = "cancelled".into();
                    item.error = Some("batch_interrupted_before_dispatch".into());
                }
            }
            changed = true;
        }
    }
    changed
}

impl Project {
    pub fn recover_chapter_batches(&self) -> Result<Manifest> {
        let render_lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.path.join("render.lock"))?;
        if render_lock.try_lock().is_err() {
            return self.read();
        }
        let _lock = self.lock()?;
        let mut m = self.read()?;
        if recover_records(&mut m) {
            return self.publish_chapter_manifest(m);
        }
        Ok(m)
    }

    pub fn render_chapter_batch(
        &self,
        expected: u64,
        ids: &[String],
        endpoint: &str,
        retry_interrupted: bool,
        control: SpeechSynthesisControl,
        progress: Arc<dyn Fn(ChapterBatchProgress) + Send + Sync>,
    ) -> Result<ChapterBatchOutcome> {
        self.render_chapter_batch_using(
            expected,
            ids,
            retry_interrupted,
            control,
            progress,
            |_, request| {
                // Cancellation is admitted between immutable segment Jobs. A Job
                // already dispatched finishes and is saved before cancellation stops
                // the next one; no provider-side interruption is claimed.
                let executor = InferRuntimeSpeechExecutor::new(
                    endpoint,
                    crate::infer_runtime::credential_path(),
                )
                .map_err(|_| Error::new("credential_unavailable"))?;
                Ok(executor.execute(request)?)
            },
        )
    }

    pub(super) fn render_chapter_batch_using(
        &self,
        expected: u64,
        ids: &[String],
        retry_interrupted: bool,
        control: SpeechSynthesisControl,
        progress: Arc<dyn Fn(ChapterBatchProgress) + Send + Sync>,
        mut execute: impl FnMut(&str, &ExecutionRequest) -> Result<ExecutionOutput>,
    ) -> Result<ChapterBatchOutcome> {
        let render_lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.path.join("render.lock"))?;
        render_lock
            .try_lock()
            .map_err(|_| Error::new("render_busy"))?;
        let (chapters, run) = {
            let _lock = self.lock()?;
            let mut m = self.read()?;
            check_version(&m, expected)?;
            let chapters = selected_chapters(&m, ids)?;
            if m.chapter_batches.len() >= 1024 {
                return Err(Error::new("project_limit"));
            }
            recover_records(&mut m);
            let selected = chapters
                .iter()
                .map(|chapter| chapter.id.as_str())
                .collect::<std::collections::BTreeSet<_>>();
            let unknown_batch = m.chapter_batches.iter().any(|batch| {
                batch.status == "interrupted_unknown"
                    && batch.chapters.iter().any(|item| {
                        selected.contains(item.chapter_id.as_str())
                            && item.status == "interrupted_unknown"
                    })
            });
            if unknown_batch && !retry_interrupted {
                return Err(Error::new("chapter_interrupted_unknown"));
            }
            for chapter in &chapters {
                let child = self.chapter_project_for(&m, &chapter.id)?;
                let current = child.recover()?;
                if current.runs.iter().any(|run| run.status == "running") {
                    return Err(Error::new("render_busy"));
                }
                if current
                    .runs
                    .iter()
                    .any(|run| run.status == "interrupted_unknown")
                {
                    if !retry_interrupted {
                        return Err(Error::new("chapter_interrupted_unknown"));
                    }
                    child.update(current.version, |manifest| {
                        for run in &mut manifest.runs {
                            if run.status == "interrupted_unknown" {
                                run.status = "interrupted_retry_authorized".into();
                                run.error = Some("retry_authorized".into());
                            }
                        }
                        Ok(())
                    })?;
                }
            }
            if retry_interrupted {
                for batch in &mut m.chapter_batches {
                    if batch.status == "interrupted_unknown" {
                        for item in &mut batch.chapters {
                            if selected.contains(item.chapter_id.as_str())
                                && item.status == "interrupted_unknown"
                            {
                                item.status = "failed".into();
                                item.error = Some("retry_authorized".into());
                            }
                        }
                        if !batch
                            .chapters
                            .iter()
                            .any(|item| item.status == "interrupted_unknown")
                        {
                            batch.status = "failed".into();
                            batch.error = Some("retry_authorized".into());
                        }
                    }
                }
            }
            let run = ChapterBatchRun {
                id: uuid::Uuid::now_v7().to_string(),
                status: "running".into(),
                settings_digest: settings_digest(&m.draft),
                error: None,
                chapters: chapters
                    .iter()
                    .map(|chapter| ChapterBatchItem {
                        chapter_id: chapter.id.clone(),
                        source_revision: None,
                        document_digest: None,
                        status: "pending".into(),
                        total_segments: 0,
                        completed_segments: 0,
                        reused_segments: 0,
                        audio_id: None,
                        error: None,
                    })
                    .collect(),
            };
            m.chapter_batches.push(run.clone());
            self.publish_chapter_manifest(m)?;
            (chapters, run)
        };
        let mut any_failure = false;
        let mut cancelled = false;
        let mut batch_error = None;
        for (index, chapter) in chapters.iter().enumerate() {
            if control.is_cancelled() {
                cancelled = true;
                self.update_batch(&run.id, |batch| {
                    for item in batch.chapters.iter_mut().skip(index) {
                        item.status = "cancelled".into();
                        item.error = Some("speech_cancelled".into());
                    }
                    Ok(())
                })?;
                break;
            }
            let result =
                self.render_one_chapter(&run, chapter, index, &control, &progress, &mut execute);
            if let Err(error) = result {
                let code = error.code;
                any_failure = true;
                batch_error = Some(code.clone());
                self.update_batch(&run.id, |batch| {
                    batch.chapters[index].status = if code == "speech_cancelled" {
                        "cancelled"
                    } else {
                        "failed"
                    }
                    .into();
                    batch.chapters[index].error = Some(code.clone());
                    Ok(())
                })?;
                if code == "speech_cancelled" {
                    cancelled = true;
                }
                // A settings change or stale source invalidates the batch
                // snapshot. Keep completed output, then stop all later dispatch.
                if matches!(
                    code.as_str(),
                    "speech_cancelled"
                        | "chapter_settings_changed"
                        | "stale_source"
                        | "stale_project"
                        | "invalid_chapter_owner"
                ) {
                    self.update_batch(&run.id, |batch| {
                        for item in batch.chapters.iter_mut().skip(index + 1) {
                            item.status = "cancelled".into();
                            item.error = Some("batch_stopped_before_dispatch".into());
                        }
                        Ok(())
                    })?;
                    break;
                }
            }
        }
        let manifest = self.update_batch(&run.id, |batch| {
            batch.status = if cancelled {
                "cancelled"
            } else if any_failure {
                "failed"
            } else {
                "completed"
            }
            .into();
            batch.error = batch_error;
            Ok(())
        })?;
        let batch = manifest
            .chapter_batches
            .iter()
            .find(|batch| batch.id == run.id)
            .ok_or_else(|| Error::new("invalid_chapter_batch"))?
            .clone();
        Ok(ChapterBatchOutcome { manifest, batch })
    }

    fn render_one_chapter(
        &self,
        batch: &ChapterBatchRun,
        chapter: &Chapter,
        index: usize,
        control: &SpeechSynthesisControl,
        progress: &Arc<dyn Fn(ChapterBatchProgress) + Send + Sync>,
        execute: &mut impl FnMut(&str, &ExecutionRequest) -> Result<ExecutionOutput>,
    ) -> Result<()> {
        let parent = self.read()?;
        self.validate_batch_snapshot(batch, chapter)?;
        let child = self.chapter_project_for(&parent, &chapter.id)?;
        let before = child.read()?;
        let document = shared_document(&parent.draft, &before.draft);
        let mut current = self.sync_chapter_document(&parent, &child, document)?;
        let plan = child.segment_plan_for(&current)?;
        if !plan.ready || plan.segments.is_empty() {
            return Err(Error::new("script_not_ready"));
        }
        if !plan.adopted {
            current = child.adopt_script(current.version)?;
        }
        let source_id = current
            .source_head
            .clone()
            .ok_or_else(|| Error::new("source_missing"))?;
        let digest = current.draft.digest();
        let definitions = child.compile_segments(&current.draft)?.segments;
        self.update_batch(&batch.id, |batch| {
            let item = &mut batch.chapters[index];
            item.source_revision = Some(source_id.clone());
            item.document_digest = Some(digest.clone());
            item.total_segments = definitions.len();
            item.status = "running".into();
            Ok(())
        })?;
        let existing = current
            .audio
            .iter()
            .filter(|audio| Some(&audio.id) == current.audio_head.as_ref())
            .chain(current.candidates.iter().rev())
            .find(|audio| {
                audio.source_revision == source_id
                    && audio.document_digest == digest
                    && audio.segment_assembly.as_ref().is_none_or(|assembly| {
                        assembly.selections.iter().all(|selection| {
                            current
                                .segment_selections
                                .get(&selection.key)
                                .is_none_or(|id| id == &selection.segment_id)
                        })
                    })
            })
            .cloned();
        if let Some(audio) = existing {
            if audio.segment_assembly.is_some() {
                child.validate_segment_assembly(&current, &audio)?;
            } else if crate::audio::parse_pcm_s16le_wav(
                &child.object(&audio.content)?,
                audio.contract.origin,
            )? != audio.contract
            {
                return Err(Error::new("corrupt_audio"));
            }
            self.validate_batch_snapshot(batch, chapter)?;
            self.update_batch(&batch.id, |batch| {
                let item = &mut batch.chapters[index];
                item.status = "skipped".into();
                item.completed_segments = definitions.len();
                item.reused_segments = definitions.len();
                item.audio_id = Some(audio.id);
                Ok(())
            })?;
            progress(ChapterBatchProgress {
                batch_id: batch.id.clone(),
                chapter_id: chapter.id.clone(),
                chapter_index: index,
                total_chapters: batch.chapters.len(),
                completed_segments: definitions.len(),
                total_segments: definitions.len(),
                status: "skipped".into(),
            });
            return Ok(());
        }
        let mut selections = Vec::with_capacity(definitions.len());
        let mut reused = 0;
        for definition in &definitions {
            self.validate_batch_snapshot(batch, chapter)?;
            if control.is_cancelled() {
                return Err(Error::new("speech_cancelled"));
            }
            current = child.read()?;
            if current.source_head.as_ref() != Some(&source_id) || current.draft.digest() != digest
            {
                return Err(Error::new("stale_source"));
            }
            // Honor a prior explicit segment choice first, then reuse the most
            // recent exact candidate. Bytes and receipts are revalidated here.
            let selected_id = current.segment_selections.get(&definition.key);
            let available = selected_id
                .and_then(|id| {
                    current
                        .accepted_segments
                        .iter()
                        .find(|record| &record.id == id)
                })
                .or_else(|| {
                    current
                        .segment_candidates
                        .iter()
                        .rev()
                        .chain(current.accepted_segments.iter().rev())
                        .find(|record| record.key == definition.key)
                })
                .cloned();
            let selected = if let Some(record) = available {
                child.validate_segment(definition, &record)?;
                reused += 1;
                record
            } else {
                current = child.render_segment_using(
                    current.version,
                    &definition.key,
                    SpeechSynthesisControl::default(),
                    Arc::new(|_, _| {}),
                    |request| {
                        // Recheck immediately at the dispatch boundary, after the
                        // child's durable reservation, so global voice changes
                        // cannot silently send a different source snapshot.
                        self.validate_batch_snapshot(batch, chapter)?;
                        execute(&chapter.id, request)
                    },
                )?;
                current
                    .segment_candidates
                    .iter()
                    .rev()
                    .find(|record| record.key == definition.key)
                    .cloned()
                    .ok_or_else(|| Error::new("segment_missing"))?
            };
            selections.push(SegmentSelection {
                key: definition.key.clone(),
                segment_id: selected.id,
            });
            self.update_batch(&batch.id, |batch| {
                let item = &mut batch.chapters[index];
                item.completed_segments = selections.len();
                item.reused_segments = reused;
                Ok(())
            })?;
            progress(ChapterBatchProgress {
                batch_id: batch.id.clone(),
                chapter_id: chapter.id.clone(),
                chapter_index: index,
                total_chapters: batch.chapters.len(),
                completed_segments: selections.len(),
                total_segments: definitions.len(),
                status: "running".into(),
            });
        }
        self.validate_batch_snapshot(batch, chapter)?;
        if control.is_cancelled() {
            return Err(Error::new("speech_cancelled"));
        }
        current = child.read()?;
        if current.source_head.as_ref() != Some(&source_id) || current.draft.digest() != digest {
            return Err(Error::new("stale_source"));
        }
        let assembled = child.assemble_segment_selection(current.version, &selections)?;
        let audio = assembled
            .candidates
            .last()
            .ok_or_else(|| Error::new("audio_missing"))?;
        self.validate_batch_snapshot(batch, chapter)?;
        self.update_batch(&batch.id, |batch| {
            let item = &mut batch.chapters[index];
            item.status = "completed".into();
            item.audio_id = Some(audio.id.clone());
            Ok(())
        })?;
        progress(ChapterBatchProgress {
            batch_id: batch.id.clone(),
            chapter_id: chapter.id.clone(),
            chapter_index: index,
            total_chapters: batch.chapters.len(),
            completed_segments: definitions.len(),
            total_segments: definitions.len(),
            status: "completed".into(),
        });
        Ok(())
    }

    fn validate_batch_snapshot(&self, batch: &ChapterBatchRun, chapter: &Chapter) -> Result<()> {
        let current = self.read()?;
        if settings_digest(&current.draft) != batch.settings_digest {
            return Err(Error::new("chapter_settings_changed"));
        }
        if !current.chapters.iter().any(|record| {
            record.id == chapter.id && record.project_id == chapter.project_id && !record.archived
        }) {
            return Err(Error::new("chapter_missing"));
        }
        self.chapter_project_for(&current, &chapter.id)?;
        Ok(())
    }

    fn update_batch(
        &self,
        id: &str,
        f: impl FnOnce(&mut ChapterBatchRun) -> Result<()>,
    ) -> Result<Manifest> {
        let _lock = self.lock()?;
        let mut m = self.read()?;
        let batch = m
            .chapter_batches
            .iter_mut()
            .find(|batch| batch.id == id)
            .ok_or_else(|| Error::new("invalid_chapter_batch"))?;
        f(batch)?;
        self.publish_chapter_manifest(m)
    }
}
