//! Live settings modules: `org.quire.SettingsModule1` (design/22-SETTINGS.md section 9.4).
//!
//! Runtime state that is not a file (accounts, local runtimes, later Wi-Fi and Bluetooth) is
//! served by its daemon on the session bus and rendered by the Settings app with the same widgets
//! as a file schema. This module holds no state of its own:
//!
//! - the daemon implements [`LiveModule`] over its own state and hands it to [`serve`];
//! - the Settings app talks to it through [`LiveClient`] (or the raw [`SettingsModule1Proxy`]).
//!
//! The wire interface is frozen; `org.quire.SettingsModule1.xml` beside this file is its
//! introspection XML and a test keeps the skeleton equal to it:
//!
//! | Member | Signature | Meaning |
//! | --- | --- | --- |
//! | `Describe` | `() -> s` | the module's [`LiveSchema`] as JSON |
//! | `Get` | `(s) -> v` | a key's current value |
//! | `Set` | `(s, v) -> ()` | set a value, or run an action key (any value; the client sends `true`) |
//! | `Changed` | `(s, v)` signal | a key's new value, broadcast |
//!
//! Values travel as `b` (bool), `x` (integer), `s` (string) or `as` (list of strings); see
//! [`value`].
//!
//! Agent-never-settable (section 9.7): a schema that marks an `accounts.` or `ai.` key (or any
//! [`AGENT_NEVER_SETTABLE`](crate::schema::AGENT_NEVER_SETTABLE) prefix) agent-settable is
//! refused by [`LiveSchema::check`], which [`serve`] runs before it exports anything and
//! `Describe` runs on every call.

mod client;
mod error;
mod module;
mod schema;
mod skeleton;
pub mod value;

pub use client::{Change, Changes, LiveClient, SettingsModule1Proxy};
pub use error::LiveError;
pub use module::{Access, Caller, LiveModule, SenderName, Verdict};
pub use schema::{LiveSchema, LiveSchemaError};
pub use skeleton::{Served, serve};

/// The D-Bus interface name.
pub const INTERFACE: &str = "org.quire.SettingsModule1";

/// The introspection XML of [`INTERFACE`], the frozen wire form.
pub const INTROSPECTION_XML: &str = include_str!("org.quire.SettingsModule1.xml");
