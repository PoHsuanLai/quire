//! The confirmation card's closed vocabularies parse back from their own slugs.

use ds_core::word::Word;
use ds_shell::confirm::model::{AlwaysChoice, ArmState, GestureMark, ScopeOffer};

fn round_trips<T: Word + std::fmt::Debug>() {
    for &word in T::ALL {
        assert_eq!(T::parse(word.slug()), Some(word), "{word:?}");
    }
    let slugs: std::collections::HashSet<_> = T::ALL.iter().map(|word| word.slug()).collect();
    assert_eq!(slugs.len(), T::ALL.len(), "{:?}", T::ALL);
}

#[test]
fn every_confirm_word_parses_back_from_its_slug() {
    round_trips::<AlwaysChoice>();
    round_trips::<ArmState>();
    round_trips::<GestureMark>();
    round_trips::<ScopeOffer>();
}
