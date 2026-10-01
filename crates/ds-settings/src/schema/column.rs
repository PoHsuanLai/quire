//! One column of a list-of-tables key (design/22-SETTINGS.md section 9.1, `KeyKind::Rows`): what
//! `#[derive(SettingsRow)]` emits per field of the row struct.

use serde::{Deserialize, Serialize};

use super::key::Label;

/// A column's key in each row's table: the row struct's field name, as serde writes it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ColumnName(pub String);

/// The values a column holds. Scalar only: a row cannot contain rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "v", rename_all = "snake_case")]
pub enum ColumnKind {
    /// Free text. `pattern`, when present, is a regular expression the whole value must match
    /// (it is data for the Settings app to validate with; the owning program validates again
    /// at load).
    Text {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pattern: Option<String>,
    },
    /// One word of a closed enum, by its stored spelling, in declaration order.
    Choice { variants: Vec<String> },
}

/// One column of a rows key: a cell in every row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Column {
    /// The key of the cell in the row's table.
    pub name: ColumnName,
    /// What a person calls the column.
    pub label: Label,
    /// What the cell holds.
    pub kind: ColumnKind,
}

#[cfg(test)]
mod tests {
    use super::{Column, ColumnKind, ColumnName};
    use crate::schema::Label;

    fn column(kind: ColumnKind) -> Column {
        Column {
            name: ColumnName("app".to_owned()),
            label: Label("App".to_owned()),
            kind,
        }
    }

    #[test]
    fn a_text_column_without_a_pattern_writes_no_pattern() {
        let text = toml::to_string(&column(ColumnKind::Text { pattern: None })).unwrap();
        assert!(!text.contains("pattern"), "{text}");
    }

    #[test]
    fn columns_round_trip_through_toml() {
        let cases = [
            column(ColumnKind::Text { pattern: None }),
            column(ColumnKind::Text {
                pattern: Some("[a-z]{1,8}".to_owned()),
            }),
            column(ColumnKind::Choice {
                variants: vec!["a".to_owned(), "b".to_owned()],
            }),
        ];
        for case in cases {
            let text = toml::to_string(&case).unwrap();
            let back: Column = toml::from_str(&text).unwrap_or_else(|e| panic!("{text}: {e}"));
            assert_eq!(back, case, "{text}");
        }
    }
}
