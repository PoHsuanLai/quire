//! PresetSwatches' Custom swatch on a real Blitz document: it opens the colour's field, stops and
//! grain in a small popover under it; a drag in the field changes the look as the hand moves and
//! is settled once, when the popover closes; a preset's press is settled at once.

use dioxus::prelude::*;
use ds::components::app::space_editor::swatches::PresetSwatches;
use ds::prelude::*;
use ds::style::space::look::SpaceLook;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 600,
    height: 600,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut look = use_signal(SpaceLook::default);
    let mut changes = use_signal(|| 0u32);
    let mut commits = use_signal(|| 0u32);
    let hue = look.read().dots.first().map_or(0.0, |dot| dot.hue.round());
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            div { style: "padding:24px 24px 0 200px",
                PresetSwatches {
                    look: look(),
                    onchange: move |next: SpaceLook| {
                        look.set(next);
                        changes += 1;
                    },
                    oncommit: move |()| commits += 1,
                }
            }
            p { class: "changes", "{changes}" }
            p { class: "commits", "{commits}" }
            p { class: "hue", "{hue}" }
        }
    }
}

fn start() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(50));
    harness
}

fn text(harness: &Harness, selector: &str) -> String {
    harness.text_of(selector).unwrap_or_default()
}

#[test]
fn custom_opens_a_small_colour_popover_whose_drag_is_settled_when_it_closes() {
    let mut harness = start();
    assert_eq!(
        harness.count(".ds-preset[*|role=radio]"),
        9,
        "eight presets and Custom"
    );
    let custom = harness
        .centre(".ds-preset-custom")
        .expect("the Custom swatch");
    harness.send(Input::click(custom));
    harness.advance(ms(300));
    assert_eq!(harness.count(".ds-custom-colour"), 1, "{}", harness.html());
    let field = harness
        .rect(".ds-custom-colour .ds-field")
        .expect("the field");
    assert!(
        (field.size.height.0 - 120.0).abs() < 1.0,
        "the popover's field is the small one: {field:?}"
    );
    assert_eq!(
        harness.count(".ds-custom-colour .ds-presets"),
        0,
        "no second row of presets"
    );
    let swatch = harness.rect(".ds-preset-custom").expect("the swatch");
    let card = harness.rect(".ds-custom-colour").expect("the card");
    assert!(
        card.origin.y.0 > swatch.origin.y.0 + swatch.size.height.0,
        "under it: {card:?}"
    );

    let before = text(&harness, ".hue");
    let at = Point {
        x: Px(field.origin.x.0 + field.size.width.0 * 0.8),
        y: Px(field.origin.y.0 + field.size.height.0 * 0.5),
    };
    harness.send(Input::click(at));
    harness.advance(ms(100));
    assert_ne!(text(&harness, ".hue"), before, "the look follows the field");
    assert_ne!(text(&harness, ".changes"), "0");
    assert_eq!(
        text(&harness, ".commits"),
        "0",
        "not settled while the popover is up"
    );
    assert_eq!(
        harness.attr(".ds-preset-custom", "aria-checked").as_deref(),
        Some("true"),
        "a look of its own checks Custom"
    );

    harness.send(Input::key(ShortcutKey::Escape));
    harness.advance(ms(400));
    assert_eq!(harness.count(".ds-custom-colour"), 0, "Escape closes it");
    assert_eq!(
        text(&harness, ".commits"),
        "1",
        "settled once, as it closed"
    );
}

#[test]
fn a_preset_is_settled_at_once() {
    let mut harness = start();
    let fourth = harness.centre(".ds-preset:nth-child(4)").expect("a preset");
    harness.send(Input::click(fourth));
    harness.advance(ms(100));
    assert_eq!(text(&harness, ".changes"), "1");
    assert_eq!(text(&harness, ".commits"), "1");
    assert_eq!(
        harness
            .attr(".ds-preset:nth-child(4)", "aria-checked")
            .as_deref(),
        Some("true")
    );
}
