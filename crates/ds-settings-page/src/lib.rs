//! `SettingsPage`: a settings schema drawn as rows, the way the desktop's Settings app draws it
//! (design/22-SETTINGS.md sections 9.1 and 9.5). One titled group per schema section, one row per
//! key, and the control the key's kind gets; nothing here knows a key by name. The values are the
//! caller's, and a change is handed back as a [`SettingEdit`] for the app to save through its own
//! `Store`.
//!
//! This is its own crate because the page needs `ds` (the rows and controls) and `ds-settings`
//! (the schema types) together, and neither may name the other (`scripts/check-boundary.sh`):
//! `ds-shell` would have to take `ds-settings`' file watcher.

mod control;
mod edit;
mod groups;
mod key_row;
mod page;
mod pair;
mod section;
mod words;

pub use edit::SettingEdit;
pub use page::SettingsPage;
pub use section::SectionRows;
