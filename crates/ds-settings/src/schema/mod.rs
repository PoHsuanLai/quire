//! A program's settings schema, generated from its own settings structs
//! (design/22-SETTINGS.md section 9: "the schema is data").
//!
//! A program does not register its settings with a Settings app at runtime. It ships a schema
//! file (section 9.2), and the Settings app renders pages from every schema it finds (section
//! 9.3) — nothing here talks to a running Settings app, and nothing in the Settings app talks
//! back to a program.

mod agent;
mod behaviour;
mod column;
mod control;
mod corners;
mod corners_stored;
mod deep_link;
mod doc_table;
mod key;
mod live_action;
mod page;
mod page_decl;
mod pages;
mod place;
mod program;
mod provided;
mod traits;

pub use agent::{
    AGENT_NEVER_SETTABLE, AGENT_SETTABLE_PROPOSED, is_never_settable, never_settable_violation,
};
pub use behaviour::{CornerAction, HotCornerSettings, SwitcherSettings};
pub use column::{Column, ColumnKind, ColumnName};
pub use control::{Control, ControlName, TogglePair};
pub use corners::{CornerBinding, CornerBindings, CornerDo, CornerKey, CornerKeys, OpenItem};
pub use corners_stored::fold_older_commands;
pub use deep_link::{deep_link, deep_link_path};
pub use doc_table::{DocTableError, key_table, regenerate};
pub use ds_core::settings::{KeyPath, PageId};
pub use key::{
    AgentSetting, ChoiceGroup, ChoiceUnavailable, ChoiceWord, Exposure, GroupedChoices, Help,
    KeyKind, KeySpec, Label, Note, Section, UnavailableReason, WordLabels, kind_from_variants,
};
pub use live_action::{ActionLabel, ActionWeight, LiveAction};
pub use page::Page;
pub use page_decl::{Listing, PageDecl, PageOrder, PageSymbol, PageTitle};
pub use pages::{PageStatus, PageTable};
pub use place::{ChoicePlace, CompanyName, EngineName, HostName};
pub use program::{
    AppId, FilePath, ForeignTables, Schema, data_dirs, data_dirs_from, discover, maybe_write_schema,
};
pub use provided::{Provided, STANDARD};
pub use traits::{ListElement, SettingsRow, SettingsSchema, choice_of, kind_of, to_value};
