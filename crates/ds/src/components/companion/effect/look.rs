//! What each effect looks like and says. One match per question, so a new `EffectMark` is a
//! compile error here and nowhere else in drawing.

use ds_core::vocab::EffectMark;
use ds_style::icon::Icon;

/// The glyph an effect wears. `Execute` is the terminal, so running commands never reads as
/// the bin of `Destructive`.
#[must_use]
pub fn effect_glyph(effect: EffectMark) -> Icon {
    match effect {
        EffectMark::Read => Icon::Search,
        EffectMark::UndoableWrite => Icon::Pen,
        EffectMark::Outbound => Icon::Send,
        EffectMark::Execute => Icon::Terminal,
        EffectMark::Destructive => Icon::Trash,
    }
}

/// The words an effect says, in the card's voice.
#[must_use]
pub fn effect_words(effect: EffectMark) -> &'static str {
    match effect {
        EffectMark::Read => "Reads only",
        EffectMark::UndoableWrite => "Changes something; can be undone",
        EffectMark::Outbound => "Sends something out of this computer",
        EffectMark::Execute => "Runs commands on this computer",
        EffectMark::Destructive => "Deletes something for good",
    }
}

#[cfg(test)]
mod tests {
    use super::{effect_glyph, effect_words};
    use ds_core::vocab::EffectMark;
    use ds_core::word::Word;

    #[test]
    fn every_effect_has_its_own_glyph_and_words() {
        for (index, a) in EffectMark::ALL.iter().enumerate() {
            for b in &EffectMark::ALL[index + 1..] {
                assert_ne!(effect_glyph(*a), effect_glyph(*b), "{a:?} / {b:?}");
                assert_ne!(effect_words(*a), effect_words(*b), "{a:?} / {b:?}");
            }
        }
    }
}
