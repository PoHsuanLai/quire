//! quire's own generated stylesheet, linted under the strictest profile (the coherence rules,
//! ARCHITECTURE.md "Repo rules": every downstream crate runs this same check on its own CSS, and
//! quire's own output has to pass it too).
//!
//! The details grammar's two rules are judged per sheet in `details_lint.rs`, which names the
//! sheets allowed to loop. Two rules exist only for consumers and are set aside whole: [`Rule::DsInternals`] (quire is
//! the one crate that styles `.ds-*` and `.ds[data-*]`) and [`Rule::Keyframes`] (quire is where
//! keyframes live). Every other rule must be clean, [`Rule::RawSpacing`] included (the sheets
//! read `--s-*` since the polish pass), or name its exact selector and reason below.

use ds_lint::{Exception, LintConfig, Offence, Profile, Rule, stylesheet};

/// Custom properties a component writes inline per element, which no stylesheet block declares:
/// the avatar's colours (`Avatar`), the slider's fraction (`Slider`), the spark angle and the
/// heal distance (design/05-MOTION.md section 5), an external icon's size (`IconView`), and a
/// Space dot's stops (`SpaceDot` and the Space editor's dots), and a tinted plate's
/// stops and ink per scheme (`IconView { plate_tint }`), and the level
/// control's rubber band and segment stagger (the capsule `Slider`).
const INLINE_VARS: &[&str] = &[
    "--av-bg",
    "--av-fg",
    "--f",
    "--a",
    "--dy",
    "--present-p",
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
    // A notification group's layer count (`NotificationCard`).
    "--layers",
    // A swiped card's offset (`NotificationCard { swipe }`).
    "--swipe-dx",
    // An animated emoji's disc (`AnimatedEmoji { disc }`, design/25).
    "--em-disc",
    // A stepping spinner's angle (`ProgressIndicator`, design/26 R4).
    "--turn",
    // A running loop's step: the barber pole's slide (`ProgressIndicator`).
    "--step",
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
        ..LintConfig::new(&ds_shell::kits())
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
        // The sheet opens `@layer ds` itself.
        Rule::LayerDs => offence.selector.starts_with("@layer ds"),
        // Judged sheet by sheet, with each allowed loop's reason, in `details_lint.rs`.
        Rule::InfiniteLoop | Rule::OffGrammarTiming => true,
        _ => false,
    }
}

#[test]
fn self_lint() {
    let rest: Vec<String> = stylesheet(ds_shell::stylesheet(), &config())
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
    let offences = stylesheet(ds_shell::stylesheet(), &bare);
    let stale: Vec<&str> = EXCEPTIONS
        .iter()
        .filter(|exception| !offences.iter().any(|offence| exception.covers(offence)))
        .map(|exception| exception.selector)
        .collect();
    assert!(stale.is_empty(), "exceptions that match nothing: {stale:?}");
}
