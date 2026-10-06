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
mod deep_link;
mod key;
mod live_action;
mod program;
mod traits;

pub use agent::{
    AGENT_NEVER_SETTABLE, AGENT_SETTABLE_PROPOSED, is_never_settable, never_settable_violation,
};
pub use behaviour::{CornerAction, HotCornerSettings, SwitcherSettings};
pub use column::{Column, ColumnKind, ColumnName};
pub use deep_link::{deep_link, deep_link_path};
pub use key::{
    AgentSetting, ChoiceGroup, ChoiceUnavailable, ChoiceWord, Exposure, GroupedChoices, Help,
    KeyKind, KeyPath, KeySpec, Label, Page, Section, UnavailableReason, Widget, WordLabels,
    kind_from_variants,
};
pub use live_action::{ActionLabel, ActionWeight, LiveAction};
pub use program::{
    AppId, FilePath, Schema, data_dirs, data_dirs_from, discover, maybe_write_schema,
};
pub use traits::{ListElement, SettingsRow, SettingsSchema, choice_of, kind_of, to_value};
