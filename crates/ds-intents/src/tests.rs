use crate::{ChipKind, FieldMode, Removal, SummonAnswerMark};
use ds_core::word::Word;

/// Every member of `T` parses back from its own slug, and no two share one.
fn round_trips<T: Word + std::fmt::Debug>() {
    for &word in T::ALL {
        assert_eq!(T::parse(word.slug()), Some(word), "{word:?}");
    }
    let slugs: std::collections::HashSet<_> = T::ALL.iter().map(|word| word.slug()).collect();
    assert_eq!(slugs.len(), T::ALL.len(), "{:?}", T::ALL);
}

#[test]
fn every_mark_word_parses_back_from_its_slug() {
    round_trips::<ChipKind>();
    round_trips::<FieldMode>();
    round_trips::<Removal>();
    round_trips::<SummonAnswerMark>();
}

#[test]
fn a_field_is_an_input_until_a_summon_takes_it() {
    assert_eq!(FieldMode::default(), FieldMode::Input);
    assert_eq!(Removal::default(), Removal::Removable);
}
