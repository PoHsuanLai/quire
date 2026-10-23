//! Whether the person's styles are drawn at all: `appearance.custom_styles`, "Use my styles"
//! (design `customization/DESIGN.md` section 7.4).

use ds_core::word::Word;
use serde::{Deserialize, Serialize};

/// Whether the look and the person's two stylesheets are drawn. `Off` is the way back from a
/// style that broke a surface: `Ds` then draws none of the three. The words are `use` and `off`,
/// which Settings draws as a switch (one word is `off`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Word)]
#[serde(rename_all = "snake_case")]
pub enum CustomStyles {
    /// Draw the look and the person's sheets.
    #[default]
    Use,
    /// Draw none of them.
    Off,
}
