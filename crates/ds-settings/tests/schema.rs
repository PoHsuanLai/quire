//! design/22-SETTINGS.md section 9.6's acceptance list, for the two domains this wave's derive
//! covers: `AppearanceSettings` and `IconsSettings` (`crates/ds-settings/src/settings.rs`).
//!
//! Run as an integration test, not a unit test, because `every_key_in_catalogue_has_a_spec`
//! reads a file outside the crate (`crates/ds-settings/tests/keys.rs` does the same, for the
//! same reason: `cargo test`'s working directory for a `tests/` binary is the crate root).

use std::path::Path;

use ds_settings::schema::{KeySpec, Page, Schema, SettingsSchema, deep_link_path};
use ds_settings::{AppearanceSettings, IconsSettings};

const DOC_PATH: &str = "../../design/22-SETTINGS.md";

fn schemas() -> Vec<Schema> {
    vec![AppearanceSettings::schema(), IconsSettings::schema()]
}

// ---------------------------------------------------------------------------------------------
// schema_round_trip

#[test]
fn schema_round_trip() {
    for schema in schemas() {
        let text = schema.to_toml();
        let back = Schema::from_toml(&text).unwrap_or_else(|e| panic!("{text}: {e}"));
        assert_eq!(back, schema, "{text}");
    }
}

// ---------------------------------------------------------------------------------------------
// widget_for_kind_is_total

#[test]
fn widget_for_kind_is_total() {
    // Every kind the derive actually emitted for these two domains resolves to a widget
    // without panicking; `ds_settings::schema::key::tests::every_kind_maps_to_exactly_one_widget`
    // (unit test, same crate) covers every `KeyKind` variant, including the ones neither
    // domain here happens to use (`Text`, `Colour`, `Shortcut`, `List`).
    let mut kinds = 0;
    for schema in schemas() {
        for key in &schema.key {
            let _ = key.kind.widget();
            kinds += 1;
        }
    }
    assert!(kinds > 0, "no keys to check");
}

// ---------------------------------------------------------------------------------------------
// deep_link_resolves

#[test]
fn deep_link_resolves() {
    let schemas = schemas();
    assert_eq!(
        deep_link_path(&schemas, "dark mode").map(|p| p.0),
        Some("appearance.theme".to_owned()),
        "\"dark mode\" should resolve to appearance.theme through its help text"
    );
    // The field is `motion_level` (`design/22-SETTINGS.md` section 3.1's own key,
    // `appearance.motion_level`; the derive's path is mechanically `<domain>.<field>`, so it is
    // not literally `appearance.motion` — see this crate's handback report). The doc's own
    // wording for the key is "reduce motion" (section 3.1's `MotionLevel::Reduced` row), which
    // is in the field's help text.
    assert_eq!(
        deep_link_path(&schemas, "reduce motion").map(|p| p.0),
        Some("appearance.motion_level".to_owned()),
        "\"reduce motion\" should resolve to appearance.motion_level through its help text"
    );
}

// ---------------------------------------------------------------------------------------------
// every_key_in_catalogue_has_a_spec

/// `appearance.*` keys the doc assigns to `sill`, not `AppearanceSettings`
/// (FINDINGS.md F15, mirrored from `crates/ds-settings/tests/keys.rs`'s `SILL_OWNED`).
const SILL_OWNED: &[&str] = &[
    "notifications.banner_material",
    "control_center.material",
    "launcher.material",
];

/// `appearance.motion_level` is also addressable as `motion.level` (section 3.2): a doc row,
/// not a second Rust field.
const DOC_ALIASES: &[&str] = &["motion.level"];

fn keys_in_section(doc: &str, heading: &str, next_heading: &str) -> Vec<String> {
    let start = doc
        .find(heading)
        .unwrap_or_else(|| panic!("{heading:?} not found in {DOC_PATH}"));
    let after = &doc[start..];
    let end = after
        .find(next_heading)
        .unwrap_or_else(|| panic!("{next_heading:?} not found after {heading:?} in {DOC_PATH}"));
    let section = &after[..end];
    section
        .lines()
        .filter_map(|line| {
            let first_cell = line.split('|').nth(1)?.trim();
            let key = first_cell.strip_prefix('`')?.strip_suffix('`')?;
            Some(key.to_owned())
        })
        .collect()
}

#[test]
fn every_key_in_catalogue_has_a_spec() {
    let doc =
        std::fs::read_to_string(Path::new(DOC_PATH)).unwrap_or_else(|e| panic!("{DOC_PATH}: {e}"));

    let mut doc_keys = keys_in_section(&doc, "### 3.1 `appearance`", "### 3.2 `motion`");
    doc_keys.extend(keys_in_section(&doc, "### 3.2 `motion`", "### 3.3 `icons`"));
    doc_keys.extend(keys_in_section(&doc, "### 3.3 `icons`", "### 3.4 `bar`"));
    assert!(!doc_keys.is_empty(), "no keys parsed from {DOC_PATH}");

    let spec_paths: Vec<String> = schemas()
        .iter()
        .flat_map(|schema| schema.key.iter().map(|key: &KeySpec| key.path.0.clone()))
        .collect();

    let mut seen = Vec::new();
    for doc_key in &doc_keys {
        if SILL_OWNED.contains(&doc_key.as_str()) || DOC_ALIASES.contains(&doc_key.as_str()) {
            continue;
        }
        assert!(
            spec_paths.contains(doc_key),
            "{doc_key}: no KeySpec with that path (have {spec_paths:?})"
        );
        seen.push(doc_key.clone());
    }
    for path in &spec_paths {
        assert!(
            seen.contains(path),
            "{path} has a KeySpec but no row in {DOC_PATH} sections 3.1-3.3"
        );
    }
}

#[test]
fn every_sill_owned_and_alias_row_named_here_is_still_in_the_doc() {
    let doc =
        std::fs::read_to_string(Path::new(DOC_PATH)).unwrap_or_else(|e| panic!("{DOC_PATH}: {e}"));
    let mut doc_keys = keys_in_section(&doc, "### 3.1 `appearance`", "### 3.2 `motion`");
    doc_keys.extend(keys_in_section(&doc, "### 3.2 `motion`", "### 3.3 `icons`"));
    for key in SILL_OWNED.iter().chain(DOC_ALIASES) {
        assert!(
            doc_keys.iter().any(|k| k == key),
            "{key} listed in SILL_OWNED/DOC_ALIASES but not found in the doc"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// The struct-level acceptance items (b)-(c) from the derive's own promise: `AppearanceSettings`
// and `IconsSettings` both carry the derive, resolve to the right file/page, and Advanced keys
// are actually marked Advanced.

#[test]
fn both_domains_resolve_to_quire_appearance_toml() {
    for schema in schemas() {
        assert_eq!(schema.app.0, "quire");
        assert_eq!(schema.file.0, "quire/appearance.toml");
        assert_eq!(schema.version, 1);
        for key in &schema.key {
            assert_eq!(key.page, Page::Appearance, "{}", key.path.0);
        }
    }
}

#[test]
fn icons_keys_are_all_advanced() {
    let icons = IconsSettings::schema();
    assert_eq!(icons.key.len(), 6);
    for key in &icons.key {
        assert_eq!(
            key.exposure,
            ds_settings::schema::Exposure::Advanced,
            "{} should be advanced (design/22-SETTINGS.md section 5)",
            key.path.0
        );
    }
}
