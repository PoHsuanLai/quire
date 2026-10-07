//! The clock rule, held over the design system's sources (`ds`, `ds-core`, `ds-style`, `ds-motion`,
//! `ds-shell`): the design system reads time only through `ds::base::time::clock` (`now`, `since`, `sleep`), so a harness's virtual clock reaches every timer and
//! every "now". A direct `Instant::now()` (or `.elapsed()` on an `Instant`, which this scan
//! cannot tell from a tween's own `elapsed`, so review catches it) or a
//! `futures_timer` sleep would stay on the wall clock and drift from the harness again.
//! `ds-core`'s `src/time/` itself and test modules (everything from a file's `#[cfg(test)]` on, and
//! `*tests.rs` files) are exempt; comments are skipped.
//!
//! The second half holds the tests: every harness a test builds names its clock. Time in a test
//! is virtual unless the test asserts something about real time (and then says so with
//! `.with_clock(Clock::Wall)` and the reason), so a loaded machine cannot move what a test sees.

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

/// The crates whose sources the rule covers, next to this one.
const CRATES: &[&str] = &["ds", "ds-core", "ds-style", "ds-motion", "ds-shell"];

#[test]
fn ds_reads_time_only_through_ds_time() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let found: Vec<String> = CRATES
        .iter()
        .flat_map(|name| {
            let src = crates.join(name).join("src");
            sources(&src)
                .into_iter()
                .filter(|path| !exempt(path.strip_prefix(&src).unwrap_or(path)))
                .flat_map(|path| offences(&path))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(
        found.is_empty(),
        "read the time through ds::base::time::clock::{{now, since, sleep}} instead:\n{}",
        found.join("\n")
    );
}

/// The test sources the harness-clock rule covers: every crate's `tests/` (`ds-harness`'s own
/// tests drive both clocks on purpose) and the gallery's in-crate tests.
fn test_sources() -> Vec<PathBuf> {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let tests = std::fs::read_dir(&crates)
        .unwrap_or_else(|error| panic!("{crates:?}: {error}"))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.file_name().is_some_and(|name| name != "ds-harness"))
        .map(|path| path.join("tests"))
        .filter(|path| path.is_dir())
        .flat_map(|path| sources(&path));
    let consumer = crates.join("../examples/consumer/tests");
    let gallery = crates.join("ds-gallery/src/tests.rs");
    tests
        .chain(sources(&consumer))
        .chain(gallery.is_file().then_some(gallery))
        .collect()
}

/// Files that still build a harness without naming a clock, each with the lane that owns the fix.
const UNCHOSEN: &[(&str, &str)] = &[];

/// The text with every `//` comment line blanked, so a call named in prose is not a call.
fn code_of(text: &str) -> String {
    text.lines()
        .map(|line| {
            if line.trim_start().starts_with("//") {
                ""
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The text inside the parentheses that open at `open` (a `(`), and the index after the `)`.
fn parens(text: &str, open: usize) -> (&str, usize) {
    let mut depth = 0_usize;
    for (at, byte) in text.bytes().enumerate().skip(open) {
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return (&text[open + 1..at], at + 1);
                }
            }
            _ => {}
        }
    }
    panic!(
        "unbalanced parentheses after {:?}",
        &text[open..text.len().min(open + 60)]
    );
}

/// The top-level comma-separated arguments of a call's argument text.
fn arguments(args: &str) -> Vec<&str> {
    let mut depth = 0_i32;
    let mut start = 0;
    let mut out = Vec::new();
    for (at, byte) in args.bytes().enumerate() {
        match byte {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b',' if depth == 0 => {
                out.push(args[start..at].trim());
                start = at + 1;
            }
            _ => {}
        }
    }
    out.push(args[start..].trim());
    out
}

/// Whether the builder chain that starts at `HarnessConfig::new(` at `at` calls `with_clock`.
fn chain_names_a_clock(text: &str, at: usize) -> bool {
    let (_, mut end) = parens(text, at + "HarnessConfig::new".len());
    loop {
        let rest = text[end..].trim_start();
        let Some(method) = rest.strip_prefix('.') else {
            return false;
        };
        let name: String = method
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if name == "with_clock" {
            return true;
        }
        let open = text.len() - method.len() + name.len();
        if !text[open..].starts_with('(') {
            return false;
        }
        end = parens(text, open).1;
    }
}

/// The calls in one file's code that leave the clock to the default, as `path:line: call`.
fn clockless(path: &Path) -> Vec<String> {
    let text = code_of(&std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path:?}: {e}")));
    let place = |at: usize| {
        format!(
            "{}:{}",
            path.display(),
            text[..at].matches('\n').count() + 1
        )
    };
    let mut found = Vec::new();
    for call in ["Harness::new(", "Harness::try_new("] {
        for (at, _) in text.match_indices(call) {
            let (args, _) = parens(&text, at + call.len() - 1);
            let second = arguments(args).get(1).copied().unwrap_or_default();
            if second != "config" && !second.contains("HarnessConfig") {
                found.push(format!("{}: {call}.., {second})", place(at)));
            }
        }
    }
    for (at, _) in text.match_indices("HarnessConfig::new(") {
        if !chain_names_a_clock(&text, at) {
            found.push(format!(
                "{}: HarnessConfig::new(..) without with_clock",
                place(at)
            ));
        }
    }
    found
}

#[test]
fn every_test_harness_names_its_clock() {
    let found: Vec<String> = test_sources()
        .into_iter()
        .filter(|path| !path.ends_with("tests/it/clock_rule.rs"))
        .filter(|path| !UNCHOSEN.iter().any(|(allowed, _)| path.ends_with(allowed)))
        .flat_map(|path| clockless(&path))
        .collect();
    assert!(
        found.is_empty(),
        "build the harness with HarnessConfig::new(view).with_clock(Clock::Virtual) (or Clock::Wall \
         with the reason, when the test asserts real time):\n{}",
        found.join("\n")
    );
}
