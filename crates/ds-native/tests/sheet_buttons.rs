//! Sheet and modal parts, Q92 and Q93, on a real Blitz document: a Danger at
//! `ButtonSize::Regular` stands as tall as the Secondary beside it (a Danger with no size keeps
//! its Mini height), and a press on a disabled button, or on a disabled icon button, fires
//! nothing, while the same press on its enabled neighbour does, and it is drawn faded: its
//! darkest ink is far lighter than the same button's enabled.

use dioxus::prelude::*;
use ds::{
    Appearance, Availability, Button, ButtonSize, ButtonVariant, Ds, Icon, IconButton,
    IconButtonVariant, Material,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 200,
    scale_percent: 100,
};

/// A power menu's row, and a row of the same actions disabled.
#[allow(non_snake_case)]
fn Page() -> Element {
    let mut log = use_signal(Vec::<&'static str>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "row", style: "padding:20px; display:flex; gap:8px; align-items:flex-start",
                span { class: "cancel",
                    Button { variant: ButtonVariant::Secondary, label: "Cancel", onclick: move |_| log.with_mut(|log| log.push("cancel")) }
                }
                span { class: "restart",
                    Button { variant: ButtonVariant::Danger, size: ButtonSize::Regular, label: "Restart", onclick: move |_| log.with_mut(|log| log.push("restart")) }
                }
                span { class: "mini-danger",
                    Button { variant: ButtonVariant::Danger, label: "Delete", onclick: move |_| {} }
                }
                span { class: "suspend",
                    Button { variant: ButtonVariant::Danger, size: ButtonSize::Regular, label: "Suspend", availability: Availability::Disabled, onclick: move |_| log.with_mut(|log| log.push("suspend")) }
                }
                span { class: "lock",
                    IconButton { variant: IconButtonVariant::Tool, icon: Icon::Lock, label: "Lock", availability: Availability::Disabled, onclick: move |_| log.with_mut(|log| log.push("lock")) }
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn page() -> Harness {
    let mut harness = Harness::new(Page, VIEW);
    harness.advance(Duration::from_millis(50));
    harness
}

#[test]
fn a_regular_danger_is_as_tall_as_the_secondary_beside_it() {
    let harness = page();
    let height = |selector: &str| {
        harness
            .rect(selector)
            .unwrap_or_else(|| panic!("{selector} is laid out"))
            .size
            .height
            .0
    };
    let secondary = height(".cancel .ds-button");
    assert_eq!(height(".restart .ds-button"), secondary);
    assert!(
        height(".mini-danger .ds-button") < secondary - 4.0,
        "a Danger with no size keeps the Mini's height"
    );
}

#[test]
fn a_press_on_a_disabled_button_fires_nothing() {
    let mut harness = page();
    for selector in [".suspend .ds-button", ".lock .ds-icon-button"] {
        let at = harness.centre(selector).expect("the button is laid out");
        harness.click(at);
        harness.advance(Duration::from_millis(50));
    }
    assert_eq!(harness.text_of(".log").as_deref(), Some(""));
    // The same press on the enabled neighbour is heard, so the silence is the button's.
    let restart = harness.centre(".restart .ds-button").expect("laid out");
    harness.click(restart);
    harness.advance(Duration::from_millis(50));
    assert_eq!(harness.text_of(".log").as_deref(), Some("restart"));
}

/// The darkest channel sum inside `selector`'s box: its label's ink.
fn darkest(harness: &Harness, frame: &image::RgbaImage, selector: &str) -> u32 {
    let rect = harness.rect(selector).expect("laid out");
    let (x0, y0) = (rect.origin.x.0 as u32 + 2, rect.origin.y.0 as u32 + 2);
    let (x1, y1) = (
        (rect.origin.x.0 + rect.size.width.0) as u32 - 2,
        (rect.origin.y.0 + rect.size.height.0) as u32 - 2,
    );
    (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .map(|(x, y)| {
            let [r, g, b, _] = frame.get_pixel(x, y).0;
            u32::from(r) + u32::from(g) + u32::from(b)
        })
        .min()
        .unwrap_or(0)
}

#[test]
fn a_disabled_button_is_drawn_at_a_third() {
    let mut harness = page();
    let frame = harness.render().expect("the page renders");
    let enabled = darkest(&harness, &frame, ".restart .ds-button");
    let disabled = darkest(&harness, &frame, ".suspend .ds-button");
    // The same Danger Regular: at .35 over the light frame its ink rises by well over a third
    // of the way to the ground.
    assert!(
        disabled > enabled + 150,
        "enabled ink {enabled}, disabled ink {disabled}"
    );
}
