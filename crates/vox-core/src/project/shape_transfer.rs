//! Shape file admission and atomic independent-project creation. No Shape database access.
mod loader;
mod schema;
use super::{
    AudioRevision, Document, Error, Manifest, Project, Result, ScriptFormat, SourceRevision,
};
use crate::{
    ContentDigest, SpeechSynthesisControl, SpeechSynthesisOperation, SpeechVoiceSelection,
};
use schema::*;
pub use schema::{ImportReceipt, PreservedDraft, SourcePin};
use std::{
    fs::{self, File, OpenOptions},
    path::Path,
};

pub(crate) fn check_cancel(cancelled: &impl Fn() -> bool) -> Result<()> {
    if cancelled() {
        Err(Error::new("shape_transfer_cancelled"))
    } else {
        Ok(())
    }
}
pub(super) fn unsupported(m: &Manifest, document: &Document) -> bool {
    m.shape_import
        .as_ref()
        .is_some_and(|r| r.unsupported_documents.contains(&document.digest()))
}
pub(super) fn require_executable(m: &Manifest, document: &Document) -> Result<()> {
    if unsupported(m, document) {
        return Err(Error::new("shape_settings_incomplete"));
    }
    document.operation().map(|_| ())
}
impl Manifest {
    pub fn analysis(&self) -> serde_json::Value {
        let mut analysis = self.draft.analysis();
        if unsupported(self, &self.draft) {
            analysis["ready"] = false.into();
            analysis["migration_issues"] =
                serde_json::json!(self.shape_import.as_ref().map(|r| &r.issues));
        }
        analysis
    }
}
impl Project {
    /// Validates an entire package before atomically publishing a new independent `.vox` project.
    /// A matching replay opens the existing project and never writes its manifest.
    pub fn import_shape_transfer(
        transfer: &Path,
        destination: &Path,
        control: &SpeechSynthesisControl,
    ) -> Result<Self> {
        Self::import_shape_transfer_expected(transfer, destination, None, control)
    }
    pub fn import_shape_transfer_expected(
        transfer: &Path,
        destination: &Path,
        expected_blake3: Option<&str>,
        control: &SpeechSynthesisControl,
    ) -> Result<Self> {
        Self::import_shape_transfer_checked(transfer, destination, expected_blake3, &|| {
            control.is_cancelled()
        })
    }
    fn import_shape_transfer_checked(
        transfer: &Path,
        destination: &Path,
        expected_blake3: Option<&str>,
        cancelled: &impl Fn() -> bool,
    ) -> Result<Self> {
        if !destination.is_absolute()
            || destination.extension().and_then(|s| s.to_str()) != Some("vox")
        {
            return Err(Error::new("project_extension"));
        }
        let verified = loader::Verified::load(transfer, cancelled)?;
        let digest = ContentDigest::from_bytes(&verified.raw).to_string();
        if expected_blake3.is_some_and(|expected| expected != digest) {
            return Err(Error::new("shape_transfer_hash"));
        }
        let parent = destination.parent().ok_or_else(invalid)?.canonicalize()?;
        let name = destination
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(invalid)?;
        let destination = parent.join(name);
        let lock_path = parent.join(format!(".{name}.shape-import.lock"));
        if fs::symlink_metadata(&lock_path).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(Error::new("shape_transfer_path"));
        }
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let lock = options.open(lock_path)?;
        lock.lock()?;
        if fs::symlink_metadata(&destination).is_ok() {
            if fs::symlink_metadata(&destination)?.file_type().is_symlink() {
                return Err(Error::new("shape_transfer_path"));
            }
            let existing =
                Self::open(&destination).map_err(|_| Error::new("shape_transfer_conflict"))?;
            let m = existing.read()?;
            let receipt = m
                .shape_import
                .as_ref()
                .ok_or_else(|| Error::new("shape_transfer_conflict"))?;
            if receipt.transfer_digest != digest
                || receipt.request_id != verified.transfer.request_id
            {
                return Err(Error::new("shape_transfer_conflict"));
            }
            existing.object(&receipt.original_manifest)?;
            check_cancel(cancelled)?;
            return Ok(existing);
        }
        check_cancel(cancelled)?;
        let stage = parent.join(format!(".vox-shape-{}.vox", uuid::Uuid::now_v7()));
        let mut cleanup = Staging {
            path: stage.clone(),
        };
        let project = Self::create(&stage, "Imported from Shape")?;
        for (content, bytes) in verified.objects.values() {
            check_cancel(cancelled)?;
            if project.put_object(bytes, &content.media_type)? != *content {
                return Err(invalid());
            }
        }
        let original_manifest = project.put_object(&verified.raw, "application/json")?;
        let mut m = project.read()?;
        let mut receipt = ImportReceipt {
            schema: "vox.shape-import/1".into(),
            request_id: verified.transfer.request_id.clone(),
            transfer_digest: digest,
            producer_project_id: verified.transfer.origin.project_id.clone(),
            original_manifest,
            materials: verified.objects.values().map(|(c, _)| c.clone()).collect(),
            drafts: vec![],
            unsupported_documents: vec![],
            issues: vec![],
        };
        if let Some(audio) = &verified.transfer.accepted_audio
            && let Some(text) = &audio.input_text
        {
            let document = document(
                &verified.text(text)?,
                text.contract.as_ref(),
                audio.operation.as_ref(),
            );
            add_source(&mut m, text, document, false);
        }
        if let Some(speech) = &verified.transfer.speech_draft {
            let text = speech
                .input_text
                .as_ref()
                .map(|input| verified.text(input))
                .transpose()?
                .unwrap_or_default();
            let fallback = Contract::TextDocument(
                if speech
                    .operation
                    .as_ref()
                    .is_some_and(|op| op.script.is_some())
                {
                    TextFormat::SpeechScript {
                        revision: crate::speech_script::SPEECH_SCRIPT_REVISION.into(),
                    }
                } else {
                    TextFormat::Plain
                },
            );
            let mut document = document(
                &text,
                speech
                    .input_text
                    .as_ref()
                    .and_then(|input| input.contract.as_ref())
                    .or(Some(&fallback)),
                speech.operation.as_ref(),
            );
            if speech.operation.is_none() {
                document.voice.clear();
            }
            if let Some(input) = &speech.input_text
                && !m.sources.iter().any(|source| {
                    source
                        .id
                        .starts_with(&format!("shape-source-{}-", input.source.revision_id))
                })
            {
                add_source(
                    &mut m,
                    input,
                    self::document(&text, input.contract.as_ref(), None),
                    false,
                );
            }
            receipt.drafts.push(PreservedDraft {
                kind: "speech".into(),
                status: "unaccepted".into(),
                source: speech.input.clone(),
                document,
            });
        }
        if let Some(text) = &verified.transfer.accepted_text {
            // Accepted text has no accepted synthesis configuration of its own. Keep that boundary.
            let document = document(&verified.text(text)?, text.contract.as_ref(), None);
            add_source(&mut m, text, document.clone(), true);
            m.draft = document;
        }
        if let Some(speech) = receipt
            .drafts
            .iter()
            .find(|d| d.kind == "speech" && d.source.is_some())
        {
            m.draft = speech.document.clone();
        }
        if let Some(authoring) = &verified.transfer.authoring_draft {
            let text =
                String::from_utf8(verified.objects[&authoring.text.content.digest].1.clone())
                    .map_err(|_| invalid())?;
            let mut document = document(&text, Some(&authoring.text.contract), None);
            if verified
                .transfer
                .speech_draft
                .as_ref()
                .is_some_and(|s| s.operation.is_none())
            {
                document.voice.clear();
            }
            receipt.drafts.push(PreservedDraft {
                kind: "authoring".into(),
                status: "unaccepted".into(),
                source: None,
                document: document.clone(),
            });
            m.draft = document;
        }
        if verified
            .transfer
            .speech_draft
            .as_ref()
            .is_some_and(|s| s.input.is_none())
        {
            receipt.issues.push("speech_source_unbound".into());
        }
        if verified
            .transfer
            .speech_draft
            .as_ref()
            .is_some_and(|s| s.operation.is_none())
        {
            m.draft.voice.clear();
            receipt.issues.push("speech_settings_unconfigured".into());
        }
        for cue in &verified.transfer.cue_materials {
            let Contract::AudioClip(contract) = &cue.contract else {
                return Err(invalid());
            };
            m.sound_library.push(super::materials::SoundMaterial {
                id: uuid::Uuid::now_v7().to_string(),
                name: format!("Shape cue {}", &cue.content.digest.to_string()[..12]),
                original: cue.content.clone(),
                content: Some(cue.content.clone()),
                contract: contract.clone(),
                origin: "shape_import".into(),
                generation: None,
            });
        }
        if let Some(audio) = &verified.transfer.accepted_audio {
            let Contract::AudioClip(contract) =
                audio.material.contract.as_ref().ok_or_else(invalid)?
            else {
                return Err(invalid());
            };
            let source_document = audio
                .input_text
                .as_ref()
                .map(|input| {
                    verified.text(input).map(|text| {
                        document(&text, input.contract.as_ref(), audio.operation.as_ref())
                    })
                })
                .transpose()?;
            let source = source_document
                .as_ref()
                .and_then(|document| m.sources.iter().find(|s| &s.document == document));
            let provenance = audio
                .receipt
                .as_ref()
                .and_then(|r| r.get("external_provenance"))
                .and_then(|v| {
                    serde_json::from_value::<crate::ExternalExecutionProvenance>(v.clone()).ok()
                });
            let id = format!("shape-audio-{}", audio.material.source.revision_id);
            m.audio.push(AudioRevision {
                id: id.clone(),
                source_revision: source.map_or_else(
                    || format!("external:{}", audio.material.source.revision_id),
                    |s| s.id.clone(),
                ),
                document_digest: source.map_or_else(String::new, |s| s.document.digest()),
                content: audio.material.content.clone(),
                contract: contract.clone(),
                provenance,
                origin: if source.is_some() {
                    "shape_import"
                } else {
                    "shape_import_unbound"
                }
                .into(),
                parent_audio: None,
                source_project: Some(verified.transfer.origin.project_id.clone()),
                edit: None,
            });
            m.audio_head = Some(id);
            if source.is_none() {
                receipt
                    .issues
                    .push("accepted_audio_source_unverified".into());
            }
        }
        for draft in &receipt.drafts {
            if draft.kind == "speech"
                && verified
                    .transfer
                    .speech_draft
                    .as_ref()
                    .and_then(|speech| speech.operation.as_ref())
                    .is_some_and(|operation| {
                        draft.document.operation().ok().as_ref() != Some(operation)
                    })
            {
                receipt.unsupported_documents.push(draft.document.digest());
                receipt
                    .issues
                    .push("speech_operation_requires_review".into());
            }
        }
        for document in std::iter::once(&m.draft)
            .chain(m.sources.iter().map(|s| &s.document))
            .chain(receipt.drafts.iter().map(|d| &d.document))
        {
            let missing_reference =
                document.reference_voice.is_some() || !document.options.reference_roles.is_empty();
            if document.operation().is_err() || missing_reference {
                let digest = document.digest();
                if !receipt.unsupported_documents.contains(&digest) {
                    receipt.unsupported_documents.push(digest);
                }
                let issue = if missing_reference {
                    "reference_voice_requires_reimport"
                } else {
                    "speech_settings_not_executable"
                }
                .to_owned();
                if !receipt.issues.contains(&issue) {
                    receipt.issues.push(issue);
                }
            }
        }
        m.shape_import = Some(receipt);
        project.write(&m)?;
        project.read()?;
        check_cancel(cancelled)?;
        // Reserve the final empty directory under the import lock. Existing destinations are never replaced.
        fs::create_dir(&destination).map_err(|_| Error::new("shape_transfer_conflict"))?;
        if let Err(error) = fs::rename(&stage, &destination) {
            let _ = fs::remove_dir(&destination); // Only removes our still-empty reservation.
            return Err(error.into());
        }
        cleanup.path.clear();
        File::open(&parent)?.sync_all()?;
        // After publication success remains success even if a cancellation arrives late.
        Self::open(&destination)
    }
    /// Restores an imported unaccepted draft without adopting it or changing accepted audio.
    pub fn restore_shape_draft(&self, expected: u64, kind: &str) -> Result<Manifest> {
        self.update(expected, |m| {
            let document = m
                .shape_import
                .as_ref()
                .and_then(|r| r.drafts.iter().find(|d| d.kind == kind))
                .ok_or_else(|| Error::new("shape_draft_missing"))?
                .document
                .clone();
            m.draft = document;
            Ok(())
        })
        .map(|x| x.0)
    }
    pub fn shape_transfer_receipt(&self) -> Result<serde_json::Value> {
        let m = self.read()?;
        let receipt = m
            .shape_import
            .ok_or_else(|| Error::new("shape_draft_missing"))?;
        let bytes = self.object(&receipt.original_manifest)?;
        serde_json::from_slice(&bytes).map_err(|_| invalid())
    }
}
fn add_source(m: &mut Manifest, material: &Material, document: Document, head: bool) {
    let id = format!(
        "shape-source-{}-{}",
        material.source.revision_id,
        document.digest()
    );
    if !m.sources.iter().any(|s| s.id == id) {
        m.sources.push(SourceRevision {
            id: id.clone(),
            source_digest: ContentDigest::from_bytes(document.text.as_bytes()).to_string(),
            document,
        });
    }
    if head {
        m.source_head = Some(id);
    }
}
fn document(
    text: &str,
    contract: Option<&Contract>,
    operation: Option<&SpeechSynthesisOperation>,
) -> Document {
    let mut d = Document {
        text: text.into(),
        ..Document::default()
    };
    if matches!(
        contract,
        Some(Contract::TextDocument(TextFormat::SpeechScript { .. }))
    ) {
        d.format = ScriptFormat::ProductionScript;
    }
    if let Some(op) = operation {
        d.format = if op.script.is_some() {
            ScriptFormat::ProductionScript
        } else {
            ScriptFormat::Plain
        };
        d.language = op.language.clone();
        d.speed_milli = op.speed_milli;
        d.delivery = op.delivery;
        match &op.voice {
            SpeechVoiceSelection::Preset(voice) => {
                d.voice = voice.alias.as_str().into();
                d.voice_catalog_revision = Some(voice.catalog_revision.clone());
            }
            SpeechVoiceSelection::AuthorizedReference(reference) => {
                d.reference_voice = Some(reference.clone());
            }
        }
        if let Some(options) = &op.script {
            d.options = options.clone();
        }
        if !op.synthetic_disclosure_required {
            d.voice.clear();
        }
    }
    if let Some(Contract::TextDocument(TextFormat::SpeechScript { revision })) = contract
        && revision != crate::speech_script::SPEECH_SCRIPT_REVISION
    {
        d.options.revision = revision.clone();
    }
    d
}
struct Staging {
    path: std::path::PathBuf,
}
impl Drop for Staging {
    fn drop(&mut self) {
        if !self.path.as_os_str().is_empty() {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
#[cfg(test)]
mod tests;
