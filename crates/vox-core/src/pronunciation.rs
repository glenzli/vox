//! Literal project pronunciation rules. Matching never reads generated output again.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PronunciationRule {
    pub text: String,
    pub spoken: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

pub(crate) fn valid_rules(rules: &[PronunciationRule]) -> bool {
    rules.len() <= 128
        && rules.iter().all(|r| {
            !r.text.trim().is_empty()
                && r.text.len() <= 256
                && !r.spoken.trim().is_empty()
                && r.spoken.len() <= 1024
                && !r.text.chars().any(char::is_control)
                && !r.spoken.chars().any(char::is_control)
                && r.language.as_ref().is_none_or(|s| {
                    crate::infer_runtime::speech::SPEECH_LANGUAGES.contains(&s.as_str())
                })
                && r.role.as_ref().is_none_or(|s| {
                    !s.trim().is_empty() && s.len() <= 256 && !s.chars().any(char::is_control)
                })
        })
        && rules
            .iter()
            .map(|r| {
                r.text.len()
                    + r.spoken.len()
                    + r.language.as_ref().map_or(0, String::len)
                    + r.role.as_ref().map_or(0, String::len)
            })
            .sum::<usize>()
            <= 16_384
        && rules.iter().enumerate().all(|(i, r)| {
            !rules[..i]
                .iter()
                .any(|old| old.text == r.text && old.language == r.language && old.role == r.role)
        })
}

/// Keep only matching rules and erase context after it has been resolved. The
/// resulting operation is executable on its own and remains stable when unrelated
/// project rules change.
pub(crate) fn for_context(
    text: &str,
    rules: &[PronunciationRule],
    language: &str,
    role: Option<&str>,
) -> Vec<PronunciationRule> {
    let mut selected = Vec::<(usize, PronunciationRule)>::new();
    for rule in rules.iter().filter(|r| {
        r.language.as_ref().is_none_or(|s| s == language)
            && r.role.as_deref().is_none_or(|s| Some(s) == role)
            && text.contains(&r.text)
    }) {
        let specificity = usize::from(rule.language.is_some()) + usize::from(rule.role.is_some());
        let normalized = PronunciationRule {
            text: rule.text.clone(),
            spoken: rule.spoken.clone(),
            language: None,
            role: None,
        };
        if let Some(existing) = selected.iter_mut().find(|(_, r)| r.text == rule.text) {
            if specificity > existing.0 {
                *existing = (specificity, normalized);
            }
        } else {
            selected.push((specificity, normalized));
        }
    }
    selected.into_iter().map(|(_, rule)| rule).collect()
}

/// Left to right, longest literal match; declaration order breaks equal-length
/// ties. Bound expansion and never recursively match replacement bytes.
pub(crate) fn apply(text: &str, rules: &[PronunciationRule]) -> Option<String> {
    if !valid_rules(rules) {
        return None;
    }
    let mut result = String::new();
    let mut offset = 0;
    while offset < text.len() {
        let rest = &text[offset..];
        let selected = rules
            .iter()
            .enumerate()
            .filter(|(_, r)| rest.starts_with(&r.text))
            .max_by_key(|(i, r)| (r.text.len(), std::cmp::Reverse(*i)))
            .map(|(_, r)| r);
        if let Some(rule) = selected {
            result.push_str(&rule.spoken);
            offset += rule.text.len();
        } else {
            let ch = rest.chars().next()?;
            result.push(ch);
            offset += ch.len_utf8();
        }
        if result.len() > 16_384 {
            return None;
        }
    }
    Some(result)
}

#[cfg(test)]
mod tests;
