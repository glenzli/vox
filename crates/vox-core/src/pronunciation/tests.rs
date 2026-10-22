use super::*;

fn rule(text: &str, spoken: &str) -> PronunciationRule {
    PronunciationRule {
        text: text.into(),
        spoken: spoken.into(),
        language: None,
        role: None,
    }
}

#[test]
fn longest_match_non_recursive_and_utf8_are_deterministic() {
    let rules = vec![
        rule("PC", "computer"),
        rule("PCP", "PC"),
        rule("行", "hang"),
    ];
    assert_eq!(apply("PCP 银行 PC", &rules).unwrap(), "PC 银hang computer");
}

#[test]
fn context_is_exact_and_unrelated_rules_do_not_change_segment_settings() {
    let mut r = rule("行", "hang");
    r.language = Some("Chinese".into());
    r.role = Some("A".into());
    let rules = vec![r, rule("absent", "unused")];
    assert!(for_context("行", &rules, "Chinese", Some("B")).is_empty());
    let selected = for_context("行", &rules, "Chinese", Some("A"));
    assert_eq!(selected, vec![rule("行", "hang")]);
}

#[test]
fn rejects_ambiguous_empty_and_unbounded_rules() {
    assert!(!valid_rules(&[rule("", "x")]));
    assert!(!valid_rules(&[rule("x", "\n")]));
    assert!(!valid_rules(&[rule("x", "a"), rule("x", "b")]));
    assert!(apply("x", &[rule("", "x")]).is_none());
    assert!(apply(&"x".repeat(100), &[rule("x", &"a".repeat(1024))]).is_none());
}

#[test]
fn context_specific_rule_overrides_global_regardless_of_declaration_order() {
    let global = rule("行", "xing");
    let mut scoped = rule("行", "hang");
    scoped.role = Some("Banker".into());
    let mut exact = rule("行", "hang2");
    exact.role = Some("Banker".into());
    exact.language = Some("Chinese".into());
    let selected = for_context("银行", &[global, scoped, exact], "Chinese", Some("Banker"));
    assert_eq!(apply("银行", &selected).unwrap(), "银hang2");
    let mut invalid = rule("x", "y");
    invalid.language = Some("en".into());
    assert!(!valid_rules(&[invalid]));
}
