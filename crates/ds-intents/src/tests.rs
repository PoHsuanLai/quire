use crate::{
    ChipKind, FieldMode, HeardEndMark, HeardMark, InputLevel, Removal, SummonAnswerMark,
    SummonOriginMark,
};
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
    round_trips::<SummonOriginMark>();
}

#[test]
fn what_a_field_hears_never_shows_the_words_in_debug() {
    let words = "lisbon receipts";
    let marks = [
        HeardMark::Tail(words.into()),
        HeardMark::Committed(words.into()),
        HeardMark::Ended(HeardEndMark::Send(words.into())),
    ];
    for mark in marks {
        let shown = format!("{mark:?}");
        assert!(!shown.contains("lisbon"), "{shown}");
    }
    assert_eq!(
        format!("{:?}", HeardMark::Level(InputLevel(420))),
        "Level(420)"
    );
    assert_eq!(
        format!("{:?}", HeardMark::Ended(HeardEndMark::Nothing)),
        "Ended(Nothing)"
    );
}
