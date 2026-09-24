//! mailo gaps 2, lists, on a real Blitz document: a strip button's click does not open its row,
//! the name and the time hand the pointer's entry and exit to the caller, the row hands over its
//! own entry and press, and the strip shows on the caller's say with no pointer on the row.

use dioxus::prelude::*;
use ds::components::vocab::{Emphasis, PulseKey, Selection, StaggerIndex};
use ds::{
    ActionId, Anim, Appearance, Ds, HoverStrip, Icon, ListRow, Material, PartHooks, Point,
    Presence, Px, Shown, StripAction,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 240,
    scale_percent: 100,
};

/// Log a line from inside a handler.
fn noter(mut log: Signal<Vec<String>>) -> impl FnMut(&str) + Copy {
    move |line: &str| log.with_mut(|log| log.push(line.to_string()))
}

/// Hooks that log `part` entering and leaving.
fn hooks(log: Signal<Vec<String>>, part: &'static str) -> PartHooks {
    let mut note = noter(log);
    let mut out = noter(log);
    PartHooks {
        onpointerenter: EventHandler::new(move |_| note(&format!("{part}-enter"))),
        onpointerleave: EventHandler::new(move |_| out(&format!("{part}-leave"))),
    }
}

/// Whether the row carries its strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Strip {
    /// A one-button strip, shown by `shown` (`None`: on hover).
    Carried(Option<Shown>),
    /// No strip: the time is not under it.
    Absent,
}

/// One row in a list, logging what reaches the page.
#[component]
fn Page(strip: Strip) -> Element {
    let log = use_signal(Vec::<String>::new);
    let mut note = noter(log);
    let actions = vec![StripAction {
        id: ActionId("archive".to_string()),
        icon: Icon::Archive,
        label: "Archive".to_string(),
        fly: "Archive → out of Inbox".to_string(),
        onhover: None,
        onclick: EventHandler::new(move |_| note("archive")),
    }];
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            ul { class: "list", style: "width:600px; padding:20px; margin:0",
                ListRow {
                    selection: Selection::Unselected,
                    emphasis: Emphasis::Strong,
                    index: StaggerIndex::new(0),
                    presence: Presence::Present,
                    name: "Dana Okafor",
                    via: None,
                    subject: "Re: UIDL stability",
                    snippet: None,
                    time: "09:41",
                    tags: rsx! {},
                    star: None,
                    star_pulse: PulseKey::rest(Anim::StarPop),
                    strip: match strip {
                        Strip::Carried(shown) => Some(rsx! { HoverStrip { actions, shown } }),
                        Strip::Absent => None,
                    },
                    onclick: move |_| note("open"),
                    on_sender: hooks(log, "sender"),
                    on_time: hooks(log, "time"),
                    onpointerenter: move |_| note("row-enter"),
                    onpointerdown: move |_| note("row-down"),
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

#[allow(non_snake_case)]
fn Hovered() -> Element {
    rsx! { Page { strip: Strip::Carried(None) } }
}

#[allow(non_snake_case)]
fn Bare() -> Element {
    rsx! { Page { strip: Strip::Absent } }
}

#[allow(non_snake_case)]
fn Revealed() -> Element {
    rsx! { Page { strip: Strip::Carried(Some(Shown::Visible)) } }
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

fn centre(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

/// Where the strip's first button is painted: the strip is centred on its row by
/// `translateY(-50%)`, which the layout rect Blitz reports leaves out (hit testing applies it).
fn strip_button(harness: &Harness) -> Point {
    let lift = harness
        .rect(".ds-strip")
        .map_or(0.0, |strip| strip.size.height.0 / 2.0);
    let at = centre(harness, ".ds-strip .ds-icon-button");
    Point {
        x: at.x,
        y: Px(at.y.0 - lift),
    }
}

/// A point well outside the row.
const AWAY: Point = Point {
    x: Px(700.0),
    y: Px(220.0),
};

#[test]
fn a_strip_click_does_not_open_the_row() {
    let mut harness = Harness::new(Hovered, VIEW);
    harness.advance(Duration::from_millis(50));
    // Over the row first, so the strip is revealed by hover and takes the pointer.
    harness.pointer_move(centre(&harness, ".ds-row-sub"));
    harness.advance(Duration::from_millis(400));
    harness.click(strip_button(&harness));
    harness.advance(Duration::from_millis(100));
    let seen = log(&harness);
    assert!(seen.contains("archive"), "the strip button acted: {seen}");
    assert!(!seen.contains("open"), "and the row did not open: {seen}");
    // The row itself still opens.
    harness.click(centre(&harness, ".ds-row-sub"));
    harness.advance(Duration::from_millis(50));
    assert!(log(&harness).ends_with("open"), "{}", log(&harness));
}

#[test]
fn the_name_the_time_and_the_row_hand_the_pointer_to_the_caller() {
    let mut harness = Harness::new(Bare, VIEW);
    harness.advance(Duration::from_millis(50));
    harness.pointer_move(centre(&harness, ".ds-row-name"));
    harness.pointer_move(centre(&harness, ".ds-row-sub"));
    harness.pointer_move(centre(&harness, ".ds-row-time"));
    harness.pointer_move(AWAY);
    harness.advance(Duration::from_millis(50));
    assert_eq!(
        log(&harness),
        "row-enter,sender-enter,sender-leave,time-enter,time-leave"
    );
    harness.pointer_down(centre(&harness, ".ds-row-sub"));
    harness.advance(Duration::from_millis(50));
    assert!(log(&harness).ends_with("row-down"), "{}", log(&harness));
}

#[test]
fn the_caller_shows_the_strip_with_no_pointer_on_the_row() {
    let mut harness = Harness::new(Revealed, VIEW);
    harness.advance(Duration::from_millis(400));
    assert_eq!(
        harness.attr(".ds-strip", "data-shown").as_deref(),
        Some("visible")
    );
    // The pointer never came over the row; the revealed strip takes a click anyway.
    let button = strip_button(&harness);
    harness.pointer_down(button);
    harness.pointer_up(button);
    harness.advance(Duration::from_millis(100));
    let seen = log(&harness);
    assert!(seen.contains("archive"), "{seen}");
    assert!(!seen.contains("open"), "{seen}");
}
