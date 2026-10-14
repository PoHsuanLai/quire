//! One section of a page: the schema's rows, and the app's own rows beside them.

use crate::edit::{SettingEdit, value_of};
use crate::key_row::KeyRow;
use dioxus::prelude::*;
use ds::prelude::*;
use ds_settings::schema::KeySpec;

/// Rows an app adds to a page beside the schema's: `rows` are drawn at the end of the section
/// titled `section`, or, when the schema has no section of that name, as a section of their own
/// after the schema's. A row of `rows` is any component, usually a `FieldRow`.
#[derive(Debug, Clone, PartialEq)]
pub struct SectionRows {
    pub section: String,
    pub rows: Element,
    /// A note about the whole group, drawn under it in the help type.
    pub footer: Option<String>,
}

impl SectionRows {
    /// `rows` for the section titled `section`, with no footer.
    pub fn new(section: impl Into<String>, rows: Element) -> Self {
        Self {
            section: section.into(),
            rows,
            footer: None,
        }
    }

    /// The same rows with `footer` under the group.
    pub fn footer(self, footer: impl Into<String>) -> Self {
        Self {
            footer: Some(footer.into()),
            ..self
        }
    }
}

/// One section: its title over a row per key, then `children`, and the note that explains the
/// whole group, if the caller has one, under it.
#[component]
pub(crate) fn SchemaSection(
    title: String,
    keys: Vec<KeySpec>,
    values: toml::Value,
    onedit: EventHandler<SettingEdit>,
    #[props(default)] children: Element,
    #[props(default)] footer: Option<String>,
) -> Element {
    rsx! {
        FormSection { title: (!title.is_empty()).then(|| title.clone()), footer,
            for key in keys {
                KeyRow { key: "{key.path.0}", value: value_of(&values, &key), spec: key.clone(), onedit }
            }
            {children}
        }
    }
}
