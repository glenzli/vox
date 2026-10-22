//! Collection adapter: chapter identities are resolved only by the owning project.
use super::*;
use vox_core::project::ScriptFormat;

fn ids(v: &Value) -> Result<Vec<String>> {
    if v["chapter_ids"].is_null() {
        return Ok(Vec::new());
    }
    serde_json::from_value(v["chapter_ids"].clone()).map_err(|_| Error::new("invalid_request"))
}

fn collection_snapshot(p: &Project, m: Manifest, selected: Option<&str>) -> Result<Value> {
    let mut result = snapshot(p, m.clone());
    result["parent_path"] = json!(p.path);
    result["parent_project"] = json!(m);
    if let Some(id) = selected {
        let child = p.chapter_project(id)?;
        let state = p.chapter_state(id)?;
        result["path"] = json!(child.path);
        result["project"] = json!(state.project);
        result["analysis"] = state.analysis;
        result["segment_plan"] = json!(state.segment_plan);
        result["chapter"] = json!(state.chapter);
        result["shared_settings_current"] = json!(state.shared_settings_current);
    }
    Ok(result)
}

pub(super) fn execute(v: &Value) -> Result<Value> {
    let p = project(v)?;
    let action = field(v, "action")?;
    let id = v["chapter_id"].as_str();
    let mut selected = id;
    let m = match action {
        "chapter_list" => p.read()?,
        "chapter_open" => {
            let child = p.chapter_project(field(v, "chapter_id")?)?;
            child.recover()?;
            p.read()?
        }
        "chapter_add" => p.add_chapter(
            version(v)?,
            field(v, "title")?,
            v["text"].as_str().unwrap_or(""),
            if v["format"].is_null() {
                ScriptFormat::Plain
            } else {
                serde_json::from_value(v["format"].clone())?
            },
        )?,
        "chapter_save" => p.save_chapter(
            version(v)?,
            field(v, "chapter_id")?,
            serde_json::from_value(v["document"].clone())?,
        )?,
        "chapter_rename" => {
            p.rename_chapter(version(v)?, field(v, "chapter_id")?, field(v, "title")?)?
        }
        "chapter_reorder" => p.reorder_chapters(version(v)?, &ids(v)?)?,
        "chapter_archive" => {
            selected = None;
            p.archive_chapter(
                version(v)?,
                field(v, "chapter_id")?,
                v["archived"].as_bool().unwrap_or(true),
            )?
        }
        "chapter_shared_settings" => {
            p.publish_chapter_settings(version(v)?, field(v, "chapter_id")?)?
        }
        "chapter_accept" => {
            p.accept_chapter(version(v)?, field(v, "chapter_id")?, field(v, "audio_id")?)?
        }
        "chapter_import" => {
            let files: Vec<String> = serde_json::from_value(v["files"].clone())?;
            if files.is_empty() || files.len() > 128 {
                return Err(Error::new("invalid_request"));
            }
            let mut current = p.read()?;
            if current.version != version(v)? {
                return Err(Error::new("stale_project"));
            }
            for file in files {
                current = p.import_chapters(current.version, Path::new(&file), 64 * 1024)?;
            }
            current
        }
        "chapter_export" => {
            let exported =
                p.export_chapters(version(v)?, &ids(v)?, Path::new(field(v, "destination")?))?;
            return Ok(json!({"exported":true,"destination":v["destination"],"manifest":exported}));
        }
        "chapter_begin_echo" => {
            let destination = Path::new(field(v, "destination")?);
            let (m, ticket) = p.begin_chapter_echo(
                version(v)?,
                field(v, "chapter_id")?,
                field(v, "audio_id")?,
                destination,
            )?;
            let mut result = collection_snapshot(&p, m, id)?;
            result["handoff"] = json!(ticket);
            result["source_file"] = json!(destination.join("source.wav"));
            result["handoff_file"] = json!(destination.join("handoff.json"));
            return Ok(result);
        }
        "chapter_review_echo" => p.review_chapter_echo(
            version(v)?,
            field(v, "chapter_id")?,
            Path::new(field(v, "handoff")?),
            Path::new(field(v, "file")?),
            v["allow_pcm24"].as_bool().unwrap_or(false),
        )?,
        _ => return Err(Error::new("unknown_action")),
    };
    collection_snapshot(&p, m, selected)
}

pub(super) fn batch(v: &Value, control: SpeechSynthesisControl) -> Result<Value> {
    let p = project(v)?;
    let request_id = v["request_id"].clone();
    let progress = Arc::new(move |progress: vox_core::project::ChapterBatchProgress| {
        let mut event = json!(progress);
        event["event"] = json!("progress");
        event["request_id"] = request_id.clone();
        event["completed"] = event["completed_segments"].clone();
        event["total"] = event["total_segments"].clone();
        emit(event);
    });
    let outcome = p.render_chapter_batch(
        version(v)?,
        &ids(v)?,
        v["endpoint"].as_str().unwrap_or(""),
        v["retry_interrupted"].as_bool().unwrap_or(false),
        control,
        progress,
    )?;
    let mut result = collection_snapshot(&p, outcome.manifest, v["chapter_id"].as_str())?;
    result["chapter_batch"] = json!(outcome.batch);
    Ok(result)
}
