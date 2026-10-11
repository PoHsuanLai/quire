//! A module read once: its schema and every key's value, agreeing with each other.

use std::collections::BTreeMap;

use super::{LiveError, LiveSchema};
use crate::schema::KeyPath;

/// What `Snapshot` returns: a module's [`LiveSchema`] and the value of every key in it.
///
/// Every key in `values` is in `schema` and every key in `schema` has a value, so a reader never
/// sees a half-changed module (a `Describe` that lists an account and a `Get` that no longer
/// knows it). A module that can read its state in one go overrides [`LiveModule::snapshot`]
/// (accountd answers from one lock); the default reads `Describe` then each key.
///
/// [`LiveModule::snapshot`]: super::LiveModule::snapshot
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub schema: LiveSchema,
    pub values: BTreeMap<KeyPath, toml::Value>,
}

impl Snapshot {
    /// The snapshot of `schema` given what a `Get` answered for each of its keys.
    ///
    /// A key that answered [`LiveError::UnknownKey`] went away since the schema was read: it is
    /// dropped from the schema as well, so the two agree. Any other failure is the read's failure.
    /// A key in `reads` that the schema does not list is ignored.
    pub fn from_reads(
        mut schema: LiveSchema,
        reads: impl IntoIterator<Item = (KeyPath, Result<toml::Value, LiveError>)>,
    ) -> Result<Snapshot, LiveError> {
        let mut values = BTreeMap::new();
        for (key, read) in reads {
            match read {
                Ok(value) => {
                    values.insert(key, value);
                }
                Err(LiveError::UnknownKey(_)) => {}
                Err(other) => return Err(other),
            }
        }
        schema.key.retain(|spec| values.contains_key(&spec.path));
        values.retain(|key, _| schema.key.iter().any(|spec| spec.path == *key));
        Ok(Snapshot { schema, values })
    }
}
