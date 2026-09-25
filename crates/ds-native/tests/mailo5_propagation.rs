//! mailo gaps 5, a press kept at its button, on a real Blitz document: a `Button` or
//! `IconButton` with `propagation: Propagation::Stop` inside a `<summary>` fires its own
//! `onclick` and leaves the `<details>` as it was, while a `Bubble` one (the default) lets the
//! click reach the summary, which toggles its details.

use dioxus::prelude::*;
use ds::{
    Appearance, Button, ButtonVariant, Ds, Icon, IconButton, IconButtonVariant, Material, Point,
    Propagation,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 360,
    scale_percent: 100,
};

/// Three collapsible sections, each with a header action; the page logs every press and every
/// click that reaches a summary.
#[allow(non_snake_case)]
fn Page() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut note = move |entry: &str| log.with_mut(|log| log.push(entry.to_string()));
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            details { class: "stop",
                summary { onclick: move |_| note("summary:stop"),
                    "Quoted message "
                    Button {
                        variant: ButtonVariant::Mini,
                        label: "Remove",
                        propagation: Propagation::Stop,
                        onclick: move |_| note("press:stop"),
                    }
                }
                p { "The quote." }
            }
            details { class: "bubble",
                summary { onclick: move |_| note("summary:bubble"),
                    "Attachments "
                    Button {
                        variant: ButtonVariant::Mini,
                        label: "Add",
                        onclick: move |_| note("press:bubble"),
                    }
                }
                p { "Two files." }
            }
            details { class: "glyph",
                summary { onclick: move |_| note("summary:glyph"),
                    "Signature "
                    IconButton {
                        variant: IconButtonVariant::Tool,
                        icon: Icon::X,
                        label: "Remove signature",
                        propagation: Propagation::Stop,
                        onclick: move |_| note("press:glyph"),
                    }
                }
                p { "Po-Hsuan" }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn centre(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

fn is_open(harness: &Harness, section: &str) -> bool {
    harness
        .attr(&format!("details.{section}"), "open")
        .is_some()
}

#[test]
fn a_stopped_press_leaves_its_details_closed_and_a_bubbling_one_toggles_it() {
    let mut harness = Harness::new(Page, VIEW);
    harness.advance(Duration::from_millis(50));
    for section in ["stop", "bubble", "glyph"] {
        assert!(!is_open(&harness, section), "{section} starts closed");
    }
    let stop = centre(&harness, "details.stop .ds-button");
    harness.click(stop);
    harness.advance(Duration::from_millis(50));
    assert!(!is_open(&harness, "stop"), "a Stop press does not toggle");
    assert_eq!(harness.text_of(".log").as_deref(), Some("press:stop"));

    let glyph = centre(&harness, "details.glyph .ds-icon-button");
    harness.click(glyph);
    harness.advance(Duration::from_millis(50));
    assert!(
        !is_open(&harness, "glyph"),
        "a Stop icon press does not toggle"
    );
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some("press:stop,press:glyph")
    );

    let bubble = centre(&harness, "details.bubble .ds-button");
    harness.click(bubble);
    harness.advance(Duration::from_millis(50));
    assert!(is_open(&harness, "bubble"), "a Bubble press toggles");
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some("press:stop,press:glyph,press:bubble,summary:bubble")
    );
    // The summary text itself still toggles a section whose button stops.
    let header = harness
        .rect("details.stop summary")
        .expect("the summary is laid out");
    harness.click(Point {
        x: header.origin.x + ds::Px(20.0),
        y: header.origin.y + ds::Px(header.size.height.0 / 2.0),
    });
    harness.advance(Duration::from_millis(50));
    assert!(
        is_open(&harness, "stop"),
        "the summary still toggles its details"
    );
}
