//! Bounded independent sound batches: every completed result stays a candidate.
use super::{Error, Manifest, Project, Result, SoundMetadata, valid_name};
use crate::SpeechSynthesisControl;
use crate::infer_runtime::sound::{SoundReceipt, SoundSpec};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoundCategory {
    #[default]
    Sfx,
    Ambience,
    ShortMusic,
}

impl SoundCategory {
    fn constrain(self, spec: &SoundSpec) -> SoundSpec {
        let mut spec = spec.clone();
        spec.music = self == Self::ShortMusic;
        if spec.music {
            spec.prompt
                .push_str(" Instrumental music only. No vocals, singing or speech.");
        }
        spec
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SoundBatchState {
    Completed,
    Cancelled,
    PartialFailed,
}

#[derive(Debug, Serialize)]
pub struct SoundBatchOutcome {
    pub manifest: Manifest,
    pub batch_id: String,
    pub requested: u8,
    pub completed: u8,
    pub state: SoundBatchState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

struct SoundBatchPlan {
    batch_id: String,
    name: String,
    authored_prompt: String,
    category: SoundCategory,
    spec: SoundSpec,
    variations: u8,
}

impl SoundBatchPlan {
    fn new(name: &str, spec: SoundSpec, category: SoundCategory, variations: u8) -> Result<Self> {
        if !valid_name(name) || !(1..=4).contains(&variations) {
            return Err(Error::new("invalid_sound_request"));
        }
        let authored_prompt = spec.prompt.clone();
        let spec = SoundSpec {
            prompt: spec.prompt.replace(['\n', '\r', '\t'], " "),
            ..spec
        };
        spec.validate()?;
        let constrained = category.constrain(&spec);
        constrained.validate()?;
        // Leave room for the variation suffix within the material-name contract.
        if variations > 1 && name.len() > 240 {
            return Err(Error::new("invalid_sound_request"));
        }
        Ok(Self {
            batch_id: uuid::Uuid::now_v7().to_string(),
            name: name.into(),
            authored_prompt,
            category,
            spec: constrained,
            variations,
        })
    }
    fn variant(&self, index: u8) -> SoundSpec {
        let mut spec = self.spec.clone();
        spec.seed = spec.seed.wrapping_add(u32::from(index));
        spec
    }
    fn name(&self, index: u8) -> String {
        if self.variations == 1 {
            self.name.clone()
        } else {
            format!("{} · {}", self.name, index + 1)
        }
    }
    fn metadata(&self, index: u8) -> SoundMetadata {
        SoundMetadata {
            category: self.category,
            authored_prompt: self.authored_prompt.clone(),
            batch_id: self.batch_id.clone(),
            variation: index + 1,
            variations: self.variations,
        }
    }
}

impl Project {
    #[allow(clippy::too_many_arguments)]
    pub fn generate_sound_batch(
        &self,
        expected: u64,
        endpoint: &str,
        name: &str,
        spec: SoundSpec,
        category: SoundCategory,
        variations: u8,
        control: &SpeechSynthesisControl,
    ) -> Result<SoundBatchOutcome> {
        let plan = SoundBatchPlan::new(name, spec, category, variations)?;
        let mut prepared = None;
        self.generate_sound_batch_using(expected, plan, control, |spec, control| {
            let generated = crate::infer_runtime::sound::generate_with_prepared(
                endpoint,
                spec,
                control,
                prepared.clone(),
            )?;
            prepared = Some(generated.1.prepared.clone());
            Ok(generated)
        })
    }

    fn generate_sound_batch_using(
        &self,
        expected: u64,
        plan: SoundBatchPlan,
        control: &SpeechSynthesisControl,
        mut generate: impl FnMut(SoundSpec, &SpeechSynthesisControl) -> Result<(Vec<u8>, SoundReceipt)>,
    ) -> Result<SoundBatchOutcome> {
        let lock = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.path.join("render.lock"))?;
        lock.try_lock().map_err(|_| Error::new("render_busy"))?;
        let manifest = self.read()?;
        if manifest.version != expected {
            return Err(Error::new("stale_project"));
        }
        if manifest.sound_library.len() + usize::from(plan.variations) > 256 {
            return Err(Error::new("project_limit"));
        }
        let mut outcome = SoundBatchOutcome {
            manifest,
            batch_id: plan.batch_id.clone(),
            requested: plan.variations,
            completed: 0,
            state: SoundBatchState::Completed,
            error: None,
        };
        for index in 0..plan.variations {
            if control.is_cancelled() {
                outcome.state = SoundBatchState::Cancelled;
                break;
            }
            let result = generate(plan.variant(index), control).and_then(|(bytes, receipt)| {
                self.add_sound_with_metadata(
                    outcome.manifest.version,
                    &plan.name(index),
                    &bytes,
                    "generated",
                    Some(receipt),
                    Some(plan.metadata(index)),
                )
            });
            match result {
                Ok(manifest) => {
                    outcome.manifest = manifest;
                    outcome.completed += 1;
                }
                Err(error) if control.is_cancelled() => {
                    outcome.state = SoundBatchState::Cancelled;
                    outcome.error = Some(error.code);
                    outcome.manifest = self.read()?;
                    break;
                }
                Err(error) if outcome.completed > 0 => {
                    outcome.state = SoundBatchState::PartialFailed;
                    outcome.error = Some(error.code);
                    outcome.manifest = self.read()?;
                    break;
                }
                Err(error) => return Err(error),
            }
        }
        Ok(outcome)
    }
}

#[cfg(test)]
mod tests;
