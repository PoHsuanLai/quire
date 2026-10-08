//! The Controls page's glyphs: every `Icon` at the bar's 22 px, set by set, named by its variant,
//! so a new glyph (the control set) is seen beside the ones it must match.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::companion::effect::view::EffectTag;
use ds::prelude::*;
use ds::style::icon::render::{Glyph, IconPx};
use ds::style::icon::style::GlyphStyle;
use ds_core::vocab::EffectMark;
use ds_core::word::Word;

/// The sets in `Icon::ALL`'s order, each with its heading.
const SETS: [(&str, &[Icon]); 4] = [
    ("mailo", Icon::MAILO),
    ("shell", Icon::SHELL),
    ("actions", Icon::ACTIONS),
    ("control", Icon::CONTROL),
];

/// The sizes the solid forms are checked at (design/08-ICONS.md section 1.4): 12, 14, 16, 20, 24.
const SIZES: [IconSize; 5] = [
    IconSize::Tiny,
    IconSize::Compact,
    IconSize::Base,
    IconSize::Px(IconPx(20)),
    IconSize::Px(IconPx(24)),
];

/// The glyphs the viewer app and the pairs draw, each at every size in [`SIZES`].
const AT_SIZES: [Icon; 24] = [
    Icon::File,
    Icon::Folder,
    Icon::Image,
    Icon::Camera,
    Icon::Plus,
    Icon::Minus,
    Icon::ChevronUp,
    Icon::ChevronDown,
    Icon::ChevronLeft,
    Icon::ChevronRight,
    Icon::Undo,
    Icon::Refresh,
    Icon::Search,
    Icon::Maximize,
    Icon::Columns,
    Icon::Code,
    Icon::Play,
    Icon::Pause,
    Icon::SkipBack,
    Icon::SkipForward,
    Icon::TriangleAlert,
    Icon::Star,
    Icon::Heart,
    Icon::Bell,
];

/// Every glyph, grouped by set.
#[component]
pub fn Glyphs() -> Element {
    rsx! {
        Section { title: "Glyphs", note: "Every Icon, solid (the default), at 22 px (IconSize::Bar), by set: the mailo set, the shell set (ending in the control center's own Switches), the actions, and the control set the control center, the power menu and Now Playing draw (Lucide, ISC).",
            span { class: "g-name", "sizes: 12, 14, 16, 20, 24 px, solid then outline" }
            div { class: "g-grid8",
                for icon in AT_SIZES {
                    Specimen { name: format!("{icon:?}"),
                        span { style: "display:flex; gap:10px; align-items:center",
                            for size in SIZES {
                                Glyph { icon, size }
                            }
                        }
                        span { style: "display:flex; gap:10px; align-items:center",
                            for size in SIZES {
                                Glyph { icon, size, style: GlyphStyle::Outline }
                            }
                        }
                    }
                }
            }
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
            span { class: "g-name", "effect marks: what an action does to the world, in order of severity" }
            div { class: "g-grid8",
                for effect in EffectMark::ALL {
                    Specimen { name: effect.slug(), EffectTag { effect: *effect } }
                }
            }
        }
    }
}
