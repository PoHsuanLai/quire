//! The clock rule, held over the design system's sources (sill Q380): ds reads time only through
//! `ds::time` (`now`, `since`, `sleep`), so a harness's virtual clock reaches every timer and
//! every "now". A direct `Instant::now()` (or `.elapsed()` on an `Instant`, which this scan
//! cannot tell from a tween's own `elapsed`, so review catches it) or a
//! `futures_timer` sleep would stay on the wall clock and drift from the harness again.
//! `src/time/` itself and test modules (everything from a file's `#[cfg(test)]` on, and
//! `*tests.rs` files) are exempt; comments are skipped.

use std::path::{Path, PathBuf};

/// Every `.rs` file under `dir`, recursively.
fn sources(dir: &Path) -> Vec<PathBuf> {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|error| panic!("{dir:?}: {error}"));
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .flat_map(|path| {
            if path.is_dir() {
                sources(&path)
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                vec![path]
            } else {
                Vec::new()
            }
        })
        .collect()
}

const FORBIDDEN: &[&str] = &["Instant::now", "futures_timer::", "Delay::new"];

fn exempt(path: &Path) -> bool {
    path.components().any(|part| part.as_os_str() == "time")
        || path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().ends_with("tests.rs"))
}

/// The offending lines of one file's non-test code, as `path:line: text`.
fn offences(path: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    text.lines()
        .enumerate()
        .take_while(|(_, line)| !line.trim_start().starts_with("#[cfg(test)]"))
        .filter(|(_, line)| !line.trim_start().starts_with("//"))
        .filter(|(_, line)| FORBIDDEN.iter().any(|needle| line.contains(needle)))
        .map(|(index, line)| format!("{}:{}: {}", path.display(), index + 1, line.trim()))
        .collect()
}

#[test]
fn ds_reads_time_only_through_ds_time() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let found: Vec<String> = sources(&src)
        .into_iter()
        .filter(|path| !exempt(path.strip_prefix(&src).unwrap_or(path)))
        .flat_map(|path| offences(&path))
        .collect();
    assert!(
        found.is_empty(),
        "read the time through ds::time::{{now, since, sleep}} instead:\n{}",
        found.join("\n")
    );
}
