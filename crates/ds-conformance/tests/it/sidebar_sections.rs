//! A sidebar of several sections and a foot on a real Blitz document (design/30 section 2.7): the
//! foot stays at the bottom while the sections scroll above it, a section of the caller's own
//! content sits among the lists, one cursor runs across the lists, and the sidebar paints its own
//! ground by default and nothing (the window's tint shows) with `SidebarFill::Clear`, in the
//! same inks.

use dioxus::prelude::*;
use ds::base::vocab::RowState;
use ds::components::chrome::sidebar::Sidebar;
use ds::components::chrome::sidebar_model::{SidebarFill, SidebarSection};
use ds::prelude::*;
use ds::style::space::presets::PRESETS;
use ds::style::tokens::colour::ColourToken;
use ds::style::tokens::hex::Hex;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Part, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 300,
    height: 400,
    scale_percent: 100,
};

thread_local! {
    static CLEAR: Cell<bool> = const { Cell::new(false) };
    static ROWS: Cell<u8> = const { Cell::new(4) };
    static FILL: Cell<bool> = const { Cell::new(false) };
    static PICKED: Cell<u8> = const { Cell::new(0) };
}

fn rows(from: u8, count: u8, here: u8, pick: EventHandler<u8>) -> Vec<ListItem<u8>> {
    (from..from + count)
        .map(|key| {
            ListItem::row(
                key,
                format!("Place {key}"),
                rsx! {
                    Row {
                        title: TextLine::from(format!("Place {key}")),
                        state: RowState {
                            selection: Selection::of(&here, &key),
                            ..RowState::default()
                        },
                        onclick: move |_| pick.call(key),
                    }
                },
            )
        })
        .collect()
}

fn sections(count: u8, here: u8, pick: EventHandler<u8>) -> Vec<SidebarSection<u8>> {
    let pinned = SidebarSection::Custom(
        rsx! { div { class: "custom-section", style: "height:40px", "Pinned" } },
    );
    if FILL.with(Cell::get) {
        let long = SidebarSection::Fill(rsx! {
            div { class: "the-fill-list", style: "overflow-y:auto",
                for page in 0..60u8 {
                    div { key: "{page}", class: "fill-item", style: "height:24px", "Page {page}" }
                }
            }
        });
        return vec![pinned, long];
    }
    vec![
        pinned,
        SidebarSection::List(rows(1, count, here, pick)),
        SidebarSection::List(rows(100, count, here, pick)),
    ]
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut here = use_signal(|| 1u8);
    let pick = EventHandler::new(move |key: u8| {
        PICKED.with(|cell| cell.set(key));
        here.set(key);
    });
    let count = ROWS.with(Cell::get);
    let fill = if CLEAR.with(Cell::get) {
        SidebarFill::Clear
    } else {
        SidebarFill::Material
    };
    let look = SpaceLook {
        dots: PRESETS[0].dots.to_vec(),
        grain: Grain(0),
        ..SpaceLook::default()
    };
    rsx! {
        Ds { appearance: Appearance::default(), look, material: Material::Window, extent: RootExtent::Viewport,
            div { style: "height:400px;width:260px",
                Sidebar::<u8> {
                    label: "Mail",
                    fill,
                    cursor: Some(here()),
                    onselect: move |key| pick.call(key),
                    sections: sections(count, here(), pick),
                    foot: rsx! { span { class: "the-foot", "Home" } },
                }
            }
        }
    }
}

fn start(clear: bool, count: u8) -> Harness {
    FILL.with(|cell| cell.set(false));
    CLEAR.with(|cell| cell.set(clear));
    ROWS.with(|cell| cell.set(count));
    PICKED.with(|cell| cell.set(0));
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(300));
    harness
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1.0
}

#[test]
fn the_foot_stays_at_the_bottom_while_the_sections_scroll_above_it() {
    let harness = start(false, 30);
    let side = harness.rect(".ds-sidebar").expect("the sidebar");
    let foot = harness.rect(".ds-sidebar-foot").expect("the foot");
    let body = harness.rect(".ds-sidebar-body").expect("the body");
    assert!(
        near(
            foot.origin.y.0 + foot.size.height.0,
            side.origin.y.0 + side.size.height.0
        ),
        "the foot rests on the sidebar's bottom: {foot:?} in {side:?}"
    );
    assert!(
        body.origin.y.0 + body.size.height.0 <= foot.origin.y.0 + 0.5,
        "the body ends where the foot begins: {body:?} {foot:?}"
    );
    assert_eq!(harness.count(".ds-sidebar-section"), 3);
}

#[test]
fn a_short_sidebar_keeps_its_foot_at_the_bottom_not_under_the_last_row() {
    let harness = start(false, 1);
    let side = harness.rect(".ds-sidebar").expect("the sidebar");
    let foot = harness.rect(".ds-sidebar-foot").expect("the foot");
    let last = harness
        .rect(".ds-sidebar-section:last-child")
        .expect("a section");
    assert!(
        near(
            foot.origin.y.0 + foot.size.height.0,
            side.origin.y.0 + side.size.height.0
        ),
        "{foot:?} in {side:?}"
    );
    assert!(
        foot.origin.y.0 > last.origin.y.0 + last.size.height.0 + 20.0,
        "free room lies between the sections and the foot: {last:?} {foot:?}"
    );
}

#[test]
fn a_custom_section_sits_first_and_the_lists_follow_in_order() {
    let harness = start(false, 2);
    let custom = harness.rect(".custom-section").expect("the custom section");
    let first = harness
        .rect("[data-key='1']")
        .or_else(|| harness.rect(".ds-sidebar-section:nth-child(2)"))
        .expect("the first list");
    let second = harness
        .rect(".ds-sidebar-section:nth-child(3)")
        .expect("the second list");
    assert!(
        custom.origin.y.0 + custom.size.height.0 <= first.origin.y.0 + 0.5,
        "{custom:?} {first:?}"
    );
    assert!(
        first.origin.y.0 + first.size.height.0 <= second.origin.y.0 + 0.5,
        "{first:?} {second:?}"
    );
}

#[test]
fn a_row_in_the_second_list_moves_the_one_cursor() {
    let mut harness = start(false, 2);
    let at = harness
        .centre(".ds-sidebar-section:nth-child(3) .ds-row")
        .expect("a row");
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(100));
    assert_eq!(
        PICKED.with(Cell::get),
        100,
        "the first row of the second list"
    );
    assert_eq!(
        harness.count(".ds-row[aria-selected=true]"),
        1,
        "one row is selected across both lists"
    );
    assert_eq!(
        harness.count(".ds-sidebar-section:nth-child(3) .ds-row[aria-selected=true]"),
        1,
        "and it is the one clicked"
    );
}

fn matches(ink: ds_harness::Srgba, hex: Hex) -> bool {
    let [r, g, b] = hex.0;
    let [ir, ig, ib, _] = ink.0;
    [(ir, r), (ig, g), (ib, b)]
        .iter()
        .all(|&(got, want)| (got * 255.0 - f32::from(want)).abs() < 2.0)
}

#[test]
fn a_clear_sidebar_paints_nothing_and_keeps_the_ordinary_inks() {
    let own = start(false, 2);
    let clear = start(true, 2);
    assert_eq!(own.attr(".ds-sidebar", "data-fill"), None);
    assert_eq!(
        clear.attr(".ds-sidebar", "data-fill").as_deref(),
        Some("clear")
    );
    let ground = own.fill_of(".ds-sidebar", Part::Element).expect("fill");
    assert!(ground.0[3] > 0.99, "its own ground is opaque: {ground:?}");
    let none = clear.fill_of(".ds-sidebar", Part::Element).expect("fill");
    assert!(
        none.0[3] < 0.01,
        "the window's tint shows through: {none:?}"
    );
    let paper_ink = Hex::parse(&ColourToken::Ink.value(Scheme::Light).css()).expect("hex");
    for (name, harness) in [("own", &own), ("clear", &clear)] {
        let ink = harness.ink_of(".ds-sidebar").expect("ink");
        assert!(matches(ink, paper_ink), "{name}: {ink:?} vs {paper_ink:?}");
    }
}

fn started_with_fill() -> Harness {
    FILL.with(|cell| cell.set(true));
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(300));
    harness
}

fn bottom(rect: &Rect) -> f32 {
    rect.origin.y.0 + rect.size.height.0
}

#[test]
fn a_fill_section_takes_the_height_the_rows_above_it_leave() {
    let harness = started_with_fill();
    let body = harness.rect(".ds-sidebar-body").expect("the body");
    let pinned = harness.rect(".custom-section").expect("the rows above");
    let fill = harness.rect(".ds-sidebar-section-fill").expect("the fill");
    let list = harness.rect(".the-fill-list").expect("the list");
    assert_eq!(harness.count(".ds-sidebar-section-fill"), 1);
    assert!(
        fill.origin.y.0 >= bottom(&pinned) - 0.5,
        "the fill sits under the rows above: {pinned:?} {fill:?}"
    );
    assert!(
        bottom(&body) - bottom(&fill) < 16.0,
        "the fill runs to the body's end, less its padding: {body:?} {fill:?}"
    );
    assert!(
        near(list.size.height.0, fill.size.height.0),
        "the content is given the fill's whole height: {list:?} {fill:?}"
    );
    assert!(
        fill.size.height.0 < 60.0 * 24.0,
        "the fill is the room left, not the content's height: {fill:?}"
    );
    let foot = harness.rect(".ds-sidebar-foot").expect("the foot");
    assert!(
        foot.origin.y.0 >= bottom(&body) - 0.5,
        "the foot stays below the body: {foot:?} {body:?}"
    );
}

#[test]
fn with_a_fill_the_body_does_not_scroll_and_the_list_does() {
    let mut harness = started_with_fill();
    let pinned = harness.rect(".custom-section").expect("the rows above");
    let first = harness.rect(".fill-item").expect("the first item");
    let over_pinned = harness.centre(".custom-section").expect("centre");
    harness.send(Input::wheel(over_pinned, Px(0.0), Px(-200.0)));
    harness.advance(Duration::from_millis(50));
    assert_eq!(
        harness.rect(".custom-section"),
        Some(pinned),
        "the body does not scroll as a whole"
    );
    assert_eq!(
        harness.rect(".fill-item"),
        Some(first),
        "nor does the list move"
    );
    let over_list = harness.centre(".the-fill-list").expect("centre");
    harness.send(Input::wheel(over_list, Px(0.0), Px(-200.0)));
    harness.advance(Duration::from_millis(50));
    let moved = harness.rect(".fill-item").expect("the first item");
    assert!(
        moved.origin.y.0 < first.origin.y.0 - 20.0,
        "the list scrolls inside the fill: {first:?} then {moved:?}"
    );
    assert_eq!(
        harness.rect(".custom-section"),
        Some(pinned),
        "and the rows above stay"
    );
}

#[test]
fn without_a_fill_the_body_stays_the_scrolling_one() {
    let harness = start(false, 3);
    assert_eq!(harness.count(".ds-sidebar-section-fill"), 0);
    assert_eq!(harness.count(".ds-sidebar-body-split"), 0);
}
