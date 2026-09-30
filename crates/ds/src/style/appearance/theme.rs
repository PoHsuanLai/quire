//! Which palette a surface resolves to: the three-way choice a person makes ([`Theme`]) and
//! the two-way answer the stylesheet needs ([`Scheme`]).
//!
//! quire resolves the scheme in Rust and always writes an explicit `data-theme`, never leaving
//! `System` to a `prefers-color-scheme` media guard (design/05-MOTION.md section 9 rule 11,
//! [`crate::resolve`]).

use ds_core::word::Word;
use serde::{Deserialize, Serialize};

/// Which palette the window resolves to.
///
/// Three states and not a bool: "follow the desktop" is a different choice from "light",
/// and a client that cannot express it either ignores the desktop or cannot be overridden
/// when the desktop is wrong.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default, Word)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    /// Follow the desktop.
    #[default]
    System,
    /// The light palette, even when the desktop is dark.
    Light,
    /// The dark palette, even when the desktop is light.
    Dark,
}

/// The palette a surface actually paints: a [`Theme`] with "follow the desktop" answered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default, Word)]
#[serde(rename_all = "snake_case")]
pub enum Scheme {
    /// The light Post palette.
    #[default]
    Light,
    /// The dark Post palette.
    Dark,
}
