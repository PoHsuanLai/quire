//! quire's own generated stylesheet, linted under the strictest profile (ORCHESTRATION
//! coherence rule 1: every downstream crate runs this same check on its own CSS, and quire's
//! own output has to pass it too).
//!
//! Two rules exist only for consumers and are set aside whole: [`Rule::DsInternals`] (quire is
//! the one crate that styles `.ds-*` and `.ds[data-*]`) and [`Rule::Keyframes`] (quire is where
//! keyframes live). Every other rule must be clean, or name its exact selector and reason below.

#[path = "support/golden.rs"]
#[allow(dead_code)] // Only the directory scan is used here.
mod golden;

use ds::lint::{Exception, LintConfig, Offence, Profile, Rule, markup, stylesheet};

/// Custom properties a component writes inline per element, which no stylesheet block declares:
/// the avatar's colours (`Avatar`), the slider's fraction (`Slider`), the spark angle and the
/// heal distance (design/05-MOTION.md section 5).
const INLINE_VARS: &[&str] = &["--av-bg", "--av-fg", "--f", "--a", "--dy"];

const EXCEPTIONS: &[Exception] = &[
    Exception {
        rule: Rule::RootSelector,
        selector: "html,body",
        reason: "the reset keeps the document transparent so a shell surface shows the desktop",
    },
    Exception {
        rule: Rule::HexColour,
        selector: ".ds-truncate",
        reason: "the end fade's mask is alpha only; its #000 is never painted (spike S13)",
    },
    Exception {
        rule: Rule::RawFontSize,
        selector: ".ds-mono",
        reason: "machine facts are a size smaller than whatever they sit in: .78em (`S:56`)",
    },
];

fn config() -> LintConfig {
    LintConfig {
        profile: Profile::Strict,
        own_vars: INLINE_VARS.iter().map(|name| (*name).to_owned()).collect(),
        exceptions: EXCEPTIONS,
    }
}

/// Whether `offence` is one of the two consumer-only rules on quire's own ground.
fn quires_own(offence: &Offence) -> bool {
    match offence.rule {
        Rule::DsInternals => offence.selector.starts_with(".ds"),
        Rule::Keyframes => offence.selector.starts_with("@keyframes "),
        _ => false,
    }
}

#[test]
fn self_lint() {
    let rest: Vec<String> = stylesheet(ds::stylesheet(), &config())
        .into_iter()
        .filter(|offence| !quires_own(offence))
        .map(|offence| {
            format!(
                "{:?} {}:{}: {}",
                offence.rule, offence.line, offence.column, offence.text
            )
        })
        .collect();
    assert!(rest.is_empty(), "{}", rest.join("\n"));
}

#[test]
fn every_exception_still_suppresses_something() {
    let bare = LintConfig {
        exceptions: &[],
        ..config()
    };
    let offences = stylesheet(ds::stylesheet(), &bare);
    let stale: Vec<&str> = EXCEPTIONS
        .iter()
        .filter(|exception| !offences.iter().any(|offence| exception.covers(offence)))
        .map(|exception| exception.selector)
        .collect();
    assert!(stale.is_empty(), "exceptions that match nothing: {stale:?}");
}

/// What quire's own components write inline that a consumer may not: the avatar's ground
/// colour is computed per address or account and written as a hex (design/04-COMPONENTS.md
/// O-7 — still open; the ground itself cannot be a fixed token, only its letter could). The
/// letter itself is `--on-hue` now (O-3, resolved; wave 2's schema brief), so this exception no
/// longer covers `#fff` — only the computed ground remains.
const MARKUP_EXCEPTIONS: &[Exception] = &[Exception {
    rule: Rule::HexColour,
    selector: "span.ds-avatar",
    reason: "the person hue and account colour are computed per face (O-7, open); the letter \
             is `--on-hue` now, not a literal (O-3, resolved)",
}];

/// Coherence rule 2 on quire's own output: every control golden, rendered markup, uses only
/// classes the stylesheet styles and no hand-written SVG or form control.
#[test]
fn every_control_golden_lints_clean() {
    let config = LintConfig {
        exceptions: MARKUP_EXCEPTIONS,
        ..LintConfig::default()
    };
    let goldens = golden::all_in("controls");
    assert!(goldens.len() > 60, "only {} goldens", goldens.len());
    let failures: Vec<String> = goldens
        .iter()
        .flat_map(|(name, html)| {
            markup(html, ds::stylesheet(), &config)
                .into_iter()
                .map(move |offence| format!("{name}: {:?} {}", offence.rule, offence.text))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
