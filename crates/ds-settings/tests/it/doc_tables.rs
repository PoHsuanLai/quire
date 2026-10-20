//! design/22-SETTINGS.md's key tables for quire's own domains are generated from the schema
//! (`scripts/gen-doc-keys.sh`), so a default, a range or a page cannot drift from the code.
//! Reads a file outside the crate, so it is an integration test: `cargo test` runs it from the
//! crate root.

use std::path::Path;

use ds_settings::quire_schema;
use ds_settings::schema::regenerate;

const DOC_PATH: &str = "../../design/22-SETTINGS.md";

fn doc() -> String {
    std::fs::read_to_string(Path::new(DOC_PATH)).unwrap_or_else(|e| panic!("{DOC_PATH}: {e}"))
}

#[test]
fn the_generated_tables_match_the_schema_byte_for_byte() {
    let doc = doc();
    let fresh = regenerate(&doc, &[quire_schema()]).unwrap_or_else(|e| panic!("{e}"));
    assert!(
        fresh == doc,
        "design/22-SETTINGS.md is out of date: run scripts/gen-doc-keys.sh"
    );
}

#[test]
fn every_domain_of_quires_schema_has_a_generated_table() {
    let doc = doc();
    for domain in ["appearance", "icons", "region"] {
        let begin = format!("<!-- keys:begin {domain} -->");
        let end = format!("<!-- keys:end {domain} -->");
        assert!(
            doc.contains(&begin) && doc.contains(&end),
            "no generated table for `{domain}` in {DOC_PATH}"
        );
    }
}
