//! One settings key, as the derive emits it and the Settings app renders it
//! (design/22-SETTINGS.md section 9.1).

use serde::{Deserialize, Serialize};

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
    Apps,
    /// A third-party program's own page, named by its app id.
    App(String),
}

/// File-only in v1 (section 5: "Advanced" = no Settings UI control) versus rendered on its
/// page. Not a `bool`: `CONVENTIONS.md#11-quire-addenda-2026-09-24`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Exposure {
    /// Rendered on its page.
    Basic,
    /// File only in v1; the Settings app renders it under a disclosure (section 9.3).
    Advanced,
}

/// A schema version number (section 2's file `version`, section 9.5's "stays in the schema for
/// one version").
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Version(pub u16);

/// Whether a key is still live, or was removed from its struct and is kept one version for a
/// clean-up offer (section 9.5). The derive always emits `No`: it only ever sees the struct's
/// current fields, never a field that used to exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "v", rename_all = "snake_case")]
pub enum Deprecated {
    /// Still a live field.
    No,
    /// Removed after this version; the Settings app may offer to clean it up.
    Since(Version),
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
/// `Cow<str>` or a field marked `#[settings(text)]` is [`KeyKind::Text`]; `Hex` is [`KeyKind::Colour`]; `Vec<_>` is [`KeyKind::List`].
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
            KeyKind::List(_) => Widget::RowsEditor,
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
    pub deprecated: Deprecated,
}

/// Every variant count from 1 up, mapped to the [`KeyKind`] it picks (section 9.1: "a two-
/// variant enum -> Toggle; 3..=5 variants -> Segmented; more -> Menu"; one variant is a
/// read-only [`KeyKind::Fixed`]).
///
/// Pure: takes the words, returns the kind. The derive's `kind_of::<T>()` (`crate::schema`)
/// is the only caller that reaches for `SchemaVariants` to get them.
pub fn kind_from_variants(mut words: Vec<String>) -> KeyKind {
    match words.len() {
        0 => panic!("SchemaVariants: a settings enum needs at least one variant, got none"),
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
    use super::{KeyKind, Widget, kind_from_variants};

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
    }
}
