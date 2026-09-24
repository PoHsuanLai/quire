//! mailo gaps 4, hook-keyed hover cards, on a real Blitz document: a caller drives the intent
//! machine from its own pointer hooks through `use_hover_intent`, placing the card against an
//! element it measured or a rect it already has; with no layout (a measurer that answers
//! nothing) the card still opens, in place, and the 450 ms open, 150 ms close and 400 ms warm
//! window still hold.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::{
    Appearance, Ds, Flow, HostMeasure, HoverAnchor, HoverCard, HoverKey, HoverKind, Material,
    Measured, MountedRef, Point, Px, Rect, Size, use_hover_intent,
};
use ds_native::{Harness, Viewport};
use probe::rect;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 480,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

const PINS: [&str; 3] = ["Dana Okafor", "Sam Lindqvist", "Mei Chen"];

/// What the page places its cards against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Placing {
    /// The item's own element, measured; the card floats beside it.
    Element,
    /// The pointer's point as a zero rect; the card floats below it.
    Point,
    /// Nothing, under a measurer that answers nothing; the card is drawn in place.
    Unplaced,
}

/// A measurer with no layout to read.
fn unknown(_: &MountedData) -> Measured {
    Measured::Unknown
}

/// The pointer's client point as a zero-size rect.
fn at_pointer(event: &PointerEvent) -> Rect {
    let point = event.client_coordinates();
    Rect {
        origin: Point {
            x: Px(point.x as f32),
            y: Px(point.y as f32),
        },
        size: Size::default(),
    }
}

/// One pinned item keying its card on its own hooks, as mailo's pins do.
#[component]
fn Pin(index: usize, name: &'static str, placing: Placing) -> Element {
    let driver = use_hover_intent();
    let mut element = use_signal(|| None::<MountedRef>);
    let key = HoverKey(format!("pin:{index}"));
    let kind = match placing {
        Placing::Point => HoverKind::Sender,
        Placing::Element | Placing::Unplaced => HoverKind::Side,
    };
    rsx! {
        li {
            class: "pin",
            style: "padding:6px 8px",
            onmounted: move |event| element.set(Some(MountedRef(event.data()))),
            onpointerenter: move |event| {
                let anchor = match (placing, element.peek().clone()) {
                    (Placing::Element, Some(mounted)) => HoverAnchor::Element(mounted),
                    (Placing::Point, _) => HoverAnchor::Rect(at_pointer(&event)),
                    (Placing::Element, None) | (Placing::Unplaced, _) => HoverAnchor::Unplaced,
                };
                driver.over(key.clone(), kind, anchor);
            },
            onpointerleave: move |_| driver.out(),
            onpointerdown: move |_| driver.press(),
            "{name}"
        }
    }
}

#[component]
fn Page(placing: Placing) -> Element {
    if placing == Placing::Unplaced {
        use_context_provider(|| HostMeasure(unknown));
    }
    let hub = use_hover_intent().hub();
    let open = hub.open().or(hub.leaving());
    let flow = match placing {
        Placing::Unplaced => Flow::Inline,
        Placing::Element | Placing::Point => Flow::Floating,
    };
    let card = open.map(|(key, kind)| {
        rsx! {
            HoverCard { key: "{key.0}", kind, flow,
                p { class: "card-of", "{key.0}" }
            }
        }
    });
    rsx! {
        div { style: "height:460px; padding:20px",
            ul { class: "pins", style: "width:200px; margin:0; padding:0; list-style:none",
                for (index , name) in PINS.into_iter().enumerate() {
                    Pin { key: "{name}", index, name, placing }
                }
            }
            div { class: "slot", style: "margin-top:24px",
                if flow == Flow::Inline {
                    {card.clone()}
                }
            }
        }
        if flow == Flow::Floating {
            {card}
        }
    }
}

#[allow(non_snake_case)]
fn ByElement() -> Element {
    rsx! { Ds { appearance: Appearance::default(), material: Material::Window, Page { placing: Placing::Element } } }
}

#[allow(non_snake_case)]
fn ByPoint() -> Element {
    rsx! { Ds { appearance: Appearance::default(), material: Material::Window, Page { placing: Placing::Point } } }
}

#[allow(non_snake_case)]
fn Unplaced() -> Element {
    rsx! { Ds { appearance: Appearance::default(), material: Material::Window, Page { placing: Placing::Unplaced } } }
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

const AWAY: Point = Point {
    x: Px(600.0),
    y: Px(400.0),
};

/// The pin at `n` (1-based).
fn pin(n: usize) -> String {
    format!("ul.pins > li:nth-child({n})")
}

#[test]
fn a_card_keyed_on_the_callers_hooks_opens_beside_its_measured_element() {
    let mut harness = Harness::new(ByElement, VIEW);
    harness.advance(ms(50));
    harness.pointer_move(centre(&harness, &pin(2)));
    harness.advance(ms(400));
    assert_eq!(harness.count(".ds-hovercard"), 0, "open before 450 ms");
    harness.advance(ms(150));
    assert_eq!(harness.text_of(".card-of").as_deref(), Some("pin:1"));
    harness.advance(ms(100));
    // A side card: 10 right of the item, 6 above its top (design/06 section 3).
    let item = rect(&harness, &pin(2));
    let card = rect(&harness, ".ds-hovercard");
    assert!(
        near(card.origin.x, item.origin.x.0 + item.size.width.0 + 10.0)
            && near(card.origin.y, item.origin.y.0 - 6.0),
        "{card:?} beside {item:?}"
    );
    // Warm: the next pin's card replaces it at once.
    harness.pointer_move(centre(&harness, &pin(3)));
    harness.advance(ms(60));
    assert_eq!(harness.text_of(".card-of").as_deref(), Some("pin:2"));
}

#[test]
fn a_card_keyed_on_a_rect_the_caller_has_opens_below_it() {
    let mut harness = Harness::new(ByPoint, VIEW);
    harness.advance(ms(50));
    let at = centre(&harness, &pin(1));
    harness.pointer_move(at);
    harness.advance(ms(600));
    let card = rect(&harness, ".ds-hovercard");
    // A sender card: at the anchor's left, 6 below it; the anchor is the entry point.
    assert!(
        near(card.origin.x, at.x.0) && near(card.origin.y, at.y.0 + 6.0),
        "{card:?} below {at:?}"
    );
}

#[test]
fn with_no_layout_an_unplaced_card_opens_in_place_on_the_hubs_timing() {
    let mut harness = Harness::new(Unplaced, VIEW);
    harness.advance(ms(50));
    harness.pointer_move(centre(&harness, &pin(1)));
    harness.advance(ms(400));
    assert_eq!(harness.count(".ds-hovercard"), 0, "open before 450 ms");
    harness.advance(ms(100));
    assert_eq!(
        harness.text_of(".slot > .ds-hovercard .card-of").as_deref(),
        Some("pin:0"),
        "{}",
        harness.html()
    );
    assert_eq!(
        harness.attr(".ds-hovercard", "data-flow").as_deref(),
        Some("inline")
    );
    assert_eq!(harness.attr(".ds-hovercard", "style"), None);
    // Out: 150 ms to close, then `hc-out` plays before the card goes.
    harness.pointer_move(AWAY);
    harness.advance(ms(100));
    assert_ne!(
        harness.attr(".ds-hovercard", "data-presence").as_deref(),
        Some("leaving"),
        "closing before 150 ms"
    );
    harness.advance(ms(100));
    assert_eq!(
        harness.attr(".ds-hovercard", "data-presence").as_deref(),
        Some("leaving")
    );
    harness.advance(ms(250));
    assert_eq!(harness.count(".ds-hovercard"), 0, "{}", harness.html());
    // Still inside the 400 ms warm window: the next pin opens at once.
    harness.pointer_move(centre(&harness, &pin(2)));
    harness.advance(ms(60));
    assert_eq!(harness.text_of(".card-of").as_deref(), Some("pin:1"));
    // A press takes it away at once, not warm.
    harness.pointer_down(centre(&harness, &pin(2)));
    harness.advance(ms(30));
    assert_eq!(harness.count(".ds-hovercard"), 0, "{}", harness.html());
}
