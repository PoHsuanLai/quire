//! What can go wrong reading or writing settings. Reading never fails the caller (a damaged
//! file is the defaults); writing and watching can.

use std::path::PathBuf;

/// A settings write, watch or portal read that did not happen.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    /// A file or directory could not be read, written or renamed.
    #[error("{path}: {source}")]
    Io {
        /// Which path.
        path: PathBuf,
        /// Why.
        source: std::io::Error,
    },
    /// The value could not be written as TOML.
    #[error("encoding settings as TOML: {0}")]
    Encode(#[from] toml::ser::Error),
    /// The value could not be written as JSON.
    #[error("encoding settings as JSON: {0}")]
    EncodeJson(#[from] serde_json::Error),
    /// The directory watch could not be started.
    #[error("watching settings: {0}")]
    Watch(#[from] notify::Error),
    /// The settings portal could not be reached or answered something unexpected.
    #[error("settings portal: {0}")]
    Portal(#[from] zbus::Error),
}
