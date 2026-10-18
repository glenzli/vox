//! Display projection of the authoritative plan; never changes execution or source.
use super::{SpeechScriptEventKind as Kind, SpeechScriptPlan};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// Groups repetitions and carries rejected source lines into a read-only preview.
/// Authored-note identity comes from the parser, not a text-prefix heuristic.
///
/// # Panics
/// Only if serialization or the internally constructed display object/array shape is invalid.
#[must_use]
pub fn reading_preview(source: &str, plan: &SpeechScriptPlan, note_lines: &[u32]) -> Value {
    let authored: BTreeSet<_> = note_lines.iter().copied().collect();
    let source_lines: Vec<_> = source.lines().collect();
    let mut lines = BTreeMap::new();
    for event in &plan.events {
        if matches!(event.kind, Kind::Note { .. }) && !authored.contains(&event.line) {
            continue;
        }
        let mut block = serde_json::to_value(event).expect("fixed reading event serializes");
        block["line_end"] = json!(event.line);
        block["issues"] = json!([]);
        lines.insert(event.line, block);
    }
    for issue in &plan.issues {
        let block = lines.entry(issue.line).or_insert_with(|| {
            json!({"kind":"issue", "line":issue.line, "line_end":issue.line,
                "text":source_lines.get(issue.line.saturating_sub(1) as usize).copied().unwrap_or(""),
                "issues":[]})
        });
        let issues = block["issues"]
            .as_array_mut()
            .expect("reading issues array");
        if !issues.iter().any(|value| value == &issue.code) {
            issues.push(json!(issue.code));
        }
    }
    let mut blocks = Vec::new();
    let mut repeated: Option<Value> = None;
    for (_, block) in lines {
        match block["kind"].as_str() {
            Some("repeat_start") => {
                if let Some(previous) = repeated.take() {
                    blocks.push(previous);
                }
                let mut group = block;
                group["kind"] = json!("repeat");
                group["blocks"] = json!([]);
                group["closed"] = json!(false);
                repeated = Some(group);
            }
            Some("repeat_end") => {
                if let Some(mut group) = repeated.take() {
                    group["closed"] = json!(true);
                    group["line_end"] = block["line"].clone();
                    blocks.push(group);
                }
            }
            _ => {
                let target = repeated.as_mut().map_or(&mut blocks, |group| {
                    group["blocks"]
                        .as_array_mut()
                        .expect("reading repeat array")
                });
                append(target, block);
            }
        }
    }
    if let Some(group) = repeated {
        blocks.push(group);
    }
    json!({"source_text":source, "production":plan.production, "blocks":blocks})
}

fn append(blocks: &mut Vec<Value>, block: Value) {
    if let Some(previous) = blocks.last_mut()
        && block["kind"] == "speech"
        && previous["kind"] == "speech"
        && ["role", "language", "delivery"]
            .iter()
            .all(|key| previous[key] == block[key])
        && previous["issues"] == json!([])
        && block["issues"] == json!([])
    {
        let gap = if block["line"].as_u64().unwrap_or(0)
            > previous["line_end"].as_u64().unwrap_or(0) + 1
        {
            "\n\n"
        } else {
            "\n"
        };
        previous["text"] = json!(format!(
            "{}{gap}{}",
            previous["text"].as_str().unwrap_or(""),
            block["text"].as_str().unwrap_or("")
        ));
        previous["line_end"] = block["line_end"].clone();
    } else {
        blocks.push(block);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::speech_script::{parse_speech_script, parse_speech_script_for_review};

    #[test]
    fn groups_repeat_metadata_without_mutating_plan_or_notes() {
        let source = "[role: Guide]\n[speaker: Guide]\n[language: English]\n[note: speaker: keep this authored reminder]\n[repeat: 2; gap: 0.4s]\nHello.\nAgain.\n[end-repeat]\n[language: Chinese]\n你好。";
        let (plan, notes) = parse_speech_script_for_review(source);
        let before = serde_json::to_vec(&plan).unwrap();
        let digest = plan.digest();
        let preview = reading_preview(source, &plan, &notes);
        assert_eq!(preview["source_text"], source);
        let blocks = preview["blocks"].as_array().unwrap();
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0]["kind"], "note");
        assert_eq!(blocks[0]["text"], "speaker: keep this authored reminder");
        assert_eq!(blocks[1]["kind"], "repeat");
        assert_eq!(blocks[1]["count"], 2);
        assert_eq!(blocks[1]["gap_ms"], 400);
        assert_eq!(blocks[1]["closed"], true);
        assert_eq!(blocks[1]["blocks"][0]["text"], "Hello.\nAgain.");
        assert_eq!(blocks[1]["blocks"][0]["language"], "English");
        assert_eq!(blocks[2]["language"], "Chinese");
        assert_eq!(serde_json::to_vec(&plan).unwrap(), before);
        assert_eq!(plan.digest(), digest);
        assert_eq!(plan, parse_speech_script(source));
    }

    #[test]
    fn rejected_source_is_visible_in_order_and_unclosed_repeats_stay_explicit() {
        let source = "[role: Guide]\r\n[language: Chinese]\r\n未选择角色的原文 😀\r\n[pause: 0.3s]\r\n[speaker: Guide]\r\n[repeat: 2]\r\nHello.";
        let (plan, notes) = parse_speech_script_for_review(source);
        let preview = reading_preview(source, &plan, &notes);
        let blocks = preview["blocks"].as_array().unwrap();
        assert_eq!(blocks[0]["kind"], "issue");
        assert_eq!(blocks[0]["text"], "未选择角色的原文 😀");
        assert_eq!(blocks[0]["line"], 3);
        assert_eq!(blocks[0]["issues"], json!(["missing_speaker"]));
        assert_eq!(blocks[1]["kind"], "pause");
        assert_eq!(blocks[2]["kind"], "repeat");
        assert_eq!(blocks[2]["closed"], false);
        assert_eq!(blocks[2]["issues"], json!(["unclosed_repeat"]));
        assert_eq!(blocks[2]["blocks"][0]["text"], "Hello.");
    }
}
