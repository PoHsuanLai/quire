//! The lenient reader every settings file uses (design/22-SETTINGS.md section 2, "Unknown
//! values fall back to the field's default"): a bad value costs that one key, never the file.
//!
//! The doc sketches this as a `deserialize_with` helper that falls back to the field *type's*
//! `Default`; that would turn a bad `material_tint_alpha` into 0 rather than the key's shipped
//! 80. So the fallback is the *struct's* default for that key: start from `D::default()` as a
//! key tree, lay each key the file sets over it one at a time, and keep a key only if `D` still
//! deserializes with it. A key `D` accepts but does not store back is one nobody reads: it is
//! reported in [`Loaded::unknown`] and gone on the next save.

mod text;
mod wire;

use crate::doc::{FileName, Format};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt;
pub(crate) use text::{FileText, file_text};
use wire::{JsonWire, TomlWire, Wire};

/// A settings file as read: the value, and everything in the file that did not become part of
/// it. Reading never fails; the caller reports what it finds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loaded<D> {
    /// Every valid key laid over the type's defaults.
    pub value: D,
    /// Keys the type does not read, dropped.
    pub unknown: Vec<UnknownKey>,
    /// Keys whose value was not valid for their field; the field's default was used.
    pub invalid: Vec<InvalidKey>,
}

/// A key path (`dock.puppy`) the file sets and the type does not read. The path stops at the
/// first table the type has no key for, so a whole unknown table is one entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnknownKey(pub String);

/// A key whose value the type refused.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InvalidKey {
    /// The dotted key path; empty when the whole file is not valid text and nothing in it could
    /// be read.
    pub path: String,
    /// Why the value was refused.
    pub reason: String,
}

impl fmt::Display for UnknownKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown key `{}` is not read and not kept", self.0)
    }
}

impl fmt::Display for InvalidKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.path.as_str() {
            "" => write!(
                f,
                "not valid, the previous or default settings are used: {}",
                self.reason
            ),
            path => write!(
                f,
                "`{path}` is not valid, its default is used: {}",
                self.reason
            ),
        }
    }
}

impl<D> Loaded<D> {
    /// A read that found nothing to report.
    pub fn clean(value: D) -> Self {
        Loaded {
            value,
            unknown: Vec::new(),
            invalid: Vec::new(),
        }
    }

    /// `value` for a file that is not valid text at all: the whole file is the one invalid key.
    pub fn garbled(value: D, reason: String) -> Self {
        Loaded {
            value,
            unknown: Vec::new(),
            invalid: vec![InvalidKey {
                path: String::new(),
                reason,
            }],
        }
    }

    /// One line per unknown or invalid key, each beginning with the file's name, for the
    /// caller's own log.
    pub fn diagnostics(&self, file: FileName) -> Vec<String> {
        let name = file.0;
        self.unknown
            .iter()
            .map(|key| format!("{name}: {key}"))
            .chain(self.invalid.iter().map(|key| format!("{name}: {key}")))
            .collect()
    }
}

impl<D: Default> Default for Loaded<D> {
    fn default() -> Self {
        Loaded::clean(D::default())
    }
}

/// What reading a file's text produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Read<D> {
    /// The text was a file of this format; some keys may still have been refused.
    Loaded(Loaded<D>),
    /// The text is not this format at all (a half-written file, a hand edit with a syntax
    /// error): nothing in it could be read.
    Garbled {
        /// Why the text does not parse.
        reason: String,
    },
}

/// `text` as a `D` in `format`, keeping every key that is valid and the default for every key
/// that is not.
pub(crate) fn read<D>(text: &str, format: Format) -> Read<D>
where
    D: Serialize + DeserializeOwned + Default,
{
    match format {
        Format::Toml => overlay::<D, TomlWire>(text),
        Format::Json => overlay::<D, JsonWire>(text),
        Format::Css => raw::<D>(text),
    }
}

/// `text` as the one string a raw document holds: every text is valid, so nothing is reported.
fn raw<D: DeserializeOwned>(text: &str) -> Read<D> {
    match serde_json::from_value(serde_json::Value::String(text.to_owned())) {
        Ok(value) => Read::Loaded(Loaded::clean(value)),
        Err(error) => Read::Garbled {
            reason: error.to_string(),
        },
    }
}

fn overlay<D, W>(text: &str) -> Read<D>
where
    D: Serialize + DeserializeOwned + Default,
    W: Wire,
{
    let given = match W::parse(text) {
        Ok(given) => given,
        Err(reason) => return Read::Garbled { reason },
    };
    let Some(mut tree) = W::encode(&D::default()) else {
        return Read::Loaded(Loaded::clean(D::default()));
    };
    let mut unknown: Vec<UnknownKey> = Vec::new();
    let mut invalid = Vec::new();
    for (path, value) in W::leaves(&given) {
        let mut candidate = tree.clone();
        W::set(&mut candidate, &path, value);
        match W::decode::<D>(&candidate) {
            Err(reason) => invalid.push(InvalidKey {
                path: path.join("."),
                reason,
            }),
            Ok(decoded) => match W::encode(&decoded) {
                Some(stored) if !W::has(&stored, &path) => {
                    let key = UnknownKey(unread_prefix::<W>(&stored, &path));
                    if !unknown.contains(&key) {
                        unknown.push(key);
                    }
                }
                _ => tree = candidate,
            },
        }
    }
    let value = W::decode(&tree).unwrap_or_default();
    Read::Loaded(Loaded {
        value,
        unknown,
        invalid,
    })
}

/// The shortest prefix of `path` that `stored` has no key for, dotted.
fn unread_prefix<W: Wire>(stored: &W::Table, path: &[String]) -> String {
    let end = (1..=path.len())
        .find(|&len| !W::has(stored, &path[..len]))
        .unwrap_or(path.len());
    path[..end].join(".")
}

#[cfg(test)]
mod tests;
