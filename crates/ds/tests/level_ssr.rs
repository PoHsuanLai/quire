//! The level control and the OSD card as markup (the user's brief of 2026-09-25; sill FINDINGS
//! Q74 to Q76): each look, each glyph state, read-only and interactive, and the OSD card shown at
//! either anchor and hidden. Each golden is `tests/snapshots/level/<name>.html`, and every `ds-`
//! class in it must be styled by the stylesheet.
//!
//! `DS_BLESS=1 cargo test -p ds --test level_ssr` rewrites the goldens.

#[path = "support/golden.rs"]
mod golden;

use dioxus::prelude::*;
use ds::{
    Appearance, Ds, Fraction, Inject, Level, LevelControl, LevelGlyph, LevelLook, LevelMode,
    Material, Muting, Osd, OsdPosition, RootChrome, Shown, Tick,
};

#[derive(Props, Clone)]
struct HostProps {
    make: fn() -> Element,
}

impl PartialEq for HostProps {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

fn host(props: HostProps) -> Element {
    (props.make)()
}

fn render(make: fn() -> Element) -> String {
    let mut dom = VirtualDom::new_with_props(host, HostProps { make });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

const HEARD: LevelGlyph = LevelGlyph::Volume(Muting::Audible);

/// `body` under a transparent Osd root, as a shell surface draws it.
fn osd_root(body: Element) -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Osd, chrome: Some(RootChrome::Transparent), stylesheet: Inject::Host,
            {body}
        }
    }
}

type Case = (&'static str, fn() -> Element);

const CASES: &[Case] = &[
    ("capsule-volume-40", || {
        osd_root(rsx! { LevelControl { label: "Volume", value: Fraction(400), glyph: HEARD } })
    }),
    ("capsule-knob-volume-40", || {
        osd_root(
            rsx! { LevelControl { label: "Volume", value: Fraction(400), glyph: HEARD, look: LevelLook::CapsuleKnob } },
        )
    }),
    ("segments-volume-40", || {
        osd_root(
            rsx! { LevelControl { label: "Volume", value: Fraction(400), glyph: HEARD, look: LevelLook::Segments } },
        )
    }),
    ("capsule-muted", || {
        osd_root(
            rsx! { LevelControl { label: "Volume", value: Fraction(400), glyph: LevelGlyph::Volume(Muting::Muted) } },
        )
    }),
    ("capsule-brightness-30-read-only", || {
        osd_root(
            rsx! { LevelControl { label: "Brightness", value: Fraction(300), glyph: LevelGlyph::Brightness, mode: LevelMode::ReadOnly } },
        )
    }),
    ("capsule-tick", || {
        osd_root(
            rsx! { LevelControl { label: "Volume", value: Fraction(1000), glyph: HEARD, tick: Tick::Quiet } },
        )
    }),
    ("osd-top-right", || {
        osd_root(rsx! {
            Osd { shown: Shown::Visible, label: "MA270U", level: Level { value: Fraction(620), glyph: HEARD } }
        })
    }),
    ("osd-bottom-centre", || {
        osd_root(rsx! {
            Osd { shown: Shown::Visible, label: "Display", position: OsdPosition::BottomCentre, level: Level { value: Fraction(300), glyph: LevelGlyph::Brightness } }
        })
    }),
    ("osd-hidden", || {
        osd_root(rsx! {
            Osd { shown: Shown::Hidden, label: "Sound", level: Level { value: Fraction(500), glyph: HEARD } }
        })
    }),
];

#[test]
fn every_level_state_matches_its_golden() {
    let failures: Vec<String> = CASES
        .iter()
        .filter_map(|(name, make)| {
            golden::check(&format!("level/{name}.html"), &render(*make)).err()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_class_is_styled() {
    let sheet = ds::stylesheet();
    for (name, make) in CASES {
        let html = render(*make);
        for class in html
            .split("class=\"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .flat_map(str::split_whitespace)
            .filter(|class| class.starts_with("ds-"))
        {
            let needle = format!(".{class}");
            let styled = sheet.match_indices(&needle).any(|(at, _)| {
                !sheet[at + needle.len()..]
                    .starts_with(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            });
            assert!(styled, "{name}: .{class} is not styled");
        }
    }
}

#[test]
fn a_read_only_level_takes_no_input_and_a_control_does() {
    let read_only = render(CASES[4].1);
    assert!(read_only.contains("role=\"progressbar\""), "{read_only}");
    assert!(!read_only.contains("tabindex"), "{read_only}");
    assert!(!read_only.contains("ds-level-hit"), "{read_only}");
    let control = render(CASES[0].1);
    assert!(
        control.contains("role=\"slider\"") && control.contains("tabindex=\"0\""),
        "{control}"
    );
    assert!(control.contains("aria-valuenow=\"40\""), "{control}");
}

#[test]
fn the_capsule_draws_its_glyph_twice_and_the_other_looks_once() {
    let parts = |html: &str| html.matches("data-part=\"body\"").count();
    assert_eq!(parts(&render(CASES[0].1)), 2, "on the well and in the fill");
    assert_eq!(parts(&render(CASES[1].1)), 1);
    assert_eq!(parts(&render(CASES[2].1)), 1);
    let segments = render(CASES[2].1);
    assert_eq!(segments.matches("class=\"ds-level-seg\"").count(), 16);
    assert_eq!(
        segments.matches("data-on=\"on\"").count() - 3,
        6,
        "6 squares and the 3 glyph parts shown at 40 %"
    );
}
