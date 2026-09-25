//! The Controls page's mailo gaps 6 specimens: the "more" glyphs. Split from `controls.rs` to
//! keep that page under its size.

use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::{Glyph, Icon, IconButton, IconButtonVariant, IconSize};

/// The two ellipses at the glyph sizes a row and a header use, and on the buttons that open a
/// row's menu.
#[component]
pub fn MoreGlyphs() -> Element {
    rsx! {
        Section { title: "Glyphs: more", note: "Icon::Ellipsis and Icon::EllipsisVertical (Lucide ellipsis, ellipsis-vertical): a row's or a header's overflow menu, drawn on the glyph grid instead of a typed ⋯.",
            div { class: "g-row g-row-top",
                for (icon, name) in [(Icon::Ellipsis, "Ellipsis"), (Icon::EllipsisVertical, "EllipsisVertical")] {
                    Specimen { name,
                        div { class: "g-row",
                            Glyph { icon, size: IconSize::Compact }
                            Glyph { icon, size: IconSize::Base }
                            Glyph { icon, size: IconSize::Large }
                            IconButton { variant: IconButtonVariant::Strip, icon, label: "More actions", onclick: |_| {} }
                            IconButton { variant: IconButtonVariant::Tool, icon, label: "More actions", onclick: |_| {} }
                        }
                    }
                }
            }
        }
    }
}
