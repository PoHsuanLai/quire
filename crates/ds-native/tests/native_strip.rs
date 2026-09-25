//! Native focus, the hover strip's hit area on Blitz (FINDINGS "Native focus"): in a 74 px row
//! the strip is the design's box (design/04-COMPONENTS.md section 17: right 8, vertically
//! centred, padding 3, buttons 26 with gap 3, a hairline border), its layout rect is where it
//! paints and takes the pointer, a hidden strip takes no hits, and a click at the row's centre
//! reaches the row unless a shown strip is wider than half the row.

use dioxus::prelude::*;
use ds::components::vocab::{Emphasis, PulseKey, Selection, StaggerIndex};
use ds::{
    ActionId, Anim, Appearance, Ds, HoverStrip, Icon, ListRow, Material, Point, Presence, Px, Rect,
    Shown, StripAction,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 240,
    scale_percent: 100,
};

/// The row's height mailo reports; the test's own style, not a quire value.
const ROW: &str = ".ds-row{height:74px;box-sizing:border-box}";

/// How wide the row is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Width {
    /// A 560 px row: a four-button strip stays right of its centre.
    Wide,
    /// A 300 px row: a five-button strip (150 px) crosses its centre.
    Narrow,
}

/// Four buttons, or five in the narrow row.
fn actions(width: Width) -> Vec<StripAction> {
    let ids: &[&str] = match width {
        Width::Wide => &["archive", "snooze", "label", "read"],
        Width::Narrow => &["archive", "snooze", "label", "read", "move"],
    };
    ids.iter()
        .map(|id| StripAction {
            id: ActionId((*id).to_string()),
            icon: Icon::Archive,
            label: (*id).to_string(),
            fly: (*id).to_string(),
            onhover: None,
            onclick: EventHandler::new(|_| {}),
        })
        .collect()
}

/// One row with its strip, logging what a click reaches.
#[component]
fn Page(shown: Option<Shown>, width: Width) -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let list = match width {
        Width::Wide => "width:600px; padding:20px; margin:0",
        Width::Narrow => "width:340px; padding:20px; margin:0",
    };
    let actions = actions(width);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            style { {ROW} }
            ul { class: "list", style: list,
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
                    strip: rsx! {
                        HoverStrip {
                            actions,
                            shown,
                            on_press: move |id: ActionId| log.with_mut(|log| log.push(format!("press:{}", id.0))),
                        }
                    },
                    onclick: move |_| log.with_mut(|log| log.push("open".to_string())),
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

#[allow(non_snake_case)]
fn Hidden() -> Element {
    rsx! { Page { shown: Some(Shown::Hidden), width: Width::Wide } }
}

#[allow(non_snake_case)]
fn Visible() -> Element {
    rsx! { Page { shown: Some(Shown::Visible), width: Width::Wide } }
}

#[allow(non_snake_case)]
fn OnHover() -> Element {
    rsx! { Page { shown: None, width: Width::Wide } }
}

#[allow(non_snake_case)]
fn NarrowHidden() -> Element {
    rsx! { Page { shown: Some(Shown::Hidden), width: Width::Narrow } }
}

#[allow(non_snake_case)]
fn NarrowVisible() -> Element {
    rsx! { Page { shown: Some(Shown::Visible), width: Width::Narrow } }
}

fn rect(harness: &Harness, selector: &str) -> Rect {
    harness
        .rect(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

fn settled(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, VIEW);
    harness.advance(Duration::from_millis(600));
    harness
}

/// Click the row's centre and read what heard it.
fn click_centre(app: fn() -> Element) -> String {
    let mut harness = settled(app);
    let centre = harness.centre(".ds-row").expect("the row");
    harness.click(centre);
    harness.advance(Duration::from_millis(100));
    harness.text_of(".log").unwrap_or_default()
}

/// The strip's box relative to its row: (left from the row's right edge, top, width, height).
fn strip_box(harness: &Harness) -> (f32, f32, f32, f32) {
    let row = rect(harness, ".ds-row");
    let strip = rect(harness, ".ds-strip");
    let row_right = row.origin.x.0 + row.size.width.0;
    let strip_right = strip.origin.x.0 + strip.size.width.0;
    (
        row_right - strip_right,
        strip.origin.y.0 - row.origin.y.0,
        strip.size.width.0,
        strip.size.height.0,
    )
}

#[test]
fn the_strip_is_the_designs_box_in_a_74px_row() {
    let harness = settled(Visible);
    assert_eq!(rect(&harness, ".ds-row").size.height.0, 74.0);
    // Right 8 inside the row's hairline; 34 tall (26 + 3 + 3 + 1 + 1) centred in 74, so 20
    // from the top; four buttons of 26 with three gaps of 3, plus padding and border: 121.
    assert_eq!(strip_box(&harness), (9.0, 20.0, 121.0, 34.0));
}

#[test]
fn the_strip_takes_the_pointer_where_its_rect_says() {
    let mut harness = settled(Visible);
    harness.pointer_move(harness.centre(".ds-row").expect("the row"));
    let strip = rect(&harness, ".ds-strip");
    let button = rect(&harness, ".ds-strip .ds-icon-button");
    let inside = |x: f32, y: f32| Point { x: Px(x), y: Px(y) };
    let (x0, y0) = (strip.origin.x.0, strip.origin.y.0);
    let (x1, y1) = (x0 + strip.size.width.0, y0 + strip.size.height.0);
    for corner in [
        inside(x0 + 1.0, y0 + 1.0),
        inside(x1 - 1.0, y0 + 1.0),
        inside(x0 + 1.0, y1 - 1.0),
        inside(x1 - 1.0, y1 - 1.0),
    ] {
        assert!(
            harness.hits(corner, ".ds-strip"),
            "{corner:?} misses the strip"
        );
    }
    for outside in [inside(x0 + 20.0, y0 - 2.0), inside(x0 + 20.0, y1 + 2.0)] {
        assert!(
            !harness.hits(outside, ".ds-strip"),
            "{outside:?} hits the strip"
        );
    }
    let middle = inside(
        button.origin.x.0 + button.size.width.0 / 2.0,
        button.origin.y.0 + button.size.height.0 / 2.0,
    );
    assert!(harness.hits(middle, ".ds-strip .ds-icon-button"));
}

#[test]
fn a_hidden_strip_takes_no_hits_even_under_the_pointer() {
    let mut harness = settled(Hidden);
    let button = harness
        .centre(".ds-strip .ds-icon-button")
        .expect("a button");
    harness.pointer_move(button);
    harness.advance(Duration::from_millis(400));
    assert!(!harness.hits(button, ".ds-strip"));
    assert!(harness.hits(button, ".ds-row"));
}

#[test]
fn a_centre_click_reaches_the_row_hidden_shown_or_hovered() {
    for app in [Hidden as fn() -> Element, Visible, OnHover] {
        assert_eq!(click_centre(app), "open");
    }
}

#[test]
fn a_shown_strip_wider_than_half_its_row_takes_the_centre() {
    // 300 px row, five buttons: 150 wide, its left edge 159 from the row's right, past the
    // centre at 150. The design's box, the same in a browser: the strip is on top.
    let harness = settled(NarrowVisible);
    assert_eq!(strip_box(&harness), (9.0, 20.0, 150.0, 34.0));
    assert_eq!(click_centre(NarrowVisible), "press:archive");
    assert_eq!(click_centre(NarrowHidden), "open");
}
