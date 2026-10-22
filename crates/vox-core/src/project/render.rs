use super::*;
use crate::{
    AudioOriginDisclosure, CapabilityId, ContentDigest, ContentRef, ExecutionInput,
    ExecutionRequest, Executor, InferRuntimeSpeechExecutor, SpeechSynthesisControl,
    TransformationId,
};
use std::{fs::OpenOptions, sync::Arc};
impl Project {
    pub fn render_inputs(&self, document: &Document) -> Result<Vec<ExecutionInput>> {
        let text = document.text.as_bytes().to_vec();
        let content = ContentRef::new(
            ContentDigest::from_bytes(&text),
            "text/plain; charset=utf-8",
            text.len() as u64,
        )
        .map_err(|_| Error::new("invalid_content"))?;
        let mut inputs = vec![
            ExecutionInput::materialized(content, text)
                .map_err(|_| Error::new("invalid_content"))?,
        ];
        let script_options =
            (document.format == ScriptFormat::ProductionScript).then_some(&document.options);
        for cue in script_options.into_iter().flat_map(|o| o.cues.values()) {
            if let crate::speech_script::SpeechCueAction::Audio { content } = cue {
                inputs.push(
                    ExecutionInput::materialized(content.clone(), self.object(content)?)
                        .map_err(|_| Error::new("invalid_cue"))?,
                );
            }
        }
        for reference in document.reference_voice.iter().chain(
            script_options
                .into_iter()
                .flat_map(|o| o.reference_roles.values()),
        ) {
            let content = reference
                .audio
                .as_ref()
                .ok_or_else(|| Error::new("reference_missing"))?;
            if !inputs.iter().any(|i| i.content() == content) {
                inputs.push(
                    ExecutionInput::materialized(content.clone(), self.object(content)?)
                        .map_err(|_| Error::new("invalid_reference"))?,
                );
            }
        }
        Ok(inputs)
    }
    pub fn render(
        &self,
        expected: u64,
        endpoint: &str,
        control: SpeechSynthesisControl,
        progress: Arc<dyn Fn(usize, usize) + Send + Sync>,
    ) -> Result<Manifest> {
        let render_lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.path.join("render.lock"))?;
        render_lock
            .try_lock()
            .map_err(|_| Error::new("render_busy"))?;
        let m = self.read()?;
        if m.version != expected {
            return Err(Error::new("stale_project"));
        }
        let source = m
            .sources
            .iter()
            .find(|s| Some(&s.id) == m.source_head.as_ref())
            .cloned()
            .ok_or_else(|| Error::new("source_missing"))?;
        super::shape_transfer::require_executable(&m, &source.document)?;
        if source.document != m.draft {
            return Err(Error::new("unadopted_script"));
        }
        if !source.document.analysis()["ready"]
            .as_bool()
            .unwrap_or(false)
        {
            return Err(Error::new("script_not_ready"));
        }
        let inputs = self.render_inputs(&source.document)?;
        let op = source.document.operation()?;
        let request = ExecutionRequest::new_materialized(
            TransformationId::new(),
            CapabilityId::new("audio.speech_synthesize")
                .map_err(|_| Error::new("invalid_request"))?,
            inputs,
            serde_json::to_vec(&op)?,
            "audio/wav",
        )
        .map_err(|_| Error::new("invalid_request"))?;
        let run_id = uuid::Uuid::now_v7().to_string();
        let (reserved, _) = self.update(expected, |m| {
            if m.runs.len() >= 2048 || m.candidates.len() + m.audio.len() >= 1024 {
                return Err(Error::new("project_limit"));
            }
            for old in &mut m.runs {
                if old.status == "running" {
                    old.status = "interrupted_unknown".into();
                }
            }
            m.runs.push(RenderRun {
                id: run_id.clone(),
                source_revision: source.id.clone(),
                status: "running".into(),
                error: None,
            });
            Ok(())
        })?;
        let result = (|| {
            let executor =
                InferRuntimeSpeechExecutor::new(endpoint, crate::infer_runtime::credential_path())
                    .map_err(|_| Error::new("credential_unavailable"))?
                    .with_control(control.clone());
            let finished = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let done = finished.clone();
            let observer = control.clone();
            let poller = std::thread::spawn(move || {
                let mut last = (usize::MAX, usize::MAX);
                while !done.load(std::sync::atomic::Ordering::Relaxed) {
                    let now = (observer.completed(), observer.total());
                    if now != last {
                        progress(now.0, now.1);
                        last = now;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            });
            let result = executor.execute(&request);
            finished.store(true, std::sync::atomic::Ordering::Relaxed);
            let _ = poller.join();
            let output = result?;
            if control.is_cancelled() {
                return Err(Error::new("speech_cancelled"));
            }
            let contract = crate::audio::parse_pcm_s16le_wav(
                &output.bytes,
                AudioOriginDisclosure::SyntheticSpeech,
            )?;
            let content = self.put_object(&output.bytes, "audio/wav")?;
            let record = AudioRevision {
                id: uuid::Uuid::now_v7().to_string(),
                source_revision: source.id.clone(),
                document_digest: source.document.digest(),
                content,
                contract,
                provenance: output.external_provenance,
                origin: "synthesis".into(),
                parent_audio: None,
                source_project: None,
                edit: None,
                segment_assembly: None,
            };
            self.update(reserved.version, |m| {
                if m.source_head.as_ref() != Some(&source.id) {
                    return Err(Error::new("stale_source"));
                }
                m.candidates.push(record);
                let run = m
                    .runs
                    .iter_mut()
                    .find(|r| r.id == run_id)
                    .ok_or_else(|| Error::new("invalid_run"))?;
                run.status = "completed".into();
                Ok(())
            })
            .map(|x| x.0)
        })();
        if let Err(error) = &result {
            let _lock = self.lock()?;
            let mut current = self.read()?;
            if let Some(run) = current.runs.iter_mut().find(|r| r.id == run_id) {
                run.status = if error.code == "speech_cancelled" {
                    "cancelled"
                } else {
                    "failed"
                }
                .into();
                run.error = Some(error.code.clone());
                current.version += 1;
                self.write(&current)?;
            }
        }
        result
    }
    pub fn recover(&self) -> Result<Manifest> {
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
        let mut changed = false;
        for run in &mut m.runs {
            if run.status == "running" {
                run.status = "interrupted_unknown".into();
                changed = true;
            }
        }
        changed |= super::voice_design::recover_records(&mut m);
        changed |= super::chapters::recover_records(&mut m);
        if changed {
            m.version += 1;
            self.write(&m)?;
        }
        Ok(m)
    }
}
