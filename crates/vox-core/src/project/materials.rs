//! Immutable reference voices and reviewable sound materials within one project.
use super::{Error, Manifest, Project, Result};
use crate::{
    AudioOriginDisclosure, AudioValueContract, AuthorizedVoiceReference, ContentRef, RevisionId,
    VoiceAuthorizationScope, audio::parse_pcm_s16le_wav,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceVoice {
    pub id: String,
    pub name: String,
    pub reference: AuthorizedVoiceReference,
    pub contract: AudioValueContract,
    pub origin: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundMaterial {
    pub id: String,
    pub name: String,
    pub original: ContentRef,
    /// Prepared script PCM is created only by explicit acceptance.
    pub content: Option<ContentRef>,
    pub contract: AudioValueContract,
    pub origin: String,
    pub generation: Option<crate::infer_runtime::sound::SoundReceipt>,
}
fn valid_name(name: &str) -> bool {
    !name.trim().is_empty() && name.len() <= 256 && !name.chars().any(char::is_control)
}
pub(super) fn validate_libraries(m: &Manifest) -> Result<()> {
    if m.voice_library.iter().any(|v| {
        !valid_name(&v.name)
            || v.id != v.reference.source_revision_id.to_string()
            || !v.reference.is_executable()
    }) || m
        .sound_library
        .iter()
        .any(|v| !valid_name(&v.name) || uuid::Uuid::parse_str(&v.id).is_err())
    {
        return Err(Error::new("invalid_material"));
    }
    // Bindings must match a complete immutable project-owned source, never an arbitrary path/claim.
    for document in std::iter::once(&m.draft).chain(m.sources.iter().map(|s| &s.document)) {
        if super::shape_transfer::unsupported(m, document) {
            continue;
        }
        for reference in document
            .reference_voice
            .iter()
            .chain(document.options.reference_roles.values())
        {
            if !m.voice_library.iter().any(|v| &v.reference == reference) {
                return Err(Error::new("reference_missing"));
            }
        }
    }
    Ok(())
}
impl Project {
    pub fn import_reference(
        &self,
        expected: u64,
        name: &str,
        transcript: &str,
        file: &Path,
        consent: bool,
        recorded: bool,
    ) -> Result<Manifest> {
        if !consent {
            return Err(Error::new("voice_consent_required"));
        }
        if !valid_name(name) || transcript.trim().is_empty() || transcript.len() > 16_384 {
            return Err(Error::new("reference_text_required"));
        }
        let bytes = super::store::read_bounded(file, 25 * 1024 * 1024)?;
        let contract = parse_pcm_s16le_wav(
            &bytes,
            if recorded {
                AudioOriginDisclosure::RecordedSource
            } else {
                AudioOriginDisclosure::ImportedUnverified
            },
        )?;
        // Bounded local reference; very long or effectively empty captures are not useful model input.
        if contract.channels > 2
            || contract.duration_millis() < 500
            || contract.duration_millis() > 120_000
        {
            return Err(Error::new("reference_duration"));
        }
        let audio = self.put_object(&bytes, "audio/wav")?;
        let mut reference = AuthorizedVoiceReference::new(
            RevisionId::new(),
            uuid::Uuid::now_v7().to_string(),
            VoiceAuthorizationScope::VoiceClone,
            true,
            true,
        )
        .map_err(|_| Error::new("invalid_reference"))?;
        reference.audio = Some(audio);
        reference.transcript = Some(transcript.into());
        self.update(expected, |m| {
            if m.voice_library.len() >= 128 {
                return Err(Error::new("project_limit"));
            }
            m.voice_library.push(ReferenceVoice {
                id: reference.source_revision_id.to_string(),
                name: name.into(),
                reference,
                contract,
                origin: if recorded { "recorded" } else { "imported" }.into(),
            });
            Ok(())
        })
        .map(|x| x.0)
    }
    pub fn bind_reference(&self, expected: u64, role: &str, id: &str) -> Result<Manifest> {
        self.update(expected, |m| {
            let reference = m
                .voice_library
                .iter()
                .find(|v| v.id == id)
                .ok_or_else(|| Error::new("reference_missing"))?
                .reference
                .clone();
            m.draft.speed_milli = 1000;
            if role.is_empty() {
                m.draft.reference_voice = Some(reference);
            } else {
                m.draft.options.roles.remove(role);
                m.draft
                    .options
                    .reference_roles
                    .insert(role.into(), reference);
            }
            m.draft.operation()?;
            Ok(())
        })
        .map(|x| x.0)
    }
    pub fn add_sound(
        &self,
        expected: u64,
        name: &str,
        bytes: &[u8],
        origin: &str,
        generation: Option<crate::infer_runtime::sound::SoundReceipt>,
    ) -> Result<Manifest> {
        if !valid_name(name) {
            return Err(Error::new("invalid_material"));
        }
        if origin == "generated" {
            generation
                .as_ref()
                .ok_or_else(|| Error::new("invalid_provenance"))?
                .validate(bytes)?;
        } else if !matches!(origin, "builtin" | "imported") || generation.is_some() {
            return Err(Error::new("invalid_material"));
        }
        let contract = parse_pcm_s16le_wav(
            bytes,
            if matches!(origin, "generated" | "builtin") {
                AudioOriginDisclosure::SyntheticSound
            } else {
                AudioOriginDisclosure::ImportedUnverified
            },
        )?;
        if contract.duration_millis() > 120_000 || contract.channels > 2 {
            return Err(Error::new("sound_too_long"));
        }
        let original = self.put_object(bytes, "audio/wav")?;
        self.update(expected, |m| {
            if m.sound_library.len() >= 256 {
                return Err(Error::new("project_limit"));
            }
            m.sound_library.push(SoundMaterial {
                id: uuid::Uuid::now_v7().to_string(),
                name: name.into(),
                original,
                content: None,
                contract,
                origin: origin.into(),
                generation,
            });
            Ok(())
        })
        .map(|x| x.0)
    }
    pub fn import_sound(&self, expected: u64, name: &str, file: &Path) -> Result<Manifest> {
        let bytes = super::store::read_bounded(file, 25 * 1024 * 1024)?;
        self.add_sound(expected, name, &bytes, "imported", None)
    }
    pub fn accept_sound(&self, expected: u64, id: &str) -> Result<Manifest> {
        let m = self.read()?;
        if m.version != expected {
            return Err(Error::new("stale_project"));
        }
        let material = m
            .sound_library
            .iter()
            .find(|s| s.id == id)
            .ok_or_else(|| Error::new("material_missing"))?;
        let bytes = self.object(&material.original)?;
        if let Some(receipt) = &material.generation {
            receipt.validate(&bytes)?;
        }
        let prepared = crate::materials::script_wav(&bytes)?;
        crate::assembly::validate_cue(&prepared)?;
        let content = self.put_object(&prepared, "audio/wav")?;
        self.update(expected, |m| {
            m.sound_library
                .iter_mut()
                .find(|s| s.id == id)
                .ok_or_else(|| Error::new("material_missing"))?
                .content = Some(content);
            Ok(())
        })
        .map(|x| x.0)
    }
    pub fn bind_sound(&self, expected: u64, label: &str, id: &str) -> Result<Manifest> {
        self.update(expected, |m| {
            let content = m
                .sound_library
                .iter()
                .find(|s| s.id == id)
                .and_then(|s| s.content.clone())
                .ok_or_else(|| Error::new("material_not_accepted"))?;
            crate::assembly::validate_cue(&self.object(&content)?)?;
            m.draft.options.cues.insert(
                label.into(),
                crate::speech_script::SpeechCueAction::Audio { content },
            );
            m.draft.operation()?;
            Ok(())
        })
        .map(|x| x.0)
    }
    pub fn discard_sound(&self, expected: u64, id: &str) -> Result<Manifest> {
        self.update(expected, |m| {
            let index = m
                .sound_library
                .iter()
                .position(|s| s.id == id && s.content.is_none())
                .ok_or_else(|| Error::new("candidate_missing"))?;
            m.sound_library.remove(index);
            Ok(())
        })
        .map(|x| x.0)
    }
    pub fn material_file(&self, kind: &str, id: &str) -> Result<std::path::PathBuf> {
        let m = self.read()?;
        let content = if kind == "voice" {
            m.voice_library
                .iter()
                .find(|s| s.id == id)
                .and_then(|s| s.reference.audio.as_ref())
        } else {
            m.sound_library
                .iter()
                .find(|s| s.id == id)
                .map(|s| &s.original)
        }
        .ok_or_else(|| Error::new("material_missing"))?;
        let bytes = self.object(content)?;
        let dir = std::env::temp_dir().join("vox-materials").join(&m.id);
        super::store::private_directory(&dir)?;
        let path = dir.join(format!("{}.wav", content.digest));
        std::fs::write(&path, bytes)?;
        Ok(path)
    }
    pub fn export_material(&self, kind: &str, id: &str, destination: &Path) -> Result<()> {
        use std::io::Write;
        let bytes = std::fs::read(self.material_file(kind, id)?)?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)
            .map_err(|_| Error::new("destination_exists"))?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        Ok(())
    }
}

impl Project {
    pub fn generate_sound(
        &self,
        expected: u64,
        endpoint: &str,
        name: &str,
        spec: crate::infer_runtime::sound::SoundSpec,
        control: &crate::SpeechSynthesisControl,
    ) -> Result<Manifest> {
        let lock = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.path.join("render.lock"))?;
        lock.try_lock().map_err(|_| Error::new("render_busy"))?;
        if self.read()?.version != expected {
            return Err(Error::new("stale_project"));
        }
        let (bytes, receipt) = crate::infer_runtime::sound::generate(endpoint, spec, control)?;
        self.add_sound(expected, name, &bytes, "generated", Some(receipt))
    }
}

impl Project {
    pub fn material_echo(&self, kind: &str, id: &str) -> Result<std::path::PathBuf> {
        let source = std::fs::read(self.material_file(kind, id)?)?;
        let m = self.read()?;
        let generated = kind == "sound"
            && m.sound_library
                .iter()
                .any(|s| s.id == id && s.generation.is_some());
        let bytes = if generated {
            crate::handoff::with_generated_disclosure(&source)?
        } else {
            source
        };
        let dir = std::env::temp_dir()
            .join("vox-material-edits")
            .join(uuid::Uuid::now_v7().to_string());
        super::store::private_directory(&dir)?;
        let path = dir.join("source.wav");
        std::fs::write(&path, &bytes)?;
        std::fs::write(
            dir.join("source.json"),
            serde_json::to_vec_pretty(
                &serde_json::json!({"schema":"vox.material-working-copy/1","project":m.id,"kind":kind,"material":id,"working_digest":crate::ContentDigest::from_bytes(&bytes),"return":"import a new reviewed material; original stays immutable"}),
            )?,
        )?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests;
