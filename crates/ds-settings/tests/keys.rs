//! design/22-SETTINGS.md section 6 acceptance test 5, `appearance` domain only: every key
//! `### 3.1 appearance` names has a field on [`ds_settings::AppearanceSettings`] reachable by
//! stripping the `appearance.` prefix and splitting on `_`/`.` — catches a key added to the doc
//! and forgotten in code, or a Rust field with no doc entry, in either direction.
//!
//! Run as an integration test (not a unit test) because it reads a file outside the crate: the
//! path is relative to the crate root, which is where `cargo test` sets the working directory
//! for a test binary in `tests/`.

use std::path::Path;

const DOC_PATH: &str = "../../design/22-SETTINGS.md";

/// `appearance.*` keys the doc assigns to `sill`, not `AppearanceSettings`
/// (FINDINGS.md F15: "Material keys for banners, control center and launcher live in sill's
/// settings, not `AppearanceSettings`"). Named explicitly, per section 6's "noted inline in the
/// test, not silently skipped", rather than filtered by a shape nobody can see at a glance.
const SILL_OWNED: &[&str] = &[
    "notifications.banner_material",
    "control_center.material",
    "launcher.material",
];

/// The fields `AppearanceSettings` actually has (`crates/ds-settings/src/settings.rs`), mirrored
/// here so the doc and the struct can be compared without runtime reflection.
const APPEARANCE_SETTINGS_FIELDS: &[&str] = &[
    "theme",
    "look",
    "warmth",
    "accent",
    "motion_level",
    "material_tint_alpha",
    "material_highlight_light",
    "material_highlight_dark",
    "material_hairline_light",
    "material_hairline_dark",
    "material_shadow_strength",
    "material_vibrancy",
];

/// The `Key` column of every data row in one `###`-level section of the doc, in file order.
/// A data row is `| \`dotted.key\` | ... |`; the header and separator rows do not start their
/// first cell with a backtick and are skipped.
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
fn every_appearance_key_in_the_doc_has_a_field_in_the_schema() {
    let doc =
        std::fs::read_to_string(Path::new(DOC_PATH)).unwrap_or_else(|e| panic!("{DOC_PATH}: {e}"));

    let keys = keys_in_section(&doc, "### 3.1 `appearance`", "### 3.2 `motion`");
    assert!(
        !keys.is_empty(),
        "no keys parsed from section 3.1 of {DOC_PATH}"
    );

    let mut seen_fields = Vec::new();
    for key in &keys {
        if SILL_OWNED.contains(&key.as_str()) {
            continue;
        }
        let field = key
            .strip_prefix("appearance.")
            .unwrap_or_else(|| panic!("{key} is not `appearance.*` and not in SILL_OWNED"));
        assert!(
            APPEARANCE_SETTINGS_FIELDS.contains(&field),
            "{key}: no `AppearanceSettings` field named {field:?} (have {APPEARANCE_SETTINGS_FIELDS:?})"
        );
        seen_fields.push(field);
    }

    for field in APPEARANCE_SETTINGS_FIELDS {
        assert!(
            seen_fields.contains(field),
            "AppearanceSettings.{field} has no `appearance.{field}` row in {DOC_PATH} section 3.1"
        );
    }
}

#[test]
fn every_sill_owned_row_named_here_is_still_in_the_doc() {
    // Guards SILL_OWNED itself against drifting out of sync with the doc (a row renamed or
    // removed there should fail loudly here, not make the test above vacuously skip nothing).
    let doc =
        std::fs::read_to_string(Path::new(DOC_PATH)).unwrap_or_else(|e| panic!("{DOC_PATH}: {e}"));
    let keys = keys_in_section(&doc, "### 3.1 `appearance`", "### 3.2 `motion`");
    for key in SILL_OWNED {
        assert!(
            keys.iter().any(|k| k == key),
            "{key} listed in SILL_OWNED but not in the doc"
        );
    }
}
