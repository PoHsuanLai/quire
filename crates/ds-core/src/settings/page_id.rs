//! A page of Settings, named by a stable id.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A page's id: `dock`, `wallpaper`, `mouse-and-gestures`. Lower-case words joined by hyphens;
/// the id is the page's address in a deep link (`page:wallpaper`) and in a schema's `[[page]]`
/// table. A third-party program's page is named by its app id.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PageId(pub String);

impl PageId {
    /// The id written as `text`.
    pub fn new(text: &str) -> PageId {
        PageId(text.to_owned())
    }

    /// The id as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
