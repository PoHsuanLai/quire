//! The app's menu items and shortcuts as the conformance file `<AppName>.ui.toml`: each is the
//! face of an action of the app's intents manifest, or it is interface only and says why. An app
//! implements [`AppCommand`](ds_core::command::AppCommand) for the command type its menu items
//! yield and its shortcuts fire, reads its menu bar, its own menus and its shortcut table into
//! rows, and writes the file from them; `docket-eval --check-app` then reads it. The writer lives
//! here and the checker with the manifest, so `ds` names neither the manifest nor the checker.

mod model;
mod rows;
mod write;

pub use model::{CommandSource, UiCommand, UiManifest, UiOnlyRow, UiVocab};
pub use rows::{CommandRow, bar_rows, menu_rows, shortcut_rows};
pub use write::UiManifestError;

#[cfg(test)]
mod tests;
