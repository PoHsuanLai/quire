//! Every `Word` enum that is stored in a settings file writes the same word as its attribute:
//! a serde rename and a slug that drifted apart would make a saved choice unreadable.

use crate::shell::battery::device_glyph::Device;
use crate::shell::emoji::id::EmojiId;
use crate::shell::widget::kind::{WidgetHost, WidgetSize};
use crate::style::appearance::{
    accent::Accent,
    material::Material,
    motion::{Motion, MotionLevel},
    peek::PeekMode,
    theme::{Scheme, Theme},
    typeface::Typeface,
};
use crate::style::look::Look;
use crate::style::tokens::label_hue::LabelHue;
use ds_core::testing::word_matches_serde;

#[test]
fn stored_vocabularies_serialise_as_their_slugs() {
    word_matches_serde::<Accent>();
    word_matches_serde::<Device>();
    word_matches_serde::<EmojiId>();
    word_matches_serde::<LabelHue>();
    word_matches_serde::<Look>();
    word_matches_serde::<Material>();
    word_matches_serde::<Motion>();
    word_matches_serde::<MotionLevel>();
    word_matches_serde::<PeekMode>();
    word_matches_serde::<Scheme>();
    word_matches_serde::<Theme>();
    word_matches_serde::<Typeface>();
    word_matches_serde::<WidgetHost>();
    word_matches_serde::<WidgetSize>();
}
