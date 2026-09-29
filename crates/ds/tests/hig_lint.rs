//! The HIG guardrails (design/27-HIG-PARITY.md section 7, H0): one passing and one failing case
//! per warning rule, that they stay warnings (never in `stylesheet` or `markup`, never failing
//! `assert_clean`), that Standard does not run them, and quire's own stylesheet under them.

use ds::lint::{
    Exception, LintConfig, Offence, Profile, Rule, Severity, WARNINGS, assert_clean, markup,
    markup_warnings, stylesheet, warnings,
};

fn strict() -> LintConfig {
    LintConfig {
        profile: Profile::Strict,
        ..LintConfig::default()
    }
}

fn fires(offences: &[Offence], rule: Rule) -> Fired {
    if offences.iter().any(|offence| offence.rule == rule) {
        Fired::Yes
    } else {
        Fired::No
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fired {
    Yes,
    No,
}

struct CssCase {
    name: &'static str,
    css: &'static str,
    rule: Rule,
    expect: Fired,
}

const CSS_CASES: &[CssCase] = &[
    CssCase {
        name: "a blur, which vello_cpu drops",
        css: ".orb { filter: blur(4px); }",
        rule: Rule::FilterNotPainted,
        expect: Fired::Yes,
    },
    CssCase {
        name: "a filter of none",
        css: ".orb { filter: none; }",
        rule: Rule::FilterNotPainted,
        expect: Fired::No,
    },
    CssCase {
        name: "pointer on a control",
        css: ".row { cursor: pointer; }",
        rule: Rule::PointerCursor,
        expect: Fired::Yes,
    },
    CssCase {
        name: "the arrow on a control",
        css: ".row { cursor: default; }",
        rule: Rule::PointerCursor,
        expect: Fired::No,
    },
    CssCase {
        name: "pointer on an anchor",
        css: "a[*|href] { cursor: pointer; }",
        rule: Rule::PointerCursor,
        expect: Fired::No,
    },
    CssCase {
        name: "pointer on a link class, hovered, under a parent",
        css: ".note .run-link:hover { cursor: pointer; }",
        rule: Rule::PointerCursor,
        expect: Fired::No,
    },
    CssCase {
        name: "pointer on a role=link",
        css: ".card [*|role=link] { cursor: pointer; }",
        rule: Rule::PointerCursor,
        expect: Fired::No,
    },
    CssCase {
        name: "pointer on a list with one control in it",
        css: ".run-link, .row { cursor: pointer; }",
        rule: Rule::PointerCursor,
        expect: Fired::Yes,
    },
    CssCase {
        name: "pointer on a control inside a link",
        css: ".link-list .row { cursor: pointer; }",
        rule: Rule::PointerCursor,
        expect: Fired::Yes,
    },
    CssCase {
        name: "a literal under the floor",
        css: ".tag { font-size: 9px; }",
        rule: Rule::MinFontSize,
        expect: Fired::Yes,
    },
    CssCase {
        name: "a pt literal under the floor",
        css: ".tag { font-size: 7pt; }",
        rule: Rule::MinFontSize,
        expect: Fired::Yes,
    },
    CssCase {
        name: "the shorthand's size under the floor",
        css: ".tag { font: 600 9.5px/1.2 var(--font-ui); }",
        rule: Rule::MinFontSize,
        expect: Fired::Yes,
    },
    CssCase {
        name: "a token under the floor",
        css: ".mark { font-size: var(--fs-pico); }",
        rule: Rule::MinFontSize,
        expect: Fired::Yes,
    },
    CssCase {
        name: "the floored tokens",
        css: ".a { font-size: var(--fs-micro); } .b { font-size: var(--fs-nano); } \
              .c { font-size: var(--fs-dial); } .d { font-size: var(--fs-help); }",
        rule: Rule::MinFontSize,
        expect: Fired::No,
    },
    CssCase {
        name: "a literal at the floor, and a line height under it",
        css: ".tag { font-size: 10px; } .t { font: 13px/8px var(--font-ui); }",
        rule: Rule::MinFontSize,
        expect: Fired::No,
    },
    CssCase {
        name: "a fixed radius on a focus ring",
        css: ".x:focus { outline: var(--focus-ring) solid var(--accent); border-radius: var(--r-tiny); }",
        rule: Rule::FocusRingShape,
        expect: Fired::Yes,
    },
    CssCase {
        name: "a ring one gap out from the control's radius",
        css: ".x:focus { border-radius: calc(var(--r-btn) + var(--focus-gap)); }",
        rule: Rule::FocusRingShape,
        expect: Fired::No,
    },
    CssCase {
        name: "a radius outside a focus rule",
        css: ".x { border-radius: var(--r-tiny); } .x::focus-ring { border-radius: 0; }",
        rule: Rule::FocusRingShape,
        expect: Fired::No,
    },
];

struct MarkupCase {
    name: &'static str,
    html: &'static str,
    rule: Rule,
    expect: Fired,
}

const MARKUP_CASES: &[MarkupCase] = &[
    MarkupCase {
        name: "an icon button with nothing to say",
        html: "<button class=\"ds-icon-button\"><svg class=\"ds-ic\" aria-hidden=\"true\"></svg></button>",
        rule: Rule::UnnamedControl,
        expect: Fired::Yes,
    },
    MarkupCase {
        name: "an icon button with a label",
        html: "<button class=\"ds-icon-button\" aria-label=\"Close\"><svg class=\"ds-ic\" aria-hidden=\"true\"></svg></button>",
        rule: Rule::UnnamedControl,
        expect: Fired::No,
    },
    MarkupCase {
        name: "a button whose text is inside a span",
        html: "<button class=\"ds-button\"><span>Send</span><!--x--></button>",
        rule: Rule::UnnamedControl,
        expect: Fired::No,
    },
    MarkupCase {
        name: "hidden text does not name",
        html: "<div role=\"button\"><span aria-hidden=\"true\">x</span></div>",
        rule: Rule::UnnamedControl,
        expect: Fired::Yes,
    },
    MarkupCase {
        name: "a field named by a label",
        html: "<label for=\"q\">Search</label><input id=\"q\" class=\"ds-input\">",
        rule: Rule::UnnamedControl,
        expect: Fired::No,
    },
    MarkupCase {
        name: "a bare field",
        html: "<input class=\"ds-input\" type=\"text\">",
        rule: Rule::UnnamedControl,
        expect: Fired::Yes,
    },
    MarkupCase {
        name: "a control inside a hidden subtree",
        html: "<div aria-hidden=\"true\"><button class=\"ds-button\"></button></div>",
        rule: Rule::UnnamedControl,
        expect: Fired::No,
    },
    MarkupCase {
        name: "three dots in a menu item",
        html: "<div role=\"menuitem\">Rename...</div>",
        rule: Rule::ThreeDots,
        expect: Fired::Yes,
    },
    MarkupCase {
        name: "three dots in a label attribute",
        html: "<button aria-label=\"More...\"></button>",
        rule: Rule::ThreeDots,
        expect: Fired::Yes,
    },
    MarkupCase {
        name: "an ellipsis",
        html: "<div role=\"menuitem\">Rename…</div>",
        rule: Rule::ThreeDots,
        expect: Fired::No,
    },
    MarkupCase {
        name: "three dots in a stylesheet",
        html: "<style>.a::after{content:\"...\"}</style><p>ok</p>",
        rule: Rule::ThreeDots,
        expect: Fired::No,
    },
];

#[test]
fn every_css_case_matches() {
    let failures: Vec<String> = CSS_CASES
        .iter()
        .filter(|case| fires(&warnings(case.css, &strict()), case.rule) != case.expect)
        .map(|case| format!("{}: {:?} expected {:?}", case.name, case.rule, case.expect))
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_markup_case_matches() {
    let failures: Vec<String> = MARKUP_CASES
        .iter()
        .filter(|case| fires(&markup_warnings(case.html, &strict()), case.rule) != case.expect)
        .map(|case| format!("{}: {:?} expected {:?}", case.name, case.rule, case.expect))
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_warning_rule_has_both_cases() {
    let seen: Vec<(Rule, Fired)> = CSS_CASES
        .iter()
        .map(|case| (case.rule, case.expect))
        .chain(MARKUP_CASES.iter().map(|case| (case.rule, case.expect)))
        .collect();
    for rule in WARNINGS {
        assert_eq!(rule.severity(), Severity::Warning, "{rule:?}");
        assert!(
            seen.contains(&(rule, Fired::Yes)),
            "{rule:?} has no failing case"
        );
        assert!(
            seen.contains(&(rule, Fired::No)),
            "{rule:?} has no passing case"
        );
    }
}

/// A warning never reaches `stylesheet` or `markup` and never fails `assert_clean`.
#[test]
fn a_warning_does_not_fail() {
    let css = ".row { cursor: pointer; font-size: var(--fs-pico); }";
    assert!(stylesheet(css, &strict()).is_empty());
    assert_eq!(warnings(css, &strict()).len(), 2);
    assert_clean(css, &strict());
    let html = "<button class=\"ds-button\"></button>";
    assert!(markup(html, ".ds-button{}", &strict()).is_empty());
    assert_eq!(markup_warnings(html, &strict()).len(), 1);
}

/// An exception silences a warning, and one that silences only a warning is not stale.
#[test]
fn an_exception_covers_a_warning() {
    const EXCEPTIONS: &[Exception] = &[Exception {
        rule: Rule::PointerCursor,
        selector: ".row",
        reason: "a test",
    }];
    let config = LintConfig {
        exceptions: EXCEPTIONS,
        ..strict()
    };
    assert!(warnings(".row { cursor: pointer; }", &config).is_empty());
    assert_clean(".row { cursor: pointer; }", &config);
}

/// quire's own stylesheet under the guardrails: every warning left is named here with its
/// reason, so a new one fails this test (quire is held to them now; consumers are warned).
#[test]
fn quires_own_sheet_warns_only_where_reviewed() {
    const REVIEWED: &[(Rule, &str, &str)] = &[
        (
            Rule::FilterNotPainted,
            ".ds-voice-orb-glow",
            "the orb's blur softens its glows on the GPU renderer; vello_cpu and the contrast() step draw them sharp (FINDINGS \"CSS filter\")",
        ),
        (
            Rule::MinFontSize,
            ".ds-provider[*|data-size=row]",
            "the in-row provider mark's letter is a drawing in an 11 px mark, not text",
        ),
        (
            Rule::FocusRingShape,
            ".ds[*|data-modality=keyboard] :focus",
            "the global ring's fixed radius is H3's to reshape (design/27 section 6.4)",
        ),
    ];
    let left: Vec<String> = warnings(ds::stylesheet(), &strict())
        .into_iter()
        .filter(|offence| {
            !REVIEWED
                .iter()
                .any(|(rule, selector, _)| offence.rule == *rule && offence.selector == *selector)
        })
        .map(|offence| format!("{:?} {}: {}", offence.rule, offence.line, offence.text))
        .collect();
    assert!(left.is_empty(), "{}", left.join("\n"));
}

/// Under System every size step the interface draws text at is at least the floor; the one
/// below it is a drawing.
#[test]
fn the_system_ramp_starts_at_the_floor() {
    use ds::Typeface;
    let under: Vec<ds::FontSize> = ds::FontSize::ALL
        .into_iter()
        .filter(|size| size.px_in(Typeface::System) < ds::FontSize::MIN_PX)
        .collect();
    assert_eq!(under, [ds::FontSize::Pico]);
    for size in ds::FontSize::FLOORED {
        assert_eq!(size.css_in(Typeface::System), "10px", "{size:?}");
    }
}

#[path = "support/golden.rs"]
#[allow(dead_code)] // Only the directory scan is used here.
mod golden;

/// Every rendered golden under the markup guardrails: the only hit is a case that renders an
/// `EditSurface` without its optional `label` (making it required is H2's, design/27 section 7).
#[test]
fn quires_goldens_warn_only_where_reviewed() {
    const REVIEWED: &[(&str, Rule)] = &[("controls/edit_surface/empty.html", Rule::UnnamedControl)];
    let left: Vec<String> = golden::all_in("")
        .into_iter()
        .filter(|(name, _)| name.ends_with(".html"))
        .flat_map(|(name, html)| {
            markup_warnings(&html, &strict())
                .into_iter()
                .filter(|offence| !REVIEWED.contains(&(name.as_str(), offence.rule)))
                .map(|offence| {
                    format!(
                        "{name}:{} {:?} {}",
                        offence.line, offence.rule, offence.text
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(left.is_empty(), "{}", left.join("\n"));
}
