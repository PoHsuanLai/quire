//! Badge: Alert and Quiet, numbers and a dot, at each size.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::components::controls::badge::{Badge, BadgeContent, BadgeTone};
use ds::prelude::*;
use ds_style::tokens::control_size::ControlSize;

/// The Badge section.
#[component]
pub fn BadgeSection() -> Element {
    let contents = [
        BadgeContent::Number(1),
        BadgeContent::Number(42),
        BadgeContent::Number(999),
        BadgeContent::Number(1200),
        BadgeContent::Dot,
    ];
    rsx! {
        Section { title: "Badge", note: "The Dock and app badge: Alert is the red capsule, Quiet the neutral one; more than 999 reads 999+, zero draws nothing (the last cell), and a dot has no number.",
            for tone in BadgeTone::ALL.iter().copied() {
                for size in [ControlSize::Mini, ControlSize::Small, ControlSize::Regular] {
                    div { class: "g-row",
                        span { class: "g-name g-type-name", "{tone.slug()} {size.slug()}" }
                        for content in contents {
                            Badge { content, tone, size }
                        }
                        Badge { content: BadgeContent::Number(0), tone, size }
                    }
                }
            }
        }
    }
}
