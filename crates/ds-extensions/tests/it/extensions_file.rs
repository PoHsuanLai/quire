//! `extensions.toml`: the schema'd file that holds the ids turned off.

use ds_extensions::{ExtensionId, ExtensionsFile, Power, extensions_schema};
use ds_settings::schema::{AgentSetting, ColumnKind, KeyKind, Page, Schema};
use ds_settings::{AppName, ConfigRoot, Store};

use crate::support::Scratch;

fn id(text: &str) -> ExtensionId {
    ExtensionId(text.to_owned())
}

#[test]
fn the_schema_is_one_rows_key_on_the_extensions_page_that_an_agent_cannot_set() {
    let schema = extensions_schema();

    assert_eq!(schema.app.0, "quire");
    assert_eq!(schema.file.0, "quire/extensions.toml");
    assert_eq!(schema.key.len(), 1);
    let key = &schema.key[0];
    assert_eq!(key.path.0, "extensions.off");
    assert_eq!(key.page, Page::declared("extensions"));
    assert_eq!(key.agent, AgentSetting::HandsOff);
    let KeyKind::Rows { columns } = &key.kind else {
        panic!("expected a rows key, got {:?}", key.kind);
    };
    assert_eq!(columns.len(), 1);
    assert_eq!(columns[0].name.0, "id");
    assert!(matches!(
        &columns[0].kind,
        ColumnKind::Text { pattern: Some(pattern) } if pattern.contains("\\.")
    ));
    assert_eq!(Schema::from_toml(&schema.to_toml()), Ok(schema));
}

#[test]
fn turning_an_extension_off_and_on_changes_only_its_row() {
    let paper = id("org.example.Paper");
    let quartz = id("org.example.Quartz");
    let none = ExtensionsFile::default().extensions;

    let off = none.with(&paper, Power::Off).with(&quartz, Power::Off);
    assert!(off.is_off(&paper) && off.is_off(&quartz));
    // Off twice is one row.
    assert_eq!(off.with(&paper, Power::Off).off.len(), 2);

    let back = off.with(&paper, Power::On);
    assert!(!back.is_off(&paper) && back.is_off(&quartz));
    assert_eq!(back.with(&quartz, Power::On), none);
}

#[test]
fn the_file_round_trips_and_names_the_ids_a_scan_takes() {
    let scratch = Scratch::new();
    let store = Store::new(
        ConfigRoot::Scratch(scratch.path().to_path_buf()),
        AppName::QUIRE,
    );
    assert!(ExtensionsFile::default().off_ids().is_empty());

    let turned_off = ExtensionsFile::default()
        .extensions
        .with(&id("org.example.Paper"), Power::Off)
        .with(&id("org.example.Quartz"), Power::Off);
    let file = ExtensionsFile {
        extensions: turned_off,
        ..ExtensionsFile::default()
    };
    store.save(&file).expect("saves");

    let path = store.path::<ExtensionsFile>().expect("a path");
    let text = std::fs::read_to_string(&path).expect("the file is there");
    assert!(text.contains("[[extensions.off]]"), "{text}");
    assert!(path.ends_with("quire/extensions.toml"), "{path:?}");

    let loaded = store.load::<ExtensionsFile>();
    assert!(loaded.invalid.is_empty() && loaded.unknown.is_empty());
    assert_eq!(loaded.value, file);
    assert_eq!(
        loaded.value.off_ids(),
        [id("org.example.Paper"), id("org.example.Quartz")]
    );
}

#[test]
fn a_blank_row_names_no_extension() {
    let file = ExtensionsFile {
        extensions: ExtensionsFile::default()
            .extensions
            .with(&id("  "), Power::Off),
        ..ExtensionsFile::default()
    };
    assert_eq!(file.extensions.off.len(), 1);
    assert!(file.off_ids().is_empty());
}
