//! Independent described-voice candidates, durable batch receipts and explicit acceptance.
use super::{Error, Manifest, Project, Result};
use crate::{
    AudioOriginDisclosure, AuthorizedVoiceReference, RevisionId, SpeechSynthesisControl,
    VoiceAuthorizationScope,
    audio::parse_pcm_s16le_wav,
    infer_runtime::voice_design::{VoiceDesignReceipt, VoiceDesignSpec},
};
use std::{fs::OpenOptions, sync::Arc};

mod model;
pub(crate) use model::validate_records;
pub use model::{
    AcceptedVoiceDesign, VoiceDesignCandidate, VoiceDesignOutcome, VoiceDesignRun, VoiceDesignState,
};

pub(super) fn recover_records(manifest: &mut Manifest) -> bool {
    let mut changed = false;
    for run in &mut manifest.voice_design_runs {
        if run.status == "running" {
            run.status = "interrupted_unknown".into();
            changed = true;
        }
    }
    changed
}

impl Project {
    pub fn generate_voice_design_batch(
        &self,
        expected: u64,
        endpoint: &str,
        spec: VoiceDesignSpec,
        variations: u8,
        control: &SpeechSynthesisControl,
        progress: Arc<dyn Fn(usize, usize) + Send + Sync>,
    ) -> Result<VoiceDesignOutcome> {
        self.generate_voice_design_using(
            expected,
            spec,
            variations,
            control,
            |spec, control| crate::infer_runtime::voice_design::generate(endpoint, spec, control),
            progress.as_ref(),
        )
    }

    fn generate_voice_design_using(
        &self,
        expected: u64,
        spec: VoiceDesignSpec,
        variations: u8,
        control: &SpeechSynthesisControl,
        mut generate: impl FnMut(
            VoiceDesignSpec,
            &SpeechSynthesisControl,
        ) -> Result<(Vec<u8>, VoiceDesignReceipt)>,
        progress: &dyn Fn(usize, usize),
    ) -> Result<VoiceDesignOutcome> {
        spec.validate()?;
        if !(1..=4).contains(&variations) {
            return Err(Error::new("invalid_voice_design"));
        }
        let producer = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.path.join("render.lock"))?;
        producer.try_lock().map_err(|_| Error::new("render_busy"))?;
        let run_id = uuid::Uuid::now_v7().to_string();
        let (manifest, _) = self.update(expected, |manifest| {
            if manifest.voice_design_candidates.len() + usize::from(variations) > 128
                || manifest.voice_design_runs.len() >= 256
            {
                return Err(Error::new("project_limit"));
            }
            recover_records(manifest);
            manifest.voice_design_runs.push(VoiceDesignRun {
                id: run_id.clone(),
                spec: spec.clone(),
                requested: variations,
                completed: 0,
                candidate_ids: Vec::new(),
                status: "running".into(),
                error: None,
                active_variation: None,
            });
            Ok(())
        })?;
        let mut outcome = VoiceDesignOutcome {
            manifest,
            run_id: run_id.clone(),
            requested: variations,
            completed: 0,
            state: VoiceDesignState::Completed,
            error: None,
        };
        progress(0, usize::from(variations));
        for variation in 1..=variations {
            if control.is_cancelled() {
                outcome.state = VoiceDesignState::Cancelled;
                break;
            }
            self.update_voice_design_run(&run_id, |manifest, run_index| {
                manifest.voice_design_runs[run_index].active_variation = Some(variation);
                Ok(())
            })?;
            let result = generate(spec.clone(), control).and_then(|(bytes, receipt)| {
                if receipt.spec != spec {
                    return Err(Error::new("invalid_provenance"));
                }
                receipt.validate(&bytes)?;
                let candidate = VoiceDesignCandidate {
                    id: uuid::Uuid::now_v7().to_string(),
                    run_id: run_id.clone(),
                    variation,
                    variations,
                    original: self.put_object(&bytes, "audio/wav")?,
                    contract: parse_pcm_s16le_wav(&bytes, AudioOriginDisclosure::SyntheticSpeech)?,
                    generation: receipt,
                    accepted_reference: None,
                };
                self.update_voice_design_run(&run_id, |manifest, run_index| {
                    if manifest.voice_design_candidates.len() >= 128 {
                        return Err(Error::new("project_limit"));
                    }
                    let run = &mut manifest.voice_design_runs[run_index];
                    if run.completed + 1 != variation || run.spec != spec {
                        return Err(Error::new("invalid_voice_design"));
                    }
                    run.completed += 1;
                    run.candidate_ids.push(candidate.id.clone());
                    run.active_variation = None;
                    manifest.voice_design_candidates.push(candidate);
                    Ok(())
                })
            });
            match result {
                Ok(manifest) => {
                    outcome.manifest = manifest;
                    outcome.completed += 1;
                    progress(usize::from(outcome.completed), usize::from(variations));
                }
                Err(error) => {
                    outcome.state = if control.is_cancelled() {
                        VoiceDesignState::Cancelled
                    } else if outcome.completed == 0 {
                        VoiceDesignState::Failed
                    } else {
                        VoiceDesignState::PartialFailed
                    };
                    outcome.error = Some(error.code);
                    break;
                }
            }
        }
        outcome.manifest = self.update_voice_design_run(&run_id, |manifest, run_index| {
            let run = &mut manifest.voice_design_runs[run_index];
            run.status = match outcome.state {
                VoiceDesignState::Completed => "completed",
                VoiceDesignState::Cancelled => "cancelled",
                VoiceDesignState::PartialFailed => "partial_failed",
                VoiceDesignState::Failed => "failed",
            }
            .into();
            run.error = outcome.error.clone();
            run.active_variation = None;
            Ok(())
        })?;
        Ok(outcome)
    }

    /// Generation does not depend on the editable script. Publish its exact result under the
    /// manifest lock while preserving independently saved drafts and other completed materials.
    fn update_voice_design_run(
        &self,
        run_id: &str,
        update: impl FnOnce(&mut Manifest, usize) -> Result<()>,
    ) -> Result<Manifest> {
        let _lock = self.lock()?;
        let mut manifest = self.read()?;
        let index = manifest
            .voice_design_runs
            .iter()
            .position(|run| run.id == run_id && run.status == "running")
            .ok_or_else(|| Error::new("invalid_voice_design"))?;
        update(&mut manifest, index)?;
        validate_records(&manifest)?;
        manifest.version = manifest
            .version
            .checked_add(1)
            .ok_or_else(|| Error::new("project_limit"))?;
        self.write(&manifest)?;
        Ok(manifest)
    }

    pub fn accept_voice_design(&self, expected: u64, id: &str, name: &str) -> Result<Manifest> {
        if !super::materials::valid_name(name) {
            return Err(Error::new("invalid_material"));
        }
        self.update(expected, |manifest| {
            if manifest.voice_library.len() >= 128 {
                return Err(Error::new("project_limit"));
            }
            let index = manifest
                .voice_design_candidates
                .iter()
                .position(|candidate| candidate.id == id)
                .ok_or_else(|| Error::new("candidate_missing"))?;
            let candidate = &manifest.voice_design_candidates[index];
            if candidate.accepted_reference.is_some() {
                return Err(Error::new("voice_design_already_accepted"));
            }
            let bytes = self.object(&candidate.original)?;
            candidate.generation.validate(&bytes)?;
            if parse_pcm_s16le_wav(&bytes, AudioOriginDisclosure::SyntheticSpeech)?
                != candidate.contract
            {
                return Err(Error::new("corrupt_audio"));
            }
            let mut reference = AuthorizedVoiceReference::new(
                RevisionId::new(),
                format!("synthetic-design-acceptance:{}", candidate.id),
                VoiceAuthorizationScope::VoiceClone,
                true,
                true,
            )
            .map_err(|_| Error::new("invalid_reference"))?;
            reference.audio = Some(candidate.original.clone());
            reference.transcript = Some(candidate.generation.spec.text.clone());
            let voice = super::materials::ReferenceVoice {
                id: reference.source_revision_id.to_string(),
                name: name.into(),
                reference,
                contract: candidate.contract.clone(),
                origin: "designed".into(),
                design: Some(AcceptedVoiceDesign::from_candidate(candidate)),
            };
            manifest.voice_design_candidates[index].accepted_reference = Some(voice.id.clone());
            manifest.voice_library.push(voice);
            super::materials::validate_libraries(manifest)?;
            validate_records(manifest)?;
            Ok(())
        })
        .map(|result| result.0)
    }

    pub fn discard_voice_design(&self, expected: u64, id: &str) -> Result<Manifest> {
        self.update(expected, |manifest| {
            let index = manifest
                .voice_design_candidates
                .iter()
                .position(|candidate| candidate.id == id && candidate.accepted_reference.is_none())
                .ok_or_else(|| Error::new("candidate_missing"))?;
            manifest.voice_design_candidates.remove(index);
            Ok(())
        })
        .map(|result| result.0)
    }
}

#[cfg(test)]
mod tests;
