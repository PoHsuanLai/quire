//! `LintConfig.exceptions`: an exception silences exactly its (rule, selector) pair, in a
//! stylesheet and in markup, and `assert_clean` says how much each one silenced and fails on
//! one that silenced nothing unless `LintConfig.stale` is `Stale::Report`.

use ds::lint::{Exception, LintConfig, Rule, Stale, assert_clean, markup, stylesheet};

const TRUNCATE_MASK: &[Exception] = &[Exception {
    rule: Rule::HexColour,
    selector: ".fade",
    reason: "mask alpha only",
}];

#[test]
fn an_exception_suppresses_exactly_its_rule_and_selector() {
    let css = ".fade { mask-image: linear-gradient(#000, transparent); }\n\
               .fade .x { color: #000; }\n\
               .fade { color: red; }";
    let config = LintConfig {
        exceptions: TRUNCATE_MASK,
        ..LintConfig::default()
    };
    let kept: Vec<(Rule, String)> = stylesheet(css, &config)
        .into_iter()
        .map(|offence| (offence.rule, offence.selector))
        .collect();
    assert_eq!(
        kept,
        vec![
            (Rule::HexColour, ".fade .x".to_owned()),
            (Rule::NamedColour, ".fade".to_owned()),
        ],
        "the other selector and the other rule are not suppressed"
    );
    let bare = stylesheet(css, &LintConfig::default());
    assert_eq!(bare.len(), kept.len() + 1, "without it, it fires");
}

#[test]
fn a_markup_exception_names_the_element() {
    const INLINE: &[Exception] = &[Exception {
        rule: Rule::HexColour,
        selector: "span.ds-space-swatch",
        reason: "the editor's swatch paints its picked colour inline",
    }];
    let html = "<span class=\"ds-space-swatch\" style=\"background:#944242\"></span>";
    let bare = markup(html, ".ds-space-swatch{}", &LintConfig::default());
    assert_eq!(bare.len(), 1, "{bare:?}");
    assert_eq!(bare[0].selector, "span.ds-space-swatch");
    let config = LintConfig {
        exceptions: INLINE,
        ..LintConfig::default()
    };
    assert!(markup(html, ".ds-space-swatch{}", &config).is_empty());
}

#[test]
fn assert_clean_passes_when_exceptions_cover_everything() {
    let config = LintConfig {
        exceptions: TRUNCATE_MASK,
        ..LintConfig::default()
    };
    assert_clean(
        ".fade { mask-image: linear-gradient(#000, transparent); }",
        &config,
    );
}

#[test]
#[should_panic(expected = "1 x HexColour on .fade (mask alpha only)")]
fn assert_clean_counts_what_each_exception_suppressed() {
    let config = LintConfig {
        exceptions: TRUNCATE_MASK,
        ..LintConfig::default()
    };
    assert_clean(
        ".fade { mask-image: linear-gradient(#000, transparent); color: red; }",
        &config,
    );
}

/// An exception for an offence that has gone: the `.fade` rule no longer writes `#000`.
const CLEAN_FADE: &str = ".fade { mask-image: linear-gradient(var(--ink), transparent); }";

#[test]
#[should_panic(
    expected = "1 stale exception(s), which suppressed nothing:\n  HexColour on .fade (mask alpha only)"
)]
fn a_stale_exception_fails_by_default() {
    let config = LintConfig {
        exceptions: TRUNCATE_MASK,
        ..LintConfig::default()
    };
    assert_eq!(config.stale, Stale::Fail, "failing is the default");
    assert_clean(CLEAN_FADE, &config);
}

#[test]
fn a_stale_exception_is_only_reported_when_asked() {
    let config = LintConfig {
        exceptions: TRUNCATE_MASK,
        stale: Stale::Report,
        ..LintConfig::default()
    };
    assert_clean(CLEAN_FADE, &config);
}

#[test]
#[should_panic(expected = "stale exception(s)")]
fn an_offence_and_a_stale_exception_are_both_named() {
    const TWO: &[Exception] = &[
        Exception {
            rule: Rule::HexColour,
            selector: ".fade",
            reason: "mask alpha only",
        },
        Exception {
            rule: Rule::NamedColour,
            selector: ".gone",
            reason: "a rule since deleted",
        },
    ];
    let config = LintConfig {
        exceptions: TWO,
        ..LintConfig::default()
    };
    assert_clean(
        ".fade { mask-image: linear-gradient(#000, transparent); color: red; }",
        &config,
    );
}
