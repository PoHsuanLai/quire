//! A hint that stands beside its target (a caller-driven tooltip or dock label, a popover) is
//! never painted before it is placed: until its target's rect and its own size are measured it is
//! `visibility:hidden`, so it cannot flash at the overlay's top-left corner; once placed it stands
//! beside its target.

use dioxus::prelude::*;
use ds::{Appearance, Ds, Material, RootExtent, Shown, Tooltip};
use ds_native::{Clock, Harness, HarnessConfig, Viewport};
use ds_shell::DockLabel;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 400,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            div { style: "position:absolute;left:300px;top:200px",
                Tooltip { text: "Snooze until tomorrow", shown: Some(Shown::Visible),
                    span { id: "a", style: "display:inline-block;width:60px;height:24px", "Snooze" }
                }
            }
            div { style: "position:absolute;left:100px;top:300px",
                DockLabel { text: "Files", shown: Some(Shown::Visible),
                    span { id: "b", style: "display:inline-block;width:48px;height:48px", "Files" }
                }
            }
        }
    }
}

#[test]
fn a_shown_hint_is_hidden_until_placed_and_never_at_the_origin() {
    let mut harness =
        Harness::with_config(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    for selector in [".ds-tooltip", ".ds-dock-label"] {
        let style = harness.attr(selector, "style").unwrap_or_default();
        assert!(
            style.contains("visibility:hidden"),
            "{selector} unplaced: {style}"
        );
    }
    harness.advance(Duration::from_millis(300));
    for (selector, target) in [(".ds-tooltip", "#a"), (".ds-dock-label", "#b")] {
        let style = harness.attr(selector, "style").unwrap_or_default();
        assert!(
            !style.contains("visibility:hidden"),
            "{selector} placed: {style}"
        );
        let hint = harness.rect(selector).expect("the hint");
        let target = harness.rect(target).expect("the target");
        assert!(
            (hint.origin.x.0 - target.origin.x.0).abs() < 200.0
                && (hint.origin.y.0 - target.origin.y.0).abs() < 60.0,
            "{selector} stands beside its target: {hint:?} {target:?}"
        );
    }
}
