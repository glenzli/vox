use super::{
    SegmentAssembly, SegmentSelection, SegmentTiming, plan::adopted_source, render::leaf_provenance,
};
use crate::{
    AudioOriginDisclosure, ContentDigest, ExternalExecutionProvenance, SpeechSegmentProvenance,
    assembly::{SCRIPT_SAMPLE_RATE, ScriptActionKind, SpeechScriptAssembly, SpeechScriptPiece},
    audio::{SpeechWaveAssembly, decode_pcm_s16le_wav, parse_pcm_s16le_wav},
    project::{AudioRevision, Document, Error, Manifest, Project, Result, ScriptFormat},
    speech_script::parse_speech_script,
};
use std::fs::OpenOptions;

impl Project {
    /// Assemble only explicitly selected immutable segments. No model is called
    /// and the result remains a whole-take candidate until normal acceptance.
    pub fn assemble_segments(&self, expected: u64) -> Result<Manifest> {
        let m = self.read()?;
        if m.version != expected {
            return Err(Error::new("stale_project"));
        }
        let source = adopted_source(&m)?;
        let compiled = self.compile_segments(&source.document)?;
        let selections = compiled
            .segments
            .iter()
            .map(|s| {
                m.segment_selections
                    .get(&s.key)
                    .map(|id| SegmentSelection {
                        key: s.key.clone(),
                        segment_id: id.clone(),
                    })
                    .ok_or_else(|| Error::new("segment_selection_missing"))
            })
            .collect::<Result<Vec<_>>>()?;
        self.assemble_segment_selection(expected, &selections)
    }

    /// A chapter batch explicitly pins validated alternatives for review. It
    /// does not promote segment candidates or change accepted selections.
    pub(in crate::project) fn assemble_segment_selection(
        &self,
        expected: u64,
        selections: &[SegmentSelection],
    ) -> Result<Manifest> {
        let render_lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.path.join("render.lock"))?;
        render_lock
            .try_lock()
            .map_err(|_| Error::new("render_busy"))?;
        let m = self.read()?;
        if m.version != expected {
            return Err(Error::new("stale_project"));
        }
        let source = adopted_source(&m)?;
        let (bytes, provenance, assembly) =
            self.assemble_selected(&m, &source.document, selections)?;
        let record = AudioRevision {
            id: uuid::Uuid::now_v7().to_string(),
            source_revision: source.id.clone(),
            document_digest: source.document.digest(),
            content: self.put_object(&bytes, "audio/wav")?,
            contract: parse_pcm_s16le_wav(&bytes, AudioOriginDisclosure::SyntheticSpeech)?,
            provenance: Some(provenance),
            origin: "segment_assembly".into(),
            parent_audio: m.audio_head.clone(),
            source_project: None,
            edit: None,
            segment_assembly: Some(assembly),
        };
        self.update(expected, |m| {
            if m.audio.len() + m.candidates.len() >= 1024 {
                return Err(Error::new("project_limit"));
            }
            m.candidates.push(record);
            Ok(())
        })
        .map(|x| x.0)
    }

    pub(crate) fn validate_segment_assembly(&self, m: &Manifest, r: &AudioRevision) -> Result<()> {
        let source = m
            .sources
            .iter()
            .find(|s| s.id == r.source_revision)
            .ok_or_else(|| Error::new("source_missing"))?;
        let metadata = r
            .segment_assembly
            .as_ref()
            .ok_or_else(|| Error::new("invalid_segment_assembly"))?;
        if r.document_digest != source.document.digest()
            || metadata.document_digest != r.document_digest
        {
            return Err(Error::new("stale_source"));
        }
        let (bytes, provenance, rebuilt) =
            self.assemble_selected(m, &source.document, &metadata.selections)?;
        if bytes != self.object(&r.content)?
            || r.provenance.as_ref() != Some(&provenance)
            || *metadata != rebuilt
            || parse_pcm_s16le_wav(&bytes, r.contract.origin)? != r.contract
        {
            return Err(Error::new("invalid_segment_assembly"));
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn assemble_selected(
        &self,
        m: &Manifest,
        document: &Document,
        selections: &[SegmentSelection],
    ) -> Result<(Vec<u8>, ExternalExecutionProvenance, SegmentAssembly)> {
        let compiled = self.compile_segments(document)?;
        if compiled.segments.len() != selections.len() || selections.is_empty() {
            return Err(Error::new("invalid_segment_assembly"));
        }
        let mut selected = Vec::with_capacity(selections.len());
        for (definition, selection) in compiled.segments.iter().zip(selections) {
            if definition.key != selection.key {
                return Err(Error::new("stale_segment"));
            }
            let record = m
                .accepted_segments
                .iter()
                .chain(&m.segment_candidates)
                .find(|r| r.id == selection.segment_id && r.key == selection.key)
                .ok_or_else(|| Error::new("segment_selection_missing"))?;
            self.validate_segment(definition, record)?;
            selected.push(record);
        }
        let mut wave = SpeechWaveAssembly::default();
        let mut receipts = Vec::new();
        let plan = parse_speech_script(&document.text);
        let mut script_assembly = SpeechScriptAssembly {
            source_digest: plan.source_digest,
            operation_digest: ContentDigest::from_bytes(&serde_json::to_vec(
                &document.operation()?,
            )?),
            plan_digest: plan.digest(),
            pieces: Vec::new(),
        };
        let mut ranges: Vec<std::ops::Range<usize>> = Vec::new();
        let mut piece_timings: Vec<Option<SegmentTiming>> = Vec::new();
        let mut timings = Vec::new();
        let mut input_start = 0_u32;
        for action in compiled.actions {
            let start = wave.pcm_len();
            let start_frame = (start / 2) as u64;
            let (piece, timing) = match action.kind {
                ScriptActionKind::Replay { piece } => {
                    let index = piece as usize;
                    let original = script_assembly
                        .pieces
                        .get(index)
                        .ok_or_else(|| Error::new("invalid_segment_assembly"))?
                        .clone();
                    wave.replay_pcm(
                        ranges
                            .get(index)
                            .ok_or_else(|| Error::new("invalid_segment_assembly"))?
                            .clone(),
                    )?;
                    let timing = piece_timings
                        .get(index)
                        .cloned()
                        .flatten()
                        .map(|mut timing| {
                            timing.start_frame = start_frame;
                            timing.end_frame = start_frame + original.frames;
                            timing
                        });
                    (
                        SpeechScriptPiece {
                            event: action.event,
                            replay_of: Some(piece),
                            ..original
                        },
                        timing,
                    )
                }
                ScriptActionKind::Local { pcm } => {
                    wave.push_pcm(SCRIPT_SAMPLE_RATE, 1, &pcm)?;
                    (
                        SpeechScriptPiece {
                            event: action.event,
                            frames: (pcm.len() / 2) as u64,
                            pcm_digest: ContentDigest::from_bytes(&pcm),
                            speech_segment: None,
                            replay_of: None,
                        },
                        None,
                    )
                }
                ScriptActionKind::Speech { text, .. } => {
                    let index = receipts.len();
                    let record = selected
                        .get(index)
                        .ok_or_else(|| Error::new("invalid_segment_assembly"))?;
                    let bytes = self.object(&record.content)?;
                    let (_, pcm) =
                        decode_pcm_s16le_wav(&bytes, AudioOriginDisclosure::SyntheticSpeech)?;
                    wave.push_pcm(SCRIPT_SAMPLE_RATE, 1, pcm)?;
                    let input_end = input_start
                        + u32::try_from(text.len()).map_err(|_| Error::new("script_too_large"))?;
                    receipts.push(SpeechSegmentProvenance {
                        job_id: record.job_id.clone(),
                        input_start,
                        input_end,
                        input_digest: ContentDigest::from_bytes(text.as_bytes()),
                        spoken_digest: (record.text != record.spoken_text)
                            .then(|| ContentDigest::from_bytes(record.spoken_text.as_bytes())),
                        output_digest: record.content.digest,
                        frames: record.contract.frame_count,
                        runtime: Box::new(leaf_provenance(record)?),
                    });
                    input_start = input_end;
                    (
                        SpeechScriptPiece {
                            event: action.event,
                            frames: record.contract.frame_count,
                            pcm_digest: ContentDigest::from_bytes(pcm),
                            speech_segment: Some(index as u32),
                            replay_of: None,
                        },
                        Some(SegmentTiming {
                            segment_id: record.id.clone(),
                            key: record.key.clone(),
                            text,
                            start_frame,
                            end_frame: start_frame + record.contract.frame_count,
                        }),
                    )
                }
            };
            if let Some(timing) = &timing {
                timings.push(timing.clone());
            }
            piece_timings.push(timing);
            ranges.push(start..wave.pcm_len());
            script_assembly.pieces.push(piece);
        }
        let first = receipts
            .first()
            .ok_or_else(|| Error::new("segment_selection_missing"))?;
        let mut provenance = *first.runtime.clone();
        provenance.speech_segments = receipts;
        if document.format == ScriptFormat::ProductionScript {
            provenance.speech_script = Some(script_assembly);
        }
        let bytes = wave.finish()?;
        if !provenance.is_bounded() {
            return Err(Error::new("invalid_provenance"));
        }
        if document.format == ScriptFormat::ProductionScript {
            crate::assembly::validate_output(
                &document.text,
                &document.operation()?,
                &self.render_inputs(document)?,
                &provenance,
                &bytes,
            )?;
        }
        let metadata = SegmentAssembly {
            schema: "vox.segment-assembly/1".into(),
            document_digest: document.digest(),
            selections: selections.to_vec(),
            timings,
        };
        Ok((bytes, provenance, metadata))
    }
}
