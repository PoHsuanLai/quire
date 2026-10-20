//! Where in Settings a link points: a page, one key, or one account. The types live in
//! [`ds_core::settings`], pure, so a compositor can hold a target without this crate; this module
//! keeps their old path.

pub use ds_core::settings::{AccountRef, SettingsTarget, TargetError};
