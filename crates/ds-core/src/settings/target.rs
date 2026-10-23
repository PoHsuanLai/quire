//! Where in Settings a link points: a page, one key, one account, or one Extension. The payload of
//! the `settings.page.show` intent and the one type every entrance to Settings hands over (the bar,
//! the launcher, the control centre, the command line, an agent).
//!
//! The text form is `page:wallpaper`, `key:dock.magnified_size_px`, `account:<id>`,
//! `extension:<id>`. One `FromStr` and one `Display` are the whole grammar; they round-trip.

use std::fmt;
use std::str::FromStr;

use super::{KeyPath, PageId};

/// An account, named by the id the accounts service gave it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AccountRef(pub String);

/// An Extension, named by the reverse-DNS id its manifest gives it (`org.example.Paper`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExtensionId(pub String);

/// A place in Settings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SettingsTarget {
    /// A page, by id: `page:wallpaper`.
    Page(PageId),
    /// One key, shown on its own page: `key:dock.magnified_size_px`.
    Key(KeyPath),
    /// One account's detail: `account:<id>`.
    Account(AccountRef),
    /// One Extension's row in Settings > Extensions: `extension:<id>`.
    Extension(ExtensionId),
}

/// Why a text is not a [`SettingsTarget`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum TargetError {
    /// The text does not start with `page:`, `key:`, `account:` or `extension:`.
    #[error(
        "{0:?} does not say where to go: expected page:<id>, key:<path>, account:<id> or extension:<id>"
    )]
    NoKind(String),
    /// A kind other than `page`, `key`, `account` or `extension`.
    #[error("{0:?} is not a place in Settings; expected page, key, account or extension")]
    UnknownKind(String),
    /// Nothing after the colon.
    #[error("{0} needs a name after the colon")]
    NoName(&'static str),
    /// The name holds a space or a control character.
    #[error("{0:?} is not a name: it holds a space or a control character")]
    BadName(String),
}

impl SettingsTarget {
    /// The word before the colon.
    fn kind(&self) -> &'static str {
        match self {
            SettingsTarget::Page(_) => "page",
            SettingsTarget::Key(_) => "key",
            SettingsTarget::Account(_) => "account",
            SettingsTarget::Extension(_) => "extension",
        }
    }

    /// The name after the colon.
    fn name(&self) -> &str {
        match self {
            SettingsTarget::Page(id) => id.as_str(),
            SettingsTarget::Key(path) => &path.0,
            SettingsTarget::Account(account) => &account.0,
            SettingsTarget::Extension(extension) => &extension.0,
        }
    }
}

impl fmt::Display for SettingsTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.kind(), self.name())
    }
}

impl FromStr for SettingsTarget {
    type Err = TargetError;

    fn from_str(text: &str) -> Result<SettingsTarget, TargetError> {
        let (kind, name) = text
            .split_once(':')
            .ok_or_else(|| TargetError::NoKind(text.to_owned()))?;
        let kind = match kind {
            "page" => "page",
            "key" => "key",
            "account" => "account",
            "extension" => "extension",
            other => return Err(TargetError::UnknownKind(other.to_owned())),
        };
        if name.is_empty() {
            return Err(TargetError::NoName(kind));
        }
        if name.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(TargetError::BadName(name.to_owned()));
        }
        Ok(match kind {
            "page" => SettingsTarget::Page(PageId::new(name)),
            "key" => SettingsTarget::Key(KeyPath(name.to_owned())),
            "account" => SettingsTarget::Account(AccountRef(name.to_owned())),
            _ => SettingsTarget::Extension(ExtensionId(name.to_owned())),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{AccountRef, ExtensionId, SettingsTarget, TargetError};
    use crate::settings::{KeyPath, PageId};

    #[test]
    fn every_target_prints_and_parses_back() {
        let cases = [
            (
                SettingsTarget::Page(PageId::new("wallpaper")),
                "page:wallpaper",
            ),
            (
                SettingsTarget::Page(PageId::new("app:com.example.App")),
                "page:app:com.example.App",
            ),
            (
                SettingsTarget::Key(KeyPath("dock.magnified_size_px".to_owned())),
                "key:dock.magnified_size_px",
            ),
            (
                SettingsTarget::Account(AccountRef("work/mail".to_owned())),
                "account:work/mail",
            ),
            (
                SettingsTarget::Extension(ExtensionId("org.example.Paper".to_owned())),
                "extension:org.example.Paper",
            ),
        ];
        for (target, text) in cases {
            assert_eq!(target.to_string(), text);
            assert_eq!(text.parse::<SettingsTarget>(), Ok(target), "{text}");
        }
    }

    #[test]
    fn a_text_that_is_not_a_place_says_why() {
        let cases = [
            ("", TargetError::NoKind(String::new())),
            ("wallpaper", TargetError::NoKind("wallpaper".to_owned())),
            (
                "pane:wallpaper",
                TargetError::UnknownKind("pane".to_owned()),
            ),
            (
                "Page:wallpaper",
                TargetError::UnknownKind("Page".to_owned()),
            ),
            ("page:", TargetError::NoName("page")),
            ("key:", TargetError::NoName("key")),
            ("account:", TargetError::NoName("account")),
            ("extension:", TargetError::NoName("extension")),
            (
                "extension:org.example. Paper",
                TargetError::BadName("org.example. Paper".to_owned()),
            ),
            (
                "extensions:org.example.Paper",
                TargetError::UnknownKind("extensions".to_owned()),
            ),
            (
                "key:dock. tile",
                TargetError::BadName("dock. tile".to_owned()),
            ),
            ("page:a\nb", TargetError::BadName("a\nb".to_owned())),
        ];
        for (text, want) in cases {
            assert_eq!(text.parse::<SettingsTarget>(), Err(want), "{text:?}");
        }
    }
}
