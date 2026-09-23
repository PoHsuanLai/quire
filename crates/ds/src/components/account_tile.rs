//! AccountTile: an account in this Space, as a square tile on the frame
//! (design/04-COMPONENTS.md section 27).

use crate::components::avatar::{Avatar, AvatarSize, AvatarTone};
use crate::components::count::{Count, CountPlace};
use crate::components::provider_mark::{MarkSize, MarkStyle, Provider, ProviderMark};
use crate::components::vocab::Switch;
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use crate::tokens::{Colour, Hex};
use dioxus::prelude::*;

/// Whose tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccountFace {
    /// Every account: the inbox glyph.
    All,
    /// One account.
    One {
        /// Its letter.
        initial: char,
        /// Its colour.
        colour: Colour,
        /// Its provider's mark.
        provider: Provider,
    },
}

/// How much of its chroma an unpressed tile's avatar keeps: S's `saturate(.55)`, computed in
/// Rust because Blitz paints no `filter` (O-26).
const UNPRESSED_CHROMA: f64 = 0.55;

/// The avatar colour a tile shows: the account's own when pressed, desaturated when not.
fn tile_colour(colour: Colour, pressed: Switch) -> Colour {
    match (pressed, colour) {
        (Switch::On, colour) => colour,
        (Switch::Off, Colour::Solid(hex)) => Colour::Solid(desaturated(hex)),
        (Switch::Off, Colour::Alpha(hex, alpha)) => Colour::Alpha(desaturated(hex), alpha),
    }
}

/// `hex` with its OKLCH chroma scaled by [`UNPRESSED_CHROMA`], lightness and hue kept.
fn desaturated(hex: Hex) -> Hex {
    let [l, a, b] = oklab(hex);
    from_oklab([l, a * UNPRESSED_CHROMA, b * UNPRESSED_CHROMA])
}

/// sRGB to OKLab (Björn Ottosson's matrices, the inverse of `space::palette`'s).
fn oklab(Hex(rgb): Hex) -> [f64; 3] {
    let [r, g, b] = rgb.map(|channel| linear(f64::from(channel) / 255.0));
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}

/// OKLab back to 8-bit sRGB, clamped into gamut.
fn from_oklab([lightness, a, b]: [f64; 3]) -> Hex {
    let l = (lightness + 0.3963377774 * a + 0.2158037573 * b).powi(3);
    let m = (lightness - 0.1055613458 * a - 0.0638541728 * b).powi(3);
    let s = (lightness - 0.0894841775 * a - 1.2914855480 * b).powi(3);
    let rgb = [
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    ];
    // Rounded and clamped to 0..=255 first, so the cast cannot truncate.
    Hex(rgb.map(|channel| (encode(channel) * 255.0).round().clamp(0.0, 255.0) as u8))
}

/// sRGB transfer, decoding.
fn linear(channel: f64) -> f64 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

/// sRGB transfer, encoding.
fn encode(channel: f64) -> f64 {
    let channel = channel.clamp(0.0, 1.0);
    if channel <= 0.0031308 {
        12.92 * channel
    } else {
        1.055 * channel.powf(1.0 / 2.4) - 0.055
    }
}

/// The tile's `aria-label`. The face carries no address, so a single account is named by its
/// letter and provider. TODO(O-16 / section 27): the doc labels the tile with the account's
/// address (`S:1236`), which `AccountFace` does not hold.
fn label(account: AccountFace) -> String {
    match account {
        AccountFace::All => "All accounts".to_string(),
        AccountFace::One {
            initial, provider, ..
        } => format!("{initial}, {} account", provider.name()),
    }
}

/// An account tile: a Pin IconButton holding the account's avatar, its provider mark and its
/// unread count. Pressed when it is the list's filter, or the Space's only account.
#[component]
pub fn AccountTile(
    account: AccountFace,
    pressed: Switch,
    unread: u32,
    onclick: EventHandler<()>,
) -> Element {
    let face = match account {
        AccountFace::All => rsx! {
            span { class: "ds-avatar", "data-size": "28", "data-tone": "all",
                Glyph { icon: Icon::Inbox, size: IconSize::Large }
            }
        },
        AccountFace::One {
            initial,
            colour,
            provider,
        } => rsx! {
            Avatar {
                initial,
                size: AvatarSize::Size28,
                tone: AvatarTone::Account(tile_colour(colour, pressed)),
            }
            ProviderMark { provider, size: MarkSize::Tile, style: MarkStyle::Letter }
        },
    };
    rsx! {
        button {
            r#type: "button",
            class: "ds-icon-button ds-account-tile",
            "data-variant": "pin",
            "aria-pressed": pressed.aria(),
            "aria-label": label(account),
            onclick: move |_| onclick.call(()),
            {face}
            Count { value: unread, place: CountPlace::Tile }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{desaturated, oklab, tile_colour};
    use crate::components::vocab::Switch;
    use crate::tokens::{Colour, Hex};

    #[test]
    fn a_grey_stays_grey_and_a_hue_loses_chroma() {
        let grey = Hex([0x80, 0x80, 0x80]);
        assert_eq!(desaturated(grey), grey);
        let violet = Hex([0x5b, 0x4f, 0xc4]);
        let [l0, a0, b0] = oklab(violet);
        let [l1, a1, b1] = oklab(desaturated(violet));
        let chroma = |a: f64, b: f64| a.hypot(b);
        assert!((l1 - l0).abs() < 0.01, "lightness kept: {l0} {l1}");
        let kept = chroma(a1, b1) / chroma(a0, b0);
        assert!((kept - 0.55).abs() < 0.03, "chroma kept {kept}");
    }

    #[test]
    fn only_an_unpressed_tile_is_desaturated() {
        let colour = Colour::Solid(Hex([0x5b, 0x4f, 0xc4]));
        assert_eq!(tile_colour(colour, Switch::On), colour);
        assert_ne!(tile_colour(colour, Switch::Off), colour);
    }
}
