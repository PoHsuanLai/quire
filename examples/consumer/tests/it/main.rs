//! The crate's integration tests: one executable, one module per former test file.
//! A separate test target needs a stated reason (CONVENTIONS.md, Tests).

mod coherence;
mod guide;

#[path = "../../../../crates/ds/tests/it/support/module_guard.rs"]
mod module_guard;

#[test]
fn every_test_file_is_a_module() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/it");
    let left_out = module_guard::undeclared(&dir);
    assert!(left_out.is_empty(), "not a module of main.rs, so never run: {left_out:?}");
}
