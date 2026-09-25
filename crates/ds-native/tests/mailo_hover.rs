//! mailo gaps 2, hover cards, on a real Blitz document: a hover target drawn as the list's own
//! `li` opens its card after the intent wait and places it against the item, moving between
//! items switches the card, and a row time's tip opens small and on one line below its time.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::{
    Appearance, Ds, HoverCard, HoverKey, HoverKind, HoverTarget, Material, Point, Px,
    TargetElement, use_hover_hub,
};
use ds_native::harness::settle_until;
use ds_native::{Harness, Viewport};
use probe::rect;
use std::time::{Duration, Instant};

const VIEW: Viewport = Viewport {
    width: 720,
    height: 480,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

const PINS: [&str; 3] = ["Dana Okafor", "Sam Lindqvist", "Mei Chen"];

/// A pinned list whose items are hover targets themselves, and a time with its tip.
#[allow(non_snake_case)]
fn Page() -> Element {
    let hub = use_hover_hub();
    rsx! {
        div { style: "height:460px; padding:20px",
            ul { class: "pins", style: "width:200px; margin:0; padding:0",
                for (index , name) in PINS.into_iter().enumerate() {
                    HoverTarget {
                        key: "{name}",
                        hover_key: HoverKey(format!("pin:{index}")),
                        kind: HoverKind::Side,
                        as_: TargetElement::Li,
                        "{name}"
                    }
                }
            }
            p { style: "margin-top:40px",
                HoverTarget { hover_key: HoverKey("time:1".into()), kind: HoverKind::Tip, "09:41" }
            }
        }
        if let Some((open, kind)) = hub.open() {
            HoverCard { key: "{open.0}", kind,
                match kind {
                    HoverKind::Tip => rsx! { "Wed 23 Sep 2026, 09:41, 10:41 their time (Lagos)" },
                    _ => rsx! { p { class: "card-of", "{open.0}" } },
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn App() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, Page {} }
    }
}

/// Whether two lengths agree to the half pixel (a layout rounds its fractions).
fn near(a: Px, b: f32) -> bool {
    (a.0 - b).abs() < 0.5
}

fn centre(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

#[test]
fn a_hover_target_on_an_li_opens_its_card_beside_the_item() {
    let mut harness = Harness::new(App, VIEW);
    harness.advance(ms(50));
    assert_eq!(
        harness.count("ul.pins > li.ds-hover-target"),
        3,
        "{}",
        harness.html()
    );
    let second = "ul.pins > li:nth-child(2)";
    let entered = Instant::now();
    harness.pointer_move(centre(&harness, second));
    // Well under half the 450 ms open delay (fixed 2026-09-25, FINDINGS "Timing tests"): the
    // old 400 ms check flaked under load, since `advance` only guarantees *at least* the time
    // asked for, and a busy machine can stretch it past the boundary it meant to stop short of.
    harness.advance(ms(150));
    assert_eq!(
        harness.count(".ds-hovercard"),
        0,
        "not open well before 450 ms"
    );
    let opened = settle_until(&mut harness, |h| h.count(".ds-hovercard") == 1);
    assert!(
        opened.duration_since(entered) >= Duration::from_millis(450),
        "the card opened only once the intent wait had fully run: {:?}",
        opened.duration_since(entered)
    );
    assert_eq!(harness.text_of(".card-of").as_deref(), Some("pin:1"));
    // A side card: 10 right of the item, 6 above its top (design/06 section 3).
    harness.advance(ms(100));
    let item = rect(&harness, second);
    let card = rect(&harness, ".ds-hovercard");
    assert!(
        near(card.origin.x, item.origin.x.0 + item.size.width.0 + 10.0)
            && near(card.origin.y, item.origin.y.0 - 6.0),
        "{card:?} beside {item:?}"
    );
    // Warm: the next item's card replaces it at once.
    harness.pointer_move(centre(&harness, "ul.pins > li:nth-child(3)"));
    harness.advance(ms(60));
    assert_eq!(harness.text_of(".card-of").as_deref(), Some("pin:2"));
}

#[test]
fn a_time_tip_opens_small_on_one_line_below_its_time() {
    let mut harness = Harness::new(App, VIEW);
    harness.advance(ms(50));
    let time = "p .ds-hover-target";
    harness.pointer_move(centre(&harness, time));
    harness.advance(ms(600));
    assert_eq!(
        harness.attr(".ds-hovercard", "data-kind").as_deref(),
        Some("tip"),
        "{}",
        harness.html()
    );
    let target = rect(&harness, time);
    let tip = rect(&harness, ".ds-hovercard");
    assert!(
        near(tip.origin.x, target.origin.x.0)
            && near(tip.origin.y, target.origin.y.0 + target.size.height.0 + 6.0),
        "{tip:?} under {target:?}"
    );
    // At most 260 wide, and one 12 px line in 6 px of padding: two lines would pass 45.
    assert!(tip.size.width.0 <= 260.0, "tooltip-sized: {tip:?}");
    assert!(tip.size.height.0 < 36.0, "one line: {tip:?}");
}
