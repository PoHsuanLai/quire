//! The Controls page's tinted plates (sill FINDINGS Q72): a neutral plate under
//! `IconView { plate_tint }` in the Work and the Home Space's Monochrome tint and in Muted, beside
//! the untinted plate, in the light scheme and the dark. The third-party raster on each is
//! re-coloured by `ds::icon::retint` with the same style and tint, so plate and icon read as one
//! hue; the glyph takes the tinted ink.

use super::app_icons::{APPS, app_icon_in};
use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::icon::{IconStyle, Tint};
use ds::{
    Icon, IconSize, IconSource, IconView, Material, PRESETS, PlateFamily, PlateTint, Scheme,
    Surface,
};

/// One column: its caption and the style and tint it draws in.
struct Column {
    name: &'static str,
    style: IconStyle,
    tint: Tint,
}

fn columns() -> [Column; 4] {
    [
        Column {
            name: "Colour (no tint)",
            style: IconStyle::Colour,
            tint: Tint::NEUTRAL,
        },
        Column {
            name: "Monochrome, Work",
            style: IconStyle::Monochrome,
            tint: Tint::space(PRESETS[0].dots),
        },
        Column {
            name: "Monochrome, Home",
            style: IconStyle::Monochrome,
            tint: Tint::space(PRESETS[1].dots),
        },
        Column {
            name: "Muted",
            style: IconStyle::Muted,
            tint: Tint::NEUTRAL,
        },
    ]
}

/// The tinted plates section.
#[component]
pub fn PlateTints() -> Element {
    rsx! {
        Section { title: "Tinted plates", note: "IconView {{ plate: Some(PlateFamily::Neutral), plate_tint: PlateTint::of(style, tint) }}: the plate's stops and ink re-coloured by retint's rule for both schemes, the root's data-theme picking one. Each tile's raster went through retint with the same pair; the glyph tile is the symbolic fallback.",
            for scheme in Scheme::ALL {
                Surface { material: Material::Popover, theme: Some(scheme),
                    div { class: "g-row g-row-top",
                        for column in columns() {
                            Specimen { name: format!("{} ({})", column.name, scheme.slug()),
                                div { class: "g-row",
                                    IconView {
                                        source: app_icon_in(APPS[0].1, IconSize::Tile48, column.style, column.tint)
                                            .unwrap_or(IconSource::Glyph(Icon::Window)),
                                        size: IconSize::Tile48,
                                        plate: Some(PlateFamily::Neutral),
                                        plate_tint: PlateTint::of(column.style, column.tint),
                                    }
                                    IconView {
                                        source: IconSource::Glyph(Icon::Window),
                                        size: IconSize::Tile48,
                                        plate: Some(PlateFamily::Neutral),
                                        plate_tint: PlateTint::of(column.style, column.tint),
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
