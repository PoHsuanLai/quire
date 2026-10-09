//! The voice orb as markup: idle, active, a small one (96 px) and one with the caller's colours
//! and period (128 px, 15 s), each matching its golden under `tests/snapshots/voice_orb/`, lint
//! clean, using only `ds-` classes the stylesheet styles, and carrying the state it is in.
//!
//! `DS_BLESS=1 cargo test -p ds --features lint --test it voice_orb_ssr` rewrites the goldens.

use crate::support::golden;
use crate::support::hygiene;

use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::components::content::voice_orb::model::{OrbColour, OrbColours};
use ds::components::content::voice_orb::view::VoiceOrb;
use ds::prelude::*;
use ds_core::vocab::Activity;
use ds_lint::LintConfig;
use ds_style::tokens::hex::Hex;
use std::time::Duration;

fn root(body: Element) -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, stylesheet: Inject::Host,
            {body}
        }
    }
}

fn idle() -> Element {
    root(rsx! { VoiceOrb {} })
}

fn active() -> Element {
    root(rsx! { VoiceOrb { activity: Activity::Active } })
}

fn small() -> Element {
    root(rsx! { VoiceOrb { size: Px(96.0), activity: Activity::Active } })
}

fn tiny() -> Element {
    root(rsx! { VoiceOrb { size: Px(24.0) } })
}

fn custom() -> Element {
    let colours = OrbColours {
        bg: OrbColour::Custom(Hex([0xF6, 0xF1, 0xE7])),
        c1: OrbColour::Custom(Hex([0xE8, 0x8B, 0x5A])),
        c2: OrbColour::Custom(Hex([0x6B, 0xB3, 0x9C])),
        c3: OrbColour::Custom(Hex([0xE3, 0xC0, 0x5B])),
    };
    root(rsx! {
        VoiceOrb { size: Px(128.0), colours, period: Duration::from_secs(15), activity: Activity::Active }
    })
}

fn named() -> Element {
    root(rsx! { VoiceOrb { aria_label: "Listening" } })
}

/// A specimen: its golden name and how it is made.
type Specimen = (&'static str, fn() -> Element);

const SPECIMENS: &[Specimen] = &[
    ("idle-192", idle),
    ("active-192", active),
    ("active-96", small),
    ("idle-24", tiny),
    ("active-128-custom-15s", custom),
    ("idle-192-named", named),
];

fn render(make: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(make);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// The orb's own markup, without the root and its stylesheet.
fn orb(html: &str) -> &str {
    let at = html
        .find("<div class=\"ds-voice-orb\"")
        .expect("an orb is drawn");
    let rest = &html[at..];
    let end = rest.find("</span></div>").expect("the orb closes") + "</span></div>".len();
    &rest[..end]
}

#[test]
fn every_specimen_matches_its_golden() {
    let failures: Vec<String> = SPECIMENS
        .iter()
        .filter_map(|(name, make)| {
            golden::check(&format!("voice_orb/{name}.html"), orb(&render(*make))).err()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Lint offences and unstyled classes of every specimen, swept by `golden_hygiene`.
pub(crate) fn hygiene_failures() -> Vec<String> {
    let sheet = ds::stylesheet();
    let config = LintConfig::new(&ds::kits());
    SPECIMENS
        .iter()
        .flat_map(|(name, make)| {
            let html = render(*make);
            let mut found = hygiene::offences(name, &html, sheet, &config);
            found.extend(hygiene::unstyled(name, orb(&html), sheet));
            found
        })
        .collect()
}

/// The markup says which state it is in, that a small orb has no mask, and that only a named
/// orb is exposed to assistive technology.
#[test]
fn the_markup_carries_the_state() {
    let idle = render(idle);
    assert!(idle.contains("data-activity=\"inactive\""), "{idle}");
    assert!(idle.contains("aria-hidden=\"true\""), "{idle}");
    let active = render(active);
    assert!(active.contains("data-activity=\"active\""), "{active}");
    assert!(active.contains("--orb-size:192px;"), "{active}");
    let tiny = render(tiny);
    assert!(tiny.contains("data-mask=\"off\""), "{tiny}");
    assert!(!tiny.contains("--orb-mask"), "{tiny}");
    let named = render(named);
    assert!(named.contains("role=\"img\""), "{named}");
    assert!(named.contains("aria-label=\"Listening\""), "{named}");
    assert!(!named.contains("aria-hidden"), "{named}");
    let custom = render(custom);
    assert!(custom.contains("--orb-c1:#e88b5a;"), "{custom}");
}
