//! `#[derive(SettingsSchema)]` on the shapes sill needed and the derive used to refuse (sill
//! FINDINGS Q5): text by type (`String`, `PathBuf`, `Cow<str>`), a text newtype through
//! `#[settings(text)]`, and a one-variant enum. The refusals (`MissingRange`, an empty enum)
//! are compile errors, tested on the expansion in `crates/ds-settings-derive/src`.

use std::borrow::Cow;
use std::path::PathBuf;

use ds_settings::schema::{KeyKind, Page, SettingsSchema, Widget};
use ds_settings::{Ms, SettingsSchema};
use serde::{Deserialize, Serialize};

/// A sound theme's name: text, though the type is a newtype.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
struct SoundTheme(String);

/// The one material a panel offers in v1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, SettingsSchema)]
#[serde(rename_all = "snake_case")]
enum PanelMaterial {
    #[default]
    Sheet,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SettingsSchema)]
#[serde(default)]
#[settings(file = "probe/settings.toml", domain = "probe", page = Page::App("probe".to_owned()))]
struct Probe {
    #[settings(label = "Name")]
    name: String,
    #[settings(label = "Folder")]
    folder: PathBuf,
    #[settings(label = "Greeting")]
    greeting: Cow<'static, str>,
    #[settings(label = "Sound theme", text)]
    sound_theme: SoundTheme,
    #[settings(label = "Material")]
    material: PanelMaterial,
    #[settings(label = "Delay", range = "0..=1000", unit = "ms")]
    delay_ms: Ms,
}

impl Default for Probe {
    fn default() -> Self {
        Probe {
            name: "quire".to_owned(),
            folder: PathBuf::from("/tmp"),
            greeting: Cow::Borrowed("hello"),
            sound_theme: SoundTheme("freedesktop".to_owned()),
            material: PanelMaterial::Sheet,
            delay_ms: Ms(200),
        }
    }
}

#[test]
fn every_field_gets_the_kind_its_type_or_attribute_names() {
    let schema = Probe::schema();
    let kinds: Vec<(&str, &KeyKind)> = schema
        .key
        .iter()
        .map(|key| (key.path.0.as_str(), &key.kind))
        .collect();
    let fixed = KeyKind::Fixed {
        variant: "sheet".to_owned(),
    };
    let bounded = KeyKind::Bounded {
        min: 0,
        max: 1000,
        unit: Some("ms".to_owned()),
    };
    let want: Vec<(&str, &KeyKind)> = vec![
        ("probe.name", &KeyKind::Text),
        ("probe.folder", &KeyKind::Text),
        ("probe.greeting", &KeyKind::Text),
        ("probe.sound_theme", &KeyKind::Text),
        ("probe.material", &fixed),
        ("probe.delay_ms", &bounded),
    ];
    assert_eq!(kinds, want);
}

#[test]
fn a_one_variant_enum_is_a_readout_with_its_default() {
    let schema = Probe::schema();
    let key = schema
        .key
        .iter()
        .find(|key| key.path.0 == "probe.material")
        .unwrap_or_else(|| panic!("no material key"));
    assert_eq!(key.kind.widget(), Widget::Readout);
    assert_eq!(key.default, toml::Value::String("sheet".to_owned()));
}

#[test]
fn text_defaults_are_their_strings_and_the_schema_round_trips() {
    let schema = Probe::schema();
    let default_of = |path: &str| {
        schema
            .key
            .iter()
            .find(|key| key.path.0 == path)
            .map(|key| key.default.clone())
    };
    assert_eq!(
        default_of("probe.sound_theme"),
        Some(toml::Value::String("freedesktop".to_owned()))
    );
    assert_eq!(
        default_of("probe.folder"),
        Some(toml::Value::String("/tmp".to_owned()))
    );
    let text = schema.to_toml();
    let back = ds_settings::schema::Schema::from_toml(&text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(back, schema, "{text}");
}
