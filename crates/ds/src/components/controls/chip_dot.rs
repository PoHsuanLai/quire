//! The colour dot of a `Chip`: one of the person's tag colours, or any CSS colour.

use ds_core::word::Word;
use ds_style::tokens::tag_colour::TagColour;

/// What a chip's dot is painted with.
#[derive(Debug, Clone, PartialEq)]
pub enum ChipDot {
    /// A tag colour the person chose (Finder's tags): painted `var(--tag-<hue>)`, so it follows
    /// the scheme and the contrast preference. A dot-only chip is named by the colour's word.
    Tag(TagColour),
    /// Any CSS colour.
    Colour(String),
}

impl ChipDot {
    /// The value of the custom property `--chip-dot`.
    pub(super) fn paint(&self) -> String {
        match self {
            ChipDot::Tag(tag) => format!("var({})", tag.var().as_str()),
            ChipDot::Colour(css) => css.clone(),
        }
    }

    /// What names a chip that has this dot and no text: the tag colour's word ("Red"); a bare
    /// CSS colour has none.
    pub(super) fn name(&self) -> Option<&'static str> {
        match self {
            ChipDot::Tag(tag) => Some(tag.label()),
            ChipDot::Colour(_) => None,
        }
    }
}
