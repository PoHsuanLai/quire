//! What a paste carried, as the clipboard gave it to the host.

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
