//! Type: the four faces with the weights they ship, and every step of the size ramp drawn in
//! each face.

use super::Section;
use dioxus::prelude::*;
use ds::{FACES, Face, FaceStyle, Family, FontSize};

/// The faces, in the order the stylesheet names them.
const FAMILIES: [Family; 4] = Family::ALL;

/// The pangram each face is shown with.
const PANGRAM: &str = "Sphinx of black quartz, judge my vow — 0123456789";

/// The type page.
#[component]
pub fn TypePage() -> Element {
    rsx! {
        Section { title: "Faces", note: "Each face at --fs-display, regular and bold, with the files ds::FACES ships for it.",
            for family in FAMILIES {
                div { class: "g-col",
                    div { style: "font-family:{family.var().reference()};font-size:var(--fs-display);font-weight:400", "{PANGRAM}" }
                    div { style: "font-family:{family.var().reference()};font-size:var(--fs-display);font-weight:700", "{PANGRAM}" }
                    span { class: "g-code", "{family.var().as_str()}: {family.stack()}" }
                    span { class: "g-code", "{files(family)}" }
                }
            }
        }
        Section { title: "Size ramp", note: "Every --fs step in the ui, display and data faces.",
            div { class: "g-table g-cols5",
                span { class: "g-head", "token" }
                span { class: "g-head", "size" }
                span { class: "g-head", "ui" }
                span { class: "g-head", "display" }
                span { class: "g-head", "data" }
                for size in FontSize::ALL {
                    span { class: "g-code", "{size.var().as_str()}" }
                    span { class: "g-code", "{size.css()}" }
                    for family in [Family::Ui, Family::Display, Family::Data] {
                        span {
                            class: "ds-truncate",
                            style: "font-family:{family.var().reference()};font-size:{size.var().reference()}",
                            "Quire 0123"
                        }
                    }
                }
            }
        }
        Section { title: "Utilities", note: "The text classes any component may add.",
            div { class: "g-col",
                span { class: "ds-eyebrow", "ds-eyebrow: the eyebrow over a section" }
                span { "ds-mono: " span { class: "ds-mono", "a1b2c3d4 · 2026-09-24T09:41Z" } }
                span { class: "ds-tabular", "ds-tabular: 1111 · 8888 · 1234" }
                h1 { "An h1 in the display face" }
                h2 { "An h2 in the display face" }
                h3 { "An h3 in the display face" }
            }
        }
    }
}

/// The files a family ships: `normal 200-800 latin, …`.
fn files(family: Family) -> String {
    FACES
        .iter()
        .filter(|face| face.family == family)
        .map(describe)
        .collect::<Vec<_>>()
        .join(" · ")
}

fn describe(face: &Face) -> String {
    let style = match face.style {
        FaceStyle::Normal => "normal",
        FaceStyle::Italic => "italic",
    };
    let weight = if face.weight.min == face.weight.max {
        face.weight.min.to_string()
    } else {
        format!("{}-{}", face.weight.min, face.weight.max)
    };
    let subset = match face.subset {
        ds::Subset::Latin => "latin",
        ds::Subset::LatinExt => "latin-ext",
    };
    format!("{style} {weight} {subset}")
}
