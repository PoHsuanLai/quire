//! Sheet and modal parts, Q92, on a real Blitz document: a Danger at `ButtonSize::Regular`
//! stands as tall as the Secondary beside it (a Danger with no size keeps its Mini height).

use dioxus::prelude::*;
use ds::{Appearance, Button, ButtonSize, ButtonVariant, Ds, Material};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 200,
    scale_percent: 100,
};

/// A power menu's row.
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
