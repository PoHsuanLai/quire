//! A toolbar pick hands over the button it came from (design/30 section 2.7): a menu hung from
//! that anchor stands under that button, whichever item it is, and an item that was behind the
//! overflow chevron reports the chevron.

use dioxus::prelude::*;
use ds::components::chrome::toolbar::model::{Picked, ToolbarItem, ToolbarRoom};
use ds::components::chrome::toolbar::view::Toolbar;
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 600,
    height: 300,
    scale_percent: 100,
};

thread_local! {
    static ROOM: Cell<f32> = const { Cell::new(560.0) };
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut hung = use_signal(|| None::<(u8, Anchor)>);
    let leading = vec![ToolbarItem::new(1u8, "Back", Icon::ChevronLeft)];
    let trailing = vec![
        ToolbarItem::new(2u8, "Share", Icon::Upload),
        ToolbarItem::new(3u8, "Tag", Icon::Tag),
        ToolbarItem::new(4u8, "Search", Icon::Search),
    ];
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            Toolbar::<u8> {
                leading,
                trailing,
                title: Some(TextLine::from("Docs")),
                room: ToolbarRoom::Fixed(Px(ROOM.with(Cell::get))),
                onpick: move |pick: Picked<u8>| {
                    if let Some(anchor) = pick.anchor {
                        hung.set(Some((pick.value, anchor)));
                    }
                },
            }
            if let Some((_, anchor)) = hung() {
                Menu::<u8> {
                    placement: MenuPlacement::Popup,
                    anchor,
                    items: vec![MenuItem::new(9u8, "Work"), MenuItem::new(8u8, "Travel")],
                    onpick: move |_| hung.set(None),
                    onclose: move |()| hung.set(None),
                }
            }
        }
    }
}

fn start(room: f32) -> Harness {
    ROOM.with(|cell| cell.set(room));
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(200));
    harness
}

/// Pick the button at `selector` and report where the menu stands against the button.
fn menu_under(
    harness: &mut Harness,
    selector: &str,
) -> (
    ds::base::geometry::units::Rect,
    ds::base::geometry::units::Rect,
) {
    let button = harness.rect(selector).expect("the button");
    let at = harness.centre(selector).expect("its centre");
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(400));
    (button, harness.rect(".ds-menu").expect("the menu"))
}

#[test]
fn a_menu_hung_from_a_trailing_item_stands_under_that_button() {
    let mut harness = start(560.0);
    let (button, menu) = menu_under(
        &mut harness,
        ".ds-toolbar-trailing > .ds-button:nth-child(2)",
    );
    assert!(
        menu.origin.y.0 >= button.origin.y.0 + button.size.height.0 - 12.0,
        "under the button: {menu:?} {button:?}"
    );
    let (left, right) = (button.origin.x.0, button.origin.x.0 + button.size.width.0);
    assert!(
        menu.origin.x.0 < right + 4.0 && menu.origin.x.0 + menu.size.width.0 > left - 4.0,
        "over the button's own columns: {menu:?} {button:?}"
    );
}

#[test]
fn a_menu_hung_from_the_leading_item_stands_at_the_leading_edge_not_the_centre() {
    let mut harness = start(560.0);
    let (button, menu) = menu_under(&mut harness, ".ds-toolbar-leading > .ds-button");
    assert!(button.origin.x.0 < 40.0, "{button:?}");
    assert!(menu.origin.x.0 < 80.0, "the menu is at the left: {menu:?}");
}

#[test]
fn an_item_behind_the_chevron_reports_the_chevron() {
    let mut harness = start(260.0);
    let chevron = harness
        .centre(".ds-toolbar-trailing > .ds-button:last-child")
        .expect("the chevron");
    harness.send(Input::click(chevron));
    harness.advance(Duration::from_millis(400));
    let first = harness
        .centre(".ds-menu-item:nth-of-type(1)")
        .expect("a hidden item");
    harness.send(Input::click(first));
    harness.advance(Duration::from_millis(700));
    // The toolbar's pick opened our own menu, from the chevron's rect.
    let menu = harness.rect(".ds-menu").expect("the hung menu");
    let button = harness
        .rect(".ds-toolbar-trailing > .ds-button:last-child")
        .expect("the chevron");
    let (left, right) = (button.origin.x.0, button.origin.x.0 + button.size.width.0);
    assert!(
        menu.origin.x.0 < right + 4.0 && menu.origin.x.0 + menu.size.width.0 > left - 4.0,
        "{menu:?} {button:?}"
    );
}
