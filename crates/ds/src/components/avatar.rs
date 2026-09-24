//! Avatar: a person or account as a coloured disc with one letter (design/04-COMPONENTS.md
//! section 11).
//!
//! The component doc writes `Person(Avatar)` and `Tile::Avatar(Avatar)` for "an avatar's
//! description"; `#[component] fn Avatar` owns that name in both namespaces, so the description
//! is [`AvatarFace`].

use crate::tokens::{Colour, Hex};
use dioxus::prelude::*;

/// An avatar's size, in logical pixels (`data-size`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AvatarSize {
    /// 16: sidebar favicon.
    Size16,
    /// 18: person chip.
    Size18,
    /// 20: event attendees, slim menu tile.
    Size20,
    /// 22: hover-card message rows.
    Size22,
    /// 26: pinned tile.
    Size26,
    /// 28: reader meta, account tile.
    Size28,
    /// 30: the sync halo's account ring.
    Size30,
    /// 34: hover-card person header, rich menu tile.
    Size34,
}

/// Round, or the favicon's rounded square.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AvatarShape {
    /// A disc.
    #[default]
    Round,
    /// A rounded square.
    Square,
}

/// The hue for a person with no stored colour, hashed from their address or name (design/03-COLOR.md
/// section 13): the same as [`PersonHue::of`], so a consumer never keeps its own hash or hex
/// table. Paint it with `AvatarTone::Person`, or [`PersonHue::colour`] where a component takes
/// a [`Colour`]; the eight stored-colour swatches are [`crate::PersonSwatch`].
pub fn person_hue(name: &str) -> PersonHue {
    PersonHue::of(name)
}

/// A person's hue, hashed from their address: `h = (h x 31 + code) mod 360`, drawn
/// `hsl(h, 38%, 42%)` and converted to hex in Rust (O-7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PersonHue(pub u16);

impl PersonHue {
    /// The hue for `address`.
    pub fn of(address: &str) -> Self {
        // UTF-16 code units, as the prototype's `charCodeAt` walks them (`S:1879`).
        let hue = address
            .encode_utf16()
            .fold(0u32, |h, code| (h * 31 + u32::from(code)) % 360);
        PersonHue(u16::try_from(hue).unwrap_or_default())
    }

    /// The disc colour: `hsl(h, 38%, 42%)` as a hex, so no colour function reaches CSS (O-7).
    pub fn hex(self) -> Hex {
        hsl_to_hex(f64::from(self.0 % 360), 0.38, 0.42)
    }

    /// The disc colour as a [`Colour`], for a component that takes one.
    pub fn colour(self) -> Colour {
        Colour::Solid(self.hex())
    }
}

/// `hsl(h, s, l)` to 8-bit sRGB, the CSS Color 4 conversion.
fn hsl_to_hex(hue: f64, saturation: f64, lightness: f64) -> Hex {
    let a = saturation * lightness.min(1.0 - lightness);
    let channel = |n: f64| {
        let k = (n + hue / 30.0) % 12.0;
        let value = lightness - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0);
        // Rounded and clamped to 0..=255 first, so the cast cannot truncate.
        (value * 255.0).round().clamp(0.0, 255.0) as u8
    };
    Hex([channel(0.0), channel(8.0), channel(4.0)])
}

/// Where an avatar's colours come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AvatarTone {
    /// `--ink` ground, `--paper` letter.
    Ink,
    /// The account's colour, white letter.
    Account(Colour),
    /// The person hash, white letter.
    Person(PersonHue),
    /// `--ink-soft` ground: event attendees.
    Stack,
}

/// An avatar, as data: what a chip, a menu tile or a sidebar item embeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AvatarFace {
    /// The letter.
    pub initial: char,
    /// The size.
    pub size: AvatarSize,
    /// The colours.
    pub tone: AvatarTone,
    /// Round or square.
    pub shape: AvatarShape,
}

impl AvatarSize {
    /// The `data-size` number.
    fn px(self) -> u8 {
        match self {
            AvatarSize::Size16 => 16,
            AvatarSize::Size18 => 18,
            AvatarSize::Size20 => 20,
            AvatarSize::Size22 => 22,
            AvatarSize::Size26 => 26,
            AvatarSize::Size28 => 28,
            AvatarSize::Size30 => 30,
            AvatarSize::Size34 => 34,
        }
    }
}

impl AvatarShape {
    /// The `data-shape` word.
    fn slug(self) -> &'static str {
        match self {
            AvatarShape::Round => "round",
            AvatarShape::Square => "square",
        }
    }
}

impl AvatarTone {
    /// The `data-tone` word: the stack tone also draws the attendee ring and overlap.
    fn slug(self) -> &'static str {
        match self {
            AvatarTone::Ink => "ink",
            AvatarTone::Account(_) => "account",
            AvatarTone::Person(_) => "person",
            AvatarTone::Stack => "stack",
        }
    }

    /// The inline `--av-bg` and `--av-fg`. Token tones name their tokens; the account colour
    /// and the person hash are hexes computed here (O-7, still open: the ground itself is
    /// per-address or per-account, so it cannot be a fixed token). The letter on a coloured
    /// disc is `--on-hue` (`S:116`, `S:1238`; O-3, resolved).
    fn style(self) -> String {
        let (ground, letter) = match self {
            AvatarTone::Ink => ("var(--ink)".to_string(), "var(--paper)"),
            AvatarTone::Stack => ("var(--ink-soft)".to_string(), "var(--paper)"),
            AvatarTone::Account(colour) => (colour.css(), "var(--on-hue)"),
            AvatarTone::Person(hue) => (hue.hex().css(), "var(--on-hue)"),
        };
        format!("--av-bg:{ground};--av-fg:{letter}")
    }
}

/// An avatar's description, drawn.
pub(crate) fn face(face: AvatarFace) -> Element {
    rsx! {
        Avatar {
            initial: face.initial,
            size: face.size,
            tone: face.tone,
            shape: face.shape,
        }
    }
}

/// A coloured disc with one letter.
#[component]
pub fn Avatar(
    initial: char,
    size: AvatarSize,
    tone: AvatarTone,
    #[props(default)] shape: AvatarShape,
) -> Element {
    let px = size.px();
    rsx! {
        span {
            class: "ds-avatar",
            "data-size": "{px}",
            "data-shape": shape.slug(),
            "data-tone": tone.slug(),
            style: tone.style(),
            "{initial}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PersonHue;

    #[test]
    fn the_person_hash_matches_the_prototype() {
        // h = (h * 31 + code) % 360 over "dana": d 100, a 97, n 110, a 97.
        let want = ((((100 * 31 + 97) % 360) * 31 + 110) % 360 * 31 + 97) % 360;
        assert_eq!(PersonHue::of("dana"), PersonHue(want));
        assert_eq!(PersonHue::of(""), PersonHue(0));
    }

    #[test]
    fn person_hue_is_the_hash_and_its_colour_the_disc() {
        assert_eq!(super::person_hue("dana"), PersonHue::of("dana"));
        // "ab": (97 * 31 + 98) % 360 = 225; hsl(225, 38%, 42%) = rgb(66.4, 86.7, 147.8).
        assert_eq!(super::person_hue("ab"), PersonHue(225));
        assert_eq!(super::person_hue("ab").colour().css(), "#425794");
    }

    #[test]
    fn hsl_reaches_hex() {
        // hsl(0, 38%, 42%) = rgb(148.4, 66.4, 66.4); hsl(120, ...) and hsl(240, ...) rotate it.
        const CASES: &[(u16, &str)] = &[(0, "#944242"), (120, "#429442"), (240, "#424294")];
        for (hue, want) in CASES {
            assert_eq!(PersonHue(*hue).hex().css(), *want, "hue {hue}");
        }
    }
}
