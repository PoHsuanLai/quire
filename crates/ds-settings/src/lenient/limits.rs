//! The ranges a schema gives its keys, enforced when a file is read: a number outside its range
//! is brought to the nearest end and reported, so a hand edit (`tile_size_px = 5000`) cannot
//! reach a consumer that trusted the slider.

use crate::schema::{KeyKind, KeySpec, Schema};
use std::collections::BTreeMap;
use std::fmt;

/// The ends of a key's range, both included.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LimitBounds {
    pub min: i64,
    pub max: i64,
}

/// The range of every bounded key of a document, by dotted key path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Limits {
    bounds: BTreeMap<String, LimitBounds>,
}

impl Limits {
    /// No ranges: every value is kept as the file gives it.
    pub fn none() -> Limits {
        Limits::default()
    }

    /// The ranges of the bounded keys among `keys`. A key whose low end is above its high end
    /// (a hand-written schema) has no range.
    pub fn of_keys(keys: &[KeySpec]) -> Limits {
        let bounds = keys
            .iter()
            .filter_map(|key| match &key.kind {
                KeyKind::Bounded { min, max, .. } if min <= max => Some((
                    key.path.0.clone(),
                    LimitBounds {
                        min: *min,
                        max: *max,
                    },
                )),
                _ => None,
            })
            .collect();
        Limits { bounds }
    }

    /// The ranges of the bounded keys of `schema`.
    pub fn of_schema(schema: &Schema) -> Limits {
        Limits::of_keys(&schema.key)
    }

    /// The range of the key at `path`, if it has one.
    pub fn bounds(&self, path: &str) -> Option<LimitBounds> {
        self.bounds.get(path).copied()
    }
}

/// A key whose value was outside its range and was brought to the nearest end.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClampedKey {
    /// The dotted key path.
    pub path: String,
    /// The value the file gave, as text.
    pub was: String,
    /// The value in use, as text.
    pub now: String,
}

impl fmt::Display for ClampedKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "`{}` is {}, outside its range; {} is used",
            self.path, self.was, self.now
        )
    }
}
