//! Clipping known text to a known width in Rust, where `.ds-truncate`'s mask fade will not do:
//! the hover card's two-line message, a fixed-width snippet (design/04-COMPONENTS.md
//! "Truncation", design/02-TYPE.md section 10).
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

/// `text` cut to at most `chars` characters, the last of them `…` when it was cut.
pub fn clip_chars(text: &str, chars: usize) -> String {
    todo!()
}
