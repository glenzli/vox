//! Explicit, pinned file round trip with Echo's independent editor. Never opens its Catalog.
mod pcm24;
use crate::{
    ContentDigest, ContentRef,
    audio::parse_pcm_s16le_wav,
    project::{AudioRevision, Error, Manifest, Project, Result, store::read_bounded},
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ticket {
    pub schema: String,
    pub id: String,
    pub project_id: String,
    pub audio_revision: String,
    pub source_revision: String,
    pub input: ContentRef,
    pub working_digest: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditReceipt {
    pub ticket_id: String,
    pub editor: String,
    pub input_digest: String,
    pub returned_digest: String,
    pub normalization: String,
    pub return_kind: String,
}
impl Project {
    pub fn begin_echo(
        &self,
        expected: u64,
        audio_id: &str,
        destination: &Path,
    ) -> Result<(Manifest, Ticket)> {
        let m = self.read()?;
        let r = m
            .audio
            .iter()
            .find(|r| r.id == audio_id)
            .ok_or_else(|| Error::new("accept_before_echo"))?;
        if Some(&r.source_revision) != m.source_head.as_ref() {
            return Err(Error::new("stale_source"));
        }
        let bytes = self.object(&r.content)?;
        // Echo's public compact declaration is carried in a WAV INFO comment.
        let working = with_generated_disclosure(&bytes)?;
        let ticket = Ticket {
            schema: "vox.echo-handoff/1".into(),
            id: uuid::Uuid::now_v7().to_string(),
            project_id: m.id,
            audio_revision: r.id.clone(),
            source_revision: r.source_revision.clone(),
            input: r.content.clone(),
            working_digest: ContentDigest::from_bytes(&working).to_string(),
        };
        fs::create_dir(destination).map_err(|_| Error::new("destination_exists"))?;
        write_new(&destination.join("source.wav"), &working)?;
        let (m, _) = self.update(expected, |m| {
            if m.handoffs.len() >= 1024 {
                return Err(Error::new("project_limit"));
            }
            m.handoffs.push(ticket.clone());
            Ok(())
        })?;
        write_new(
            &destination.join("handoff.json"),
            &serde_json::to_vec_pretty(&ticket)?,
        )?;
        Ok((m, ticket))
    }
    pub fn review_echo(
        &self,
        expected: u64,
        ticket_path: &Path,
        returned: &Path,
        allow_pcm24: bool,
    ) -> Result<Manifest> {
        let ticket: Ticket = serde_json::from_slice(&read_bounded(ticket_path, 128 * 1024)?)?;
        let m = self.read()?;
        if ticket.schema != "vox.echo-handoff/1"
            || ticket.project_id != m.id
            || !m.handoffs.contains(&ticket)
        {
            return Err(Error::new("invalid_handoff"));
        }
        let parent = m
            .audio
            .iter()
            .find(|r| r.id == ticket.audio_revision)
            .ok_or_else(|| Error::new("audio_missing"))?;
        if parent.content != ticket.input
            || parent.source_revision != ticket.source_revision
            || m.source_head.as_ref() != Some(&parent.source_revision)
        {
            return Err(Error::new("stale_source"));
        }
        self.object(&ticket.input)?;
        let raw = read_bounded(returned, 128 * 1024 * 1024)?;
        let convert = pcm24::is_pcm24(&raw);
        if convert && !allow_pcm24 {
            return Err(Error::new("pcm24_confirmation_required"));
        }
        let normalized = if convert {
            pcm24::convert(&raw)?
        } else {
            raw.clone()
        };
        let contract = parse_pcm_s16le_wav(&normalized, parent.contract.origin)?;
        let content = self.put_object(&normalized, "audio/wav")?;
        let receipt = EditReceipt {
            ticket_id: ticket.id,
            editor: "echo".into(),
            input_digest: ticket.input.digest.to_string(),
            returned_digest: ContentDigest::from_bytes(&raw).to_string(),
            normalization: if convert {
                "pcm24_to_pcm16_round_nearest"
            } else {
                "none"
            }
            .into(),
            return_kind: "user_selected_file".into(),
        };
        let r = AudioRevision {
            id: uuid::Uuid::now_v7().to_string(),
            source_revision: parent.source_revision.clone(),
            document_digest: parent.document_digest.clone(),
            content,
            contract,
            provenance: None,
            origin: "echo_file_return".into(),
            parent_audio: Some(parent.id.clone()),
            source_project: None,
            edit: Some(receipt),
        };
        self.update(expected, |m| {
            if m.source_head.as_ref() != Some(&r.source_revision) {
                return Err(Error::new("stale_source"));
            }
            if m.candidates.len() + m.audio.len() >= 1024 {
                return Err(Error::new("project_limit"));
            }
            m.candidates.push(r);
            Ok(())
        })
        .map(|x| x.0)
    }
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut f = OpenOptions::new().write(true).create_new(true).open(path)?;
    f.write_all(bytes)?;
    f.sync_all()?;
    Ok(())
}
fn with_generated_disclosure(bytes: &[u8]) -> Result<Vec<u8>> {
    parse_pcm_s16le_wav(bytes, crate::AudioOriginDisclosure::SyntheticSpeech)?;
    let comment=b"Echo source disclosure: {\"schema\":\"echo.source-disclosure.v1\",\"scope\":\"referenced_sources\",\"kinds\":[\"ai_generated\"]}\0";
    let mut chunk = b"INFOICMT".to_vec();
    chunk.extend((comment.len() as u32).to_le_bytes());
    chunk.extend(comment);
    if !comment.len().is_multiple_of(2) {
        chunk.push(0);
    }
    let mut out = bytes.to_vec();
    out.extend(b"LIST");
    out.extend((chunk.len() as u32).to_le_bytes());
    out.extend(chunk);
    let size = (out.len() - 8) as u32;
    out[4..8].copy_from_slice(&size.to_le_bytes());
    Ok(out)
}
