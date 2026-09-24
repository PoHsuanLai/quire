//! The Controls page's dock tiles: app icons at the dock's own sizes (sill FINDINGS Q16), a pill
//! whose corner is a setting (`Surface { radius }`, Q15), and a label the dock's machine shows
//! and hides (`Tooltip { shown }`, Q17).

use super::app_icons::{APPS, app_icon};
use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::{
    Corner, Icon, IconPx, IconSize, IconSource, IconView, Material, Px, Shown, Surface, Tooltip,
    TooltipKind,
};

/// An app's icon at `size`, or the window glyph when it cannot be drawn.
fn icon(hue: [u8; 3], size: IconSize) -> IconSource {
    app_icon(hue, size).unwrap_or(IconSource::Glyph(Icon::Window))
}

/// The dock tiles section.
#[component]
pub fn DockTiles() -> Element {
    let sizes = [
        ("Tile48", IconSize::Tile48),
        ("Px(71)", IconSize::Px(IconPx(71))),
        ("Tile96", IconSize::Tile96),
    ];
    rsx! {
        Section { title: "Dock tiles", note: "Icons at the dock's sizes, drawn at that size rather than scaled up; a Dock pill whose corner comes from dock.pill_radius_px (here 12); a label shown by the caller with no pointer on it (Shown::Visible) and one kept down (Shown::Hidden).",
            div { class: "g-row g-row-top",
                for (name , size) in sizes {
                    Specimen { name: "{name}",
                        div { class: "g-row",
                            IconView { source: icon(APPS[0].1, size) }
                            IconView { source: Icon::Folder.into(), size }
                        }
                    }
                }
                Specimen { name: "Dock pill, radius Px(12), labels by the caller",
                    div { class: "g-dock",
                        Surface { material: Material::Dock, radius: Some(Corner::Px(Px(12.0))),
                            div { class: "g-dock-tiles",
                                for (index , (name , hue)) in APPS.iter().enumerate() {
                                    Tooltip {
                                        kind: TooltipKind::Fly,
                                        text: *name,
                                        shown: Some(if index == 1 { Shown::Visible } else { Shown::Hidden }),
                                        IconView { source: icon(*hue, IconSize::Tile48) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
