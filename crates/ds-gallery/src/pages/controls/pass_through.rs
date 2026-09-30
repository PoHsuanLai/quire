//! The Controls page's "more" glyphs, and buttons carrying a consumer's `data-*` and class.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::controls::button_model::{Bezel, ImagePosition};
use ds::components::controls::press::Propagation;
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::{DataAttr, DataName, ExtraClass};
use ds::style::icon::render::Glyph;
use ds::style::tokens::control_size::ControlSize;

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
                            Button { bezel: Bezel::Toolbar, image: ImagePosition::Only, icon, label: "More actions", onclick: |_| {} }
                            Button { bezel: Bezel::Toolbar, size: ControlSize::Large, image: ImagePosition::Only, icon, label: "More actions", onclick: |_| {} }
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
        Section { title: "Button: the consumer's data and class", note: "data: vec![DataAttr::new(DataName::parse(\"folder\")?, path)] writes data-folder on the button itself, and extra_class: ExtraClass::parse(\"g-reveal\")? appends the consumer's class after quire's (here the gallery's own reveal: faint until the row is hovered). A ds- class or name, or data-variant, is refused when it is built.",
            div { class: "g-row g-row-top",
                Specimen { name: "data-folder", code: "data-folder=\"INBOX/Receipts\"",
                    div { class: "g-row",
                        Button { common: Common { data: folder("INBOX/Receipts"), ..Common::default() }, bezel: Bezel::Toolbar, label: "Receipts", onclick: |_| {} }
                        Button { common: Common { data: folder("INBOX/Receipts"), ..Common::default() }, bezel: Bezel::Toolbar, image: ImagePosition::Only, icon: Icon::Ellipsis, label: "Actions for Receipts", propagation: Propagation::Stop, onclick: |_| {} }
                    }
                }
                Specimen { name: "extra_class", code: "class=\"ds-button g-reveal\"",
                    div { class: "g-row g-reveal-row",
                        span { "Hover this row" }
                        Button { common: Common { extra_class: reveal.clone(), ..Common::default() }, size: ControlSize::Mini, label: "Reply", onclick: |_| {} }
                        Button { common: Common { extra_class: reveal, ..Common::default() }, bezel: Bezel::Toolbar, size: ControlSize::Large, image: ImagePosition::Only, icon: Icon::EllipsisVertical, label: "More", onclick: |_| {} }
                    }
                }
            }
        }
    }
}
