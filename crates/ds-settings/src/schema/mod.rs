//! A program's settings schema, generated from its own settings structs
//! (design/22-SETTINGS.md section 9: "the schema is data").
//!
//! A program does not register its settings with a Settings app at runtime. It ships a schema
//! file (section 9.2), and the Settings app renders pages from every schema it finds (section
//! 9.3) — nothing here talks to a running Settings app, and nothing in the Settings app talks
//! back to a program.

mod deep_link;
mod foreign;
mod key;
mod program;
mod traits;

pub use deep_link::{deep_link, deep_link_path};
pub use key::{
    Deprecated, Exposure, Help, KeyKind, KeyPath, KeySpec, Label, Page, Section, Version, Widget,
    kind_from_variants,
};
pub use program::{
    AppId, FilePath, Schema, Stale, data_dirs, discover, discover_reporting, maybe_write_schema,
};
pub use traits::{SchemaVariants, SettingsSchema, kind_of, to_value};
