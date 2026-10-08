//! Today's state types: the recent items per Space, and what is parked.

use super::model::SpaceId;
use super::time::Epoch;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// One recently opened item, in one Space.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry<I> {
    /// Which Space it was opened in. The same item in two Spaces is two shortcuts.
    pub space: SpaceId,
    /// The app's item: a thread, a file, a page. Called `thread` in mailo's earlier files.
    #[serde(alias = "thread")]
    pub item: I,
    /// When it was last opened. Opening it again moves this forward.
    pub last_opened: Epoch,
}

/// Something put aside in one Space, which stays until it is reopened or discarded: parking is
/// not a timer, because the thing itself is kept elsewhere and this is only the way back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parked<K> {
    /// Which Space it was put aside in.
    pub space: SpaceId,
    /// The app's parked thing: a draft. Called `draft` in mailo's earlier files.
    #[serde(alias = "draft")]
    pub item: K,
    /// What its row says.
    pub title: String,
    /// When it was parked. The newest is first.
    pub parked: Epoch,
}

/// The parked kind of an app that parks nothing. Has no values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoPark {}

/// The items currently in Today, most recently opened first, and what is parked.
///
/// Entries expire [`super::IDLE`] after they were last opened; parked things never expire.
/// Closing or expiring an entry never touches the app's data. Written to `today.json` in the
/// state directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    default,
    bound(deserialize = "I: DeserializeOwned, K: DeserializeOwned")
)]
pub struct Today<I, K = NoPark> {
    /// Recent items. One that cannot be read drops only itself.
    #[serde(deserialize_with = "tolerant")]
    pub entries: Vec<Entry<I>>,
    /// Parked things, newest first. Called `drafts` in the files mailo wrote before the kit.
    #[serde(alias = "drafts", deserialize_with = "tolerant")]
    pub parked: Vec<Parked<K>>,
}

impl<I, K> Default for Today<I, K> {
    fn default() -> Self {
        Today {
            entries: Vec::new(),
            parked: Vec::new(),
        }
    }
}

/// A list read item by item: one that does not read is dropped, and a value that is not a list
/// is an empty one, as `recall` is read.
fn tolerant<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    Ok(match Value::deserialize(deserializer)? {
        Value::Array(items) => items
            .into_iter()
            .filter_map(|item| serde_json::from_value(item).ok())
            .collect(),
        _ => Vec::new(),
    })
}
