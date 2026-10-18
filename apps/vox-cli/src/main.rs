//! Bounded JSON adapter shared by humans, AI and the native Qt application.
use serde_json::{Value, json};
use std::{
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use vox_core::{
    SpeechSynthesisControl,
    project::{Document, Error, Manifest, Project, Result},
};

fn emit(value: Value) {
    let mut out = io::stdout().lock();
    let _ = serde_json::to_writer(&mut out, &value);
    let _ = writeln!(out);
    let _ = out.flush();
}
fn field<'a>(v: &'a Value, k: &str) -> Result<&'a str> {
    v[k].as_str().ok_or_else(|| Error::new("invalid_request"))
}
fn version(v: &Value) -> Result<u64> {
    v["version"]
        .as_u64()
        .ok_or_else(|| Error::new("expected_version_required"))
}
fn project(v: &Value) -> Result<Project> {
    Project::open(Path::new(field(v, "path")?))
}
fn snapshot(p: &Project, m: Manifest) -> Value {
    let analysis = m.draft.analysis();
    json!({"project":m,"path":p.path,"analysis":analysis})
}
fn voices() -> Value {
    json!(
        vox_core::SPEECH_PRESETS
            .iter()
            .map(|v| json!({"key":v.key,"alias":v.alias,"language":v.language}))
            .collect::<Vec<_>>()
    )
}

fn execute(v: &Value) -> Result<Value> {
    let action = field(v, "action")?;
    if action == "capabilities" {
        return Ok(
            json!({"schema":"vox.capabilities/1","project_schema":"vox.project/1","voices":voices(),"languages":["auto","Chinese","English","Japanese","Korean","German","French","Russian","Portuguese","Spanish","Italian"],"operations":["create","open","save","adopt","validate","render","cancel","accept","discard","playback","export","import_script","import_cue","restore_source","begin_echo","review_echo"],"forms":["narration","dialogue","listening"],"voice_reference":false,"acceptance":"explicit","runtime_app":"vox"}),
        );
    }
    if action == "probe" {
        return Ok(vox_core::infer_runtime::probe(
            v["endpoint"].as_str().unwrap_or(""),
        ));
    }
    if action == "create" {
        let p = Project::create(
            Path::new(field(v, "path")?),
            v["title"].as_str().unwrap_or("Untitled"),
        )?;
        return Ok(snapshot(&p, p.read()?));
    }
    let p = project(v)?;
    let m = match action {
        "open" => p.recover()?,
        "status" => p.read()?,
        "validate" => return Ok(p.read()?.draft.analysis()),
        "save" => {
            let draft: Document = serde_json::from_value(v["document"].clone())?;
            p.save(version(v)?, draft, v["title"].as_str().map(str::to_owned))?
        }
        "adopt" => p.adopt_script(version(v)?)?,
        "accept" => p.accept(version(v)?, field(v, "audio_id")?)?,
        "discard" => p.discard(version(v)?, field(v, "audio_id")?)?,
        "restore_source" => p.restore_source(version(v)?, field(v, "source_id")?)?,
        "playback" => {
            let id = field(v, "audio_id")?;
            let audio = p.audio_revision(id)?;
            let bytes = p.object(&audio.content)?;
            return Ok(
                json!({"file":p.playback(id)?,"peaks":vox_core::audio::waveform_peaks(&bytes,160)?}),
            );
        }
        "export" => {
            p.export(field(v, "audio_id")?, Path::new(field(v, "destination")?))?;
            return Ok(json!({"exported":true,"destination":v["destination"]}));
        }
        "import_script" => {
            let bytes = std::fs::read(field(v, "file")?)?;
            if bytes.len() > 64 * 1024 {
                return Err(Error::new("script_too_large"));
            }
            let text = String::from_utf8(bytes).map_err(|_| Error::new("utf8_required"))?;
            let mut draft = p.read()?.draft;
            draft.text = text;
            p.save(version(v)?, draft, None)?
        }
        "import_cue" => p.import_cue(
            version(v)?,
            field(v, "label")?,
            Path::new(field(v, "file")?),
        )?,
        "begin_echo" => {
            let (m, t) = p.begin_echo(
                version(v)?,
                field(v, "audio_id")?,
                Path::new(field(v, "destination")?),
            )?;
            let mut value = snapshot(&p, m);
            value["handoff"] = json!(t);
            value["source_file"] = json!(Path::new(field(v, "destination")?).join("source.wav"));
            value["handoff_file"] = json!(Path::new(field(v, "destination")?).join("handoff.json"));
            return Ok(value);
        }
        "review_echo" => p.review_echo(
            version(v)?,
            Path::new(field(v, "handoff")?),
            Path::new(field(v, "file")?),
            v["allow_pcm24"].as_bool().unwrap_or(false),
        )?,
        _ => return Err(Error::new("unknown_action")),
    };
    Ok(snapshot(&p, m))
}
fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().map(String::as_str) == Some("capabilities") {
        emit(execute(&json!({"action":"capabilities"})).expect("static capabilities"));
        return;
    }
    if args.first().map(String::as_str) == Some("--help") {
        println!(
            "vox-cli capabilities | serve | request\nrequest: one JSON request on stdin; serve: newline-delimited JSON requests and correlated events. See docs/AUTOMATION.md."
        );
        return;
    }
    let serve = args.first().map(String::as_str) == Some("serve");
    let busy = Arc::new(AtomicBool::new(false));
    let control = SpeechSynthesisControl::default();
    let mut workers = Vec::new();
    for line in io::stdin().lock().lines() {
        let line = match line {
            Ok(s) if s.len() <= 512 * 1024 => s,
            _ => {
                emit(json!({"ok":false,"error":"invalid_request"}));
                break;
            }
        };
        let v: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => {
                emit(json!({"ok":false,"error":"invalid_request"}));
                continue;
            }
        };
        let request_id = v["request_id"].clone();
        let action = v["action"].as_str().unwrap_or("");
        if action == "cancel" {
            control.cancel();
            emit(
                json!({"request_id":request_id,"ok":true,"result":{"stop_requested":true,"boundary":"after_current_segment"}}),
            );
            continue;
        }
        if action == "render" || action == "voice_preview" {
            if busy
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                emit(json!({"request_id":request_id,"ok":false,"error":"render_busy"}));
                continue;
            }
            control.resume();
            let control = control.clone();
            let busy = busy.clone();
            let worker = std::thread::spawn(move || {
                let result = if v["action"] == "voice_preview" {
                    voice_preview(&v, control)
                } else {
                    (|| {
                        let p = project(&v)?;
                        let id = v["request_id"].clone();
                        let progress = Arc::new(move |completed, total| {
                            emit(
                                json!({"event":"progress","request_id":id,"completed":completed,"total":total}),
                            )
                        });
                        let m = p.render(
                            version(&v)?,
                            v["endpoint"].as_str().unwrap_or(""),
                            control,
                            progress,
                        )?;
                        Ok(snapshot(&p, m))
                    })()
                };
                busy.store(false, Ordering::SeqCst);
                match result {
                    Ok(result) => emit(json!({"request_id":request_id,"ok":true,"result":result})),
                    Err(e) => emit(json!({"request_id":request_id,"ok":false,"error":e.code})),
                }
            });
            if serve {
                workers.retain(|w: &std::thread::JoinHandle<()>| !w.is_finished());
                workers.push(worker);
            } else {
                let _ = worker.join();
                break;
            }
        } else {
            if busy.load(Ordering::SeqCst)
                && !matches!(action, "status" | "capabilities" | "validate")
            {
                emit(json!({"request_id":request_id,"ok":false,"error":"render_busy"}));
                continue;
            }
            match execute(&v) {
                Ok(result) => emit(json!({"request_id":request_id,"ok":true,"result":result})),
                Err(e) => emit(json!({"request_id":request_id,"ok":false,"error":e.code})),
            }
            if !serve {
                break;
            }
        }
    }
    if busy.load(Ordering::SeqCst) {
        control.cancel();
    }
    for w in workers {
        let _ = w.join();
    }
}
fn voice_preview(v: &Value, control: SpeechSynthesisControl) -> Result<Value> {
    use vox_core::{
        CapabilityId, ContentDigest, ContentRef, ExecutionInput, ExecutionRequest, Executor,
        TransformationId,
    };
    let doc = Document {
        voice: field(v, "voice")?.into(),
        text: v["text"]
            .as_str()
            .unwrap_or("Hello. 欢迎使用 Vox，让文字成为声音。")
            .into(),
        ..Document::default()
    };
    if doc.text.len() > 512 {
        return Err(Error::new("sample_too_large"));
    }
    let op = doc.operation()?;
    let text = doc.text.as_bytes().to_vec();
    let content = ContentRef::new(
        ContentDigest::from_bytes(&text),
        "text/plain",
        text.len() as u64,
    )
    .map_err(|_| Error::new("invalid_request"))?;
    let request = ExecutionRequest::new_materialized(
        TransformationId::new(),
        CapabilityId::new("audio.speech_synthesize").map_err(|_| Error::new("invalid_request"))?,
        vec![
            ExecutionInput::materialized(content, text)
                .map_err(|_| Error::new("invalid_request"))?,
        ],
        serde_json::to_vec(&op)?,
        "audio/wav",
    )
    .map_err(|_| Error::new("invalid_request"))?;
    let executor = vox_core::InferRuntimeSpeechExecutor::new(
        v["endpoint"].as_str().unwrap_or(""),
        vox_core::infer_runtime::credential_path(),
    )
    .map_err(|_| Error::new("credential_unavailable"))?
    .with_control(control.clone());
    let result = executor.execute(&request)?;
    if control.is_cancelled() {
        return Err(Error::new("speech_cancelled"));
    }
    let directory = std::env::temp_dir().join("vox-voice-samples");
    std::fs::create_dir_all(&directory)?;
    let file: PathBuf = directory.join(format!("{}.wav", TransformationId::new()));
    std::fs::write(&file, &result.bytes)?;
    Ok(
        json!({"file":file,"voice":doc.voice,"synthetic":true,"peaks":vox_core::audio::waveform_peaks(&result.bytes,160)?}),
    )
}
