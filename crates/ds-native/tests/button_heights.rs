//! Sheet and modal parts, Q110: `ButtonVariant::Primary` drew with `border:0` while Secondary
//! and Danger carry a `--hair` border, so a Primary stood one `--hair` short top and bottom of
//! a Secondary or a Danger beside it — 36 px against 38 px at `ButtonSize::Regular`. Primary
//! (and Secondary, which shares its rule) now carries `border:var(--hair) solid transparent`
//! (button.css): the box model matches Danger's own `--hair` border, the paint does not
//! (`background-clip:border-box`'s default already paints `--accent` under a transparent
//! border, so Primary looks exactly as it did). This measures that Primary, Secondary and
//! Danger render the same height side by side, at `ButtonSize::Regular` and at `ButtonSize::Mini`.

use dioxus::prelude::*;
use ds::{Appearance, Button, ButtonSize, ButtonVariant, Ds, Material};
use ds_native::{Harness, Viewport};
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
                    Button { variant: ButtonVariant::Primary, label: "Shut Down", onclick: move |_| {} }
                }
                span { class: "secondary-regular",
                    Button { variant: ButtonVariant::Secondary, label: "Cancel", onclick: move |_| {} }
                }
                span { class: "danger-regular",
                    Button { variant: ButtonVariant::Danger, size: ButtonSize::Regular, label: "Restart", onclick: move |_| {} }
                }
                span { class: "primary-mini",
                    Button { variant: ButtonVariant::Primary, size: ButtonSize::Mini, label: "Shut Down", onclick: move |_| {} }
                }
                span { class: "secondary-mini",
                    Button { variant: ButtonVariant::Secondary, size: ButtonSize::Mini, label: "Cancel", onclick: move |_| {} }
                }
                span { class: "danger-mini",
                    // No size: Danger's own size is Mini (design/04-COMPONENTS.md section 1).
                    Button { variant: ButtonVariant::Danger, label: "Restart", onclick: move |_| {} }
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
