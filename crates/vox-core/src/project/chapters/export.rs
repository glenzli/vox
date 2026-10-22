use super::{
    ChapterExport, ChapterExportManifest,
    store::{check_version, selected_chapters, settings_digest},
};
use crate::{
    handoff::Ticket,
    project::{Error, Manifest, Project, Result},
};
use std::{fs, io::Write, path::Path};

impl Project {
    /// Accept exactly the reviewed whole-chapter candidate. Batch completion
    /// never calls this method and cannot advance an audio head by itself.
    pub fn accept_chapter(&self, expected: u64, id: &str, audio_id: &str) -> Result<Manifest> {
        let _lock = self.lock()?;
        let m = self.read()?;
        check_version(&m, expected)?;
        let chapter = m
            .chapters
            .iter()
            .find(|chapter| chapter.id == id && !chapter.archived)
            .ok_or_else(|| Error::new("chapter_missing"))?;
        let child = self.chapter_project_for(&m, &chapter.id)?;
        let current = child.read()?;
        require_current_settings(&m, &current)?;
        require_current_source(&current)?;
        child.accept(current.version, audio_id)?;
        self.publish_chapter_manifest(m)
    }

    pub fn begin_chapter_echo(
        &self,
        expected: u64,
        chapter_id: &str,
        audio_id: &str,
        destination: &Path,
    ) -> Result<(Manifest, Ticket)> {
        let _lock = self.lock()?;
        let m = self.read()?;
        check_version(&m, expected)?;
        let child = self.chapter_project_for(&m, chapter_id)?;
        let current = child.read()?;
        require_current_settings(&m, &current)?;
        require_current_source(&current)?;
        let (_, ticket) = child.begin_echo(current.version, audio_id, destination)?;
        Ok((self.publish_chapter_manifest(m)?, ticket))
    }

    /// The existing source-pinned Echo admission creates only a candidate. Its
    /// returned WAV still requires a separate accept_chapter action.
    pub fn review_chapter_echo(
        &self,
        expected: u64,
        chapter_id: &str,
        ticket_path: &Path,
        returned: &Path,
        allow_pcm24: bool,
    ) -> Result<Manifest> {
        let _lock = self.lock()?;
        let m = self.read()?;
        check_version(&m, expected)?;
        let child = self.chapter_project_for(&m, chapter_id)?;
        let current = child.read()?;
        require_current_settings(&m, &current)?;
        require_current_source(&current)?;
        child.review_echo(current.version, ticket_path, returned, allow_pcm24)?;
        self.publish_chapter_manifest(m)
    }

    /// Export ordered current accepted chapters, with measured SRT/VTT and a
    /// portable manifest containing relative filenames and exact source/receipts.
    /// Edited accepted WAVs remain exportable; their unavailable captions are
    /// recorded instead of inferring new timing from the edited audio.
    pub fn export_chapters(
        &self,
        expected: u64,
        ids: &[String],
        destination: &Path,
    ) -> Result<ChapterExportManifest> {
        let _lock = self.lock()?;
        let m = self.read()?;
        check_version(&m, expected)?;
        let chapters = selected_chapters(&m, ids)?;
        if destination.exists() {
            return Err(Error::new("destination_exists"));
        }
        let mut manifest = ChapterExportManifest {
            schema: "vox.chapter-export/1".into(),
            project_id: m.id.clone(),
            title: m.title.clone(),
            chapters: Vec::with_capacity(chapters.len()),
        };
        let mut selected = Vec::with_capacity(chapters.len());
        // Freeze the child manifests before writing any output. The source,
        // accepted audio and segment objects are all immutable identities.
        for (index, chapter) in chapters.iter().enumerate() {
            let child = self.chapter_project_for(&m, &chapter.id)?;
            let current = child.read()?;
            require_current_settings(&m, &current)?;
            let audio = current
                .audio
                .iter()
                .find(|audio| Some(&audio.id) == current.audio_head.as_ref())
                .ok_or_else(|| Error::new("chapter_audio_unaccepted"))?;
            let source = current
                .sources
                .iter()
                .find(|source| {
                    Some(&source.id) == current.source_head.as_ref()
                        && source.document == current.draft
                })
                .ok_or_else(|| Error::new("stale_source"))?;
            if audio.source_revision != source.id
                || audio.document_digest != source.document.digest()
            {
                return Err(Error::new("stale_source"));
            }
            let captions = audio.segment_assembly.is_some();
            if captions {
                child.validate_segment_assembly(&current, audio)?;
            } else if crate::audio::parse_pcm_s16le_wav(
                &child.object(&audio.content)?,
                audio.contract.origin,
            )? != audio.contract
            {
                return Err(Error::new("corrupt_audio"));
            }
            let stem = format!("{:03}-{}", index + 1, chapter.id);
            manifest.chapters.push(ChapterExport {
                chapter_id: chapter.id.clone(),
                title: chapter.title.clone(),
                project_id: current.id,
                source_revision: source.id.clone(),
                document_digest: audio.document_digest.clone(),
                audio_revision: audio.id.clone(),
                audio: audio.content.clone(),
                document: source.document.clone(),
                origin: audio.origin.clone(),
                provenance: audio.provenance.clone(),
                edit: audio.edit.clone(),
                parent_audio: audio.parent_audio.clone(),
                wav: format!("{stem}.wav"),
                srt: captions.then(|| format!("{stem}.srt")),
                vtt: captions.then(|| format!("{stem}.vtt")),
                subtitles_unavailable_reason: (!captions)
                    .then(|| "accepted_audio_has_no_measured_segment_timings".into()),
                sample_rate_hz: audio.contract.sample_rate_hz,
                frames: audio.contract.frame_count,
            });
            selected.push((child, audio.id.clone()));
        }
        let parent = destination
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .ok_or_else(|| Error::new("invalid_destination"))?;
        let staging = parent.join(format!(".vox-chapter-export-{}", uuid::Uuid::now_v7()));
        fs::create_dir(&staging)?;
        let result = (|| {
            for ((child, id), chapter) in selected.iter().zip(&manifest.chapters) {
                child.export(id, &staging.join(&chapter.wav))?;
                if let Some(srt) = &chapter.srt {
                    child.export_subtitles(id, &staging.join(srt), "srt")?;
                }
                if let Some(vtt) = &chapter.vtt {
                    child.export_subtitles(id, &staging.join(vtt), "vtt")?;
                }
            }
            let mut output = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(staging.join("manifest.json"))?;
            output.write_all(&serde_json::to_vec_pretty(&manifest)?)?;
            output.sync_all()?;
            fs::File::open(&staging)?.sync_all()?;
            // create_dir is the no-clobber publication reservation. Only files
            // this call created are removed if a later publication step fails.
            fs::create_dir(destination).map_err(|_| Error::new("destination_exists"))?;
            if let Err(error) = fs::rename(&staging, destination) {
                let _ = fs::remove_dir(destination);
                return Err(error.into());
            }
            fs::File::open(parent)?.sync_all()?;
            Ok::<_, Error>(())
        })();
        if result.is_err() {
            let _ = fs::remove_dir_all(&staging);
        }
        result?;
        Ok(manifest)
    }
}

fn require_current_settings(parent: &Manifest, child: &Manifest) -> Result<()> {
    if settings_digest(&parent.draft) != settings_digest(&child.draft) {
        return Err(Error::new("chapter_settings_changed"));
    }
    Ok(())
}

fn require_current_source(child: &Manifest) -> Result<()> {
    if !child.sources.iter().any(|source| {
        Some(&source.id) == child.source_head.as_ref() && source.document == child.draft
    }) {
        return Err(Error::new("stale_source"));
    }
    Ok(())
}
