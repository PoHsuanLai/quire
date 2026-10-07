//! The design system's own stylesheet, without the shell's: pinned by a golden file. The whole
//! sheet a shell surface draws with, and the rules about its shape, are `ds-shell`'s
//! `tests/stylesheet.rs`.
//!
//! `DS_BLESS=1 cargo test -p ds --test it stylesheet` rewrites the golden file.

use ds::stylesheet;
use std::path::PathBuf;

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/stylesheet.css")
}

#[test]
fn the_stylesheet_matches_its_golden_file() {
    let path = golden_path();
    if std::env::var_os("DS_BLESS").is_some() {
        std::fs::write(&path, stylesheet())
            .unwrap_or_else(|why| panic!("writing {}: {why}", path.display()));
        return;
    }
    let golden = std::fs::read_to_string(&path)
        .unwrap_or_else(|why| panic!("{}: {why}; run with DS_BLESS=1", path.display()));
    if golden != stylesheet() {
        let first = golden
            .lines()
            .zip(stylesheet().lines())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| golden.lines().count().min(stylesheet().lines().count()));
        panic!(
            "the stylesheet differs from {} from line {}; rerun with DS_BLESS=1 if intended\n\
             golden:    {:?}\ngenerated: {:?}",
            path.display(),
            first + 1,
            golden.lines().nth(first),
            stylesheet().lines().nth(first)
        );
    }
}
