use super::{
    SegmentRevision,
    plan::{SegmentDefinition, adopted_source},
};
use crate::{
    AudioOriginDisclosure, CapabilityId, ContentDigest, ContentRef, ExecutionInput,
    ExecutionOutput, ExecutionRequest, Executor, ExternalExecutionProvenance,
    InferRuntimeSpeechExecutor, SpeechSynthesisControl, SpeechVoiceSelection, TransformationId,
    project::{Error, Manifest, Project, RenderRun, Result},
};
use std::{fs::OpenOptions, path::PathBuf, sync::Arc};

impl Project {
    pub fn render_segment(
        &self,
        expected: u64,
        key: &str,
        endpoint: &str,
        control: SpeechSynthesisControl,
        progress: Arc<dyn Fn(usize, usize) + Send + Sync>,
    ) -> Result<Manifest> {
        let synthesis_control = control.clone();
        self.render_segment_using(expected, key, control, progress, |request| {
            let executor =
                InferRuntimeSpeechExecutor::new(endpoint, crate::infer_runtime::credential_path())
                    .map_err(|_| Error::new("credential_unavailable"))?
                    .with_control(synthesis_control);
            Ok(executor.execute(request)?)
        })
    }

    pub(in crate::project) fn render_segment_using(
        &self,
        expected: u64,
        key: &str,
        control: SpeechSynthesisControl,
        progress: Arc<dyn Fn(usize, usize) + Send + Sync>,
        execute: impl FnOnce(&ExecutionRequest) -> Result<ExecutionOutput>,
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
        let source = adopted_source(&m)?.clone();
        let definition = self
            .compile_segments(&source.document)?
            .segments
            .into_iter()
            .find(|s| s.key == key)
            .ok_or_else(|| Error::new("segment_missing"))?;
        if control.is_cancelled() {
            return Err(Error::new("speech_cancelled"));
        }
        let text = definition.text.as_bytes().to_vec();
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
        if let SpeechVoiceSelection::AuthorizedReference(reference) = &definition.operation.voice {
            let content = reference
                .audio
                .as_ref()
                .ok_or_else(|| Error::new("reference_missing"))?;
            inputs.push(
                ExecutionInput::materialized(content.clone(), self.object(content)?)
                    .map_err(|_| Error::new("invalid_reference"))?,
            );
        }
        let request = ExecutionRequest::new_materialized(
            TransformationId::new(),
            CapabilityId::new("audio.speech_synthesize")
                .map_err(|_| Error::new("invalid_request"))?,
            inputs,
            serde_json::to_vec(&definition.operation)?,
            "audio/wav",
        )
        .map_err(|_| Error::new("invalid_request"))?;
        let run_id = uuid::Uuid::now_v7().to_string();
        let (reserved, _) = self.update(expected, |m| {
            if m.runs.len() >= 2048
                || m.segment_candidates.len() + m.accepted_segments.len() >= 4096
            {
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
            progress(0, 1);
            let output = execute(&request)?;
            if control.is_cancelled() {
                return Err(Error::new("speech_cancelled"));
            }
            let contract = crate::audio::parse_pcm_s16le_wav(
                &output.bytes,
                AudioOriginDisclosure::SyntheticSpeech,
            )?;
            let record = SegmentRevision {
                id: uuid::Uuid::now_v7().to_string(),
                source_revision: source.id.clone(),
                document_digest: source.document.digest(),
                key: definition.key.clone(),
                text: definition.text.clone(),
                spoken_text: definition.spoken_text.clone(),
                operation: definition.operation.clone(),
                content: self.put_object(&output.bytes, "audio/wav")?,
                contract,
                provenance: output
                    .external_provenance
                    .ok_or_else(|| Error::new("invalid_provenance"))?,
                job_id: output
                    .executor_job_id
                    .ok_or_else(|| Error::new("invalid_provenance"))?,
            };
            self.validate_segment(&definition, &record)?;
            let (m, _) = self.update(reserved.version, |m| {
                if m.source_head.as_ref() != Some(&source.id) || m.draft != source.document {
                    return Err(Error::new("stale_source"));
                }
                m.segment_candidates.push(record);
                let run = m
                    .runs
                    .iter_mut()
                    .find(|r| r.id == run_id)
                    .ok_or_else(|| Error::new("invalid_run"))?;
                run.status = "completed".into();
                Ok(())
            })?;
            progress(1, 1);
            Ok(m)
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

    pub fn accept_segment(&self, expected: u64, id: &str) -> Result<Manifest> {
        self.update(expected, |m| {
            let source = adopted_source(m)?;
            if let Some(record) = m.accepted_segments.iter().find(|r| r.id == id) {
                let definition = self
                    .compile_segments(&source.document)?
                    .segments
                    .into_iter()
                    .find(|s| s.key == record.key)
                    .ok_or_else(|| Error::new("stale_segment"))?;
                self.validate_segment(&definition, record)?;
                m.segment_selections
                    .insert(record.key.clone(), record.id.clone());
                return Ok(());
            }
            let index = m
                .segment_candidates
                .iter()
                .position(|r| r.id == id)
                .ok_or_else(|| Error::new("segment_candidate_missing"))?;
            let record = &m.segment_candidates[index];
            if record.source_revision != source.id
                || record.document_digest != source.document.digest()
            {
                return Err(Error::new("stale_source"));
            }
            let definition = self
                .compile_segments(&source.document)?
                .segments
                .into_iter()
                .find(|s| s.key == record.key)
                .ok_or_else(|| Error::new("stale_segment"))?;
            self.validate_segment(&definition, record)?;
            let record = m.segment_candidates.remove(index);
            m.segment_selections
                .insert(record.key.clone(), record.id.clone());
            m.accepted_segments.push(record);
            Ok(())
        })
        .map(|x| x.0)
    }

    pub fn segment_revision(&self, id: &str) -> Result<SegmentRevision> {
        let m = self.read()?;
        m.segment_candidates
            .iter()
            .chain(&m.accepted_segments)
            .find(|r| r.id == id)
            .cloned()
            .ok_or_else(|| Error::new("segment_missing"))
    }

    pub fn segment_playback(&self, id: &str) -> Result<PathBuf> {
        let record = self.segment_revision(id)?;
        let bytes = self.object(&record.content)?;
        if crate::audio::parse_pcm_s16le_wav(&bytes, record.contract.origin)? != record.contract {
            return Err(Error::new("corrupt_audio"));
        }
        let directory = std::env::temp_dir()
            .join("vox-segment-audition")
            .join(self.read()?.id);
        crate::project::store::private_directory(&directory)?;
        let path = directory.join(format!("{}.wav", record.content.digest));
        if path.exists() {
            if std::fs::read(&path)? != bytes {
                return Err(Error::new("corrupt_object"));
            }
        } else {
            use std::io::Write;
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            options.open(&path)?.write_all(&bytes)?;
        }
        Ok(path)
    }

    pub(in crate::project) fn validate_segment(
        &self,
        definition: &SegmentDefinition,
        r: &SegmentRevision,
    ) -> Result<()> {
        if r.key != definition.key
            || r.text != definition.text
            || r.spoken_text != definition.spoken_text
            || r.operation != definition.operation
        {
            return Err(Error::new("stale_segment"));
        }
        let bytes = self.object(&r.content)?;
        if crate::audio::parse_pcm_s16le_wav(&bytes, AudioOriginDisclosure::SyntheticSpeech)?
            != r.contract
            || r.contract.sample_rate_hz != 24_000
            || r.contract.channels != 1
        {
            return Err(Error::new("corrupt_audio"));
        }
        let runtime = leaf_provenance(r)?;
        let (intent, capability, deployment) = match &r.operation.voice {
            SpeechVoiceSelection::Preset(_) => (
                "speech.synthesize",
                "infer.audio.speech@20260811.1",
                "mlx_qwen3_tts_custom_voice_1_7b",
            ),
            SpeechVoiceSelection::AuthorizedReference(_) => (
                "speech.clone_voice",
                "infer.audio.voice-clone@20260811.1",
                "mlx_qwen3_tts_base_1_7b",
            ),
        };
        if !r.provenance.is_bounded()
            || runtime.app_id != "vox"
            || runtime.intent != intent
            || runtime.capability_contract.as_deref() != Some(capability)
            || runtime.deployment != deployment
            || runtime.placement != "local"
            || runtime.requested_placement != "local_only"
            || !runtime.offline_required
            || runtime.fallback != "none"
            || runtime.max_cost_microusd != 0
        {
            return Err(Error::new("invalid_provenance"));
        }
        Ok(())
    }
}

pub(super) fn leaf_provenance(r: &SegmentRevision) -> Result<ExternalExecutionProvenance> {
    let p = &r.provenance;
    if p.speech_script.is_some() {
        return Err(Error::new("invalid_provenance"));
    }
    if p.speech_segments.is_empty() {
        return if r.text == r.spoken_text {
            Ok(p.clone())
        } else {
            Err(Error::new("invalid_provenance"))
        };
    }
    if p.speech_segments.len() != 1 {
        return Err(Error::new("invalid_provenance"));
    }
    let receipt = &p.speech_segments[0];
    if receipt.job_id != r.job_id
        || receipt.input_start != 0
        || receipt.input_end as usize != r.text.len()
        || receipt.input_digest != ContentDigest::from_bytes(r.text.as_bytes())
        || receipt.spoken_digest
            != (r.text != r.spoken_text)
                .then(|| ContentDigest::from_bytes(r.spoken_text.as_bytes()))
        || receipt.output_digest != r.content.digest
        || receipt.frames != r.contract.frame_count
    {
        return Err(Error::new("invalid_provenance"));
    }
    Ok(*receipt.runtime.clone())
}
