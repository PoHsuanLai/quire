//! How a variant name becomes a slug and a label.

/// The spelling of a slug: which separator joins a variant's words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Case {
    /// `read-only`, the attribute-value spelling.
    Kebab,
    /// `read_only`, the spelling of a stored enum's serde name.
    Snake,
}

/// The words of a `CamelCase` identifier: a new word starts at each upper-case letter that
/// follows a lower-case letter or a digit, or that starts a word inside an acronym (`URLBar`).
pub(crate) fn words(ident: &str) -> Vec<String> {
    let chars: Vec<char> = ident.chars().collect();
    let mut out: Vec<String> = Vec::new();
    for (i, &ch) in chars.iter().enumerate() {
        let starts = i == 0
            || (ch.is_uppercase()
                && (chars[i - 1].is_lowercase()
                    || chars[i - 1].is_ascii_digit()
                    || chars.get(i + 1).is_some_and(|next| next.is_lowercase())));
        if starts {
            out.push(String::new());
        }
        if let Some(word) = out.last_mut() {
            word.extend(ch.to_lowercase());
        }
    }
    out
}

/// `ident` as a slug in `case`.
pub(crate) fn slug(ident: &str, case: Case) -> String {
    let separator = match case {
        Case::Kebab => "-",
        Case::Snake => "_",
    };
    words(ident).join(separator)
}

/// `ident` as a label: the words spaced, the first capitalised (`ReadOnly` -> `Read only`).
pub(crate) fn label(ident: &str) -> String {
    let joined = words(ident).join(" ");
    let mut chars = joined.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{Case, label, slug};

    const CASES: &[(&str, &str, &str, &str)] = &[
        ("single word", "Postmark", "postmark", "Postmark"),
        ("two words", "ReadOnly", "read-only", "Read only"),
        (
            "three words",
            "SameAsLight",
            "same-as-light",
            "Same as light",
        ),
        ("acronym", "URLBar", "url-bar", "Url bar"),
        ("digit", "Grid2x", "grid2x", "Grid2x"),
    ];

    #[test]
    fn slug_and_label_follow_the_words_of_the_variant() {
        for (name, ident, want_slug, want_label) in CASES {
            assert_eq!(slug(ident, Case::Kebab), *want_slug, "{name}");
            assert_eq!(label(ident), *want_label, "{name}");
        }
    }

    #[test]
    fn snake_case_joins_with_underscores() {
        assert_eq!(slug("SameAsLight", Case::Snake), "same_as_light");
    }
}
