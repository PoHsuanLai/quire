//! What `Describe` returns: a live module's keys, in the same [`KeySpec`] shape as a file schema.

use serde::{Deserialize, Serialize};

use crate::schema::{KeyPath, KeySpec, never_settable_violation};

/// A live module's schema. JSON on the wire: `{"version":1,"key":[ ...KeySpec... ]}`.
///
/// Unlike a file [`Schema`](crate::schema::Schema) it names no app or file (the bus name and
/// object path say who serves it), and its keys may use [`KeyKind::Live`](crate::schema::KeyKind::Live).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveSchema {
    pub version: u16,
    pub key: Vec<KeySpec>,
}

/// Why a [`LiveSchema`] is refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum LiveSchemaError {
    #[error("the schema is not valid JSON of the live schema shape: {0}")]
    Parse(String),
    /// Section 9.7: an `accounts.`, `ai.`, ... key can never be marked agent-settable.
    #[error("key {} is under a never-agent-settable prefix but is marked agent = \"settable\"", .0.0)]
    NeverSettable(KeyPath),
    #[error("key {} appears twice", .0.0)]
    Duplicate(KeyPath),
}

impl LiveSchema {
    /// The JSON `Describe` returns.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self)
            .expect("LiveSchema fields are all JSON-representable: strings, numbers, enums, Vec")
    }

    /// The schema `text` holds, checked ([`LiveSchema::check`]).
    pub fn from_json(text: &str) -> Result<LiveSchema, LiveSchemaError> {
        let schema: LiveSchema =
            serde_json::from_str(text).map_err(|e| LiveSchemaError::Parse(e.to_string()))?;
        schema.check().map(|()| schema)
    }

    /// The same never-settable check a file schema gets (`Schema::from_toml`, through
    /// [`never_settable_violation`]), plus: no key twice.
    pub fn check(&self) -> Result<(), LiveSchemaError> {
        if let Some(key) = never_settable_violation(&self.key) {
            return Err(LiveSchemaError::NeverSettable(key.path.clone()));
        }
        let mut seen = std::collections::HashSet::new();
        match self.key.iter().find(|key| !seen.insert(&key.path)) {
            Some(key) => Err(LiveSchemaError::Duplicate(key.path.clone())),
            None => Ok(()),
        }
    }
}
