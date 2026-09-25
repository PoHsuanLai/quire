//! The Controls page's mailo gaps 6 specimens: the "more" glyphs, and buttons carrying a
//! consumer's `data-*` and class. Split from `controls.rs` to
//! keep that page under its size.

use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::{
    Button, ButtonVariant, DataAttr, DataName, ExtraClass, Glyph, Icon, IconButton,
    IconButtonVariant, IconSize, Propagation,
};

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

/// `data-folder="<path>"`, or nothing if the name were refused (it is not: `folder` is a plain
/// consumer word).
fn folder(path: &str) -> Vec<DataAttr> {
    DataName::parse("folder")
        .map(|name| vec![DataAttr::new(name, path)])
        .unwrap_or_default()
}

/// Buttons with the consumer's own attribute and class on the element itself: mailo's folder
/// name and its ⋯ with no wrapping span.
#[component]
pub fn PassThrough() -> Element {
    let reveal = ExtraClass::parse("g-reveal").ok();
    rsx! {
        Section { title: "Button and IconButton: the consumer's data and class", note: "data: vec![DataAttr::new(DataName::parse(\"folder\")?, path)] writes data-folder on the button itself, and extra_class: ExtraClass::parse(\"g-reveal\")? appends the consumer's class after quire's (here the gallery's own reveal: faint until the row is hovered). A ds- class or name, or data-variant, is refused when it is built.",
            div { class: "g-row g-row-top",
                Specimen { name: "data-folder", code: "data-folder=\"INBOX/Receipts\"",
                    div { class: "g-row",
                        Button { variant: ButtonVariant::Frame, label: "Receipts", data: folder("INBOX/Receipts"), onclick: |_| {} }
                        IconButton { variant: IconButtonVariant::Strip, icon: Icon::Ellipsis, label: "Actions for Receipts", data: folder("INBOX/Receipts"), propagation: Propagation::Stop, onclick: |_| {} }
                    }
                }
                Specimen { name: "extra_class", code: "class=\"ds-button g-reveal\"",
                    div { class: "g-row g-reveal-row",
                        span { "Hover this row" }
                        Button { variant: ButtonVariant::Mini, label: "Reply", extra_class: reveal.clone(), onclick: |_| {} }
                        IconButton { variant: IconButtonVariant::Tool, icon: Icon::EllipsisVertical, label: "More", extra_class: reveal, onclick: |_| {} }
                    }
                }
            }
        }
    }
}
