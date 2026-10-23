//! The crate's integration tests: one executable, one module per concept. Every test works in
//! scratch roots standing in for `$XDG_DATA_HOME` and `$XDG_DATA_DIRS`; none reads the real ones.

mod checks;
mod choices;
mod extensions_file;
mod scan;
mod support;

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
