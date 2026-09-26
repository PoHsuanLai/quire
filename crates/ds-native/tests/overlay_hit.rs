//! mailo's sender card over the message list, on a real Blitz document (FINDINGS "A part's
//! leave is not the row's enter"). Blitz's hit test follows paint order: a card over rows that
//! are `position: relative` takes the pointer and the click, whether it is `position: fixed`
//! after the rows, `position: absolute` with a z-index before or after them, or drawn through
//! quire's overlay host. What swapped mailo's card was the name's leave, which it read as "back
//! on the row"; `ListRow`'s `onpointerback` says that only when it is so.

use dioxus::prelude::*;
use ds::{
    Anim, Appearance, Ds, Emphasis, HoverAnchor, HoverCard, HoverKey, HoverKind, ListRow, Material,
    OverlayId, PartHooks, Point, Presence, PulseKey, Px, Rect, Selection, Size, StaggerIndex,
    ZLayer, use_hover_intent, use_overlays,
};
use ds_native::harness::settle_until;
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 480,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn centre(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

/// Where the floating card is drawn, and how.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Card {
    /// `position: fixed`, after the rows in the tree.
    FixedAfter,
    /// `position: fixed`, before the rows: the rows, positioned and later, paint over it.
    FixedBefore,
    /// `position: absolute; z-index: 5`, after the rows.
    RaisedAfter,
    /// `position: absolute; z-index: 5`, before the rows.
    RaisedBefore,
    /// Through quire's overlay host, on the card layer, at the end of `.ds`.
    Overlay,
}

/// Eight 40 px rows, each `position: relative`, in a list that is itself `position: relative`
/// (as mailo's `.list-col` is), logging their entry and click.
#[component]
fn Rows(log: Signal<Vec<String>>) -> Element {
    rsx! {
        div { class: "list", style: "position:relative",
            for i in 0..8 {
                div {
                    key: "{i}",
                    class: "row r{i}",
                    style: "position:relative; height:40px",
                    onpointerenter: move |_| log.write().push(format!("enter r{i}")),
                    onclick: move |_| log.write().push(format!("click r{i}")),
                    "row {i}"
                }
            }
        }
    }
}

/// The rows and a 300 by 120 card at (40, 70), over rows 1 to 4, drawn as `card` says.
#[component]
fn Page(card: Card) -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let overlays = use_overlays();
    let body = rsx! {
        div {
            class: "card",
            style: "width:300px; height:120px; background:white",
            onmouseenter: move |_| log.write().push("enter card".into()),
            button {
                class: "pin",
                onclick: move |_| log.write().push("click pin".into()),
                "Pin"
            }
        }
    };
    let floated = body.clone();
    use_effect(move || {
        if card == Card::Overlay {
            overlays.show(
                OverlayId(900),
                ZLayer::Card,
                rsx! {
                    div { class: "ds-overlay", "data-layer": "card",
                        div { class: "ds-popover", style: "left:40px; top:70px", {floated.clone()} }
                    }
                },
            );
        }
    });
    let fixed = rsx! {
        div { style: "position:fixed; left:40px; top:70px", {body.clone()} }
    };
    let raised = rsx! {
        div { style: "position:absolute; z-index:5; left:40px; top:70px", {body.clone()} }
    };
    rsx! {
        div { style: "position:relative; height:460px",
            if card == Card::FixedBefore {
                {fixed.clone()}
            }
            if card == Card::RaisedBefore {
                {raised.clone()}
            }
            Rows { log }
            if card == Card::FixedAfter {
                {fixed}
            }
            if card == Card::RaisedAfter {
                {raised}
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

macro_rules! app {
    ($name:ident, $card:expr) => {
        #[allow(non_snake_case)]
        fn $name() -> Element {
            rsx! {
                Ds { appearance: Appearance::default(), material: Material::Window,
                    Page { card: $card }
                }
            }
        }
    };
}
app!(FixedAfterApp, Card::FixedAfter);
app!(FixedBeforeApp, Card::FixedBefore);
app!(RaisedAfterApp, Card::RaisedAfter);
app!(RaisedBeforeApp, Card::RaisedBefore);
app!(OverlayApp, Card::Overlay);

/// Point at the card where it covers row 3, then click its button (which covers row 1 or 2).
fn point_and_click(app: fn() -> Element) -> (Harness, Point) {
    let mut harness = Harness::new(app, VIEW);
    harness.advance(ms(50));
    let over_row = Point {
        x: Px(200.0),
        y: Px(150.0),
    };
    harness.pointer_move(over_row);
    harness.advance(ms(30));
    let pin = centre(&harness, ".pin");
    harness.click(pin);
    harness.advance(ms(30));
    (harness, over_row)
}

#[test]
fn a_card_over_positioned_rows_takes_the_pointer_and_the_click() {
    let cases: [(Card, fn() -> Element); 4] = [
        (Card::FixedAfter, FixedAfterApp),
        (Card::RaisedAfter, RaisedAfterApp),
        (Card::RaisedBefore, RaisedBeforeApp),
        (Card::Overlay, OverlayApp),
    ];
    for (card, app) in cases {
        let (harness, over_row) = point_and_click(app);
        assert!(
            harness.hits(over_row, ".card"),
            "{card:?}: {}",
            harness.html()
        );
        assert!(
            !harness.hits(over_row, ".r3"),
            "{card:?}: row 3 is under the card"
        );
        assert_eq!(log(&harness), "enter card,click pin", "{card:?}");
    }
}

#[test]
fn rows_later_in_the_tree_paint_over_an_earlier_fixed_card() {
    // CSS 2.1 Appendix E step 8: positioned boxes with `z-index: auto` paint in tree order, so
    // `position: relative` rows after a fixed card are above it, in a browser as in Blitz. A
    // floating surface belongs in the overlay host (or needs a z-index), never before the list.
    let (harness, over_row) = point_and_click(FixedBeforeApp);
    assert!(harness.hits(over_row, ".r3"));
    assert!(!harness.hits(over_row, ".card"));
}

/// One mail row as mailo draws it: the name opens the sender card, the row the thread card,
/// and being back on the row after the name reopens the thread card.
#[component]
fn MailRow(i: usize, log: Signal<Vec<String>>) -> Element {
    let driver = use_hover_intent();
    let thread = move || {
        driver.over(
            HoverKey(format!("thread:{i}")),
            HoverKind::Thread,
            HoverAnchor::Unplaced,
        );
    };
    let sender = PartHooks {
        onpointerenter: EventHandler::new(move |event: PointerEvent| {
            // mailo's `line_at`: the part's top-left corner and one line's height.
            let client = event.client_coordinates();
            let offset = event.element_coordinates();
            let line = Rect {
                origin: Point {
                    x: Px((client.x - offset.x) as f32),
                    y: Px((client.y - offset.y) as f32),
                },
                size: Size {
                    width: Px(0.0),
                    height: Px(16.0),
                },
            };
            driver.over(
                HoverKey(format!("sender:{i}")),
                HoverKind::Sender,
                HoverAnchor::Rect(line),
            );
        }),
        onpointerleave: EventHandler::new(move |_| driver.out()),
    };
    rsx! {
        div { class: "mrow m{i}",
            ListRow {
                selection: Selection::Unselected,
                emphasis: Emphasis::Strong,
                index: StaggerIndex::new(0),
                presence: Presence::Present,
                name: "Sender {i}",
                via: None,
                subject: "Subject {i}",
                snippet: None,
                time: "09:41",
                tags: rsx! {},
                star: None,
                star_pulse: PulseKey::rest(Anim::StarPop),
                strip: None,
                onclick: move |_| log.write().push(format!("click m{i}")),
                on_sender: Some(sender),
                onpointerenter: EventHandler::new(move |_| {
                    log.write().push(format!("enter m{i}"));
                    thread();
                }),
                onpointerleave: EventHandler::new(move |_| {
                    log.write().push(format!("leave m{i}"));
                    driver.out();
                }),
                onpointerdown: EventHandler::new(move |_| driver.press()),
                onpointerback: EventHandler::new(move |_| {
                    log.write().push(format!("back m{i}"));
                    thread();
                }),
            }
        }
    }
}

#[component]
fn MailPage() -> Element {
    let log = use_signal(Vec::<String>::new);
    let hub = use_hover_intent().hub();
    let open = hub.open().or(hub.leaving());
    rsx! {
        div {
            class: "list-col",
            style: "position:relative; height:440px; overflow-y:auto; width:420px",
            for i in 0..8 {
                MailRow { key: "{i}", i, log }
            }
        }
        if let Some((key, kind)) = open {
            HoverCard { key: "{key.0}", kind,
                p { class: "card-of", "{key.0}" }
                button {
                    class: "pin",
                    style: "margin-top:24px",
                    onclick: move |_| log.clone().write().push("click pin".into()),
                    "Pin"
                }
            }
        }
        p { class: "log", {log().join(",")} }
    }
}

#[allow(non_snake_case)]
fn MailApp() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, MailPage {} }
    }
}

/// The harness with row 1's sender card open, and the name's centre.
fn sender_card_open() -> (Harness, Point) {
    let mut harness = Harness::new(MailApp, VIEW);
    harness.advance(ms(50));
    let name = centre(&harness, ".m1 .ds-row-name");
    harness.pointer_move(name);
    settle_until(&mut harness, |h| {
        h.text_of(".card-of").as_deref() == Some("sender:1")
    });
    harness.advance(ms(100));
    (harness, name)
}

/// Open row 1's sender card, move from its name to the card's button in `steps` moves (one: a
/// jump straight onto the card, as mailo's report; eight: across the 6 px gap on the row, then
/// the card over rows 2 and 3), rest, and click the button.
fn cross_to_the_button(steps: u16) {
    let (mut harness, name) = sender_card_open();
    let pin = centre(&harness, ".pin");
    let row = harness.rect(".m2").expect("row 2");
    assert!(
        pin.y.0 > row.origin.y.0,
        "the card's button is over the rows below: {pin:?}, row 2 {row:?}"
    );
    for step in 1..=steps {
        let t = f32::from(step) / f32::from(steps);
        let at = Point {
            x: Px(name.x.0 + (pin.x.0 - name.x.0) * t),
            y: Px(name.y.0 + (pin.y.0 - name.y.0) * t),
        };
        harness.pointer_move(at);
        harness.advance(ms(20));
        assert_eq!(
            harness.text_of(".card-of").as_deref(),
            Some("sender:1"),
            "step {step} at {at:?}: {}",
            log(&harness)
        );
    }
    harness.advance(ms(600));
    assert_eq!(harness.text_of(".card-of").as_deref(), Some("sender:1"));
    harness.click(centre(&harness, ".pin"));
    harness.advance(ms(30));
    let log = log(&harness);
    assert!(log.ends_with("click pin"), "{log}");
    assert!(
        !log.contains("back"),
        "the pointer left for the card: {log}"
    );
    assert!(
        !log.contains("m2") && !log.contains("m3"),
        "no row under the card heard it: {log}"
    );
}

#[test]
fn the_sender_card_stays_when_the_pointer_jumps_onto_it() {
    cross_to_the_button(1);
}

#[test]
fn the_sender_card_stays_while_the_pointer_crosses_the_row_to_it() {
    cross_to_the_button(8);
}

#[test]
fn leaving_the_name_for_its_row_reopens_the_thread_card() {
    let (mut harness, _) = sender_card_open();
    // On the row's subject line, right of the card (which covers the subject's centre).
    let card = harness.rect(".ds-hovercard").expect("the sender card");
    let row = harness.rect(".m1 .ds-row").expect("row 1");
    let on_row = Point {
        x: Px(card.origin.x.0 + card.size.width.0 + 8.0),
        y: centre(&harness, ".m1 .ds-row-sub").y,
    };
    assert!(
        on_row.x.0 < row.origin.x.0 + row.size.width.0 - 60.0,
        "{on_row:?} in {row:?}"
    );
    harness.pointer_move(on_row);
    // Not at once: the pointer may be crossing the row to the card (the close grace, 150 ms).
    harness.advance(ms(60));
    assert_eq!(harness.text_of(".card-of").as_deref(), Some("sender:1"));
    settle_until(&mut harness, |h| {
        h.text_of(".card-of").as_deref() == Some("thread:1")
    });
    assert_eq!(
        harness.text_of(".card-of").as_deref(),
        Some("thread:1"),
        "{}",
        log(&harness)
    );
    assert!(log(&harness).contains("back m1"), "{}", log(&harness));
}

/// Rows first, then a card that is `position: absolute` with no z-index inside a static box.
#[allow(non_snake_case)]
fn NestedAutoApp() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "position:relative; height:460px",
                for i in 0..8 {
                    div {
                        key: "{i}",
                        class: "row r{i}",
                        style: "position:relative; height:40px",
                        onclick: move |_| log.write().push(format!("click r{i}")),
                        "row {i}"
                    }
                }
                div {
                    div {
                        class: "card",
                        style: "position:absolute; left:40px; top:70px; width:300px; height:120px; background:white",
                    }
                }
            }
        }
    }
}

#[test]
fn blitz_orders_auto_positioned_boxes_among_siblings_only() {
    // A known Blitz deviation (FINDINGS "A part's leave is not the row's enter", Blitz
    // `layout/damage.rs` `node_to_paint_order`): CSS 2.1 Appendix E step 8 paints every
    // `z-index: auto` positioned box of a stacking context in tree order, so this card, last in
    // the tree, is above the rows in a browser. Blitz sorts positioned boxes above in-flow ones
    // per parent only: the card's static wrapper sorts below its positioned siblings, the rows,
    // and the card goes under them for paint and hit test alike. quire never draws a floating
    // surface this way (the overlay host carries a z-index); when Blitz follows step 8, this
    // flips and the assertion goes the other way.
    let mut harness = Harness::new(NestedAutoApp, VIEW);
    harness.advance(ms(50));
    let over_row = Point {
        x: Px(200.0),
        y: Px(150.0),
    };
    assert!(harness.hits(over_row, ".r3"), "{}", harness.html());
    assert!(!harness.hits(over_row, ".card"));
}
