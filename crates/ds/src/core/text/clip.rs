//! Clipping known text to a known width in Rust, where `.ds-truncate`'s mask fade will not do:
//! the hover card's two-line message, a fixed-width snippet (design/04-COMPONENTS.md
//! "Truncation", design/02-TYPE.md section 10).

/// The mark that ends clipped text.
const ELLIPSIS: char = '…';

/// `text` cut to at most `chars` characters, the last of them `…` when it was cut.
///
/// Counts Unicode scalar values and cuts only on their boundaries, so no character is ever
/// split. Text that already fits comes back unchanged; `chars == 0` leaves nothing.
pub fn clip_chars(text: &str, chars: usize) -> String {
    if text.chars().nth(chars).is_none() {
        return text.to_string();
    }
    let Some(kept) = chars.checked_sub(1) else {
        return String::new();
    };
    text.chars().take(kept).chain(Some(ELLIPSIS)).collect()
}

#[cfg(test)]
mod tests {
    use super::clip_chars;

    const CASES: &[(&str, usize, &str)] = &[
        ("", 0, ""),
        ("", 3, ""),
        ("abc", 3, "abc"),
        ("abc", 4, "abc"),
        ("abcd", 3, "ab…"),
        ("abcd", 1, "…"),
        ("abcd", 0, ""),
        ("héllo wörld", 6, "héllo…"),
        ("日本語のテキスト", 4, "日本語…"),
        ("a😀b😀c", 4, "a😀b…"),
        ("a😀b😀", 4, "a😀b😀"),
    ];

    #[test]
    fn clips_on_char_boundaries() {
        let failures: Vec<String> = CASES
            .iter()
            .filter_map(|(text, chars, want)| {
                let got = clip_chars(text, *chars);
                (got != *want)
                    .then(|| format!("clip_chars({text:?}, {chars}) = {got:?}, want {want:?}"))
            })
            .collect();
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn never_longer_than_asked() {
        for chars in 0..12 {
            let got = clip_chars("the quick brown fox", chars);
            assert!(got.chars().count() <= chars, "{chars}: {got:?}");
        }
    }
}
