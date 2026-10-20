//! Where in Settings a link points, and the two names it is made of: a page id and a key path.
//! Pure text types with no schema behind them, so a compositor or a shell that never reads a
//! settings file can still hold, parse and print a target. `ds-settings` re-exports all of them
//! (`ds_settings::target::*`, `ds_settings::schema::{PageId, KeyPath}`) and keeps everything that
//! needs its schema data.

mod key_path;
mod page_id;
mod target;

pub use key_path::KeyPath;
pub use page_id::PageId;
pub use target::{AccountRef, SettingsTarget, TargetError};
