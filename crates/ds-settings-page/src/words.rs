//! A stored word as a person reads it.

use ds_settings::schema::KeySpec;

/// A stored word as a person reads it: the schema's label when it has one, else the word with
/// its first letter capitalised and underscores as spaces.
pub(crate) fn word_label(key: &KeySpec, word: &str) -> String {
    if let Some(label) = key.labels.of(word) {
        return label.to_owned();
    }
    let spaced = word.replace('_', " ");
    let mut chars = spaced.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}
