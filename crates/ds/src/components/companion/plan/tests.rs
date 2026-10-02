use super::model::{Inclusion, PlanFinish, StopWhy};
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
fn every_plan_word_parses_back_from_its_slug() {
    round_trips::<Inclusion>();
    round_trips::<PlanFinish>();
    round_trips::<StopWhy>();
}
