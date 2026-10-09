//! What an app's menu items and shortcuts are the face of, as plain data: the action each one
//! runs, or the reason it is only interface. `ds` reads it to write the app's `<AppName>.ui.toml`
//! (`ds::components::menus::ui_manifest`), which the conformance check of the intents manifest
//! reads. Nothing here names the manifest's own types: an action is a string the app declares
//! there.

mod chord;
mod face;
mod keys;
mod names;

pub use face::{AppCommand, CommandFace, ShortcutBinding};
pub use keys::{command_keys, is_command};
pub use names::{ActionName, CommandId, EmptyReason, IntentsApp, UiOnlyReason};
