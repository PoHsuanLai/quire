//! The modifier glyphs `⌃ ⌥ ⇧ ⌘` and the keys `⌫ ↩` are drawn by the bundled faces, not by
//! whatever system face the renderer falls back to. U+2303 (control) was missing from every
//! face, so a fallback face drew it as a caret (`^`): `⌃⌘S` read "^⌘S". The probe renders a
//! KeyEquivalent per glyph in both styles and checks each is wide ink, not a caret.

use crate::support::probe;

use dioxus::prelude::*;
use ds::components::controls::key_equivalent::{KeyEquivalent, KeyStyle};
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Viewport};

const VIEW: Viewport = Viewport {
    width: 360,
    height: 150,
    scale_percent: 100,
};

fn page(theme: Theme) -> Element {
    let combo = Shortcut(vec![
        ShortcutKey::Ctrl,
        ShortcutKey::Super,
        ShortcutKey::Char('s'),
    ]);
    let all = Shortcut(vec![
        ShortcutKey::Ctrl,
        ShortcutKey::Alt,
        ShortcutKey::Shift,
        ShortcutKey::Super,
        ShortcutKey::Backspace,
        ShortcutKey::Enter,
    ]);
    rsx! {
        Ds { appearance: Appearance { theme, ..Appearance::default() }, material: Material::Window,
            div { style: "padding:12px; display:flex; flex-direction:column; gap:10px; align-items:flex-start; font-size:16px",
                span { class: "text", KeyEquivalent { shortcut: combo.clone(), style: KeyStyle::Text } }
                span { class: "cap", style: "font-size:var(--fs-note)", KeyEquivalent { shortcut: combo, style: KeyStyle::Cap, size: ControlSize::Regular } }
                // The same glyph set from the face by name, the reference for the cap's ink.
                span { class: "inter", style: "font-family:'Inter'; font-size:var(--fs-note)", "⌃⌘" }
                span { class: "every", KeyEquivalent { shortcut: all, style: KeyStyle::Text } }
            }
        }
    }
}

#[allow(non_snake_case)]
fn Light() -> Element {
    page(Theme::Light)
}

#[allow(non_snake_case)]
fn Dark() -> Element {
    page(Theme::Dark)
}

/// The box around the ink inside `selector`: every pixel that stands out from the region's own
/// ground.
fn ink_box(harness: &Harness, frame: &image::RgbaImage, selector: &str) -> (u32, u32) {
    let area = probe::rect(harness, selector);
    let seen = probe::pixels(frame, area, 0.0);
    let ground = probe::modal(&seen);
    let width = area.size.width.0 as u32;
    let marks: Vec<(u32, u32)> = seen
        .iter()
        .enumerate()
        .filter(|(_, pixel)| probe::distance(**pixel, ground) > 60)
        .map(|(at, _)| (at as u32 % width, at as u32 / width))
        .collect();
    let span = |value: fn(&(u32, u32)) -> u32| {
        let low = marks.iter().map(value).min().unwrap_or(0);
        let high = marks.iter().map(value).max().unwrap_or(0);
        if marks.is_empty() { 0 } else { high - low + 1 }
    };
    (span(|m| m.0), span(|m| m.1))
}

/// The shots, at twice the size for the eye.
#[test]
fn shots_of_control_command_s() {
    let big = Viewport {
        scale_percent: 200,
        ..VIEW
    };
    for (app, shot) in [(Light as fn() -> Element, "light"), (Dark, "dark")] {
        let mut harness = Harness::new(app, HarnessConfig::new(big).with_clock(Clock::Virtual));
        harness.advance(std::time::Duration::from_millis(50));
        let frame = harness.render().expect("renders");
        probe::keep(&frame, &format!("key-glyphs-2x-{shot}"));
    }
}

#[test]
fn control_command_s_is_drawn_by_the_bundled_faces() {
    for (app, shot) in [(Light as fn() -> Element, "light"), (Dark, "dark")] {
        let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
        harness.advance(std::time::Duration::from_millis(50));
        let frame = harness.render().expect("renders");
        // Every row has ink; the full set of modifiers is wider than ⌃⌘S.
        let (combo, _) = ink_box(&harness, &frame, ".text");
        let (every, _) = ink_box(&harness, &frame, ".every");
        assert!(
            combo > 20 && every > combo,
            "{shot}: {combo} and {every} px of ink"
        );
        // The cap's ⌃ and ⌘ come from Inter, as the reference span's do: the cap draws them as
        // wide as the face's own (a system fallback drew a caret half that size).
        let (cap_width, _) = ink_box(&harness, &frame, ".cap .ds-key-equivalent-key:nth-child(1)");
        let (reference, _) = ink_box(&harness, &frame, ".inter");
        assert!(
            cap_width * 3 >= reference,
            "{shot}: cap ⌃ {cap_width} px against Inter's ⌃⌘ {reference} px"
        );
    }
}
