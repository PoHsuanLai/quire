//! An inline menu in a card inside a popover (mailo's Spotlight panel: a field and the rows in
//! one `div` in the popover's body) is inset as a floating menu is; an avatar in a row is drawn
//! at the size its face has.

use dioxus::prelude::*;
use ds::base::geometry::placement::{Align, Side};
use ds::base::vocab::Dismiss;
use ds::components::content::avatar::{AvatarFace, AvatarShape, AvatarSize, AvatarTone};
use ds::components::overlays::popover::Arrow;
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 480,
    scale_percent: 100,
};

fn rows() -> Vec<MenuItem<u8>> {
    let avatar = |size| {
        MenuImage::Avatar(AvatarFace {
            initial: 'D',
            size,
            tone: AvatarTone::Ink,
            shape: AvatarShape::Round,
        })
    };
    vec![
        MenuItem::new(1, "One").with_image(avatar(AvatarSize::Size22)),
        MenuItem::new(2, "Two").with_image(avatar(AvatarSize::Size34)),
    ]
}

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "position:relative; height:480px",
                Popover {
                    anchor: Anchor::Point(Point { x: Px(40.0), y: Px(40.0) }),
                    placement: Placement::new(Side::Bottom, Align::Start),
                    gap: Px(0.0),
                    arrow: Arrow::None,
                    dismiss: Dismiss::Transient,
                    onclose: |()| {},
                    div { style: "display:flex; flex-direction:column; width:400px",
                        TextField {
                            kind: FieldKind::Search,
                            label: "Search".to_owned(),
                            value: String::new(),
                            oninput: |_| {},
                        }
                        div { style: "max-height:300px; overflow-y:auto",
                            Menu::<u8> {
                                placement: MenuPlacement::Popup,
                                anchor: Anchor::Point(Point::default()),
                                flow: Flow::Inline,
                                items: rows(),
                                active: MenuCursor::Controlled(Some(0)),
                                onpick: |_| {},
                                onclose: |()| {},
                            }
                        }
                    }
                }
                Menu::<u8> {
                    placement: MenuPlacement::Popup,
                    anchor: Anchor::Point(Point { x: Px(500.0), y: Px(40.0) }),
                    items: rows(),
                    onpick: |_| {},
                    onclose: |()| {},
                }
            }
        }
    }
}

#[test]
fn an_inline_menu_in_a_card_has_the_floating_menus_row_inset() {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(400));
    let rect = |selector: &str| harness.rect(selector).expect(selector);
    let floating = rect(".ds-popover.ds-menu:not([*|data-flow=inline])");
    let floating_row = rect(".ds-popover.ds-menu:not([*|data-flow=inline]) .ds-menu-item");
    let inline = rect(".ds-popover-body .ds-menu");
    let inline_row = rect(".ds-popover-body .ds-menu-item");
    let want = floating_row.origin.x.0 - floating.origin.x.0;
    let got = inline_row.origin.x.0 - inline.origin.x.0;
    assert!(want > 0.0, "the floating menu insets its rows");
    assert!((got - want).abs() < 0.5, "inline {got}, floating {want}");
}

#[test]
fn an_avatar_is_drawn_at_the_size_it_is_given() {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(400));
    let width = |size: u8| {
        harness
            .rect(&format!(
                ".ds-popover-body .ds-avatar[*|data-size=\"{size}\"]"
            ))
            .map(|rect| rect.size.width.0)
    };
    assert_eq!(width(22), Some(22.0));
    assert_eq!(width(34), Some(34.0));
}
