//! A key whose value is a list of tables (`Vec<Row>` with `#[derive(SettingsRow)]`) and a list
//! of text (`Vec<String>`): the kinds the derive gives them, the schema text they write, and
//! the settings file they round-trip through (design/22-SETTINGS.md section 9.1).

use ds_core::word::Word;
use ds_settings::schema::{
    Column, ColumnKind, ColumnName, KeyKind, Label, Page, Schema, SettingsRow, SettingsSchema,
    Widget,
};
use ds_settings::{SettingsRow, SettingsSchema};
use serde::{Deserialize, Serialize};

/// Which side a pin sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Word)]
#[serde(rename_all = "snake_case")]
#[word(case = snake)]
enum Side {
    #[default]
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SettingsRow)]
struct Override {
    #[settings(label = "App")]
    app: String,
    #[settings(label = "Profile", pattern = "[a-z0-9_-]{1,64}")]
    profile: String,
    #[settings(label = "Side")]
    side: Side,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SettingsSchema)]
#[serde(default)]
#[settings(file = "probe/settings.toml", domain = "keys", page = Page::KeyboardAndShortcuts)]
struct Keys {
    #[settings(label = "Overrides", help = "One row per app.", advanced)]
    overrides: Vec<Override>,
    #[settings(label = "Names")]
    names: Vec<String>,
    #[settings(label = "Sides")]
    sides: Vec<Side>,
}

impl Default for Keys {
    fn default() -> Self {
        Keys {
            overrides: vec![Override {
                app: "org.gnome.Nautilus".to_owned(),
                profile: "files".to_owned(),
                side: Side::Right,
            }],
            names: vec!["a".to_owned(), "b".to_owned()],
            sides: vec![Side::Left],
        }
    }
}

fn column(name: &str, label: &str, kind: ColumnKind) -> Column {
    Column {
        name: ColumnName(name.to_owned()),
        label: Label(label.to_owned()),
        kind,
    }
}

#[test]
fn a_vec_of_rows_is_rows_and_a_vec_of_text_or_words_is_a_list() {
    let schema = Keys::schema();
    let kinds: Vec<&KeyKind> = schema.key.iter().map(|key| &key.kind).collect();
    let want = [
        KeyKind::Rows {
            columns: vec![
                column("app", "App", ColumnKind::Text { pattern: None }),
                column(
                    "profile",
                    "Profile",
                    ColumnKind::Text {
                        pattern: Some("[a-z0-9_-]{1,64}".to_owned()),
                    },
                ),
                column(
                    "side",
                    "Side",
                    ColumnKind::Choice {
                        variants: vec!["left".to_owned(), "right".to_owned()],
                    },
                ),
            ],
        },
        KeyKind::List(Box::new(KeyKind::Text)),
        KeyKind::List(Box::new(KeyKind::Toggle {
            variants: ["left".to_owned(), "right".to_owned()],
        })),
    ];
    assert_eq!(kinds, want.iter().collect::<Vec<_>>());
    for kind in kinds {
        assert_eq!(kind.widget(), Widget::RowsEditor);
    }
}

#[test]
fn the_row_struct_lists_its_columns_in_field_order() {
    let names: Vec<String> = Override::columns().into_iter().map(|c| c.name.0).collect();
    assert_eq!(names, ["app", "profile", "side"]);
}

#[test]
fn the_rows_schema_writes_the_columns_as_data_and_reads_back_equal() {
    let schema = Keys::schema();
    let text = schema.to_toml();
    assert!(text.contains("kind = \"rows\""), "{text}");
    assert!(
        text.contains("pattern = \"[a-z0-9_-]{1,64}\""),
        "the pattern is written as data: {text}"
    );
    assert_eq!(
        text.matches("pattern").count(),
        1,
        "a column without a pattern writes none: {text}"
    );
    let back = Schema::from_toml(&text).unwrap_or_else(|e| panic!("{text}: {e}"));
    assert_eq!(back, schema, "{text}");
}

#[test]
fn the_default_of_a_rows_key_is_its_list_of_tables() {
    let schema = Keys::schema();
    let default = &schema.key[0].default;
    let rows = default.as_array().expect("an array of tables");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["app"].as_str(), Some("org.gnome.Nautilus"));
    assert_eq!(rows[0]["side"].as_str(), Some("right"));
}

#[test]
fn rows_round_trip_through_a_settings_file() {
    let keys = Keys::default();
    let text = toml::to_string(&keys).unwrap_or_else(|e| panic!("{e}"));
    assert!(text.contains("[[overrides]]"), "{text}");
    let back: Keys = toml::from_str(&text).unwrap_or_else(|e| panic!("{text}: {e}"));
    assert_eq!(back, keys, "{text}");

    let empty = Keys {
        overrides: Vec::new(),
        ..Keys::default()
    };
    let text = toml::to_string(&empty).unwrap_or_else(|e| panic!("{e}"));
    let back: Keys = toml::from_str(&text).unwrap_or_else(|e| panic!("{text}: {e}"));
    assert_eq!(back, empty, "{text}");
}
