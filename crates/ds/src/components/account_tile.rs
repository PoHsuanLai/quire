//! AccountTile: an account in this Space, as a square tile on the frame
//! (design/04-COMPONENTS.md section 27).

use crate::components::provider_mark::Provider;
use crate::components::vocab::Switch;
use crate::tokens::Colour;
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

/// An account tile.
#[component]
pub fn AccountTile(
    account: AccountFace,
    pressed: Switch,
    unread: u32,
    onclick: EventHandler<()>,
) -> Element {
    todo!()
}
