//! The Look: which value set the tokens are written in (design/30-CATALOGUE.md part 3).
//!
//! A Look is values only: colours, radii, fonts, grain, materials, shadows and the icon plate.
//! It never changes a component, a markup skeleton, a size, a duration, a motion level or a
//! behaviour, and no stylesheet outside the token blocks may select on it. The token families
//! (`crate::tokens`) hold each Look's values and read the one in their [`TokenScope`];
//! this module is the vocabulary they and `appearance.look` share.
//!
//! [`TokenScope`]: crate::tokens::token::TokenScope

use ds_core::word::Word;
use serde::{Deserialize, Serialize};

/// A whole visual language, as `appearance.look` names it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default, Word)]
#[serde(rename_all = "snake_case")]
pub enum Look {
    /// macOS Sonoma and Sequoia: system neutrals, flat window paper, small radii, no grain.
    #[default]
    Mac,
}
