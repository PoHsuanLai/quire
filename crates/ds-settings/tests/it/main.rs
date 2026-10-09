//! The crate's integration tests: one executable, one module per former test file.
//! A separate test target needs a stated reason (CONVENTIONS.md, Tests).
//! Separate target: `live` (feature `live`) runs the settings service on a private D-Bus.

mod appearance_file;
mod derive;
mod keys;
mod rows;
mod schema;
mod spaces_file;
mod spaces_storage;
mod store;
mod support;
mod user_style;
mod watch;

#[path = "../../../ds/tests/it/support/module_guard.rs"]
mod module_guard;

#[test]
fn every_test_file_is_a_module() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/it");
    let left_out = module_guard::undeclared(&dir);
    assert!(
        left_out.is_empty(),
        "not a module of main.rs, so never run: {left_out:?}"
    );
}
