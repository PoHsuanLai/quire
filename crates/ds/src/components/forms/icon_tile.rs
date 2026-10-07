//! IconTile: the 28 px rounded tile that leads a grouped row, a colour with a white glyph on it
//! (System Settings' General, Wi-Fi and Sound tiles; design/34-MODERN-LOOK.md sections 2.3 and
//! 3.5). Its sibling is the circular avatar a row of people leads with: the tile's other face
//! is an [`AvatarFace`], drawn by `Avatar` itself, so the two sit in the same leading slot.
//!
//! The ground is the caller's colour under a soft vertical gradient from a lighter step of the
//! same hue (design/08: matte, per-app gradient, no outline); the glyph is the on-hue ink.
//! Markup: `span.ds-icon-tile` (`style="--tile-bg;--tile-top"`) of a `Glyph`, or an `Avatar`.

use crate::components::content::avatar::{AvatarFace, face};
use dioxus::prelude::*;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};
use ds_style::tokens::hex::Hex;

/// What an icon tile is, as data: a row's leading element holds one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileFace {
    /// A rounded tile: `icon` in white on `colour`.
    Glyph(Icon, Hex),
    /// A person or account: the circular avatar.
    Avatar(AvatarFace),
}

impl TileFace {
    /// The row's `data-leading` word: a tile is its own shape, an avatar the avatar's.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            TileFace::Glyph(..) => "tile",
            TileFace::Avatar(_) => "avatar",
        }
    }
}

/// A tile face, drawn.
pub fn tile(tile: TileFace) -> Element {
    match tile {
        TileFace::Glyph(icon, colour) => rsx! {
            IconTile { icon, colour }
        },
        TileFace::Avatar(avatar) => face(avatar),
    }
}

/// How far the gradient's top is mixed toward white, in 1/100 of the way.
const TOP_MIX: u16 = 16;

/// `hex` mixed [`TOP_MIX`] percent toward white: the top of a tile's gradient. A mix in sRGB
/// stays in gamut for every colour, where a lift in lightness at fixed chroma clips a saturated
/// red.
fn lifted(hex: Hex) -> Hex {
    let Hex([r, g, b]) = hex;
    let toward = |channel: u8| {
        let channel = u16::from(channel);
        // At most 255: the mix of a channel with 255.
        u8::try_from((channel * (100 - TOP_MIX) + 255 * TOP_MIX) / 100).unwrap_or(u8::MAX)
    };
    Hex([toward(r), toward(g), toward(b)])
}

/// The rounded tile: `icon` in white on `colour`.
#[component]
pub fn IconTile(icon: Icon, colour: Hex) -> Element {
    rsx! {
        span { class: "ds-icon-tile", style: "--tile-bg:{colour.css()};--tile-top:{lifted(colour).css()}",
            Glyph { icon, size: IconSize::Compact }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::lifted;
    use ds_core::colour::{oklab::Oklab, srgb::Srgb};
    use ds_style::tokens::hex::Hex;

    #[test]
    fn the_top_of_the_gradient_is_lighter_and_the_same_hue() {
        const CASES: &[(&str, Hex)] = &[
            ("blue", Hex([0x0a, 0x84, 0xff])),
            ("red", Hex([0xff, 0x3b, 0x30])),
            ("grey", Hex([0x8e, 0x8e, 0x93])),
        ];
        for (name, hex) in CASES {
            let base = Oklab::from(Srgb::from(*hex));
            let top = Oklab::from(Srgb::from(lifted(*hex)));
            assert!(top.l > base.l + 0.02, "{name}: {} vs {}", top.l, base.l);
            let hue = |c: Oklab| c.b.atan2(c.a);
            assert!((hue(top) - hue(base)).abs() < 0.12, "{name}: hue moved");
        }
    }
}
