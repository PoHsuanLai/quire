//! ThreadRow's `more` slot on a real Blitz document: the quiet "More actions" button is laid out
//! in the tail but hidden until the row is hovered, it takes the keyboard and shows, a click
//! hands its rect to the caller, and while its menu is open it stays up with the pointer away. It
//! never overlaps the time, and a card row is exactly `thread_card_height` tall.

use dioxus::prelude::*;
use ds::base::vocab::RowState;
use ds::components::app::row_more::RowMore;
use ds::components::app::thread_height::{ThreadLines, thread_card_gap, thread_card_height};
use ds::components::app::thread_row::ThreadRow;
use ds::prelude::*;
use ds_harness::{Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 240,
    scale_percent: 100,
};

const MORE: &str = ".ds-row-more";

/// One card row carrying its `more` button, with or without a snippet and open menu.
#[component]
fn Page(lines: ThreadLines, expanded: Shown) -> Element {
    let mut log = use_signal(String::new);
    let snippet = match lines {
        ThreadLines::Two => None,
        ThreadLines::Three => Some(TextLine::from(
            "Treat UIDL as stable only while UIDVALIDITY holds.",
        )),
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            ul { class: "list", style: "width:600px; padding:20px; margin:0",
                ThreadRow {
                    state: RowState { selection: Selection::Unselected, emphasis: Emphasis::Strong, ..RowState::default() },
                    name: "Dana Okafor",
                    via: None,
                    subject: "Re: UIDL stability",
                    snippet,
                    time: "09:41",
                    tags: rsx! {},
                    star: None,
                    more: rsx! {
                        RowMore {
                            expanded,
                            onclick: move |rect: Rect| log.set(format!("rect:{}x{}", rect.size.width.0, rect.size.height.0)),
                        }
                    },
                    onclick: move |_| log.set("open".to_string()),
                }
            }
            p { class: "log", {log()} }
        }
    }
}

#[allow(non_snake_case)]
fn Three() -> Element {
    rsx! { Page { lines: ThreadLines::Three, expanded: Shown::Hidden } }
}

#[allow(non_snake_case)]
fn Two() -> Element {
    rsx! { Page { lines: ThreadLines::Two, expanded: Shown::Hidden } }
}

#[allow(non_snake_case)]
fn Open() -> Element {
    rsx! { Page { lines: ThreadLines::Three, expanded: Shown::Visible } }
}

fn settled(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(400));
    harness
}

fn rect(harness: &Harness, selector: &str) -> Rect {
    harness
        .rect(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

fn opacity(harness: &Harness) -> f32 {
    harness.opacity_of(MORE).expect("the more button")
}

/// A point well outside the row.
const AWAY: Point = Point {
    x: Px(700.0),
    y: Px(230.0),
};

#[test]
fn it_is_hidden_until_the_row_is_hovered() {
    let mut harness = settled(Three);
    assert_eq!(opacity(&harness), 0.0);
    let before = rect(&harness, MORE);
    harness.send(Input::pointer_move(
        harness.centre(".ds-thread-sub").unwrap(),
    ));
    harness.advance(Duration::from_millis(400));
    assert_eq!(opacity(&harness), 1.0);
    assert_eq!(rect(&harness, MORE), before, "the row never reflows");
    harness.send(Input::pointer_move(AWAY));
    harness.advance(Duration::from_millis(400));
    assert_eq!(opacity(&harness), 0.0);
}

#[test]
fn tab_reaches_it_and_shows_it() {
    let mut harness = settled(Three);
    for _ in 0..3 {
        if harness.focus_of(MORE) == FocusState::Focused {
            break;
        }
        harness.send(Input::key(ShortcutKey::Tab));
        harness.advance(Duration::from_millis(400));
    }
    assert_eq!(harness.focus_of(MORE), FocusState::Focused);
    assert_eq!(opacity(&harness), 1.0, "focus shows it with no pointer");
}

#[test]
fn a_click_hands_over_its_rect_and_does_not_open_the_row() {
    let mut harness = settled(Three);
    harness.send(Input::pointer_move(
        harness.centre(".ds-thread-sub").unwrap(),
    ));
    harness.advance(Duration::from_millis(400));
    harness.send(Input::click(harness.centre(MORE).unwrap()));
    harness.advance(Duration::from_millis(100));
    assert_eq!(harness.text_of(".log").as_deref(), Some("rect:26x26"));
}

#[test]
fn an_open_menu_keeps_it_visible_with_the_pointer_away() {
    let mut harness = settled(Open);
    harness.send(Input::pointer_move(AWAY));
    harness.advance(Duration::from_millis(400));
    assert_eq!(opacity(&harness), 1.0);
    assert_eq!(harness.attr(MORE, "aria-expanded").as_deref(), Some("true"));
}

#[test]
fn it_never_overlaps_the_time_or_the_text() {
    let harness = settled(Three);
    let more = rect(&harness, MORE);
    for other in [".ds-thread-time", ".ds-thread-main"] {
        let other = rect(&harness, other);
        let apart = more.right().0 <= other.left().0
            || other.right().0 <= more.left().0
            || more.bottom().0 <= other.top().0
            || other.bottom().0 <= more.top().0;
        assert!(apart, "{more:?} overlaps {other:?}");
    }
}

#[test]
fn a_card_row_is_exactly_thread_card_height_tall() {
    for (app, lines) in [
        (Two as fn() -> Element, ThreadLines::Two),
        (Three, ThreadLines::Three),
    ] {
        let harness = settled(app);
        let height = rect(&harness, ".ds-row").size.height.0;
        let want = thread_card_height(lines).0;
        assert!(
            (height - want).abs() < 0.05,
            "{lines:?}: drawn {height}, computed {want}"
        );
    }
    assert_eq!(thread_card_gap(), Px(5.0));
}
