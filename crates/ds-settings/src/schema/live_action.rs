//! The kind of a key that is a button, not a value (design/22-SETTINGS.md section 9.4): a live
//! module's "Revoke", "Remove" and "Sign in again" rows. Setting such a key runs the action in
//! the service; nothing is stored.

use serde::{Deserialize, Serialize};

/// The words on an action's button: "Revoke", "Remove account".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActionLabel(pub String);

/// How heavily the host treats an action before it runs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionWeight {
    /// Runs on one press.
    Plain,
    /// Destroys something the person cannot get back: the host asks first (a critical alert that
    /// names what goes) and styles the button as destructive.
    Destructive,
}

/// What a [`KeyKind::Live`](super::KeyKind::Live) key does when it is set.
///
/// Wire form: `{"label":"Revoke","weight":"plain"}`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LiveAction {
    pub label: ActionLabel,
    pub weight: ActionWeight,
}
