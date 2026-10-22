//! The thirteen colours a person can give a tag (`--tag-red` … `--tag-gray`): Apple's system
//! hues in their order, light and dark, resolved like the accents.
//!
//! A tag's colour is a choice the person made, so it stays one of these thirteen and never a
//! value of the app's own; an app stores the hue and paints `var(--tag-<hue>)`.

use crate::appearance::system::Contrast;
use crate::tokens::system_hue::SystemHue;
use crate::tokens::token::{CssValue, Token, TokenScope};
use ds_core::word::Word;

/// One tag colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "tag-", kind = fixed, css = tag_css)]
#[non_exhaustive]
pub enum TagColour {
    /// `--tag-red`.
    Red,
    /// `--tag-orange`.
    Orange,
    /// `--tag-yellow`.
    Yellow,
    /// `--tag-green`.
    Green,
    /// `--tag-mint`.
    Mint,
    /// `--tag-teal`.
    Teal,
    /// `--tag-cyan`.
    Cyan,
    /// `--tag-blue`.
    Blue,
    /// `--tag-indigo`.
    Indigo,
    /// `--tag-purple`.
    Purple,
    /// `--tag-pink`.
    Pink,
    /// `--tag-brown`.
    Brown,
    /// `--tag-gray`: Finder's Gray tag.
    Gray,
}

impl TagColour {
    /// The system hue this tag colour is, by position: both lists are Apple's order.
    pub fn hue(self) -> SystemHue {
        let at = Self::ALL.iter().position(|tag| *tag == self).unwrap_or(0);
        SystemHue::ALL[at]
    }
}

/// A tag colour as the stylesheet writes it: its hue in the scope's scheme.
fn tag_css(token: TagColour, scope: TokenScope) -> CssValue {
    CssValue::computed(token.hue().value(scope.scheme, Contrast::Normal).css())
}

#[cfg(test)]
mod tests {
    use super::TagColour;
    use ds_core::word::Word;

    #[test]
    fn each_tag_colour_is_the_system_hue_of_the_same_name() {
        for tag in TagColour::ALL.iter().copied() {
            assert_eq!(tag.hue().slug(), tag.slug());
            assert_eq!(tag.var().as_str(), format!("--tag-{}", tag.slug()));
        }
    }
}
