use super::*;
use crate::{AudioOriginDisclosure, ContentDigest, ContentRef, audio::parse_pcm_s16le_wav};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Project {
    pub path: PathBuf,
}
impl Project {
    pub fn create(path: &Path, title: &str) -> Result<Self> {
        if title.len() > 512 || title.trim().is_empty() {
            return Err(Error::new("invalid_title"));
        }
        if path.extension().and_then(|x| x.to_str()) != Some("vox") {
            return Err(Error::new("project_extension"));
        }
        fs::create_dir(path).map_err(|_| Error::new("project_exists"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
        }
        let project = Self {
            path: fs::canonicalize(path)?,
        };
        fs::create_dir(project.path.join("objects"))?;
        let m = Manifest {
            schema: PROJECT_SCHEMA.into(),
            id: uuid::Uuid::now_v7().to_string(),
            version: 0,
            title: title.into(),
            draft: Document::default(),
            sources: vec![],
            source_head: None,
            candidates: vec![],
            audio: vec![],
            audio_head: None,
            runs: vec![],
            handoffs: vec![],
            voice_library: vec![],
            sound_library: vec![],
            shape_import: None,
        };
        project.write(&m)?;
        Ok(project)
    }
    pub fn open(path: &Path) -> Result<Self> {
        let p = Self {
            path: fs::canonicalize(path)?,
        };
        p.read()?;
        Ok(p)
    }
    pub fn read(&self) -> Result<Manifest> {
        let bytes = read_bounded(&self.path.join("project.json"), 16 * 1024 * 1024)?;
        let m: Manifest = serde_json::from_slice(&bytes)?;
        if m.schema != PROJECT_SCHEMA
            || uuid::Uuid::parse_str(&m.id).is_err()
            || m.title.len() > 512
            || m.sources.len() > 1024
            || m.audio.len() + m.candidates.len() > 1024
            || m.runs.len() > 2048
            || m.voice_library.len() > 128
            || m.sound_library.len() > 256
        {
            return Err(Error::new("invalid_project"));
        }
        if let Some(receipt) = &m.shape_import
            && (receipt.schema != "vox.shape-import/1"
                || receipt.transfer_digest != receipt.original_manifest.digest.to_string()
                || receipt.original_manifest.media_type != "application/json"
                || receipt.original_manifest.byte_length > 8 * 1024 * 1024
                || receipt.materials.len() > 256
                || receipt.drafts.len() > 2
                || receipt.unsupported_documents.len() > 16
                || receipt.issues.len() > 16
                || receipt.drafts.iter().any(|draft| {
                    draft.status != "unaccepted"
                        || !matches!(draft.kind.as_str(), "speech" | "authoring")
                }))
        {
            return Err(Error::new("invalid_project"));
        }
        super::materials::validate_libraries(&m)?;
        if !super::shape_transfer::unsupported(&m, &m.draft) {
            m.draft.operation()?;
        }
        for source in &m.sources {
            if !super::shape_transfer::unsupported(&m, &source.document) {
                source.document.operation()?;
            }
            if source.source_digest
                != blake3::hash(source.document.text.as_bytes())
                    .to_hex()
                    .as_str()
            {
                return Err(Error::new("corrupt_source"));
            }
        }
        if m.source_head
            .as_ref()
            .is_some_and(|id| !m.sources.iter().any(|r| &r.id == id))
            || m.audio_head
                .as_ref()
                .is_some_and(|id| !m.audio.iter().any(|r| &r.id == id))
        {
            return Err(Error::new("invalid_project"));
        }
        Ok(m)
    }
    pub(crate) fn lock(&self) -> Result<File> {
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.path.join("project.lock"))?;
        lock.lock()?;
        Ok(lock)
    }
    pub(crate) fn update<T>(
        &self,
        expected: u64,
        f: impl FnOnce(&mut Manifest) -> Result<T>,
    ) -> Result<(Manifest, T)> {
        let _lock = self.lock()?;
        let mut m = self.read()?;
        if m.version != expected {
            return Err(Error::new("stale_project"));
        }
        let out = f(&mut m)?;
        m.version = m
            .version
            .checked_add(1)
            .ok_or_else(|| Error::new("project_limit"))?;
        self.write(&m)?;
        Ok((m, out))
    }
    pub(super) fn write(&self, m: &Manifest) -> Result<()> {
        let temp = self
            .path
            .join(format!(".project-{}.tmp", uuid::Uuid::now_v7()));
        let bytes = serde_json::to_vec_pretty(m)?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(Error::new("project_limit"));
        }
        let mut file = private_file(&temp)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(temp, self.path.join("project.json"))?;
        File::open(&self.path)?.sync_all()?;
        Ok(())
    }
    pub fn save(&self, expected: u64, draft: Document, title: Option<String>) -> Result<Manifest> {
        draft.operation()?;
        let mut candidate = self.read()?;
        candidate.draft = draft.clone();
        super::materials::validate_libraries(&candidate)?;
        if title
            .as_ref()
            .is_some_and(|t| t.is_empty() || t.len() > 512)
        {
            return Err(Error::new("invalid_title"));
        }
        self.update(expected, |m| {
            m.draft = draft;
            if let Some(t) = title {
                m.title = t;
            }
            Ok(())
        })
        .map(|x| x.0)
    }
    pub fn adopt_script(&self, expected: u64) -> Result<Manifest> {
        self.update(expected, |m| {
            super::shape_transfer::require_executable(m, &m.draft)?;
            if m.draft.text.trim().is_empty() {
                return Err(Error::new("empty_script"));
            }
            if m.sources.len() >= 1024 {
                return Err(Error::new("project_limit"));
            }
            if let Some(r) = m
                .sources
                .iter()
                .find(|r| Some(&r.id) == m.source_head.as_ref())
                && r.document == m.draft
            {
                return Ok(());
            }
            let id = uuid::Uuid::now_v7().to_string();
            m.sources.push(SourceRevision {
                id: id.clone(),
                source_digest: blake3::hash(m.draft.text.as_bytes()).to_hex().to_string(),
                document: m.draft.clone(),
            });
            m.source_head = Some(id);
            Ok(())
        })
        .map(|x| x.0)
    }
    pub fn restore_source(&self, expected: u64, id: &str) -> Result<Manifest> {
        self.update(expected, |m| {
            m.draft = m
                .sources
                .iter()
                .find(|r| r.id == id)
                .ok_or_else(|| Error::new("source_missing"))?
                .document
                .clone();
            Ok(())
        })
        .map(|x| x.0)
    }
    pub fn put_object(&self, bytes: &[u8], media_type: &str) -> Result<ContentRef> {
        if bytes.len() > 128 * 1024 * 1024 {
            return Err(Error::new("audio_too_large"));
        }
        let digest = ContentDigest::from_bytes(bytes);
        let path = self.path.join("objects").join(digest.to_string());
        if path.exists() {
            let old = read_bounded(&path, 128 * 1024 * 1024)?;
            if old != bytes {
                return Err(Error::new("corrupt_object"));
            }
        } else {
            let temp = self
                .path
                .join("objects")
                .join(format!(".{}.tmp", uuid::Uuid::now_v7()));
            let mut file = private_file(&temp)?;
            file.write_all(bytes)?;
            file.sync_all()?;
            match fs::hard_link(&temp, &path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    if read_bounded(&path, 128 * 1024 * 1024)? != bytes {
                        return Err(Error::new("corrupt_object"));
                    }
                }
                Err(e) => return Err(e.into()),
            };
            fs::remove_file(temp)?;
            File::open(self.path.join("objects"))?.sync_all()?;
        }
        ContentRef::new(digest, media_type, bytes.len() as u64)
            .map_err(|_| Error::new("invalid_content"))
    }
    pub fn object(&self, content: &ContentRef) -> Result<Vec<u8>> {
        let bytes = read_bounded(
            &self.path.join("objects").join(content.digest.to_string()),
            128 * 1024 * 1024,
        )?;
        if bytes.len() as u64 != content.byte_length
            || ContentDigest::from_bytes(&bytes) != content.digest
        {
            return Err(Error::new("corrupt_object"));
        }
        Ok(bytes)
    }
    pub fn audio_revision(&self, id: &str) -> Result<AudioRevision> {
        let m = self.read()?;
        m.candidates
            .iter()
            .chain(&m.audio)
            .find(|r| r.id == id)
            .cloned()
            .ok_or_else(|| Error::new("audio_missing"))
    }
    pub fn accept(&self, expected: u64, id: &str) -> Result<Manifest> {
        self.update(expected, |m| {
            let index = m
                .candidates
                .iter()
                .position(|r| r.id == id)
                .ok_or_else(|| Error::new("candidate_missing"))?;
            let r = &m.candidates[index];
            if Some(&r.source_revision) != m.source_head.as_ref() {
                return Err(Error::new("stale_source"));
            }
            let source = m
                .sources
                .iter()
                .find(|s| s.id == r.source_revision)
                .ok_or_else(|| Error::new("source_missing"))?;
            if r.document_digest != source.document.digest() {
                return Err(Error::new("stale_source"));
            }
            let bytes = self.object(&r.content)?;
            if parse_pcm_s16le_wav(&bytes, r.contract.origin)? != r.contract {
                return Err(Error::new("corrupt_audio"));
            }
            if r.origin == "synthesis" {
                let p = r
                    .provenance
                    .as_ref()
                    .ok_or_else(|| Error::new("invalid_provenance"))?;
                if p.app_id != "vox" || !p.is_bounded() {
                    return Err(Error::new("invalid_provenance"));
                }
                if source.document.format == ScriptFormat::ProductionScript {
                    let op = source.document.operation()?;
                    let inputs = self.render_inputs(&source.document)?;
                    crate::assembly::validate_output(
                        &source.document.text,
                        &op,
                        &inputs,
                        p,
                        &bytes,
                    )?;
                }
            }
            let r = m.candidates.remove(index);
            m.audio_head = Some(r.id.clone());
            m.audio.push(r);
            Ok(())
        })
        .map(|x| x.0)
    }
    pub fn discard(&self, expected: u64, id: &str) -> Result<Manifest> {
        self.update(expected, |m| {
            let n = m.candidates.len();
            m.candidates.retain(|r| r.id != id);
            if n == m.candidates.len() {
                return Err(Error::new("candidate_missing"));
            }
            Ok(())
        })
        .map(|x| x.0)
    }
    pub fn export(&self, id: &str, destination: &Path) -> Result<()> {
        let r = self.audio_revision(id)?;
        let bytes = self.object(&r.content)?;
        parse_pcm_s16le_wav(&bytes, r.contract.origin)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)
            .map_err(|_| Error::new("destination_exists"))?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        Ok(())
    }
    pub fn playback(&self, id: &str) -> Result<PathBuf> {
        let r = self.audio_revision(id)?;
        let bytes = self.object(&r.content)?;
        parse_pcm_s16le_wav(&bytes, r.contract.origin)?;
        let directory = std::env::temp_dir()
            .join("vox-audition")
            .join(&self.read()?.id);
        fs::create_dir_all(&directory)?;
        let path = directory.join(format!("{}.wav", r.content.digest));
        if !path.exists() {
            let mut f = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&path)?;
            f.write_all(&bytes)?;
        } else if fs::read(&path)? != bytes {
            return Err(Error::new("corrupt_object"));
        }
        Ok(path)
    }
    pub fn import_cue(&self, expected: u64, label: &str, path: &Path) -> Result<Manifest> {
        let bytes = read_bounded(path, 8 * 1024 * 1024)?;
        let c = parse_pcm_s16le_wav(&bytes, AudioOriginDisclosure::ImportedUnverified)?;
        if c.sample_rate_hz != 24000 || c.channels != 1 {
            return Err(Error::new("cue_format"));
        }
        let content = self.put_object(&bytes, "audio/wav")?;
        self.update(expected, |m| {
            m.draft.options.cues.insert(
                label.into(),
                crate::speech_script::SpeechCueAction::Audio { content },
            );
            m.draft.operation()?;
            Ok(())
        })
        .map(|x| x.0)
    }
}
pub(crate) fn read_bounded(path: &Path, max: u64) -> Result<Vec<u8>> {
    let file = File::open(path)?;
    if file.metadata()?.len() > max {
        return Err(Error::new("file_too_large"));
    }
    let mut bytes = Vec::new();
    file.take(max + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max {
        return Err(Error::new("file_too_large"));
    }
    Ok(bytes)
}

fn private_file(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    Ok(options.open(path)?)
}
pub(crate) fn private_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}
