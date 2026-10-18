//! Admission of an untrusted transfer package before any destination project exists.
use super::{schema::*, *};
use crate::{ContentDigest, ContentRef, audio::parse_pcm_s16le_wav};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

pub(super) struct Verified {
    pub transfer: Transfer,
    pub raw: Vec<u8>,
    pub objects: BTreeMap<ContentDigest, (ContentRef, Vec<u8>)>,
}
impl Verified {
    pub fn load(path: &Path, cancelled: &impl Fn() -> bool) -> Result<Self> {
        check_cancel(cancelled)?;
        if !path.is_absolute() {
            return Err(Error::new("shape_transfer_path"));
        }
        let manifest = if path.is_dir() {
            path.join("manifest.json")
        } else {
            path.into()
        };
        let metadata = fs::symlink_metadata(&manifest)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(Error::new("shape_transfer_path"));
        }
        let directory = manifest.parent().ok_or_else(invalid)?.canonicalize()?;
        let raw = super::super::store::read_bounded(&manifest, 8 * 1024 * 1024)?;
        let value: Value = serde_json::from_slice(&raw).map_err(|_| invalid())?;
        for key in [
            "schema",
            "request_id",
            "origin",
            "accepted_text",
            "authoring_draft",
            "speech_draft",
            "cue_materials",
            "accepted_audio",
        ] {
            if value.get(key).is_none() {
                return Err(invalid());
            }
        }
        if value["origin"].get("observed_text_head").is_none() {
            return Err(invalid());
        }
        let transfer: Transfer = serde_json::from_slice(&raw).map_err(|_| invalid())?;
        if transfer.schema != "vox.shape-transfer/1"
            || transfer.origin.producer != "shape"
            || transfer.request_id.is_empty()
            || transfer.request_id.len() > 160
            || !transfer
                .request_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
            || transfer.cue_materials.len() > 128
            || (transfer.accepted_text.is_none() && transfer.authoring_draft.is_none())
        {
            return Err(invalid());
        }
        uuid(&transfer.origin.project_id)?;
        uuid(&transfer.origin.text_artifact_id)?;
        if let Some(head) = &transfer.origin.observed_text_head {
            uuid(head)?;
        }
        if transfer
            .accepted_text
            .as_ref()
            .map(|t| &t.source.revision_id)
            != transfer.origin.observed_text_head.as_ref()
            || transfer
                .accepted_text
                .as_ref()
                .is_some_and(|t| t.source.artifact_id != transfer.origin.text_artifact_id)
        {
            return Err(invalid());
        }
        let mut objects = BTreeMap::new();
        let mut admit =
            |content: &ContentRef, contract: Option<&Contract>, relative: &str| -> Result<()> {
                check_cancel(cancelled)?;
                let text = content.media_type == "text/plain"
                    || content.media_type == "text/plain; charset=utf-8";
                let maximum = if text {
                    4 * 1024 * 1024
                } else if content.media_type == "audio/wav" {
                    128 * 1024 * 1024
                } else {
                    return Err(invalid());
                };
                if content.byte_length > maximum {
                    return Err(invalid());
                }
                let file = safe_file(&directory, relative)?;
                let bytes = super::super::store::read_bounded(&file, maximum)?;
                if bytes.len() as u64 != content.byte_length
                    || ContentDigest::from_bytes(&bytes) != content.digest
                {
                    return Err(Error::new("shape_transfer_hash"));
                }
                match (text, contract) {
                    (true, None | Some(Contract::TextDocument(_))) => {
                        std::str::from_utf8(&bytes).map_err(|_| invalid())?;
                    }
                    (false, Some(Contract::AudioClip(c)))
                        if parse_pcm_s16le_wav(&bytes, c.origin)? == *c => {}
                    _ => return Err(Error::new("shape_transfer_contract")),
                }
                if let Some((old, old_bytes)) = objects.get(&content.digest) {
                    if old != content || old_bytes != &bytes {
                        return Err(invalid());
                    }
                } else {
                    objects.insert(content.digest, (content.clone(), bytes));
                }
                if objects.values().map(|(c, _)| c.byte_length).sum::<u64>() > 168 * 1024 * 1024 {
                    return Err(invalid());
                }
                Ok(())
            };
        if let Some(text) = &transfer.accepted_text {
            material(text, &value["accepted_text"], true, &mut admit)?;
        }
        if let Some(draft) = &transfer.authoring_draft {
            if draft.status != "unaccepted"
                || draft.target_artifact_id != transfer.origin.text_artifact_id
                || draft.expected_head != transfer.origin.observed_text_head
                || draft.text.contract != Contract::TextDocument(draft.format.clone())
            {
                return Err(invalid());
            }
            graph(
                &draft.graph,
                &draft.graph_digest,
                &draft.target_artifact_id,
                draft.expected_head.as_deref(),
                &draft.settings,
                None,
            )?;
            content_keys(&value["authoring_draft"]["text"]["content"])?;
            contract_keys(&value["authoring_draft"]["text"]["contract"])?;
            admit(
                &draft.text.content,
                Some(&draft.text.contract),
                &draft.text.relative_path,
            )?;
        }
        if let Some(draft) = &transfer.speech_draft {
            if draft.status != "unaccepted"
                || draft.draft_id.is_empty()
                || draft.draft_id.len() > 256
            {
                return Err(invalid());
            }
            uuid(&draft.target_artifact_id)?;
            if let Some(head) = &draft.expected_head {
                uuid(head)?;
            }
            graph(
                &draft.graph,
                &draft.graph_digest,
                &draft.target_artifact_id,
                draft.expected_head.as_deref(),
                &draft.settings,
                Some(&draft.draft_id),
            )?;
            if draft.operation.is_some() {
                if draft.settings.is_null() || draft.issue.is_some() {
                    return Err(invalid());
                }
                operation_keys(&value["speech_draft"]["operation"])?;
            } else if !draft.settings.is_null()
                || draft.issue.as_deref() != Some("speech_settings_unconfigured")
            {
                return Err(invalid());
            }
            if draft.input.as_ref() != draft.input_text.as_ref().map(|t| &t.source) {
                return Err(invalid());
            }
            if let Some(text) = &draft.input_text {
                if text.source.artifact_id != transfer.origin.text_artifact_id {
                    return Err(invalid());
                }
                material(text, &value["speech_draft"]["input_text"], true, &mut admit)?;
            }
        }
        for (index, cue) in transfer.cue_materials.iter().enumerate() {
            keys(
                &value["cue_materials"][index],
                &["content", "contract", "relative_path"],
            )?;
            content_keys(&value["cue_materials"][index]["content"])?;
            contract_keys(&value["cue_materials"][index]["contract"])?;
            if cue.content.byte_length > 8 * 1024 * 1024 {
                return Err(invalid());
            }
            let Contract::AudioClip(c) = &cue.contract else {
                return Err(invalid());
            };
            if c.sample_rate_hz != 24000 || c.channels != 1 {
                return Err(Error::new("shape_transfer_contract"));
            }
            admit(&cue.content, Some(&cue.contract), &cue.relative_path)?;
        }
        if let Some(audio) = &transfer.accepted_audio {
            if audio.status != "accepted_external_origin" {
                return Err(invalid());
            }
            material(
                &audio.material,
                &value["accepted_audio"]["material"],
                false,
                &mut admit,
            )?;
            if audio.transformation["id"] != audio.material.revision.transformation_id
                || audio.transformation["target_artifact_id"] != audio.material.source.artifact_id
            {
                return Err(invalid());
            }
            if let Some(receipt) = &audio.receipt
                && (receipt["transformation_id"] != audio.material.revision.transformation_id
                    || receipt["outcome"]["status"] != "succeeded")
            {
                return Err(invalid());
            }
            if let Some(op) = &audio.operation {
                operation_keys(&value["accepted_audio"]["operation"])?;
                let _ = op; // Executability is assessed separately; original settings always survive.
            }
            if let Some(text) = &audio.input_text {
                material(
                    text,
                    &value["accepted_audio"]["input_text"],
                    true,
                    &mut admit,
                )?;
                let inputs = audio.transformation["inputs"]
                    .as_array()
                    .ok_or_else(invalid)?;
                if audio.operation.is_none()
                    || inputs.len() != 1
                    || inputs[0] != text.source.revision_id
                {
                    return Err(invalid());
                }
            }
        }
        if let Some(draft) = &transfer.authoring_draft {
            let settings: Value =
                serde_json::from_str(draft.settings["json"].as_str().ok_or_else(invalid)?)
                    .map_err(|_| invalid())?;
            let bytes = &objects[&draft.text.content.digest].1;
            if settings["text"].as_str().map(str::as_bytes) != Some(bytes.as_slice()) {
                return Err(invalid());
            }
        }
        for op in transfer
            .speech_draft
            .iter()
            .filter_map(|d| d.operation.as_ref())
            .chain(
                transfer
                    .accepted_audio
                    .iter()
                    .filter_map(|a| a.operation.as_ref()),
            )
        {
            for content in
                op.script
                    .iter()
                    .flat_map(|s| s.cues.values())
                    .filter_map(|cue| match cue {
                        crate::speech_script::SpeechCueAction::Audio { content } => Some(content),
                        _ => None,
                    })
            {
                if !transfer.cue_materials.iter().any(|c| &c.content == content) {
                    return Err(Error::new("shape_transfer_contract"));
                }
            }
        }
        check_cancel(cancelled)?;
        Ok(Self {
            transfer,
            raw,
            objects,
        })
    }
    pub fn text(&self, material: &Material) -> Result<String> {
        String::from_utf8(self.objects[&material.content.digest].1.clone()).map_err(|_| invalid())
    }
}
fn material(
    m: &Material,
    raw: &Value,
    text: bool,
    admit: &mut impl FnMut(&ContentRef, Option<&Contract>, &str) -> Result<()>,
) -> Result<()> {
    pin(&m.source)?;
    uuid(&m.revision.id)?;
    uuid(&m.revision.artifact_id)?;
    uuid(&m.revision.transformation_id)?;
    if m.source.revision_id != m.revision.id
        || m.source.artifact_id != m.revision.artifact_id
        || m.content != m.revision.content
        || m.contract != m.revision.content_contract
        || m.revision.parents.len() > 8
    {
        return Err(invalid());
    }
    for parent in &m.revision.parents {
        uuid(parent)?;
    }
    let _timestamp = m.revision.created_at_unix_ms;
    if text != m.content.media_type.starts_with("text/plain") {
        return Err(invalid());
    }
    content_keys(&raw["content"])?;
    contract_keys(&raw["contract"])?;
    contract_keys(&raw["revision"]["content_contract"])?;
    content_keys(&raw["revision"]["content"])?;
    admit(&m.content, m.contract.as_ref(), &m.relative_path)
}
fn graph(
    value: &Value,
    digest: &str,
    target: &str,
    head: Option<&str>,
    settings: &Value,
    draft: Option<&str>,
) -> Result<()> {
    if digest.len() != 64
        || !digest.bytes().all(|b| b.is_ascii_hexdigit())
        || value["context_artifact_id"] != target
        || value["expected_revision_id"].as_str() != head
    {
        return Err(invalid());
    }
    let nodes = value["operators"].as_array().ok_or_else(invalid)?;
    if nodes.len() > 128
        || !nodes.iter().any(|node| {
            node["configuration"] == *settings && draft.is_none_or(|id| node["id"] == id)
        })
    {
        return Err(invalid());
    }
    Ok(())
}
fn safe_file(root: &Path, relative: &str) -> Result<PathBuf> {
    if relative.is_empty() || relative.len() > 4096 || relative.contains('\\') {
        return Err(Error::new("shape_transfer_path"));
    }
    let path = Path::new(relative);
    let mut file = root.to_path_buf();
    for component in path.components() {
        let Component::Normal(name) = component else {
            return Err(Error::new("shape_transfer_path"));
        };
        file.push(name);
        if fs::symlink_metadata(&file)?.file_type().is_symlink() {
            return Err(Error::new("shape_transfer_path"));
        }
    }
    if !file.is_file() || !file.canonicalize()?.starts_with(root) {
        return Err(Error::new("shape_transfer_path"));
    }
    Ok(file)
}
