//! Blur intent and blur availability (design/03-COLOR.md section 17.1).

use ds_core::word::Word;
use serde::{Deserialize, Serialize};

/// Whether a material asks the compositor to blur what is behind it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Blur {
    /// Painted opaque; nothing behind shows.
    None,
    /// Translucent over a compositor blur of what is behind.
    Behind,
}

/// Whether the host could get a compositor blur for this surface.
///
/// `data-blur="on"` paints `--m-tint` (translucent); `data-blur="off"` paints
/// `--m-tint-solid` (alpha at least .94). The default is the safe one: solid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default, Word)]
#[serde(rename_all = "snake_case")]
pub enum BlurState {
    /// The compositor blurs behind the surface.
    #[word(slug = "on")]
    Available,
    /// No blur: paint the solid tint.
    #[default]
    #[word(slug = "off")]
    Unavailable,
}
