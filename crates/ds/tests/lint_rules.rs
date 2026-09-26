//! Table-driven coverage of every `ds::lint::Rule`: one passing and one failing case each, on
//! the `cssparser` token stream (never a substring search — CONVENTIONS "Substrings are not
//! tokens"). `mailo_cases` ports the case tables from
//! `mail-app/src/ui/style/mod.rs`'s `every_var_is_declared`, `no_colour_outside_the_palette`,
//! `current_color_is_only_a_stroke_or_fill_value` and `a_class_with_no_rule_is_named`.

use ds::lint::{LintConfig, Offence, Profile, Rule, markup, stylesheet};

fn lint(css: &str, profile: Profile) -> Vec<Offence> {
    stylesheet(
        css,
        &LintConfig {
            profile,
            ..LintConfig::default()
        },
    )
}

fn has(offences: &[Offence], rule: Rule) -> bool {
    offences.iter().any(|offence| offence.rule == rule)
}

struct Case {
    name: &'static str,
    css: &'static str,
    profile: Profile,
    rule: Rule,
    /// Whether `rule` is expected to fire somewhere in `css`.
    expect: bool,
}

/// One passing and one failing case per [`Rule`], in [`Rule::ALL`]'s order.
const CASES: &[Case] = &[
    // HexColour
    Case {
        name: "hex colour: literal fails",
        css: ".chip { color: #fff; }",
        profile: Profile::Standard,
        rule: Rule::HexColour,
        expect: true,
    },
    Case {
        name: "hex colour: token passes",
        css: ".chip { color: var(--ink); }",
        profile: Profile::Standard,
        rule: Rule::HexColour,
        expect: false,
    },
    // ColourFunction
    Case {
        name: "colour function: rgb() fails",
        css: ".chip { background: rgb(1, 2, 3); }",
        profile: Profile::Standard,
        rule: Rule::ColourFunction,
        expect: true,
    },
    Case {
        name: "colour function: token passes",
        css: ".chip { background: var(--surface); }",
        profile: Profile::Standard,
        rule: Rule::ColourFunction,
        expect: false,
    },
    // NamedColour
    Case {
        name: "named colour: red fails",
        css: ".chip { color: red; }",
        profile: Profile::Standard,
        rule: Rule::NamedColour,
        expect: true,
    },
    Case {
        name: "named colour: token passes",
        css: ".chip { color: var(--ink); }",
        profile: Profile::Standard,
        rule: Rule::NamedColour,
        expect: false,
    },
    Case {
        name: "named colour: transparent passes",
        css: ".chip { background: transparent; }",
        profile: Profile::Standard,
        rule: Rule::NamedColour,
        expect: false,
    },
    // CurrentColourOutsideStrokeFill
    Case {
        name: "currentColor on color: fails",
        css: ".label { color: currentColor; }",
        profile: Profile::Standard,
        rule: Rule::CurrentColourOutsideStrokeFill,
        expect: true,
    },
    Case {
        name: "currentColor on stroke alone: passes",
        css: ".ic { stroke: currentColor; }",
        profile: Profile::Standard,
        rule: Rule::CurrentColourOutsideStrokeFill,
        expect: false,
    },
    // RawDuration
    Case {
        name: "raw duration: 200ms fails",
        css: ".chip { transition: color 200ms; }",
        profile: Profile::Standard,
        rule: Rule::RawDuration,
        expect: true,
    },
    Case {
        name: "raw duration: token passes",
        css: ".chip { transition: color var(--t-quick); }",
        profile: Profile::Standard,
        rule: Rule::RawDuration,
        expect: false,
    },
    // RawEasing
    Case {
        name: "raw easing: ease-in-out fails",
        css: ".chip { transition-timing-function: ease-in-out; }",
        profile: Profile::Standard,
        rule: Rule::RawEasing,
        expect: true,
    },
    Case {
        name: "raw easing: token passes",
        css: ".chip { transition-timing-function: var(--e-in-out); }",
        profile: Profile::Standard,
        rule: Rule::RawEasing,
        expect: false,
    },
    // Keyframes
    Case {
        name: "keyframes: @keyframes fails",
        css: "@keyframes wiggle { from { opacity: 0; } to { opacity: 1; } }",
        profile: Profile::Standard,
        rule: Rule::Keyframes,
        expect: true,
    },
    Case {
        name: "keyframes: none passes",
        css: ".chip { animation-name: gulp; }",
        profile: Profile::Standard,
        rule: Rule::Keyframes,
        expect: false,
    },
    // UnknownAnimation
    Case {
        name: "unknown animation: made-up name fails",
        css: ".chip { animation-name: sparkle-explosion; }",
        profile: Profile::Standard,
        rule: Rule::UnknownAnimation,
        expect: true,
    },
    Case {
        name: "unknown animation: a real Anim passes",
        css: ".chip { animation-name: gulp; }",
        profile: Profile::Standard,
        rule: Rule::UnknownAnimation,
        expect: false,
    },
    // The `animation` shorthand names its keyframes too (mailo gaps 3).
    Case {
        name: "unknown animation: a made-up name in the shorthand fails",
        css: ".chip { animation: sparkle-explosion var(--t-big) var(--e-spring); }",
        profile: Profile::Standard,
        rule: Rule::UnknownAnimation,
        expect: true,
    },
    Case {
        name: "unknown animation: a made-up name after the shorthand's keywords fails",
        css: ".chip { animation: infinite alternate both paused sparkle var(--t-big); }",
        profile: Profile::Standard,
        rule: Rule::UnknownAnimation,
        expect: true,
    },
    Case {
        name: "unknown animation: the second of two shorthand animations is read too",
        css: ".chip { animation: gulp var(--t-big) var(--e-spring), sparkle var(--t-move) var(--e-out); }",
        profile: Profile::Standard,
        rule: Rule::UnknownAnimation,
        expect: true,
    },
    Case {
        name: "unknown animation: a real Anim in the shorthand passes",
        css: ".chip { animation: pill-up var(--t-big) var(--e-spring); }",
        profile: Profile::Standard,
        rule: Rule::UnknownAnimation,
        expect: false,
    },
    Case {
        name: "unknown animation: keywords, a count and its --b alias pass",
        css: ".chip { animation: var(--t-ambient) var(--e-in-out) infinite alternate backwards busy--b; }",
        profile: Profile::Standard,
        rule: Rule::UnknownAnimation,
        expect: false,
    },
    Case {
        name: "unknown animation: a raw easing's own words are not names",
        css: ".chip { animation: ease-in-out 1s steps(4, jump-end) spin; }",
        profile: Profile::Standard,
        rule: Rule::UnknownAnimation,
        expect: false,
    },
    Case {
        name: "unknown animation: none passes",
        css: ".chip { animation: none; }",
        profile: Profile::Standard,
        rule: Rule::UnknownAnimation,
        expect: false,
    },
    Case {
        name: "unknown animation: a name only a var() knows is not judged",
        css: ".chip { animation: var(--my-anim) var(--t-big); }",
        profile: Profile::Standard,
        rule: Rule::UnknownAnimation,
        expect: false,
    },
    // FontFamily
    Case {
        name: "font-family: literal fails",
        css: ".chip { font-family: Arial, sans-serif; }",
        profile: Profile::Standard,
        rule: Rule::FontFamily,
        expect: true,
    },
    Case {
        name: "font-family: absent passes",
        css: ".chip { color: var(--ink); }",
        profile: Profile::Standard,
        rule: Rule::FontFamily,
        expect: false,
    },
    Case {
        name: "font-family: a face token passes",
        css: ".chip { font-family: var(--font-data); }",
        profile: Profile::Standard,
        rule: Rule::FontFamily,
        expect: false,
    },
    Case {
        name: "font-family: the serif face token passes (mailo gaps 3)",
        css: ".serif { font-family: var(--font-serif); }",
        profile: Profile::Strict,
        rule: Rule::FontFamily,
        expect: false,
    },
    Case {
        name: "font-family: a serif named outright still fails",
        css: ".serif { font-family: Georgia, serif; }",
        profile: Profile::Strict,
        rule: Rule::FontFamily,
        expect: true,
    },
    Case {
        name: "font-family: a token with a literal fallback fails",
        css: ".chip { font-family: var(--font-data), monospace; }",
        profile: Profile::Standard,
        rule: Rule::FontFamily,
        expect: true,
    },
    // RawFontSize (Strict only)
    Case {
        name: "raw font-size: literal fails under Strict",
        css: ".chip { font-size: 14px; }",
        profile: Profile::Strict,
        rule: Rule::RawFontSize,
        expect: true,
    },
    Case {
        name: "raw font-size: token passes under Strict",
        css: ".chip { font-size: var(--fs-body); }",
        profile: Profile::Strict,
        rule: Rule::RawFontSize,
        expect: false,
    },
    // RawRadius (Strict only)
    Case {
        name: "raw radius: literal fails under Strict",
        css: ".chip { border-radius: 8px; }",
        profile: Profile::Strict,
        rule: Rule::RawRadius,
        expect: true,
    },
    Case {
        name: "raw radius: token passes under Strict",
        css: ".chip { border-radius: var(--r-chip); }",
        profile: Profile::Strict,
        rule: Rule::RawRadius,
        expect: false,
    },
    Case {
        name: "raw radius: a square corner passes under Strict",
        css: ".chip { border-radius: var(--r-pill) var(--r-pill) 0 0; }",
        profile: Profile::Strict,
        rule: Rule::RawRadius,
        expect: false,
    },
    Case {
        name: "raw radius: a custom property named -radius is not a radius",
        css: ".chip { --m-radius: 22px; }",
        profile: Profile::Strict,
        rule: Rule::RawRadius,
        expect: false,
    },
    // RawZIndex (Strict only)
    Case {
        name: "raw z-index: literal fails under Strict",
        css: ".chip { z-index: 5; }",
        profile: Profile::Strict,
        rule: Rule::RawZIndex,
        expect: true,
    },
    Case {
        name: "raw z-index: token passes under Strict",
        css: ".chip { z-index: var(--z-menu); }",
        profile: Profile::Strict,
        rule: Rule::RawZIndex,
        expect: false,
    },
    // RawSpacing
    Case {
        name: "raw spacing: a px padding fails under strict",
        css: ".chip { padding: 4px var(--s-8); }",
        profile: Profile::Strict,
        rule: Rule::RawSpacing,
        expect: true,
    },
    Case {
        name: "raw spacing: a px gap fails under strict",
        css: ".row { display: flex; column-gap: 6px; }",
        profile: Profile::Strict,
        rule: Rule::RawSpacing,
        expect: true,
    },
    Case {
        name: "raw spacing: a px margin side fails under strict",
        css: ".row { margin-inline-start: -2px; }",
        profile: Profile::Strict,
        rule: Rule::RawSpacing,
        expect: true,
    },
    Case {
        name: "raw spacing: tokens, zero and auto pass",
        css: ".chip { padding: var(--s-4) var(--s-8); margin: 0 auto; gap: var(--s-6); }",
        profile: Profile::Strict,
        rule: Rule::RawSpacing,
        expect: false,
    },
    Case {
        name: "raw spacing: a px width is not spacing",
        css: ".chip { width: 300px; scroll-margin-top: 8px; }",
        profile: Profile::Strict,
        rule: Rule::RawSpacing,
        expect: false,
    },
    Case {
        name: "raw spacing: standard allows a px padding",
        css: ".chip { padding: 4px; }",
        profile: Profile::Standard,
        rule: Rule::RawSpacing,
        expect: false,
    },
    // RawHairline (Strict only)
    Case {
        name: "raw hairline: a 1px border fails under strict",
        css: ".card { border: 1px solid var(--line); }",
        profile: Profile::Strict,
        rule: Rule::RawHairline,
        expect: true,
    },
    Case {
        name: "raw hairline: a 1px border side fails under strict",
        css: ".card { border-bottom: 1px solid var(--line-soft); }",
        profile: Profile::Strict,
        rule: Rule::RawHairline,
        expect: true,
    },
    Case {
        name: "raw hairline: a half-pixel outline width fails under strict",
        css: ".card { outline-width: .5px; }",
        profile: Profile::Strict,
        rule: Rule::RawHairline,
        expect: true,
    },
    Case {
        name: "raw hairline: a 1px-wide separator box fails under strict",
        css: ".sep { position: absolute; width: 1px; background-color: var(--f-line); }",
        profile: Profile::Strict,
        rule: Rule::RawHairline,
        expect: true,
    },
    Case {
        name: "raw hairline: the tokens, a 2px border and a radius pass",
        css: ".card { border: var(--hair) solid var(--line); outline: var(--focus-ring) solid \
              var(--accent); border-bottom-width: 2px; border-radius: 1px; min-width: 12px; } \
              .sep { height: var(--hair); }",
        profile: Profile::Strict,
        rule: Rule::RawHairline,
        expect: false,
    },
    Case {
        name: "raw hairline: standard allows a 1px border",
        css: ".card { border: 1px solid var(--line); }",
        profile: Profile::Standard,
        rule: Rule::RawHairline,
        expect: false,
    },
    // RootSelector
    Case {
        name: "root selector: :root fails",
        css: ":root { color: red; }",
        profile: Profile::Standard,
        rule: Rule::RootSelector,
        expect: true,
    },
    Case {
        name: "root selector: a class passes",
        css: ".chip { color: var(--ink); }",
        profile: Profile::Standard,
        rule: Rule::RootSelector,
        expect: false,
    },
    // DsInternals
    Case {
        name: "ds internals: .ds- class fails",
        css: ".ds-icon { color: red; }",
        profile: Profile::Standard,
        rule: Rule::DsInternals,
        expect: true,
    },
    Case {
        name: "ds internals: a consumer class passes",
        css: ".icon { color: var(--ink); }",
        profile: Profile::Standard,
        rule: Rule::DsInternals,
        expect: false,
    },
    Case {
        name: "ds internals: the root's warm-hover attribute fails",
        css: ".card[*|data-hover=warm] .fly { color: var(--ink); }",
        profile: Profile::Standard,
        rule: Rule::DsInternals,
        expect: true,
    },
    Case {
        name: "ds internals: a hover target's own key passes",
        css: ".card[*|data-hover-key] { color: var(--ink); }",
        profile: Profile::Standard,
        rule: Rule::DsInternals,
        expect: false,
    },
    Case {
        name: "ds internals: a consumer class inside the trailing slot seam passes",
        css: "[*|data-slot=trailing] .fold-more { color: var(--ink); }",
        profile: Profile::Standard,
        rule: Rule::DsInternals,
        expect: false,
    },
    Case {
        name: "ds internals: the same class reached through the slot's own class fails",
        css: ".ds-tree-item-trail .fold-more { color: var(--ink); }",
        profile: Profile::Standard,
        rule: Rule::DsInternals,
        expect: true,
    },
    // UndeclaredVar
    Case {
        name: "undeclared var: a typo fails",
        css: ".chip { color: var(--totally-not-real); }",
        profile: Profile::Standard,
        rule: Rule::UndeclaredVar,
        expect: true,
    },
    Case {
        name: "undeclared var: a real token passes",
        css: ".chip { color: var(--ink); }",
        profile: Profile::Standard,
        rule: Rule::UndeclaredVar,
        expect: false,
    },
    // Important
    Case {
        name: "important: !important fails",
        css: ".chip { color: var(--ink) !important; }",
        profile: Profile::Standard,
        rule: Rule::Important,
        expect: true,
    },
    Case {
        name: "important: absent passes",
        css: ".chip { color: var(--ink); }",
        profile: Profile::Standard,
        rule: Rule::Important,
        expect: false,
    },
    // BlitzUnsupported
    Case {
        name: "blitz unsupported: backdrop-filter fails",
        css: ".chip { backdrop-filter: blur(10px); }",
        profile: Profile::Standard,
        rule: Rule::BlitzUnsupported,
        expect: true,
    },
    Case {
        name: "blitz unsupported: absent passes",
        css: ".chip { color: var(--ink); }",
        profile: Profile::Standard,
        rule: Rule::BlitzUnsupported,
        expect: false,
    },
    Case {
        name: "blitz unsupported: scroll-behavior: smooth fails",
        css: ".ds-scroller { scroll-behavior: smooth; }",
        profile: Profile::Standard,
        rule: Rule::BlitzUnsupported,
        expect: true,
    },
    Case {
        name: "blitz unsupported: scroll-behavior: auto passes",
        css: ".ds-scroller { scroll-behavior: auto; }",
        profile: Profile::Standard,
        rule: Rule::BlitzUnsupported,
        expect: false,
    },
    // UnprefixedAttributeSelector
    Case {
        name: "unprefixed attribute: [data-open] fails",
        css: "[data-open] { color: var(--ink); }",
        profile: Profile::Standard,
        rule: Rule::UnprefixedAttributeSelector,
        expect: true,
    },
    Case {
        name: "unprefixed attribute: [*|data-open] passes",
        css: "[*|data-open] { color: var(--ink); }",
        profile: Profile::Standard,
        rule: Rule::UnprefixedAttributeSelector,
        expect: false,
    },
    // FocusPseudoClass
    Case {
        name: "focus pseudo-class: :focus-visible fails",
        css: ".btn:focus-visible { color: var(--ink); }",
        profile: Profile::Standard,
        rule: Rule::FocusPseudoClass,
        expect: true,
    },
    Case {
        name: "focus pseudo-class: :focus passes",
        css: ".btn:focus { color: var(--ink); }",
        profile: Profile::Standard,
        rule: Rule::FocusPseudoClass,
        expect: false,
    },
    // SvgPaintInCss
    Case {
        name: "svg paint in css: svg descendant fails",
        css: ".icon svg { stroke: red; }",
        profile: Profile::Standard,
        rule: Rule::SvgPaintInCss,
        expect: true,
    },
    Case {
        name: "svg paint in css: a non-svg selector passes",
        css: ".ic { stroke: currentColor; }",
        profile: Profile::Standard,
        rule: Rule::SvgPaintInCss,
        expect: false,
    },
];

/// A markup case: rendered `html` against the stylesheet `css`.
struct MarkupCase {
    name: &'static str,
    html: &'static str,
    css: &'static str,
    rule: Rule,
    expect: bool,
}

/// One passing and one failing case per markup-only [`Rule`].
const MARKUP_CASES: &[MarkupCase] = &[
    MarkupCase {
        name: "unstyled class: a class no rule names fails",
        html: "<span class=\"ds-chip zz-missing\">x</span>",
        css: ".ds-chip{}",
        rule: Rule::UnstyledClass,
        expect: true,
    },
    MarkupCase {
        name: "unstyled class: every class styled passes",
        html: "<span class=\"ds-chip\">x</span>",
        css: ".ds-chip{}",
        rule: Rule::UnstyledClass,
        expect: false,
    },
    MarkupCase {
        name: "raw markup: a hand-written svg fails",
        html: "<svg viewBox=\"0 0 24 24\"><path d=\"M0 0\"></path></svg>",
        css: "",
        rule: Rule::RawMarkup,
        expect: true,
    },
    MarkupCase {
        name: "raw markup: a hand-written button fails",
        html: "<button type=\"button\" class=\"send\">Send</button>",
        css: ".send{}",
        rule: Rule::RawMarkup,
        expect: true,
    },
    MarkupCase {
        name: "raw markup: a bare input fails",
        html: "<input type=\"text\"/>",
        css: "",
        rule: Rule::RawMarkup,
        expect: true,
    },
    MarkupCase {
        name: "raw markup: a Glyph passes",
        html: "<svg class=\"ds-ic\" viewBox=\"0 0 24 24\"></svg>",
        css: ".ds-ic{}",
        rule: Rule::RawMarkup,
        expect: false,
    },
    MarkupCase {
        name: "raw markup: a component's own marked svg passes",
        html: "<svg class=\"ds-send-ring\" data-ds-svg=\"ring\" viewBox=\"0 0 24 24\"></svg>",
        css: ".ds-send-ring{}",
        rule: Rule::RawMarkup,
        expect: false,
    },
    MarkupCase {
        name: "raw markup: a ds class alone does not make an svg quire's",
        html: "<svg class=\"ds-send-ring\" viewBox=\"0 0 24 24\"></svg>",
        css: ".ds-send-ring{}",
        rule: Rule::RawMarkup,
        expect: true,
    },
    MarkupCase {
        name: "inline style: a component's computed custom property passes",
        html: "<span class=\"ds-avatar\" style=\"--av-bg:#944242;--av-fg:var(--on-hue)\">D</span>",
        css: ".ds-avatar{}",
        rule: Rule::HexColour,
        expect: false,
    },
    MarkupCase {
        name: "inline style: a computed rgba() custom property on the root passes",
        html: "<div class=\"ds\" style=\"--f-ink:#1b1f1a;--f-pill:rgba(255,255,255,.5);\"><div class=\"ds-layer\" style=\"--f-grad:linear-gradient(#fff,#000)\"></div></div>",
        css: ".ds{} .ds-layer{}",
        rule: Rule::ColourFunction,
        expect: false,
    },
    MarkupCase {
        name: "inline style: a custom property on a consumer's element fails",
        html: "<span class=\"badge\" style=\"--bg:#944242\">D</span>",
        css: ".badge{}",
        rule: Rule::HexColour,
        expect: true,
    },
    MarkupCase {
        name: "inline style: a literal paint on a ds element still fails",
        html: "<span class=\"ds-space-swatch\" style=\"--x:1;background:#944242\"></span>",
        css: ".ds-space-swatch{}",
        rule: Rule::HexColour,
        expect: true,
    },
    MarkupCase {
        name: "inline style: a named colour after a custom property on a ds element fails",
        html: "<span class=\"ds-chip\" style=\"--a:red; color: red\">x</span>",
        css: ".ds-chip{}",
        rule: Rule::NamedColour,
        expect: true,
    },
    MarkupCase {
        name: "raw markup: a quire button passes",
        html: "<button type=\"button\" class=\"ds-button\" data-variant=\"primary\">Send</button>",
        css: ".ds-button{}",
        rule: Rule::RawMarkup,
        expect: false,
    },
];

#[test]
fn every_rule_case_matches() {
    let mut failures = Vec::new();
    for case in CASES {
        let got = has(&lint(case.css, case.profile), case.rule);
        if got != case.expect {
            failures.push(format!(
                "{}: expected {rule:?} to {verb} in `{css}`, but it did{not}",
                case.name,
                rule = case.rule,
                verb = if case.expect { "fire" } else { "stay quiet" },
                css = case.css,
                not = if got { "" } else { " not" },
            ));
        }
    }
    for case in MARKUP_CASES {
        let got = has(
            &markup(case.html, case.css, &LintConfig::default()),
            case.rule,
        );
        if got != case.expect {
            failures.push(format!(
                "{}: expected {:?} to {} in `{}`",
                case.name,
                case.rule,
                if case.expect { "fire" } else { "stay quiet" },
                case.html
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A hairline offence says which token to use.
#[test]
fn a_raw_hairline_points_at_its_token() {
    let texts: Vec<String> = lint(
        ".card { border: 1px solid var(--line); box-shadow: none; } .edge { outline: .5px solid var(--line); }",
        Profile::Strict,
    )
    .into_iter()
    .filter(|offence| offence.rule == Rule::RawHairline)
    .map(|offence| offence.text)
    .collect();
    assert_eq!(
        texts,
        [
            ".card: border: 1px (use var(--hair))",
            ".edge: outline: .5px (use var(--hairline))",
        ]
    );
}

/// `CASES` and `MARKUP_CASES` cover every `Rule`, with both a passing and a failing case — not
/// just the ones a contributor remembered to hand-pick.
#[test]
fn every_rule_variant_is_covered() {
    let cases = CASES
        .iter()
        .map(|case| (case.rule, case.expect))
        .chain(MARKUP_CASES.iter().map(|case| (case.rule, case.expect)));
    let seen: Vec<(Rule, bool)> = cases.collect();
    let mut missing = Vec::new();
    for rule in Rule::ALL {
        if !seen.contains(&(rule, true)) {
            missing.push(format!("{rule:?} has no failing case"));
        }
        if !seen.contains(&(rule, false)) {
            missing.push(format!("{rule:?} has no passing case"));
        }
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}

mod mailo_cases {
    use super::*;

    /// Ported from `mail-app/src/ui/style/mod.rs`'s identically named test: `currentColor` is
    /// the whole value of `stroke`/`fill`, or it is a colour outside the palette.
    #[test]
    fn current_color_is_only_a_stroke_or_fill_value() {
        const CASES: &[(&str, &str, &[&str])] = &[
            (".label", "color: currentColor", &[".label: currentColor"]),
            (".ic", "stroke: currentColor", &[]),
            (".ic", "fill: currentColor", &[]),
            (
                ".presets button",
                "background: currentColor",
                &[".presets button: currentColor"],
            ),
            (".ic", "stroke-width: currentColor", &[".ic: currentColor"]),
            (".ic", "stroke: currentColor extra", &[".ic: currentColor"]),
        ];
        let mut failures = Vec::new();
        for &(selector, body, expect) in CASES {
            let css = format!("{selector} {{ {body} }}");
            let got: Vec<String> = lint(&css, Profile::Standard)
                .into_iter()
                .filter(|offence| offence.rule == Rule::CurrentColourOutsideStrokeFill)
                .map(|offence| offence.text)
                .collect();
            let expect: Vec<String> = expect.iter().map(|s| (*s).to_owned()).collect();
            if got != expect {
                failures.push(format!(
                    "{selector} {{ {body} }}: got {got:?}, want {expect:?}"
                ));
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    /// Ported from `every_var_is_declared`: a misspelt token resolves to nothing at the CSS
    /// layer and is never an error there, so the lint is the one place that catches it.
    #[test]
    fn every_var_is_declared() {
        let clean = lint(
            ".row { color: var(--ink); background: var(--surface); transition: color var(--t-quick) var(--e-out); }",
            Profile::Standard,
        );
        assert!(
            !has(&clean, Rule::UndeclaredVar),
            "real tokens flagged as undeclared: {clean:?}"
        );

        let typo = lint(".row { color: var(--line-sfot); }", Profile::Standard);
        let missing: Vec<&str> = typo
            .iter()
            .filter(|offence| offence.rule == Rule::UndeclaredVar)
            .map(|offence| offence.text.as_str())
            .collect();
        assert_eq!(missing, vec![".row: --line-sfot"], "{typo:?}");
    }

    /// Ported from `no_colour_outside_the_palette`: every raw colour shape the lint promises to
    /// catch — hex, a colour function, a named colour, a system colour — is actually caught.
    #[test]
    fn no_colour_outside_the_palette() {
        const RAW_COLOURS: &[&str] = &[
            ".a { color: #ABCDEF; }",
            ".a { color: #fff; }",
            ".a { background: rgba(0, 0, 0, .5); }",
            ".a { background: hsl(200deg 50% 50%); }",
            ".a { color: white; }",
            ".a { color: CanvasText; }",
        ];
        for css in RAW_COLOURS {
            let offences = lint(css, Profile::Standard);
            let flagged = offences.iter().any(|offence| {
                matches!(
                    offence.rule,
                    Rule::HexColour | Rule::ColourFunction | Rule::NamedColour
                )
            });
            assert!(flagged, "{css} was not flagged: {offences:?}");
        }
        let clean = lint(
            ".a { color: var(--ink); background: var(--surface); }",
            Profile::Standard,
        );
        assert!(
            !clean.iter().any(|offence| matches!(
                offence.rule,
                Rule::HexColour | Rule::ColourFunction | Rule::NamedColour
            )),
            "{clean:?}"
        );
    }

    /// Ported from `a_class_with_no_rule_is_named`: the failure names the missing class, not
    /// merely "some markup was unstyled".
    #[test]
    fn a_class_with_no_rule_is_named() {
        let offences = markup(
            "<div class=\"zz-missing\"></div>",
            "",
            &LintConfig::default(),
        );
        assert_eq!(
            offences
                .iter()
                .map(|o| (o.rule, o.text.as_str()))
                .collect::<Vec<_>>(),
            vec![(
                Rule::UnstyledClass,
                "<div>: class not defined by any rule: zz-missing"
            )],
            "{offences:?}"
        );
    }
}

/// The person swatches and the status inks are declared tokens: a consumer's own CSS may paint
/// with them at the strict profile (FINDINGS "mailo gaps", items 4 and 5).
#[test]
fn the_person_swatches_and_status_inks_lint_clean() {
    let css = ".pin { background: var(--c-person-3); color: var(--on-hue); }\
               .saved { background: var(--ok); color: var(--ok-ink); }\
               .dirty { background: var(--warn); color: var(--warn-ink); }";
    let offences = lint(css, Profile::Strict);
    assert!(offences.is_empty(), "{offences:#?}");
    assert!(
        !lint(".pin { background: var(--c-person-9); }", Profile::Strict).is_empty(),
        "an undeclared swatch is an offence"
    );
}
