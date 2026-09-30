//! The pages, grouped like the components they show, and the pieces they share.

pub mod catalogue;
pub mod chrome;
pub mod content;
pub mod controls;
pub mod details;
pub mod editor;
pub mod foundations;
pub mod lists;
pub mod menus;
pub mod overlays;
pub mod settings_window;
pub mod shell;
mod specimen;
pub mod structure;

pub use specimen::{Caption, Scope, Section, Specimen};
