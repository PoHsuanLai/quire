//! Styling: every public part a stylesheet may name, on a live example, generated from
//! `ds::selectors` so the page cannot drift from the table (DESIGN.md section 7.3).

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::selectors::{AXES, COMPONENTS, ComponentSelectors, RENAMES, SURFACE, SURFACE_ATTRIBUTE};

/// A frame that keeps an absolutely placed root (a popover, a sheet) inside its own box.
const FRAME: &str = "position:relative;height:96px;overflow:hidden";

/// The styling page.
#[component]
pub fn StylingPage() -> Element {
    rsx! {
        Section {
            title: format!("Public surface, version {SURFACE}"),
            note: "Everything below is what a stylesheet may rely on. The version goes up when a public name is renamed or removed, and not when one is added. Anything not listed is internal and may change in any update.",
            div { class: "g-col",
                p { class: "g-note", "[{SURFACE_ATTRIBUTE}=<name>] is on every surface root; every token variable (--<prefix><slug>) is public." }
                p { class: "g-code", "surface {SURFACE}" }
            }
        }
        Section {
            title: "Components",
            note: "Each example is the component's root class with its parts inside it, drawn by the real stylesheet; the parts are named in the code line.",
            div { class: "g-grid3",
                for component in COMPONENTS {
                    PartExample { component: *component }
                }
            }
        }
        Section { title: "Renamed this release", note: "A renamed part keeps its old class for one release. Nothing is listed when nothing was renamed.",
            div { class: "g-col",
                for rename in RENAMES {
                    p { class: "g-code", ".{rename.old_class()} is now .{rename.new_class()}" }
                }
            }
        }
        Section { title: "Attributes", note: "What a stylesheet may select on, on any component that writes it.",
            div { class: "g-col",
                for axis in AXES {
                    Specimen { name: axis.attribute, code: axis.meaning.to_string(), span {} }
                }
            }
        }
    }
}

/// One component: its root and parts as live elements, named by their selectors.
#[component]
fn PartExample(component: ComponentSelectors) -> Element {
    let selectors = component.part_selectors();
    let code = std::iter::once(component.root_selector())
        .chain(selectors.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ");
    rsx! {
        Specimen { name: component.component, code,
            div { style: FRAME,
                div { class: component.root,
                    for part in component.parts {
                        span { class: "{component.root}-{part}" }
                    }
                }
            }
        }
    }
}
