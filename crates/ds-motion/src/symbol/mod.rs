//! Symbol effects: an icon's own motion, modelled on SF Symbols' (design/35-SYMBOL-EFFECTS.md).
//!
//! [`effect`] is the vocabulary a caller writes, [`route`] says which driver plays it on an icon,
//! [`attrs`] is what the stylesheet's share leaves on the wrapper, [`pose`] and [`timing`] are the
//! part driver's data, and `Symbol` (under the `dioxus` feature) draws it all.

pub mod attrs;
pub mod effect;
pub mod pose;
pub mod route;
pub mod timing;
#[cfg(feature = "dioxus")]
pub mod use_symbol;
#[cfg(feature = "dioxus")]
pub mod view;

/// The symbol's component sheet, appended to the stylesheet by the assembly in its cascade slot.
pub const CSS: &str = include_str!("symbol.css");
