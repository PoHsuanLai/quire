//! The public selector table: every class it lists is in the stylesheet, its names are unique,
//! and `docs/selectors.md` is the table (`DS_BLESS=1` rewrites the page; read the diff).

use ds::selectors::{AXES, COMPONENTS, markdown};
use std::collections::HashSet;
use std::path::PathBuf;

fn page() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/selectors.md")
}

/// Whether the stylesheet has a rule that selects `.class`.
fn styled(sheet: &str, class: &str) -> bool {
    let needle = format!(".{class}");
    sheet.match_indices(&needle).any(|(at, _)| {
        !sheet[at + needle.len()..]
            .starts_with(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    })
}

#[test]
fn every_listed_class_is_in_the_stylesheet() {
    let sheet = ds::stylesheet();
    let mut missing = Vec::new();
    for component in COMPONENTS {
        let classes = std::iter::once(component.root_selector()).chain(component.part_selectors());
        for selector in classes {
            let class = selector.trim_start_matches('.');
            if !styled(&sheet, class) {
                missing.push(format!("{}: {selector}", component.component));
            }
        }
    }
    assert_eq!(missing, Vec::<String>::new(), "listed but not styled");
}

#[test]
fn the_table_has_no_duplicates_and_every_root_is_a_ds_class() {
    let roots: Vec<&str> = COMPONENTS.iter().map(|c| c.root).collect();
    let unique: HashSet<&str> = roots.iter().copied().collect();
    assert_eq!(roots.len(), unique.len(), "a root is listed twice: {roots:?}");
    for component in COMPONENTS {
        assert!(component.root.starts_with("ds-"), "{}", component.root);
        let parts: HashSet<&str> = component.parts.iter().copied().collect();
        assert_eq!(parts.len(), component.parts.len(), "{}", component.component);
    }
    let axes: HashSet<&str> = AXES.iter().map(|axis| axis.attribute).collect();
    assert_eq!(axes.len(), AXES.len(), "an attribute is listed twice");
}

#[test]
fn the_doc_page_is_the_table() {
    let want = markdown();
    let path = page();
    if std::env::var("DS_BLESS").is_ok_and(|value| value == "1") {
        std::fs::write(&path, &want).expect("write docs/selectors.md");
        return;
    }
    let have = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e} (DS_BLESS=1 writes it)", path.display()));
    assert_eq!(have, want, "docs/selectors.md is stale (DS_BLESS=1 rewrites it)");
}
