//! One settings key, named by its dotted path.

use serde::{Deserialize, Serialize};

/// A dotted settings key path: `"dock.magnified_px"`. Always `<domain>.<field>`, the derive's
/// own mechanical rule (`crates/ds-settings-derive/src/gen_struct.rs`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KeyPath(pub String);
