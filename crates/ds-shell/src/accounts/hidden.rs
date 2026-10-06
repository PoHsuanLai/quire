//! Secret text on its way from a field to the host.

use std::fmt;

/// Text the person typed into a hidden field (a password, an app password, an API key).
///
/// It lives in props and in the one event that reports a keystroke, and goes nowhere else: no
/// `Display`, no serde, and a `Debug` that prints `<hidden>`, so a log line, a panic message or
/// an error chain never carries it. The host reads it with [`Hidden::reveal`], at the point it
/// hands the answer to the sign-in.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct Hidden(String);

impl Hidden {
    /// Secret text holding `text`.
    pub fn new(text: impl Into<String>) -> Self {
        Hidden(text.into())
    }

    /// The text itself, for the host to hand to the sign-in.
    pub fn reveal(&self) -> &str {
        &self.0
    }

    /// Whether nothing is typed.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for Hidden {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<hidden>")
    }
}

#[cfg(test)]
mod tests {
    use super::Hidden;
    use crate::accounts::model::FieldText;

    #[test]
    fn debug_never_prints_the_text() {
        let secret = Hidden::new("hunter2");
        let wrapped = FieldText::Secret(secret.clone());
        for shown in [format!("{secret:?}"), format!("{wrapped:?}")] {
            assert!(!shown.contains("hunter2"), "{shown}");
        }
        assert_eq!(format!("{secret:?}"), "<hidden>");
        assert_eq!(secret.reveal(), "hunter2", "the host can still read it");
    }
}
