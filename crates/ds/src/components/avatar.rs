//! Avatar: a person or account as a coloured disc with one letter (design/04-COMPONENTS.md
//! section 11).
//!
//! The component doc writes `Person(Avatar)` and `Tile::Avatar(Avatar)` for "an avatar's
//! description"; `#[component] fn Avatar` owns that name in both namespaces, so the description
//! is [`AvatarFace`].
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use crate::tokens::Colour;
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

/// A person's hue, hashed from their address: `h = (h x 31 + code) mod 360`, drawn
/// `hsl(h, 38%, 42%)` and converted to hex in Rust (O-7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PersonHue(pub u16);

impl PersonHue {
    /// The hue for `address`.
    pub fn of(address: &str) -> Self {
        todo!()
    }
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

/// A coloured disc with one letter.
#[component]
pub fn Avatar(
    initial: char,
    size: AvatarSize,
    tone: AvatarTone,
    #[props(default)] shape: AvatarShape,
) -> Element {
    todo!()
}
