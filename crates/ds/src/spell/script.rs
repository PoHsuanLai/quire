//! Which characters make words, and which text is never checked: CJK.
//!
//! CJK is detected by Unicode block, not by the locale: a Han ideograph, kana, Hangul or
//! Bopomofo character (Zhuyin's tone marks included), and the CJK punctuation and full-width
//! forms around them, is never part of a word, so CJK text is never marked. Chinese and
//! Japanese are written without spaces, a Hunspell dictionary cannot segment them, and a
//! Korean dictionary is not what a Latin surface's language names. A Latin word inside CJK text
//! (`用 Rust 寫`) is still checked.

/// Whether `c` is CJK: Han, kana, Hangul, Bopomofo, or CJK punctuation and full-width forms.
pub fn is_cjk(c: char) -> bool {
    matches!(
        u32::from(c),
        0x02C7 | 0x02CA | 0x02CB | 0x02D9 // Bopomofo tone marks
        | 0x1100..=0x11FF     // Hangul Jamo
        | 0x2E80..=0x2FDF     // CJK radicals, Kangxi radicals
        | 0x2FF0..=0x303F     // ideographic description, CJK symbols and punctuation
        | 0x3040..=0x30FF     // Hiragana, Katakana
        | 0x3100..=0x312F     // Bopomofo
        | 0x3130..=0x318F     // Hangul compatibility Jamo
        | 0x3190..=0x31FF     // Kanbun, Bopomofo extended, CJK strokes, Katakana extensions
        | 0x3200..=0x33FF     // enclosed CJK, CJK compatibility
        | 0x3400..=0x4DBF     // CJK extension A
        | 0x4E00..=0x9FFF     // CJK unified ideographs
        | 0xA960..=0xA97F     // Hangul Jamo extended A
        | 0xAC00..=0xD7FF     // Hangul syllables, Jamo extended B
        | 0xF900..=0xFAFF     // CJK compatibility ideographs
        | 0xFE30..=0xFE4F     // CJK compatibility forms
        | 0xFF00..=0xFFEF     // half-width and full-width forms
        | 0x1B000..=0x1B16F   // kana supplement and extended
        | 0x20000..=0x3134F // CJK extensions B to G, compatibility supplement
    )
}

/// A combining mark that belongs to the letter before it (a decomposed `é`).
fn is_combining(c: char) -> bool {
    matches!(
        u32::from(c),
        0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF | 0xFE20..=0xFE2F
    )
}

/// Whether `c` can be part of a word: a letter or digit that is not CJK, or a combining mark.
pub(crate) fn is_word_char(c: char) -> bool {
    (c.is_alphanumeric() && !is_cjk(c)) || is_combining(c)
}

/// An apostrophe, which joins two letters into one word (`don't`, `l’eau`).
pub(crate) fn is_apostrophe(c: char) -> bool {
    matches!(c, '\'' | '\u{2019}')
}

#[cfg(test)]
mod tests {
    use super::{is_cjk, is_word_char};

    const CJK: &[(char, bool)] = &[
        ('中', true),
        ('寫', true),
        ('か', true),
        ('カ', true),
        ('ㄓ', true),
        ('한', true),
        ('。', true),
        ('，', true),
        ('Ａ', true),
        ('𠀋', true),
        ('a', false),
        ('é', false),
        ('ß', false),
        ('я', false),
        ('1', false),
    ];

    #[test]
    fn cjk_is_found_by_block() {
        for (c, expected) in CJK {
            assert_eq!(is_cjk(*c), *expected, "{c:?}");
        }
    }

    #[test]
    fn cjk_letters_are_not_word_characters() {
        assert!(!is_word_char('中'));
        assert!(is_word_char('a'));
        assert!(is_word_char('\u{0301}'), "a combining acute");
        assert!(!is_word_char('-'));
    }
}
