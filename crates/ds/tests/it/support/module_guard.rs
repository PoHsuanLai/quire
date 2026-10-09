//! The include guard of a crate's `tests/it` directory: a test file that `main.rs` does not
//! declare as a module is never compiled, so its tests never run and nothing says so. Every
//! crate's `main.rs` includes this file and checks its own directory with it.

use std::fs;
use std::path::Path;

/// The module `line` declares, if it declares one: `mod name;`, `pub mod name;`, `mod name {`.
fn declared_by(line: &str) -> Option<&str> {
    let rest = line.trim_start();
    let rest = rest.strip_prefix("pub(crate) ").unwrap_or(rest);
    let rest = rest.strip_prefix("pub ").unwrap_or(rest);
    let rest = rest.strip_prefix("mod ")?;
    rest.split([';', ' ', '{']).next()
}

/// The files and directories of `dir` (a `tests/it`) that `dir/main.rs` leaves out: a `name.rs`
/// or a `name/mod.rs` with no `mod name`. Directories without a `mod.rs` hold data, or files a
/// `#[path]` module includes, and are not modules.
pub fn undeclared(dir: &Path) -> Vec<String> {
    let main = fs::read_to_string(dir.join("main.rs")).unwrap_or_default();
    let declared: Vec<&str> = main.lines().filter_map(declared_by).collect();
    let Ok(entries) = fs::read_dir(dir) else {
        return vec![format!("{} cannot be read", dir.display())];
    };
    let mut left_out: Vec<String> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let name = path.file_name()?.to_str()?.to_owned();
            let module = match path.is_dir() {
                true => path.join("mod.rs").exists().then_some(name.clone())?,
                false => name
                    .strip_suffix(".rs")
                    .filter(|stem| *stem != "main")?
                    .to_owned(),
            };
            (!declared.contains(&module.as_str())).then_some(name)
        })
        .collect();
    left_out.sort();
    left_out
}
