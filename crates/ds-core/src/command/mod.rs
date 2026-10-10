//! What an app's menu items and shortcuts are the face of, as plain data: the action each one
//! runs, or the reason it is only interface. `ds` reads it to write the app's `<AppName>.ui.toml`
//! (`ds::components::menus::ui_manifest`), which the conformance check of the intents manifest
//! reads. Nothing here names the manifest's own types: an action is a string the app declares
//! there.

mod chord;
mod face;
mod key_input;
mod keys;
mod names;
mod resolve;
mod shortcut_chord;
mod shortcut_for;
mod shortcut_text;

pub use face::{AppCommand, CommandFace, ShortcutBinding};
pub use key_input::{chord_key_of, key_input, modifiers_held, modifiers_of};
#[allow(deprecated)]
pub use keys::{command_keys, is_command};
pub use names::{ActionName, CommandId, EmptyReason, IntentsApp, UiOnlyReason};
pub use resolve::{chord_of, holds_primary, resolve, types_text};
pub use shortcut_for::{chord_caps, chord_text, shortcut_caps_for, shortcut_text_for};
pub use shortcut_text::{KeyCap, shortcut_caps, shortcut_text};
