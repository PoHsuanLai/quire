//! The check a stored `Word` enum runs against its serde form.

use super::Word;
use serde::Serialize;

/// Asserts that every variant of `T` serialises as its own slug, so a stored enum's settings
/// word and its attribute word are one spelling.
pub(crate) fn word_matches_serde<T: Word + Serialize>() {
    for word in T::ALL {
        assert_eq!(
            serde_json::to_value(word).ok(),
            Some(serde_json::Value::from(word.slug())),
            "the serde form of `{}` is not its slug",
            word.slug()
        );
    }
}
