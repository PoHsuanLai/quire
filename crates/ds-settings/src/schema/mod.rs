//! A program's settings schema, generated from its own settings structs
//! (design/22-SETTINGS.md section 9: "the schema is data").
//!
//! A program does not register its settings with a Settings app at runtime. It ships a schema
//! file (section 9.2), and the Settings app renders pages from every schema it finds (section
//! 9.3) — nothing here talks to a running Settings app, and nothing in the Settings app talks
//! back to a program.

mod deep_link;
mod key;
mod program;
mod traits;

pub use deep_link::{deep_link, deep_link_path};
pub use key::{
    Exposure, Help, KeyKind, KeyPath, KeySpec, Label, Page, Section, Widget, kind_from_variants,
};
pub use program::{
    AppId, FilePath, Schema, data_dirs, data_dirs_from, discover, maybe_write_schema,
};
pub use traits::{SettingsSchema, kind_of, to_value};
