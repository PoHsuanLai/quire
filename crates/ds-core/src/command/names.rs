//! The names a command's face is written with.

use serde::{Deserialize, Serialize};

/// An action as the app's intents manifest names it (`mail.draft.create`). Plain text: whether the
/// manifest declares it is the conformance check's to say, not `ds`'s.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActionName(pub String);

/// The app's own stable id for one menu item or shortcut (`file.new`), the key the conformance
/// file lists it under.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CommandId(pub String);

/// The app the intents manifest is filed under (`org.quire.Mail`), which names the conformance
/// file too.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IntentsApp(pub String);

/// Why a command is interface only and no action (`window chrome`). Never empty: a reason that
/// says nothing is the gap the conformance check exists to find.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct UiOnlyReason(String);

/// A reason with no words in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("a UI-only command needs a reason")]
pub struct EmptyReason;

impl UiOnlyReason {
    /// `text`, trimmed, when it says something.
    pub fn new(text: impl AsRef<str>) -> Result<Self, EmptyReason> {
        match text.as_ref().trim() {
            "" => Err(EmptyReason),
            words => Ok(UiOnlyReason(words.to_owned())),
        }
    }

    /// The reason as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for UiOnlyReason {
    type Error = EmptyReason;

    fn try_from(text: String) -> Result<Self, EmptyReason> {
        UiOnlyReason::new(text)
    }
}

impl From<UiOnlyReason> for String {
    fn from(reason: UiOnlyReason) -> String {
        reason.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reason_is_trimmed_and_never_empty() {
        const CASES: &[(&str, Option<&str>)] = &[
            ("window chrome", Some("window chrome")),
            ("  scrolling \n", Some("scrolling")),
            ("", None),
            ("   \t", None),
        ];
        for &(text, want) in CASES {
            let got = UiOnlyReason::new(text).ok();
            assert_eq!(got.as_ref().map(UiOnlyReason::as_str), want, "{text:?}");
        }
    }

    #[test]
    fn a_stored_empty_reason_is_refused_on_load() {
        let loaded: Result<UiOnlyReason, _> = serde_json::from_str("\"  \"");
        assert!(loaded.is_err());
        let kept: UiOnlyReason = serde_json::from_str("\"window chrome\"").unwrap();
        assert_eq!(serde_json::to_string(&kept).unwrap(), "\"window chrome\"");
    }
}
