//! Which palette a surface resolves to: the three-way choice a person makes ([`Theme`]) and
//! the two-way answer the stylesheet needs ([`Scheme`]).
//!
//! [`Theme`] moved from mailo (`mail-app/src/view.rs`). mailo's `Theme::attribute`, which
//! returned `None` for `System` so a `prefers-color-scheme` media guard could decide, is gone:
//! quire resolves the scheme in Rust and always writes an explicit `data-theme`
//! (design/05-MOTION.md section 9 rule 11, [`crate::resolve`]).

use serde::{Deserialize, Serialize};

/// Which palette the window resolves to.
///
/// Three states and not a bool: "follow the desktop" is a different choice from "light",
/// and a client that cannot express it either ignores the desktop or cannot be overridden
/// when the desktop is wrong.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default)]
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

impl Theme {
    /// Every choice, in the order a picker offers them.
    pub const ALL: [Theme; 3] = [Theme::System, Theme::Light, Theme::Dark];

    /// What a picker calls it.
    pub fn label(self) -> &'static str {
        match self {
            Theme::System => "System",
            Theme::Light => "Light",
            Theme::Dark => "Dark",
        }
    }

    /// The palette a stored word names, or [`None`] for a word that is not one.
    pub fn parse(word: &str) -> Option<Theme> {
        match word {
            "system" => Some(Theme::System),
            "light" => Some(Theme::Light),
            "dark" => Some(Theme::Dark),
            _ => None,
        }
    }
}

/// The palette a surface actually paints: a [`Theme`] with "follow the desktop" answered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum Scheme {
    /// The light Post palette.
    #[default]
    Light,
    /// The dark Post palette.
    Dark,
}

impl Scheme {
    /// Both schemes, light first: the order every legibility test sweeps them in.
    pub const ALL: [Scheme; 2] = [Scheme::Light, Scheme::Dark];

    /// The `data-theme` value on a `.ds` root.
    pub fn slug(self) -> &'static str {
        match self {
            Scheme::Light => "light",
            Scheme::Dark => "dark",
        }
    }
}
