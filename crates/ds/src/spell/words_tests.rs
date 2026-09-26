use super::{Span, word_at, words};

/// The words `words` hands a dictionary, as text.
fn checked<'a>(text: &'a str, skips: &[Span]) -> Vec<&'a str> {
    words(text, skips)
        .into_iter()
        .map(|span| &text[span.start..span.end])
        .collect()
}

const CASES: &[(&str, &[&str])] = &[
    ("Teh cat sat", &["Teh", "cat", "sat"]),
    ("Hello, world!", &["Hello", "world"]),
    ("don't stop", &["don't", "stop"]),
    ("l\u{2019}eau", &["l\u{2019}eau"]),
    ("'quoted'", &["quoted"]),
    ("well-known", &["well", "known"]),
    ("  spaced\tout\nlines ", &["spaced", "out", "lines"]),
    // URLs, email addresses and paths.
    ("see https://example.com/teh page", &["see", "page"]),
    ("(https://exmaple.org).", &[]),
    ("at www.exmaple.com now", &["at", "now"]),
    ("mailto:ada@exmaple.org", &[]),
    ("write ada.lovelace@exmaple.org today", &["write", "today"]),
    ("<grace@navy.mil>", &[]),
    ("open /usr/share/hunspell or ~/notes", &["open", "or"]),
    (
        "a@b is not an address",
        &["a", "b", "is", "not", "an", "address"],
    ),
    ("nota://", &[]),
    // Code spans.
    ("run `cargo tset` now", &["run", "now"]),
    ("an `unpaired tick", &["an", "unpaired", "tick"]),
    // All capitals and digits.
    ("NASA and HTTP", &["and"]),
    ("I think", &["think"]),
    ("mp3 2nd v1 ok", &["ok"]),
    ("McDonald iPhone", &["McDonald", "iPhone"]),
    // CJK is never a word; Latin inside it still is.
    ("用 Rust 寫程式", &["Rust"]),
    ("我喜歡teh貓", &["teh"]),
    ("こんにちは世界", &[]),
    ("안녕하세요", &[]),
    ("ㄓㄨˋ音", &[]),
    ("naïve café", &["naïve", "café"]),
    ("cafe\u{0301}", &["cafe\u{0301}"]),
];

#[test]
fn a_paragraph_cuts_into_the_words_a_dictionary_checks() {
    for (text, expected) in CASES {
        assert_eq!(checked(text, &[]), *expected, "{text:?}");
    }
}

#[test]
fn a_host_code_stretch_is_skipped() {
    let text = "call frobnicate here";
    let code = Span::new(5, 15);
    assert_eq!(checked(text, &[code]), vec!["call", "here"]);
}

#[test]
fn the_caret_touches_a_word_at_either_end_or_inside() {
    let text = "teh cat";
    let cases: &[(usize, Option<Span>)] = &[
        (0, Some(Span::new(0, 3))),
        (2, Some(Span::new(0, 3))),
        (3, Some(Span::new(0, 3))),
        (4, Some(Span::new(4, 7))),
        (7, Some(Span::new(4, 7))),
    ];
    for (offset, expected) in cases {
        assert_eq!(word_at(text, *offset), *expected, "offset {offset}");
    }
    assert_eq!(word_at("teh  cat", 4), None, "between two spaces");
}
