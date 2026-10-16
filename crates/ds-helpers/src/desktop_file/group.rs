//! One `[Group]` of a desktop file and the typed reads of its keys.

use super::escape::{split_list, unescape};
use super::locale::EntryLocale;
use std::borrow::Cow;
use std::collections::BTreeMap;

/// One key as the file gives it: its value, and one value per locale (`Name[de]`). A key that
/// only has translations has no default.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct Key {
    default: Option<String>,
    locales: BTreeMap<String, String>,
}

impl Key {
    /// The value for the first of `locales` (each tried from its most to its least specific
    /// variant) that the key is translated into, else the default.
    fn pick(&self, locales: &[EntryLocale]) -> Option<&str> {
        locales
            .iter()
            .flat_map(EntryLocale::variants)
            .find_map(|variant| self.locales.get(&variant))
            .or(self.default.as_ref())
            .map(String::as_str)
    }
}

/// A group of a desktop file: its name and its keys. Values are held as written and read through
/// the typed accessors, which take the escapes of the specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopGroup {
    pub(super) name: String,
    pub(super) keys: BTreeMap<String, Key>,
}

impl DesktopGroup {
    pub(super) fn named(name: &str) -> DesktopGroup {
        DesktopGroup {
            name: name.to_owned(),
            keys: BTreeMap::new(),
        }
    }

    /// Sets `name` (in `locale`, or its default value) to `value`, as written; a key given twice
    /// keeps the last.
    pub(super) fn set(&mut self, name: &str, locale: Option<&str>, value: &str) {
        let key = self.keys.entry(name.to_owned()).or_default();
        match locale {
            Some(locale) => {
                key.locales.insert(locale.to_owned(), value.to_owned());
            }
            None => key.default = Some(value.to_owned()),
        }
    }

    /// The group's name: `Desktop Entry`, `Desktop Action new-window`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The names of the keys the group has, once each, in sorted order (a translated key counts
    /// under its plain name).
    pub fn key_names(&self) -> impl Iterator<Item = &str> {
        self.keys.keys().map(String::as_str)
    }

    /// The default value of `key`, its escapes taken (`\s`, `\n`, `\t`, `\r`, `\\`); none when the
    /// group has no default for it.
    pub fn string(&self, key: &str) -> Option<Cow<'_, str>> {
        let raw = self.keys.get(key)?.default.as_deref()?;
        Some(unescape(raw))
    }

    /// The value of `key` for the first of `locales` it is translated into, else its default,
    /// escapes taken. `locales` run best first.
    pub fn localized(&self, key: &str, locales: &[EntryLocale]) -> Option<Cow<'_, str>> {
        let raw = self.keys.get(key)?.pick(locales)?;
        Some(unescape(raw))
    }

    /// The items of the `;` list `key` holds, escapes taken (`\;` is a semicolon inside an item);
    /// empty items are dropped, and the list is empty when the key is missing.
    pub fn list(&self, key: &str) -> Vec<String> {
        self.string_list(key, &[])
    }

    /// The `;` list `key` holds in the first of `locales` it is translated into, else its
    /// default (`Keywords[de]=...`).
    pub fn localized_list(&self, key: &str, locales: &[EntryLocale]) -> Vec<String> {
        self.string_list(key, locales)
    }

    fn string_list(&self, key: &str, locales: &[EntryLocale]) -> Vec<String> {
        self.keys
            .get(key)
            .and_then(|key| key.pick(locales))
            .map(split_list)
            .unwrap_or_default()
    }

    /// `key` as a boolean: `true` and `false` are the only values the specification has. None
    /// when the key is missing or holds anything else.
    pub fn boolean(&self, key: &str) -> Option<bool> {
        match self.keys.get(key)?.default.as_deref()? {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }

    /// Whether `key` is `true`; a missing or malformed boolean is false, which is the
    /// specification's default for every boolean key.
    pub fn is_true(&self, key: &str) -> bool {
        self.boolean(key) == Some(true)
    }
}
