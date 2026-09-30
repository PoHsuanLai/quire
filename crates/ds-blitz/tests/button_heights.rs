//! A default button (`Answers::Return`) draws its accent fill under a transparent `--hair` border,
//! as a plain push button and a destructive one carry a visible one, so all three stand the same
//! height (button.css): the box model matches, the paint does not differ
//! (`background-clip:border-box`'s default already paints `--accent` under a transparent border).
//! This measures that a default, a plain and a destructive push button render the same height side by
//! side, at `ControlSize::Regular` and at `ControlSize::Mini`.

use dioxus::prelude::*;
use ds::{Answers, ButtonRole, ControlSize};
use ds::{Appearance, Button, Ds, Material};
use ds_blitz::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 200,
    scale_percent: 100,
};

/// Primary, Secondary and Danger at Regular, then the same three at Mini, each in its own box.
#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "padding:20px; display:flex; gap:8px; align-items:flex-start",
                span { class: "primary-regular",
                    Button { answers: Answers::Return, label: "Shut Down", onclick: move |_| {} }
                }
                span { class: "secondary-regular",
                    Button { label: "Cancel", onclick: move |_| {} }
                }
                span { class: "danger-regular",
                    Button { role: ButtonRole::Destructive, size: ControlSize::Regular, label: "Restart", onclick: move |_| {} }
                }
                span { class: "primary-mini",
                    Button { answers: Answers::Return, size: ControlSize::Mini, label: "Shut Down", onclick: move |_| {} }
                }
                span { class: "secondary-mini",
                    Button { size: ControlSize::Mini, label: "Cancel", onclick: move |_| {} }
                }
                span { class: "danger-mini",
                    // No size: Danger's own size is Mini (design/04-COMPONENTS.md section 1).
                    Button { role: ButtonRole::Destructive, size: ControlSize::Mini, label: "Restart", onclick: move |_| {} }
                }
            }
        }
    }
}

fn page() -> Harness {
    let mut harness = Harness::new(Page, VIEW);
    harness.advance(Duration::from_millis(50));
    harness
}

#[test]
fn primary_secondary_and_danger_share_one_height_at_regular_and_at_mini() {
    let harness = page();
    let height = |selector: &str| {
        harness
            .rect(selector)
            .unwrap_or_else(|| panic!("{selector} is laid out"))
            .size
            .height
            .0
    };

    let regular = height(".primary-regular .ds-button");
    assert_eq!(
        height(".secondary-regular .ds-button"),
        regular,
        "secondary stands off Primary's height at Regular"
    );
    assert_eq!(
        height(".danger-regular .ds-button"),
        regular,
        "danger stands off Primary's height at Regular"
    );

    let mini = height(".primary-mini .ds-button");
    assert_eq!(
        height(".secondary-mini .ds-button"),
        mini,
        "secondary stands off Primary's height at Mini"
    );
    assert_eq!(
        height(".danger-mini .ds-button"),
        mini,
        "danger (its own size, no size prop) stands off Primary's height at Mini"
    );
    assert!(mini < regular, "Mini is smaller than Regular");
}
