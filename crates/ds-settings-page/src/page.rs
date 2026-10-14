//! `SettingsPage`: one page drawn from one or more schemas.

use crate::edit::SettingEdit;
use crate::groups::{KeyGroup, sections};
use crate::section::{SchemaSection, SectionRows};
use dioxus::prelude::*;
use ds::prelude::*;
use ds::root::common::Common;
use ds_settings::schema::Schema;

/// A settings page: a `Form` with one titled `FormSection` per section of each schema, in the
/// order each section's first key comes, and one `FieldRow` per key. The control follows the key's
/// kind: a two-word key is a switch when the words are an on/off pair and segments otherwise;
/// three or more choices are a pop-up button; a kind not drawn yet is shown read-only. Nothing
/// here knows a key by name.
///
/// `values` is the file's current content as TOML (a key not in it reads as its schema default).
/// The page holds no copy: a change reaches `onedit` as a [`SettingEdit`], the app applies it
/// (`SettingEdit::applied_to`), saves through its own `Store` and passes the new `values` back.
///
/// `extras` add the app's own rows to a section (see [`SectionRows`]); `children` come last, inside
/// the form, for whole sections the app draws itself. A page made only of the app's own rows is
/// `SettingsPage { extras }` or `SettingsPage { FormSection { .. } }` with no schemas.
#[component]
pub fn SettingsPage(
    #[props(default)] schemas: Vec<Schema>,
    #[props(default = toml::Value::Table(toml::Table::new()))] values: toml::Value,
    #[props(default)] onedit: EventHandler<SettingEdit>,
    #[props(default)] extras: Vec<SectionRows>,
    #[props(default)] children: Element,
    #[props(default)] common: Common,
) -> Element {
    let groups: Vec<KeyGroup> = schemas.iter().flat_map(sections).collect();
    let beside = |title: &str| -> Vec<SectionRows> {
        extras
            .iter()
            .filter(|extra| extra.section == title)
            .cloned()
            .collect()
    };
    let alone: Vec<SectionRows> = extras
        .iter()
        .filter(|extra| groups.iter().all(|group| group.title != extra.section))
        .cloned()
        .collect();
    rsx! {
        Form { common,
            for (at, group) in groups.iter().enumerate() {
                SchemaSection {
                    key: "{at}-{group.title}",
                    title: group.title.clone(),
                    keys: group.keys.clone(),
                    values: values.clone(),
                    onedit,
                    footer: beside(&group.title).iter().find_map(|extra| extra.footer.clone()),
                    for extra in beside(&group.title) {
                        {extra.rows}
                    }
                }
            }
            for (at, extra) in alone.into_iter().enumerate() {
                SchemaSection {
                    key: "own-{at}-{extra.section}",
                    title: extra.section.clone(),
                    keys: Vec::new(),
                    values: values.clone(),
                    onedit,
                    footer: extra.footer.clone(),
                    {extra.rows}
                }
            }
            {children}
        }
    }
}
