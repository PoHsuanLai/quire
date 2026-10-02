//! One settings key, as the derive emits it and the Settings app renders it
//! (design/22-SETTINGS.md section 9.1).

use serde::{Deserialize, Serialize};

use super::column::Column;

/// A dotted settings key path: `"dock.magnified_px"`. Always `<domain>.<field>`, the derive's
/// own mechanical rule (`crates/ds-settings-derive/src/gen_struct.rs`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KeyPath(pub String);

/// What a picker calls the key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Label(pub String);

/// The one-line explanation under the widget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Help(pub String);

/// What a picker calls each word of an enum key, by the stored word: `workspace_prev` is
/// "Previous workspace". Optional: a word without an entry is shown as its own words.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WordLabels(pub std::collections::BTreeMap<String, String>);

impl WordLabels {
    /// Whether no word has a label.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The label of `word`, if the schema gave one.
    pub fn of(&self, word: &str) -> Option<&str> {
        self.0.get(word).map(String::as_str)
    }
}

/// A key's group within its page, e.g. dock's "Magnification" section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Section(pub String);

/// The Settings app's fixed page list (design/22-SETTINGS.md section 9.3), plus one page per
/// third-party app id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "v", rename_all = "snake_case")]
pub enum Page {
    Appearance,
    Dock,
    MouseAndGestures,
    KeyboardAndShortcuts,
    Notifications,
    Spaces,
    Accounts,
    /// sill's idle service (design/22-SETTINGS.md section 3.24 `idle.*`, section 3.19
    /// `session.lock_grace_s`).
    Power,
    Apps,
    /// The companion, its models, memory and computer use (design/22-SETTINGS.md sections 3.26 to
    /// 3.30 `ai.*`, `agent.*`, `memory.*`, `cua.*`, `companion.*`).
    Intelligence,
    /// A third-party program's own page, named by its app id.
    App(String),
}

impl Page {
    /// The name a person reads for the page: a fixed page by its name in section 9.3, a
    /// third-party page by its app id.
    pub fn label(&self) -> &str {
        match self {
            Page::Appearance => "Appearance",
            Page::Dock => "Dock",
            Page::MouseAndGestures => "Mouse & Gestures",
            Page::KeyboardAndShortcuts => "Keyboard & Shortcuts",
            Page::Notifications => "Notifications",
            Page::Spaces => "Spaces",
            Page::Accounts => "Accounts",
            Page::Power => "Power",
            Page::Apps => "Apps",
            Page::Intelligence => "Intelligence",
            Page::App(id) => id,
        }
    }
}

/// File-only in v1 (section 5: "Advanced" = no Settings UI control) versus rendered on its
/// page. Not a `bool`: `CONVENTIONS.md#4-types`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Exposure {
    /// Rendered on its page.
    Basic,
    /// File only in v1; the Settings app renders it under a disclosure (section 9.3).
    Advanced,
}

/// The widget table is fixed (section 9.1): one component per [`KeyKind`], never chosen by the
/// Settings app itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Widget {
    /// A read-only label: the one value a one-variant enum can hold.
    Readout,
    Toggle,
    SegmentedControl,
    Menu,
    Slider,
    TextInput,
    AppearancePickerSwatch,
    ShortcutField,
    RowsEditor,
}

/// A key's shape: what values it can hold, and so which widget draws it.
///
/// Inferred from the field's own type by the derive (`crates/ds-settings-derive/src/shape.rs`):
/// a one-variant enum is [`KeyKind::Fixed`], a two-variant enum [`KeyKind::Toggle`], three to five [`KeyKind::Segmented`], more
/// [`KeyKind::Menu`]; a newtype with `range` is [`KeyKind::Bounded`]; `String`, `PathBuf`,
/// `Cow<str>` or a field marked `#[settings(text)]` is [`KeyKind::Text`]; `Hex` is [`KeyKind::Colour`]; `Vec<String>` and `Vec<Word enum>` are [`KeyKind::List`]; `Vec<T>` with
/// `T: SettingsRow` is [`KeyKind::Rows`].
/// [`KeyKind::Shortcut`] has no field-type rule yet (no settings key is a key binding today); a
/// live D-Bus module (section 9.4) constructs it directly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "v", rename_all = "snake_case")]
pub enum KeyKind {
    /// A one-variant enum: the key exists and has its one value, with room for a second
    /// variant later (`control_center.material`'s `Sheet` for v1).
    Fixed { variant: String },
    /// A two-variant enum: on/off, natural/traditional, ....
    Toggle { variants: [String; 2] },
    /// A three-to-five-variant enum.
    Segmented { variants: Vec<String> },
    /// A more-than-five-variant enum.
    Menu { variants: Vec<String> },
    /// A number with a floor and a ceiling, and the unit it is shown in.
    Bounded {
        min: i64,
        max: i64,
        unit: Option<String>,
    },
    /// Free text.
    Text,
    /// A colour swatch.
    Colour,
    /// A captured key chord.
    Shortcut,
    /// A rows editor over a list of the boxed kind.
    List(Box<KeyKind>),
    /// A rows editor over a list of tables, one cell per column. Derived from a `Vec<T>` field
    /// whose `T` derives `SettingsRow`.
    Rows { columns: Vec<Column> },
}

impl KeyKind {
    /// The one component that draws this kind (section 9.1's widget table; total by
    /// construction — every arm below is written, so a new [`KeyKind`] variant is a compile
    /// error here until it picks one).
    pub fn widget(&self) -> Widget {
        match self {
            KeyKind::Fixed { .. } => Widget::Readout,
            KeyKind::Toggle { .. } => Widget::Toggle,
            KeyKind::Segmented { .. } => Widget::SegmentedControl,
            KeyKind::Menu { .. } => Widget::Menu,
            KeyKind::Bounded { .. } => Widget::Slider,
            KeyKind::Text => Widget::TextInput,
            KeyKind::Colour => Widget::AppearancePickerSwatch,
            KeyKind::Shortcut => Widget::ShortcutField,
            KeyKind::List(_) | KeyKind::Rows { .. } => Widget::RowsEditor,
        }
    }
}

/// One field of one settings domain struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeySpec {
    pub path: KeyPath,
    pub kind: KeyKind,
    pub default: toml::Value,
    pub label: Label,
    pub help: Help,
    pub page: Page,
    pub section: Section,
    pub exposure: Exposure,
    /// Human labels for an enum key's words; empty for every other key and for a schema written
    /// before they existed (the field is absent then).
    #[serde(default, skip_serializing_if = "WordLabels::is_empty")]
    pub labels: WordLabels,
}

/// Every variant count from 1 up, mapped to the [`KeyKind`] it picks (section 9.1: "a two-
/// variant enum -> Toggle; 3..=5 variants -> Segmented; more -> Menu"; one variant is a
/// read-only [`KeyKind::Fixed`]).
///
/// Pure: takes the words, returns the kind. The derive's `kind_of::<T>()` (`crate::schema`)
/// is the only caller that reaches for a `Word`'s variants to get them.
pub fn kind_from_variants(mut words: Vec<String>) -> KeyKind {
    match words.len() {
        0 => panic!("a settings enum needs at least one variant, got none"),
        1 => KeyKind::Fixed {
            variant: words.remove(0),
        },
        2 => {
            let [a, b]: [String; 2] = words
                .try_into()
                .unwrap_or_else(|words: Vec<String>| panic!("checked len() == 2: {words:?}"));
            KeyKind::Toggle { variants: [a, b] }
        }
        3..=5 => KeyKind::Segmented { variants: words },
        _ => KeyKind::Menu { variants: words },
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyKind, Page, Widget, kind_from_variants};

    #[test]
    fn two_variants_are_a_toggle() {
        let kind = kind_from_variants(vec!["on".to_owned(), "off".to_owned()]);
        assert_eq!(
            kind,
            KeyKind::Toggle {
                variants: ["on".to_owned(), "off".to_owned()]
            }
        );
    }

    #[test]
    fn three_to_five_variants_are_segmented() {
        for count in 3..=5 {
            let words: Vec<String> = (0..count).map(|i| format!("v{i}")).collect();
            assert!(matches!(
                kind_from_variants(words),
                KeyKind::Segmented { .. }
            ));
        }
    }

    #[test]
    fn more_than_five_variants_are_a_menu() {
        let words: Vec<String> = (0..6).map(|i| format!("v{i}")).collect();
        assert!(matches!(kind_from_variants(words), KeyKind::Menu { .. }));
    }

    #[test]
    fn one_variant_is_fixed() {
        assert_eq!(
            kind_from_variants(vec!["only".to_owned()]),
            KeyKind::Fixed {
                variant: "only".to_owned()
            }
        );
    }

    #[test]
    #[should_panic(expected = "at least one variant")]
    fn no_variant_panics() {
        kind_from_variants(Vec::new());
    }

    #[test]
    fn every_kind_maps_to_exactly_one_widget() {
        assert_eq!(
            KeyKind::Fixed {
                variant: "a".to_owned()
            }
            .widget(),
            Widget::Readout
        );
        assert_eq!(
            KeyKind::Toggle {
                variants: ["a".to_owned(), "b".to_owned()]
            }
            .widget(),
            Widget::Toggle
        );
        assert_eq!(
            KeyKind::Segmented {
                variants: vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]
            }
            .widget(),
            Widget::SegmentedControl
        );
        assert_eq!(
            KeyKind::Menu {
                variants: (0..6).map(|i| format!("v{i}")).collect()
            }
            .widget(),
            Widget::Menu
        );
        assert_eq!(
            KeyKind::Bounded {
                min: 0,
                max: 100,
                unit: Some("%".to_owned())
            }
            .widget(),
            Widget::Slider
        );
        assert_eq!(KeyKind::Text.widget(), Widget::TextInput);
        assert_eq!(KeyKind::Colour.widget(), Widget::AppearancePickerSwatch);
        assert_eq!(KeyKind::Shortcut.widget(), Widget::ShortcutField);
        assert_eq!(
            KeyKind::List(Box::new(KeyKind::Text)).widget(),
            Widget::RowsEditor
        );
        assert_eq!(
            KeyKind::Rows {
                columns: Vec::new()
            }
            .widget(),
            Widget::RowsEditor
        );
    }

    #[test]
    fn page_intelligence_label_and_wire_form() {
        assert_eq!(Page::Intelligence.label(), "Intelligence");
        assert_eq!(
            serde_json::to_string(&Page::Intelligence).ok().as_deref(),
            Some(r#"{"kind":"intelligence"}"#)
        );
        assert_eq!(
            serde_json::from_str::<Page>(r#"{"kind":"intelligence"}"#).ok(),
            Some(Page::Intelligence)
        );
    }

    #[test]
    fn a_page_is_labelled_as_the_settings_app_names_it() {
        const CASES: &[(Page, &str)] = &[
            (Page::Appearance, "Appearance"),
            (Page::Dock, "Dock"),
            (Page::MouseAndGestures, "Mouse & Gestures"),
            (Page::KeyboardAndShortcuts, "Keyboard & Shortcuts"),
            (Page::Notifications, "Notifications"),
            (Page::Spaces, "Spaces"),
            (Page::Accounts, "Accounts"),
            (Page::Power, "Power"),
            (Page::Apps, "Apps"),
            (Page::Intelligence, "Intelligence"),
        ];
        for (page, want) in CASES {
            assert_eq!(page.label(), *want, "{page:?}");
        }
        assert_eq!(
            Page::App("com.example.App".to_owned()).label(),
            "com.example.App"
        );
    }
}

#[cfg(test)]
mod label_tests {
    use super::{KeySpec, WordLabels};

    const OLD: &str = "path = \"a.b\"\ndefault = \"x\"\nlabel = \"B\"\nhelp = \"\"\nsection = \"\"\nexposure = \"basic\"\n[kind]\nkind = \"text\"\n[page]\nkind = \"appearance\"\n";

    #[test]
    fn a_key_written_before_labels_existed_still_reads_and_writes_without_them() {
        let spec: KeySpec = toml::from_str(OLD).expect("old schema text parses");
        assert!(spec.labels.is_empty());
        assert!(!toml::to_string(&spec).unwrap().contains("labels"));
    }

    #[test]
    fn labels_round_trip_by_word() {
        let text = format!("{OLD}[labels]\nback = \"Back\"\n");
        let spec: KeySpec = toml::from_str(&text).expect("parses");
        assert_eq!(spec.labels.of("back"), Some("Back"));
        assert_eq!(spec.labels.of("other"), None);
        let again: KeySpec = toml::from_str(&toml::to_string(&spec).unwrap()).unwrap();
        assert_eq!(again.labels, WordLabels(spec.labels.0.clone()));
    }
}
