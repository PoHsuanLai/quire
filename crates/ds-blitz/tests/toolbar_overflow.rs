//! Toolbar on a real Blitz document (design/30 section 2.7): when the room is short the last
//! items go behind the chevron, whose menu picks them.

use dioxus::prelude::*;
use ds::{Appearance, Ds, Icon, Material, Px, Toolbar, ToolbarItem, ToolbarRoom};
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 300,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut picked = use_signal(|| 0u8);
    let trailing = vec![
        ToolbarItem::new(1u8, "Share", Icon::Upload),
        ToolbarItem::new(2u8, "Tag", Icon::Tag),
        ToolbarItem::new(3u8, "Search", Icon::Search),
    ];
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            Toolbar::<u8> { trailing, title: Some(ds::TextLine::from("Docs")), room: ToolbarRoom::Fixed(Px(260.0)), onpick: move |value| picked.set(value) }
            p { class: "picked", "{picked()}" }
        }
    }
}

#[test]
fn the_items_that_do_not_fit_are_picked_from_the_chevron_menu() {
    let mut harness =
        Harness::with_config(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(100));
    // 260 - 140 - 24 = 96: the chevron and one item.
    assert_eq!(
        harness.count(".ds-toolbar-trailing > .ds-button"),
        2,
        "one item and the chevron"
    );
    let chevron = harness
        .centre(".ds-toolbar-trailing > .ds-button:last-child")
        .expect("chevron");
    harness.click(chevron);
    harness.advance(Duration::from_millis(400));
    assert!(
        harness.count(".ds-menu-item") >= 2,
        "the menu lists the hidden items"
    );
    let tag = harness
        .centre(".ds-menu-item:nth-of-type(1)")
        .expect("first hidden item");
    harness.click(tag);
    harness.advance(Duration::from_millis(600));
    assert_eq!(harness.text_of(".picked").as_deref(), Some("2"));
}
