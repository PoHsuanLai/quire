//! What an [`EditSurface`](crate::EditSurface) tells its app: typed text, keys, IME composition
//! and clipboard gestures, in the order they happened. An app's adapter maps each to its editor
//! core's input (mailo builds `editor::InputEvent`s from them).

use dioxus::prelude::{Key, Modifiers};

/// One piece of input for the app's editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditInput {
    /// Text to insert at the selection: a printable key, or an IME commit outside a composition.
    Text(String),
    /// A key that is not text (Enter, Backspace, arrows, Tab) or a chord (Ctrl+B, Ctrl+Z).
    Key(KeyInput),
    /// A step of an IME composition. The app draws the preedit itself and inserts nothing until
    /// [`Composition::End`].
    Composition(Composition),
    /// A paste, with the clipboard's HTML when it had any.
    Paste(Pasted),
    /// Ctrl+X (or Shift+Delete): the app copies its selection and deletes it.
    Cut,
    /// Ctrl+C (or Ctrl+Insert): the app copies its selection.
    Copy,
}

/// A key the surface did not turn into text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyInput {
    /// The key, as the keyboard layout reads it.
    pub key: Key,
    /// The modifiers held with it.
    pub modifiers: Modifiers,
}

/// A step of an IME composition (Zhuyin, Pinyin, kana to kanji).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Composition {
    /// A composition began: the first [`Composition::Update`] follows at once.
    Start,
    /// The preedit is now `text` (empty when the IME cleared it, as it does just before a
    /// commit), with the IME's cursor inside it.
    Update {
        /// The text being composed, not yet part of the document.
        text: String,
        /// The IME's cursor or highlight in `text`, when it shows one.
        cursor: Option<PreeditCursor>,
    },
    /// The composition ended with `text` committed; empty when it was cancelled.
    End {
        /// The text to insert.
        text: String,
    },
}

/// The IME's cursor inside a preedit: UTF-8 byte offsets into its text, equal for a caret.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PreeditCursor {
    /// Where it starts.
    pub start: usize,
    /// Where it ends.
    pub end: usize,
}

/// What a paste carried.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pasted {
    /// Plain text only.
    Text(String),
    /// HTML (untrusted: the app sanitises it) and its plain-text form.
    Html {
        /// The clipboard's `text/html`.
        html: String,
        /// The clipboard's `text/plain`, empty when it had none.
        text: String,
    },
}
