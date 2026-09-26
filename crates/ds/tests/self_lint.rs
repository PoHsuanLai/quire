//! quire's own generated stylesheet, linted under the strictest profile (ORCHESTRATION
//! coherence rule 1: every downstream crate runs this same check on its own CSS, and quire's
//! own output has to pass it too).
//!
//! The details grammar's two rules are judged per sheet in `details_lint.rs`, which names the
//! sheets allowed to loop. Two rules exist only for consumers and are set aside whole: [`Rule::DsInternals`] (quire is
//! the one crate that styles `.ds-*` and `.ds[data-*]`) and [`Rule::Keyframes`] (quire is where
//! keyframes live). Every other rule must be clean, [`Rule::RawSpacing`] included (the sheets
//! read `--s-*` since the polish pass), or name its exact selector and reason below.

#[path = "support/golden.rs"]
#[allow(dead_code)] // Only the directory scan is used here.
mod golden;

use ds::lint::{Exception, LintConfig, Offence, Profile, Rule, markup, stylesheet};

/// Custom properties a component writes inline per element, which no stylesheet block declares:
/// the avatar's colours (`Avatar`), the slider's fraction (`Slider`), the spark angle and the
/// heal distance (design/05-MOTION.md section 5), an external icon's size (`IconView`), and a
/// Space dot's stops (`SpaceDot` and the Space editor's dots, mailo gaps 3), and a tinted plate's
/// stops and ink per scheme (`IconView { plate_tint }`, sill FINDINGS Q72), and the level
/// control's rubber band and segment stagger (`LevelControl`).
const INLINE_VARS: &[&str] = &[
    "--av-bg",
    "--av-fg",
    "--f",
    "--a",
    "--dy",
    "--ic-size",
    "--dot-c1",
    "--dot-c2",
    "--dot-c3",
    "--plate-base-l",
    "--plate-deep-l",
    "--plate-ink-l",
    "--plate-base-d",
    "--plate-deep-d",
    "--plate-ink-d",
    // The level control's rubber band and a segment's place in the fill's stagger.
    "--rb",
    "--i",
    // A notification group's layer count (`NotificationCard`, sill Q120).
    "--layers",
    // A swiped card's offset (`NotificationCard { swipe }`, sill Q122).
    "--swipe-dx",
    // An animated emoji's disc (`AnimatedEmoji { disc }`, design/25).
    "--em-disc",
    // A stepping spinner's angle (`Spinner`, design/26 R4).
    "--turn",
];

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
        rule: Rule::CurrentColourOutsideStrokeFill,
        selector: ".ds-ext-icon[*|data-kind=symbolic]",
        reason: "a symbolic icon is its mask over the text colour, the Glyph's currentColor \
                 stroke by other means (design/08-ICONS.md section 1.5, spike S7)",
    },
];

fn config() -> LintConfig {
    LintConfig {
        profile: Profile::Strict,
        own_vars: INLINE_VARS.iter().map(|name| (*name).to_owned()).collect(),
        exceptions: EXCEPTIONS,
        ..LintConfig::default()
    }
}

/// Whether `offence` is one of the two consumer-only rules on quire's own ground (the reset's
/// element rules scope through `:where(.ds)`).
fn quires_own(offence: &Offence) -> bool {
    match offence.rule {
        Rule::DsInternals => {
            offence.selector.starts_with(".ds") || offence.selector.starts_with(":where(.ds)")
        }
        Rule::Keyframes => offence.selector.starts_with("@keyframes "),
        // Judged sheet by sheet, with each allowed loop's reason, in `details_lint.rs`.
        Rule::InfiniteLoop | Rule::OffGrammarTiming => true,
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

/// The consumer classes the mailo gaps 6 goldens put on a button through `extra_class`, and
/// the edit surface 2 golden on the surface (mailo's `.c-body`, positioned so its text stacks
/// over a selection layer): a consumer's own sheet styles them, as it would in the app, so they
/// are in scope here and nowhere in quire's sheet.
const CONSUMER_CSS: &str = ".row-reveal{opacity:0}\n.quiet-until-hover{opacity:0}\n.fold-more{opacity:0}\n\
     .c-body{position:relative}";

/// Coherence rule 2 on quire's own output: every control golden, rendered markup, uses only
/// classes the stylesheet styles, no hand-written SVG or form control, and no literal paint
/// outside the custom properties a component computes (the avatar's `--av-bg`, O-7), which
/// the markup lint allows on a `ds-*` element without an exception.
#[test]
fn every_control_golden_lints_clean() {
    let config = LintConfig::default();
    let css = format!("{}\n{CONSUMER_CSS}", ds::stylesheet());
    let goldens = golden::all_in("controls");
    assert!(goldens.len() > 60, "only {} goldens", goldens.len());
    let failures: Vec<String> = goldens
        .iter()
        .flat_map(|(name, html)| {
            markup(html, &css, &config)
                .into_iter()
                .map(move |offence| format!("{name}: {:?} {}", offence.rule, offence.text))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
