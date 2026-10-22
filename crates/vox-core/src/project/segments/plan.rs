use super::model::{SegmentCandidatePreview, SegmentPlan, SegmentPreview};
use crate::{
    SpeechSynthesisOperation,
    assembly::{ScriptAction, ScriptActionKind},
    project::{Document, Error, Manifest, Project, Result, ScriptFormat, SourceRevision},
    speech_script::{SpeechScriptEventKind, parse_speech_script},
};
use std::collections::BTreeMap;

pub(in crate::project) struct SegmentDefinition {
    pub key: String,
    pub index: usize,
    pub line: u32,
    pub role: Option<String>,
    pub text: String,
    pub spoken_text: String,
    pub operation: SpeechSynthesisOperation,
}
pub(in crate::project) struct CompiledSegments {
    pub actions: Vec<ScriptAction>,
    pub segments: Vec<SegmentDefinition>,
}

pub(super) fn adopted_source(m: &Manifest) -> Result<&SourceRevision> {
    let source = m
        .sources
        .iter()
        .find(|s| Some(&s.id) == m.source_head.as_ref())
        .ok_or_else(|| Error::new("source_missing"))?;
    super::super::shape_transfer::require_executable(m, &source.document)?;
    if source.document != m.draft {
        return Err(Error::new("unadopted_script"));
    }
    if !source.document.analysis()["ready"]
        .as_bool()
        .unwrap_or(false)
    {
        return Err(Error::new("script_not_ready"));
    }
    Ok(source)
}

impl Project {
    pub fn segment_plan(&self) -> Result<SegmentPlan> {
        self.segment_plan_for(&self.read()?)
    }

    /// Build from the caller's exact manifest snapshot, so asynchronous adapters
    /// never combine a newer segment plan with an older project version.
    pub fn segment_plan_for(&self, m: &Manifest) -> Result<SegmentPlan> {
        let analysis = m.draft.analysis();
        let adopted = m
            .sources
            .iter()
            .any(|s| Some(&s.id) == m.source_head.as_ref() && s.document == m.draft);
        let mut plan = SegmentPlan {
            source_revision: adopted.then(|| m.source_head.clone()).flatten(),
            document_digest: m.draft.digest(),
            adopted,
            ready: analysis["ready"].as_bool().unwrap_or(false),
            all_accepted: false,
            can_assemble: false,
            issues: analysis["issues"].clone(),
            segments: Vec::new(),
        };
        if !plan.ready {
            return Ok(plan);
        }
        let compiled = match self.compile_segments(&m.draft) {
            Ok(compiled) => compiled,
            Err(error) => {
                plan.ready = false;
                if let Some(issues) = plan.issues.as_array_mut() {
                    issues.push(serde_json::json!({"line":0,"code":error.code,"text":""}));
                }
                return Ok(plan);
            }
        };
        for definition in compiled.segments {
            let candidates = m
                .segment_candidates
                .iter()
                .map(|r| (r, false))
                .chain(m.accepted_segments.iter().map(|r| (r, true)))
                .filter(|(r, _)| r.key == definition.key)
                .map(|(r, accepted)| SegmentCandidatePreview {
                    id: r.id.clone(),
                    duration_millis: r.contract.duration_millis(),
                    accepted,
                    source_revision: r.source_revision.clone(),
                })
                .collect();
            let selected = m
                .segment_selections
                .get(&definition.key)
                .and_then(|id| m.accepted_segments.iter().find(|r| &r.id == id));
            plan.segments.push(SegmentPreview {
                key: definition.key,
                index: definition.index,
                line: definition.line,
                role: definition.role,
                language: definition.operation.language,
                text: definition.text,
                spoken_text: definition.spoken_text,
                candidates,
                accepted_id: selected.map(|r| r.id.clone()),
                duration_millis: selected.map(|r| r.contract.duration_millis()),
            });
        }
        plan.all_accepted =
            !plan.segments.is_empty() && plan.segments.iter().all(|s| s.accepted_id.is_some());
        plan.can_assemble = plan.ready && plan.adopted && plan.all_accepted;
        Ok(plan)
    }

    pub(in crate::project) fn compile_segments(
        &self,
        document: &Document,
    ) -> Result<CompiledSegments> {
        let operation = document.operation()?;
        let events = parse_speech_script(&document.text).events;
        let actions = if document.format == ScriptFormat::ProductionScript {
            crate::assembly::compile(&document.text, &operation, &self.render_inputs(document)?)?
        } else {
            crate::infer_runtime::speech::narration::split_text(&document.text)
                .into_iter()
                .enumerate()
                .map(|(index, part)| {
                    let mut selected = operation.clone();
                    selected.pronunciation_rules = crate::pronunciation::for_context(
                        part,
                        &operation.pronunciation_rules,
                        &operation.language,
                        None,
                    );
                    ScriptAction {
                        event: index as u32,
                        kind: ScriptActionKind::Speech {
                            text: part.into(),
                            operation: Box::new(selected),
                        },
                    }
                })
                .collect()
        };
        let mut segments = Vec::new();
        let mut occurrences = BTreeMap::<String, usize>::new();
        let mut plain_offset = 0;
        for action in &actions {
            if let ScriptActionKind::Speech { text, operation } = &action.kind {
                if segments.len() >= 512 {
                    return Err(Error::new("speech_text_too_long"));
                }
                let spoken_text = crate::pronunciation::apply(text, &operation.pronunciation_rules)
                    .ok_or_else(|| Error::new("invalid_pronunciation"))?;
                let identity =
                    serde_json::to_vec(&("vox.segment/1", text, &spoken_text, operation))?;
                let base = blake3::hash(&identity).to_hex().to_string();
                let occurrence = occurrences.entry(base.clone()).or_default();
                *occurrence += 1;
                let (line, role) = if document.format == ScriptFormat::ProductionScript {
                    let event = events
                        .get(action.event as usize)
                        .ok_or_else(|| Error::new("invalid_speech_script"))?;
                    (
                        event.line,
                        match &event.kind {
                            SpeechScriptEventKind::Speech { role, .. } => Some(role.clone()),
                            _ => None,
                        },
                    )
                } else {
                    let line = 1 + document.text[..plain_offset]
                        .bytes()
                        .filter(|b| *b == b'\n')
                        .count() as u32;
                    plain_offset += text.len();
                    (line, None)
                };
                segments.push(SegmentDefinition {
                    key: base,
                    index: segments.len(),
                    line,
                    role,
                    text: text.clone(),
                    spoken_text,
                    operation: *operation.clone(),
                });
            }
        }
        // Duplicate passages can have deliberately different takes. If their
        // multiplicity changes, invalidate the group instead of shifting an
        // occurrence onto a different accepted alternative.
        let mut ordinals = BTreeMap::<String, usize>::new();
        for segment in &mut segments {
            let base = segment.key.clone();
            let ordinal = ordinals.entry(base.clone()).or_default();
            segment.key =
                blake3::hash(&serde_json::to_vec(&(&base, *ordinal, occurrences[&base]))?)
                    .to_hex()
                    .to_string();
            *ordinal += 1;
        }
        Ok(CompiledSegments { actions, segments })
    }
}
