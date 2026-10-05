//! A `FieldRow` with several controls: they sit on one line while they fit and the last wraps
//! under the others when the row is too narrow.

use dioxus::prelude::*;
use ds::components::fields::field_row::{FieldRow, RowLayout};
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 240,
    scale_percent: 100,
};

/// The row at `width`, with three push buttons.
fn row(width: u32) -> impl Fn() -> Element {
    move || {
        rsx! {
            Ds { appearance: Appearance::default(), material: Material::Window,
                div { style: "width:{width}px",
                    FieldRow { label: "Protection", layout: RowLayout::Form,
                        Button { label: "Sign the message", size: ControlSize::Regular, onclick: |_| {} }
                        Button { label: "Encrypt the message", size: ControlSize::Regular, onclick: |_| {} }
                        Button { label: "More options", size: ControlSize::Regular, onclick: |_| {} }
                    }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn Wide() -> Element {
    row(700)()
}

#[allow(non_snake_case)]
fn Narrow() -> Element {
    row(330)()
}

fn tops(app: fn() -> Element) -> Vec<f32> {
    let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(100));
    (1..=3)
        .map(|n| {
            harness
                .rect(&format!(
                    ".ds-field-row-control > .ds-button:nth-child({n})"
                ))
                .expect("a control")
                .origin
                .y
                .0
        })
        .collect()
}

#[test]
fn the_controls_share_a_line_when_they_fit_and_wrap_when_they_do_not() {
    let wide = tops(Wide);
    assert!(
        wide.iter().all(|y| (y - wide[0]).abs() < 1.0),
        "one line at 700 px: {wide:?}"
    );
    let narrow = tops(Narrow);
    assert!(
        narrow[2] > narrow[0] + 10.0,
        "the last control wraps under the first at 330 px: {narrow:?}"
    );
}
