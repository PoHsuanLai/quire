//! ProviderMark: a letter in the provider's colour on a white chip, never the provider's logo,
//! or the favicon the app supplies (design/04-COMPONENTS.md section 28).

use dioxus::prelude::*;

/// A mail provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provider {
    /// `G` #1A73E8.
    Google,
    /// `M` #0F6CBD.
    Microsoft,
    /// `F` #2A5DB0.
    Fastmail,
    /// `i` #3A82F7.
    ICloud,
    /// `Y` #6001D2.
    Yahoo,
    /// `@` #5D6660.
    Imap,
}

/// Where a mark sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarkSize {
    /// 14, on an account tile.
    Tile,
    /// 11, in a row's via.
    Row,
    /// 13, inline.
    Inline,
}

/// An image the app supplies, as a `data:` URI. quire never fetches.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageSource(pub String);

/// Letter or image.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MarkStyle {
    /// The provider's letter in its colour.
    Letter,
    /// The provider's own favicon.
    Image(ImageSource),
}

/// A provider mark.
#[component]
pub fn ProviderMark(provider: Provider, size: MarkSize, style: MarkStyle) -> Element {
    todo!()
}
