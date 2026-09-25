//! The Controls page's glyphs: every `Icon` at the bar's 22 px, set by set, named by its variant,
//! so a new glyph (sill FINDINGS Q81's control set) is seen beside the ones it must match.

use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::{Glyph, Icon, IconSize};

/// The sets in `Icon::ALL`'s order, each with its heading.
const SETS: [(&str, &[Icon]); 4] = [
    ("mailo", Icon::MAILO),
    ("shell", Icon::SHELL),
    ("actions", Icon::ACTIONS),
    ("control", Icon::CONTROL),
];

/// Every glyph, grouped by set.
#[component]
pub fn Glyphs() -> Element {
    rsx! {
        Section { title: "Glyphs", note: "Every Icon at 22 px (IconSize::Bar), by set: the mailo set, the shell set, the actions, and the control set the control center, the power menu and Now Playing draw (Lucide, ISC).",
            for (name , set) in SETS {
                span { class: "g-name", "{name}" }
                div { class: "g-grid8",
                    for icon in set.iter().copied() {
                        Specimen { name: format!("{icon:?}"),
                            Glyph { icon, size: IconSize::Bar }
                        }
                    }
                }
            }
        }
    }
}
