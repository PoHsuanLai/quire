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

/// The longest tail, in characters, that counts as an extension: a dot and up to seven more.
const LONGEST_EXTENSION: usize = 8;

/// `text` cut to at most `chars` characters with the `…` in the middle, as a file name is
/// shortened: the start and the end stay, and an extension (`.png`) stays whole when there is
/// room for it.
///
/// Counts Unicode scalar values and cuts only on their boundaries. Text that already fits comes
/// back unchanged; `chars == 0` leaves nothing and `chars == 1` only the mark. With an extension
/// the end keeps a third of what is left after it, the start the rest; without one the two
/// halves are even, the start the larger.
pub fn clip_middle(text: &str, chars: usize) -> String {
    let all: Vec<char> = text.chars().collect();
    if all.len() <= chars {
        return text.to_string();
    }
    let Some(room) = chars.checked_sub(1) else {
        return String::new();
    };
    let extension = extension_len(&all).filter(|extension| room >= extension + 2);
    let kept_end = extension.unwrap_or(0);
    let left = room - kept_end;
    let tail = match extension {
        Some(_) => left / 3,
        None => room / 2,
    };
    let head = match extension {
        Some(_) => left - tail,
        None => room - tail,
    };
    let end_from = all.len() - kept_end - tail;
    all[..head]
        .iter()
        .copied()
        .chain(Some(ELLIPSIS))
        .chain(all[end_from..].iter().copied())
        .collect()
}

/// How many characters the extension at the end of `all` takes, dot included: none when the
/// last dot starts the name, ends it, or sits further back than [`LONGEST_EXTENSION`].
fn extension_len(all: &[char]) -> Option<usize> {
    let dot = all.iter().rposition(|c| *c == '.')?;
    let length = all.len() - dot;
    (dot > 0
        && length > 1
        && length <= LONGEST_EXTENSION
        && !all[dot..].iter().any(|c| c.is_whitespace()))
    .then_some(length)
}

#[cfg(test)]
mod tests {
    use super::{clip_chars, clip_middle};

    const MIDDLE: &[(&str, usize, &str)] = &[
        ("", 0, ""),
        ("report.pdf", 10, "report.pdf"),
        ("report.pdf", 20, "report.pdf"),
        ("report.pdf", 0, ""),
        ("report.pdf", 1, "…"),
        ("holiday-picture-2026.png", 14, "holida…026.png"),
        ("holiday-picture-2026.png", 12, "holid…26.png"),
        ("holiday-picture-2026.png", 6, "hol…ng"),
        ("holiday-picture-2026.png", 5, "ho…ng"),
        ("holiday-picture-2026.png", 7, "ho….png"),
        ("a-long-name-with-no-extension", 11, "a-lon…nsion"),
        (".a-long-hidden-file-name", 9, ".a-l…name"),
        ("notes.of.a-long.ending-with-a-dot.", 12, "notes.…-dot."),
        ("a name.with a space in it", 10, "a nam…n it"),
        ("日本語のテキスト.txt", 8, "日本…ト.txt"),
    ];

    #[test]
    fn clips_the_middle_and_keeps_the_extension() {
        let failures: Vec<String> = MIDDLE
            .iter()
            .filter_map(|(text, chars, want)| {
                let got = clip_middle(text, *chars);
                (got != *want)
                    .then(|| format!("clip_middle({text:?}, {chars}) = {got:?}, want {want:?}"))
            })
            .collect();
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn the_middle_clip_is_never_longer_than_asked() {
        for chars in 0..30 {
            let got = clip_middle("a-very-long-holiday-picture-name-2026.png", chars);
            assert!(got.chars().count() <= chars, "{chars}: {got:?}");
        }
    }

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
