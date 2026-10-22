use super::{
    Chapter, ChapterImport, ChapterOwner, ChapterState, ChapterSummary, MAX_CHAPTER_BYTES,
    MAX_CHAPTERS, MAX_IMPORT_BYTES, model::valid_title,
};
use crate::{
    ContentRef,
    project::{Document, Error, Manifest, Project, Result, ScriptFormat},
    speech_script::SpeechCueAction,
};
use std::{collections::BTreeSet, fs, ops::Range, path::Path};

pub(super) fn settings_digest(document: &Document) -> String {
    let mut settings = document.clone();
    settings.text.clear();
    settings.format = ScriptFormat::Plain;
    settings.digest()
}

pub(super) fn shared_document(parent: &Document, authored: &Document) -> Document {
    Document {
        text: authored.text.clone(),
        format: authored.format.clone(),
        ..parent.clone()
    }
}

pub(super) fn require_collection(m: &Manifest) -> Result<()> {
    if m.chapter_owner.is_some() {
        return Err(Error::new("nested_chapter_project"));
    }
    Ok(())
}

pub(super) fn selected_chapters(m: &Manifest, ids: &[String]) -> Result<Vec<Chapter>> {
    require_collection(m)?;
    let requested = ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if requested.len() != ids.len() || ids.len() > MAX_CHAPTERS {
        return Err(Error::new("invalid_chapter_selection"));
    }
    let selected = m
        .chapters
        .iter()
        .filter(|chapter| {
            !chapter.archived && (ids.is_empty() || requested.contains(chapter.id.as_str()))
        })
        .cloned()
        .collect::<Vec<_>>();
    if selected.is_empty() || (!ids.is_empty() && selected.len() != ids.len()) {
        return Err(Error::new("chapter_missing"));
    }
    Ok(selected)
}

impl Project {
    /// Resolve only an owned chapter. No caller-provided child path is admitted.
    pub fn chapter_project(&self, id: &str) -> Result<Project> {
        self.chapter_project_for(&self.read()?, id)
    }

    pub(super) fn chapter_project_for(&self, m: &Manifest, id: &str) -> Result<Project> {
        require_collection(m)?;
        let chapter = m
            .chapters
            .iter()
            .find(|chapter| chapter.id == id)
            .ok_or_else(|| Error::new("chapter_missing"))?;
        let directory = self.path.join("chapters");
        let path = directory.join(format!("{}.vox", chapter.id));
        if fs::symlink_metadata(&directory)?.file_type().is_symlink()
            || fs::symlink_metadata(&path)?.file_type().is_symlink()
            || fs::canonicalize(&directory)?.parent() != Some(self.path.as_path())
            || fs::canonicalize(&path)?.parent() != Some(directory.as_path())
        {
            return Err(Error::new("invalid_chapter_owner"));
        }
        let child = Project::open(&path)?;
        let manifest = child.read()?;
        if manifest.id != chapter.project_id
            || manifest.chapter_owner.as_ref()
                != Some(&ChapterOwner {
                    project_id: m.id.clone(),
                    chapter_id: chapter.id.clone(),
                })
        {
            return Err(Error::new("invalid_chapter_owner"));
        }
        Ok(child)
    }

    pub fn chapter_state(&self, id: &str) -> Result<ChapterState> {
        self.chapter_state_for(&self.read()?, id)
    }

    pub fn chapter_state_for(&self, m: &Manifest, id: &str) -> Result<ChapterState> {
        let child = self.chapter_project_for(m, id)?;
        let project = child.read()?;
        let segment_plan = child.segment_plan_for(&project)?;
        Ok(ChapterState {
            chapter: m
                .chapters
                .iter()
                .find(|chapter| chapter.id == id)
                .ok_or_else(|| Error::new("chapter_missing"))?
                .clone(),
            analysis: project.draft.analysis(),
            shared_settings_current: settings_digest(&m.draft) == settings_digest(&project.draft),
            project,
            segment_plan,
        })
    }

    pub fn chapter_summaries_for(&self, m: &Manifest) -> Result<Vec<ChapterSummary>> {
        m.chapters
            .iter()
            .map(|chapter| {
                let state = self.chapter_state_for(m, &chapter.id)?;
                let current = &state.project;
                let digest = current.draft.digest();
                let source = current.sources.iter().find(|source| {
                    Some(&source.id) == current.source_head.as_ref()
                        && source.document == current.draft
                });
                let accepted = current.audio.iter().find(|audio| {
                    Some(&audio.id) == current.audio_head.as_ref()
                        && Some(&audio.source_revision) == current.source_head.as_ref()
                        && audio.document_digest == digest
                        && source.is_some()
                });
                let candidate_ids = current
                    .candidates
                    .iter()
                    .filter(|audio| {
                        Some(&audio.source_revision) == current.source_head.as_ref()
                            && audio.document_digest == digest
                            && source.is_some()
                    })
                    .map(|audio| audio.id.clone())
                    .collect::<Vec<_>>();
                let completed_segments = state
                    .segment_plan
                    .segments
                    .iter()
                    .filter(|segment| !segment.candidates.is_empty())
                    .count();
                let interrupted_unknown = current.runs.iter().any(|run| {
                    run.status == "interrupted_unknown"
                        && Some(&run.source_revision) == current.source_head.as_ref()
                }) || m.chapter_batches.iter().any(|batch| {
                    batch.status == "interrupted_unknown"
                        && batch.chapters.iter().any(|item| {
                            item.chapter_id == chapter.id && item.status == "interrupted_unknown"
                        })
                });
                let running = current.runs.iter().any(|run| {
                    run.status == "running"
                        && Some(&run.source_revision) == current.source_head.as_ref()
                }) || m.chapter_batches.iter().any(|batch| {
                    batch.status == "running"
                        && batch
                            .chapters
                            .iter()
                            .any(|item| item.chapter_id == chapter.id && item.status == "running")
                });
                let status = if chapter.archived {
                    "archived"
                } else if !state.shared_settings_current {
                    "settings_changed"
                } else if interrupted_unknown {
                    "interrupted_unknown"
                } else if running {
                    "running"
                } else if !state.segment_plan.ready {
                    "needs_script"
                } else if accepted.is_some() {
                    "accepted"
                } else if !candidate_ids.is_empty() {
                    "review"
                } else if completed_segments > 0 {
                    "partial"
                } else {
                    "ready"
                };
                Ok(ChapterSummary {
                    id: chapter.id.clone(),
                    title: chapter.title.clone(),
                    project_id: chapter.project_id.clone(),
                    archived: chapter.archived,
                    version: current.version,
                    status: status.into(),
                    ready: state.segment_plan.ready,
                    shared_settings_current: state.shared_settings_current,
                    source_revision: source.map(|source| source.id.clone()),
                    document_digest: digest,
                    total_segments: state.segment_plan.segments.len(),
                    completed_segments,
                    candidate_ids,
                    audio_id: accepted.map(|audio| audio.id.clone()),
                    interrupted_unknown,
                })
            })
            .collect()
    }

    pub fn add_chapter(
        &self,
        expected: u64,
        title: &str,
        text: &str,
        format: ScriptFormat,
    ) -> Result<Manifest> {
        if !valid_title(title) || text.len() > MAX_CHAPTER_BYTES {
            return Err(Error::new("invalid_chapter"));
        }
        let _lock = self.lock()?;
        let mut m = self.read()?;
        check_version(&m, expected)?;
        let chapter = self.create_owned_chapter(&m, title, text, format, None)?;
        m.chapters.push(chapter);
        self.publish_chapter_manifest(m)
    }

    fn create_owned_chapter(
        &self,
        m: &Manifest,
        title: &str,
        text: &str,
        format: ScriptFormat,
        import: Option<ChapterImport>,
    ) -> Result<Chapter> {
        require_collection(m)?;
        if m.chapters.len() >= MAX_CHAPTERS {
            return Err(Error::new("chapter_limit"));
        }
        let directory = self.path.join("chapters");
        if directory.exists() && fs::symlink_metadata(&directory)?.file_type().is_symlink() {
            return Err(Error::new("invalid_chapter_owner"));
        }
        super::super::store::private_directory(&directory)?;
        let id = uuid::Uuid::now_v7().to_string();
        let child = Project::create(&directory.join(format!("{id}.vox")), title)?;
        let mut document = m.draft.clone();
        document.text = text.into();
        document.format = format;
        let initialized = (|| {
            let mut child_manifest = child.read()?;
            self.copy_document_materials(m, &child, &mut child_manifest, &document)?;
            child_manifest.draft = document;
            child_manifest.chapter_owner = Some(ChapterOwner {
                project_id: m.id.clone(),
                chapter_id: id.clone(),
            });
            child_manifest.draft.operation()?;
            super::super::materials::validate_libraries(&child_manifest)?;
            super::super::voice_design::validate_records(&child_manifest)?;
            child.write(&child_manifest)?;
            Ok::<_, Error>(child_manifest)
        })();
        match initialized {
            Ok(child_manifest) => Ok(Chapter {
                id,
                title: title.into(),
                project_id: child_manifest.id,
                archived: false,
                import,
            }),
            Err(error) => {
                // This brand-new, unpublished child is the only directory this operation owns.
                let _ = fs::remove_dir_all(&child.path);
                Err(error)
            }
        }
    }

    pub fn save_chapter(&self, expected: u64, id: &str, document: Document) -> Result<Manifest> {
        if document.text.len() > MAX_CHAPTER_BYTES {
            return Err(Error::new("script_too_large"));
        }
        let _lock = self.lock()?;
        let m = self.read()?;
        check_version(&m, expected)?;
        let child = self.chapter_project_for(&m, id)?;
        let authored = shared_document(&m.draft, &document);
        self.sync_chapter_document(&m, &child, authored)?;
        self.publish_chapter_manifest(m)
    }

    /// Publish an explicitly chosen chapter's current cast/defaults/pronunciation
    /// as the collection settings, retaining both projects' immutable materials.
    pub fn publish_chapter_settings(&self, expected: u64, id: &str) -> Result<Manifest> {
        let _lock = self.lock()?;
        let mut m = self.read()?;
        check_version(&m, expected)?;
        let child = self.chapter_project_for(&m, id)?;
        let child_manifest = child.read()?;
        let settings = shared_document(&child_manifest.draft, &m.draft);
        child.copy_document_materials(&child_manifest, self, &mut m, &settings)?;
        m.draft = settings;
        m.draft.operation()?;
        super::super::materials::validate_libraries(&m)?;
        super::super::voice_design::validate_records(&m)?;
        self.publish_chapter_manifest(m)
    }

    pub(super) fn sync_chapter_document(
        &self,
        parent: &Manifest,
        child: &Project,
        document: Document,
    ) -> Result<Manifest> {
        let mut current = child.read()?;
        document.operation()?;
        self.copy_document_materials(parent, child, &mut current, &document)?;
        if current.draft == document {
            return Ok(current);
        }
        let voice_library = current.voice_library;
        let sound_library = current.sound_library;
        let voice_design_candidates = current.voice_design_candidates;
        let voice_design_runs = current.voice_design_runs;
        child
            .update(current.version, |m| {
                m.draft = document;
                m.voice_library = voice_library;
                m.sound_library = sound_library;
                m.voice_design_candidates = voice_design_candidates;
                m.voice_design_runs = voice_design_runs;
                super::super::materials::validate_libraries(m)?;
                super::super::voice_design::validate_records(m)?;
                Ok(())
            })
            .map(|result| result.0)
    }

    fn copy_document_materials(
        &self,
        source: &Manifest,
        target: &Project,
        destination: &mut Manifest,
        document: &Document,
    ) -> Result<()> {
        for reference in document
            .reference_voice
            .iter()
            .chain(document.options.reference_roles.values())
        {
            let voice = source
                .voice_library
                .iter()
                .find(|voice| &voice.reference == reference)
                .ok_or_else(|| Error::new("reference_missing"))?;
            let audio = reference
                .audio
                .as_ref()
                .ok_or_else(|| Error::new("reference_missing"))?;
            copy_object(self, target, audio)?;
            if let Some(existing) = destination
                .voice_library
                .iter()
                .find(|existing| existing.id == voice.id)
            {
                if serde_json::to_vec(existing)? != serde_json::to_vec(voice)? {
                    return Err(Error::new("material_identity_conflict"));
                }
            } else {
                if destination.voice_library.len() >= 128 {
                    return Err(Error::new("project_limit"));
                }
                destination.voice_library.push(voice.clone());
            }
            if let Some(design) = &voice.design {
                let candidate = source
                    .voice_design_candidates
                    .iter()
                    .find(|candidate| {
                        candidate.id == design.candidate_id
                            && candidate.accepted_reference.as_ref() == Some(&voice.id)
                    })
                    .ok_or_else(|| Error::new("invalid_voice_design"))?;
                let run = source
                    .voice_design_runs
                    .iter()
                    .find(|run| run.id == candidate.run_id)
                    .ok_or_else(|| Error::new("invalid_voice_design"))?;
                if let Some(existing) = destination
                    .voice_design_candidates
                    .iter()
                    .find(|existing| existing.id == candidate.id)
                {
                    if serde_json::to_vec(existing)? != serde_json::to_vec(candidate)? {
                        return Err(Error::new("material_identity_conflict"));
                    }
                } else {
                    if destination.voice_design_candidates.len() >= 128 {
                        return Err(Error::new("project_limit"));
                    }
                    destination.voice_design_candidates.push(candidate.clone());
                }
                if let Some(existing) = destination
                    .voice_design_runs
                    .iter_mut()
                    .find(|existing| existing.id == run.id)
                {
                    if existing.spec != run.spec
                        || existing.requested != run.requested
                        || !(existing.candidate_ids.starts_with(&run.candidate_ids)
                            || run.candidate_ids.starts_with(&existing.candidate_ids))
                    {
                        return Err(Error::new("material_identity_conflict"));
                    }
                    if run.completed > existing.completed
                        || (run.completed == existing.completed && existing.status != "completed")
                    {
                        *existing = run.clone();
                    }
                } else {
                    if destination.voice_design_runs.len() >= 256 {
                        return Err(Error::new("project_limit"));
                    }
                    destination.voice_design_runs.push(run.clone());
                }
            }
        }
        for cue in document.options.cues.values() {
            if let SpeechCueAction::Audio { content } = cue {
                copy_object(self, target, content)?;
                if let Some(sound) = source
                    .sound_library
                    .iter()
                    .find(|sound| sound.content.as_ref() == Some(content))
                {
                    copy_object(self, target, &sound.original)?;
                    if let Some(existing) = destination
                        .sound_library
                        .iter()
                        .find(|existing| existing.id == sound.id)
                    {
                        if serde_json::to_vec(existing)? != serde_json::to_vec(sound)? {
                            return Err(Error::new("material_identity_conflict"));
                        }
                    } else {
                        if destination.sound_library.len() >= 256 {
                            return Err(Error::new("project_limit"));
                        }
                        destination.sound_library.push(sound.clone());
                    }
                }
            }
        }
        Ok(())
    }

    pub fn rename_chapter(&self, expected: u64, id: &str, title: &str) -> Result<Manifest> {
        if !valid_title(title) {
            return Err(Error::new("invalid_title"));
        }
        self.update(expected, |m| {
            require_collection(m)?;
            let chapter = m
                .chapters
                .iter_mut()
                .find(|chapter| chapter.id == id)
                .ok_or_else(|| Error::new("chapter_missing"))?;
            chapter.title = title.into();
            Ok(())
        })
        .map(|result| result.0)
    }

    /// Remove from production without deleting source, accepted audio or tickets.
    pub fn archive_chapter(&self, expected: u64, id: &str, archived: bool) -> Result<Manifest> {
        self.update(expected, |m| {
            require_collection(m)?;
            let chapter = m
                .chapters
                .iter_mut()
                .find(|chapter| chapter.id == id)
                .ok_or_else(|| Error::new("chapter_missing"))?;
            chapter.archived = archived;
            Ok(())
        })
        .map(|result| result.0)
    }

    pub fn reorder_chapters(&self, expected: u64, ids: &[String]) -> Result<Manifest> {
        self.update(expected, |m| {
            require_collection(m)?;
            let active = m
                .chapters
                .iter()
                .filter(|chapter| !chapter.archived)
                .map(|chapter| chapter.id.as_str())
                .collect::<BTreeSet<_>>();
            let requested = ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
            if active != requested || requested.len() != ids.len() {
                return Err(Error::new("invalid_chapter_order"));
            }
            let mut ordered = ids
                .iter()
                .map(|id| {
                    m.chapters
                        .iter()
                        .find(|chapter| &chapter.id == id)
                        .cloned()
                        .ok_or_else(|| Error::new("chapter_missing"))
                })
                .collect::<Result<Vec<_>>>()?;
            ordered.extend(
                m.chapters
                    .iter()
                    .filter(|chapter| chapter.archived)
                    .cloned(),
            );
            m.chapters = ordered;
            Ok(())
        })
        .map(|result| result.0)
    }

    /// Import literal UTF-8 Markdown/text. Repeated Markdown level-one ATX
    /// headings (otherwise level-two) define chapters; fenced-code headings are
    /// ignored. Oversized sections and TXT use bounded paragraph/line splits.
    /// Preamble belongs to the first chapter. No source bytes are stripped.
    pub fn import_chapters(
        &self,
        expected: u64,
        path: &Path,
        max_bytes: usize,
    ) -> Result<Manifest> {
        if !(1024..=MAX_CHAPTER_BYTES).contains(&max_bytes) {
            return Err(Error::new("invalid_chapter_size"));
        }
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("");
        if !extension.eq_ignore_ascii_case("md") && !extension.eq_ignore_ascii_case("txt") {
            return Err(Error::new("chapter_import_format"));
        }
        let bytes = super::super::store::read_bounded(path, MAX_IMPORT_BYTES)?;
        let text = std::str::from_utf8(&bytes).map_err(|_| Error::new("invalid_utf8"))?;
        if text.trim().is_empty() {
            return Err(Error::new("empty_script"));
        }
        let markdown = extension.eq_ignore_ascii_case("md");
        let heading_level = markdown.then(|| markdown_chapter_level(text)).flatten();
        let ranges = if markdown {
            split_markdown(text, max_bytes, heading_level)
        } else {
            split_import(text, max_bytes)
        };
        let original_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| {
                !name.is_empty() && name.len() <= 512 && !name.chars().any(char::is_control)
            })
            .ok_or_else(|| Error::new("chapter_import_format"))?;
        let _lock = self.lock()?;
        let mut m = self.read()?;
        check_version(&m, expected)?;
        require_collection(&m)?;
        if m.chapters.len() + ranges.len() > MAX_CHAPTERS {
            return Err(Error::new("chapter_limit"));
        }
        let original = self.put_object(&bytes, "text/plain; charset=utf-8")?;
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("Chapter");
        for (index, range) in ranges.into_iter().enumerate() {
            let chunk = &text[range.clone()];
            let title = chapter_title(chunk, heading_level)
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    format!(
                        "{} {}",
                        stem.chars().take(120).collect::<String>(),
                        index + 1
                    )
                });
            let import = ChapterImport {
                original_name: original_name.into(),
                original: original.clone(),
                start_byte: range.start as u64,
                end_byte: range.end as u64,
            };
            let chapter =
                self.create_owned_chapter(&m, &title, chunk, ScriptFormat::Plain, Some(import))?;
            m.chapters.push(chapter);
        }
        self.publish_chapter_manifest(m)
    }

    pub(super) fn publish_chapter_manifest(&self, mut m: Manifest) -> Result<Manifest> {
        m.version = m
            .version
            .checked_add(1)
            .ok_or_else(|| Error::new("project_limit"))?;
        super::model::validate_records(&m)?;
        self.write(&m)?;
        Ok(m)
    }
}

pub(super) fn check_version(m: &Manifest, expected: u64) -> Result<()> {
    if m.version != expected {
        return Err(Error::new("stale_project"));
    }
    Ok(())
}

fn copy_object(source: &Project, target: &Project, content: &ContentRef) -> Result<()> {
    if target.put_object(&source.object(content)?, &content.media_type)? != *content {
        return Err(Error::new("corrupt_object"));
    }
    Ok(())
}

fn chapter_title(text: &str, level: Option<u8>) -> Option<&str> {
    let headings = markdown_headings(text);
    headings
        .iter()
        .find(|(_, depth, title)| level.is_none_or(|level| level == *depth) && valid_title(title))
        .or_else(|| headings.iter().find(|(_, _, title)| valid_title(title)))
        .map(|(_, _, title)| *title)
}

fn markdown_chapter_level(text: &str) -> Option<u8> {
    let headings = markdown_headings(text);
    [1, 2].into_iter().find(|level| {
        headings
            .iter()
            .filter(|(_, depth, _)| depth == level)
            .count()
            >= 2
    })
}

fn split_markdown(text: &str, max_bytes: usize, level: Option<u8>) -> Vec<Range<usize>> {
    let Some(level) = level else {
        return split_import(text, max_bytes);
    };
    let mut boundaries = vec![0];
    boundaries.extend(
        markdown_headings(text)
            .into_iter()
            .filter(|(_, depth, _)| *depth == level)
            .skip(1)
            .map(|(offset, _, _)| offset),
    );
    boundaries.push(text.len());
    boundaries
        .windows(2)
        .flat_map(|range| {
            split_import(&text[range[0]..range[1]], max_bytes)
                .into_iter()
                .map(|part| range[0] + part.start..range[0] + part.end)
        })
        .collect()
}

/// The import boundary needs only ATX headings and fenced-code recognition;
/// it never renders or normalizes Markdown. Offsets always address raw UTF-8.
fn markdown_headings(text: &str) -> Vec<(usize, u8, &str)> {
    let mut headings = Vec::new();
    let mut fence: Option<(u8, usize)> = None;
    let mut offset = 0;
    for raw in text.split_inclusive('\n') {
        let line = raw.trim_end_matches(['\r', '\n']);
        let indent = line.bytes().take_while(|byte| *byte == b' ').count();
        if indent <= 3 {
            let content = &line[indent..];
            let marker = content.as_bytes().first().copied();
            if let Some((kind, minimum)) = fence {
                if marker == Some(kind) {
                    let length = content.bytes().take_while(|byte| *byte == kind).count();
                    if length >= minimum && content[length..].trim().is_empty() {
                        fence = None;
                    }
                }
            } else if matches!(marker, Some(b'`' | b'~')) {
                let kind = marker.expect("fence marker");
                let length = content.bytes().take_while(|byte| *byte == kind).count();
                if length >= 3 && (kind == b'~' || !content[length..].contains('`')) {
                    fence = Some((kind, length));
                }
            } else if marker == Some(b'#') {
                let depth = content.bytes().take_while(|byte| *byte == b'#').count();
                let rest = &content[depth..];
                if (1..=6).contains(&depth) && (rest.is_empty() || rest.starts_with([' ', '\t'])) {
                    let title = rest.trim();
                    let without_closing = title.trim_end_matches('#');
                    let title =
                        if without_closing.is_empty() || without_closing.ends_with([' ', '\t']) {
                            without_closing.trim_end()
                        } else {
                            title
                        };
                    headings.push((offset, depth as u8, title));
                }
            }
        }
        offset += raw.len();
    }
    headings
}

pub(super) fn split_import(text: &str, max_bytes: usize) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0;
    while start < text.len() {
        let mut end = (start + max_bytes).min(text.len());
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        if end < text.len() {
            let part = &text[start..end];
            let boundary = part
                .rfind("\n# ")
                .map(|offset| offset + 1)
                .or_else(|| part.rfind("\n## ").map(|offset| offset + 1))
                .or_else(|| part.rfind("\n\n").map(|offset| offset + 2))
                .or_else(|| part.rfind('\n').map(|offset| offset + 1));
            if let Some(offset) = boundary.filter(|offset| !part[..*offset].trim().is_empty()) {
                end = start + offset;
            }
        }
        ranges.push(start..end);
        start = end;
    }
    ranges
}
