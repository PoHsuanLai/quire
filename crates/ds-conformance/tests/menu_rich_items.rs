//! Menu items on a real Blitz document: an inline menu inside a popover has the row inset a
//! floating menu has; a result row draws its matched characters, an avatar and a second line,
//! and a plain row beside it is drawn as it always was.

use dioxus::prelude::*;
use ds::base::geometry::placement::{Align, Side};
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
    let avatar = MenuImage::Avatar(AvatarFace {
        initial: 'D',
        size: AvatarSize::Size22,
        tone: AvatarTone::Ink,
        shape: AvatarShape::Round,
    });
    vec![
        MenuItem::new(1, "Invoice from Dana")
            .with_image(avatar)
            .with_marks(Marks::of_query("Invoice from Dana", "dana"))
            .with_subtitle("Re: March invoice"),
        MenuItem::new(2, "A plain row"),
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
                    gap: Px(4.0),
                    arrow: Arrow::None,
                    onclose: |()| {},
                    Menu::<u8> {
                        placement: MenuPlacement::Popup,
                        anchor: Anchor::Point(Point::default()),
                        items: rows(),
                        flow: Flow::Inline,
                        onpick: |_| {},
                        onclose: |()| {},
                    }
                }
                Menu::<u8> {
                    placement: MenuPlacement::Popup,
                    anchor: Anchor::Point(Point { x: Px(400.0), y: Px(40.0) }),
                    items: rows(),
                    onpick: |_| {},
                    onclose: |()| {},
                }
            }
        }
    }
}

fn start() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(400));
    harness
}

#[test]
fn an_inline_menu_in_a_popover_has_the_floating_menus_row_inset() {
    let harness = start();
    let rect = |selector: &str| harness.rect(selector).expect(selector);
    let floating = rect(".ds-popover.ds-menu:not([*|data-flow=inline])");
    let floating_row = rect(".ds-popover.ds-menu .ds-menu-item");
    let inline = rect(".ds-popover-body .ds-menu");
    let inline_row = rect(".ds-popover-body .ds-menu-item");
    let want = floating_row.origin.x.0 - floating.origin.x.0;
    let got = inline_row.origin.x.0 - inline.origin.x.0;
    assert!(want > 0.0, "the floating menu insets its rows");
    assert!((got - want).abs() < 0.5, "inline {got}, floating {want}");
}

#[test]
fn a_result_row_draws_marks_an_avatar_and_a_second_line() {
    let harness = start();
    let scope = ".ds-popover-body";
    assert_eq!(
        harness
            .text_of(&format!("{scope} .ds-menu-mark"))
            .as_deref(),
        Some("Dana")
    );
    assert_eq!(
        harness
            .text_of(&format!("{scope} .ds-menu-subtitle"))
            .as_deref(),
        Some("Re: March invoice")
    );
    assert_eq!(
        harness.count(&format!("{scope} .ds-menu-image[*|data-image=avatar]")),
        1
    );
    let rich = harness
        .rect(&format!("{scope} .ds-menu-item:nth-child(1)"))
        .expect("the rich row");
    let plain = harness
        .rect(&format!("{scope} .ds-menu-item:nth-child(2)"))
        .expect("the plain row");
    assert!(
        rich.size.height.0 > plain.size.height.0 + 8.0,
        "a second line makes the row taller: {rich:?} {plain:?}"
    );
    assert_eq!(
        harness.count(&format!(
            "{scope} .ds-menu-item:nth-child(2) .ds-menu-title"
        )),
        0,
        "a plain row has no title wrapper"
    );
}
