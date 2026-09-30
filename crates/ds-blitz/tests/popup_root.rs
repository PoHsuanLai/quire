//! A popup-mode root (`RootExtent::Popup`) on a real Blitz document: the menu's card lies in
//! flow at the root's origin, so the document measures the card and nothing more; a press on a
//! row reaches it, and Escape closes it. Without the mode the same root measures nothing.

use dioxus::prelude::*;
use ds::{Anchor, Appearance, Ds, Material, Menu, MenuItem, Point, Px, RootExtent, ShortcutKey};
use ds_blitz::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 400,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A popup document as a host fits it: a frame at the origin, holding the root and its menu.
#[component]
fn Popup(extent: RootExtent) -> Element {
    let mut log = use_signal(String::new);
    let mut open = use_signal(|| true);
    rsx! {
        div { style: "position:absolute; left:0; top:0",
            p { class: "log", "{log}" }
            Ds { appearance: Appearance::default(), material: Material::Popover, extent,
                if open() {
                    Menu::<usize> {
                        placement: ds::MenuPlacement::Bar,
                        anchor: Anchor::Point(Point { x: Px(200.0), y: Px(150.0) }),
                        items: ["Open", "Quit"]
                            .into_iter()
                            .enumerate()
                            .map(|(value, name)| MenuItem::new(value, name))
                            .collect::<Vec<_>>(),
                        onpick: move |value: usize| log.set(format!("pick:{value}")),
                        onclose: move |()| open.set(false),
                    }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn Fitted() -> Element {
    rsx! { Popup { extent: RootExtent::Popup } }
}

#[allow(non_snake_case)]
fn Plain() -> Element {
    rsx! { Popup { extent: RootExtent::Content } }
}

fn laid_out(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, VIEW);
    harness.advance(ms(400));
    harness
}

#[test]
fn a_popup_root_is_the_size_of_its_card_at_the_origin() {
    let harness = laid_out(Fitted);
    assert_eq!(harness.attr(".ds", "data-extent").as_deref(), Some("popup"));
    let card = harness.rect(".ds-popover").expect("the card");
    let root = harness.rect(".ds").expect("the root");
    assert!(card.size.height.0 > 20.0, "{card:?}");
    assert!(card.size.width.0 < 400.0, "not the whole surface: {card:?}");
    assert!(
        (root.size.height.0 - card.size.height.0).abs() < 1.0,
        "root {root:?} is card {card:?}"
    );
    assert_eq!(card.origin.x.0, root.origin.x.0, "{card:?}");
    assert_eq!(card.origin.y.0, root.origin.y.0, "{card:?}");
    assert_eq!(harness.count(".ds-overlay-catch"), 1);
    let catch = harness.rect(".ds-overlay-catch");
    assert!(
        catch.is_none_or(|rect| rect.size.height.0 == 0.0),
        "the catcher does not cover the surface: {catch:?}"
    );
}

#[test]
fn without_the_mode_the_same_root_is_not_the_card() {
    let harness = laid_out(Plain);
    let root = harness.rect(".ds").expect("the root");
    let card = harness.rect(".ds-popover").expect("the card");
    assert!(
        (root.size.height.0 - card.size.height.0).abs() > 1.0,
        "the trap G367 names: {root:?} vs {card:?}"
    );
}

#[test]
fn a_press_on_a_row_reaches_it_and_escape_closes() {
    let mut harness = laid_out(Fitted);
    let second = harness
        .centre(".ds-menu-item:nth-child(2) .ds-menu-label")
        .expect("the second row");
    harness.pointer_move(second);
    harness.click(second);
    harness.advance(ms(300));
    assert_eq!(harness.text_of(".log").as_deref(), Some("pick:1"));
    let mut harness = laid_out(Fitted);
    assert!(harness.is_focused(".ds-menu"), "the menu has the keyboard");
    harness.key(ShortcutKey::Escape);
    harness.advance(ms(400));
    assert_eq!(harness.count(".ds-menu"), 0);
}
