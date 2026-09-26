//! The gallery's command line: `ds-gallery [--page PAGE] [--snapshot DIR] [--level-sheet DIR]`.

use crate::page::Page;
use std::path::PathBuf;

/// What the gallery was asked to do.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Args {
    /// Open on this page (the tokens page when absent); with `--snapshot`, render only it.
    pub page: Option<Page>,
    /// Render every page and state into this directory as a contact sheet, then exit.
    pub snapshot: Option<PathBuf>,
    /// Render the level control's variant and motion sheets into this directory, then exit.
    pub level_sheet: Option<PathBuf>,
}

/// A command line that is not one the gallery understands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgsError(pub String);

impl std::fmt::Display for ArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\nusage: ds-gallery [--page {}] [--snapshot DIR] [--level-sheet DIR]",
            self.0,
            slugs()
        )
    }
}

impl std::error::Error for ArgsError {}

/// Parse the arguments after the program name.
pub fn parse(args: impl Iterator<Item = String>) -> Result<Args, ArgsError> {
    let mut args = args;
    let mut parsed = Args::default();
    while let Some(flag) = args.next() {
        let (name, inline) = match flag.split_once('=') {
            Some((name, value)) => (name.to_owned(), Some(value.to_owned())),
            None => (flag.clone(), None),
        };
        let mut value = |what: &str| {
            inline
                .clone()
                .or_else(|| args.next())
                .ok_or_else(|| ArgsError(format!("{name} needs {what}")))
        };
        match name.as_str() {
            "--page" => {
                let word = value("a page")?;
                let page = page_named(&word)
                    .ok_or_else(|| ArgsError(format!("no page is called {word:?}")))?;
                parsed.page = once(parsed.page, page, "--page")?;
            }
            "--snapshot" => {
                let dir = value("a directory")?;
                if dir.is_empty() {
                    return Err(ArgsError("--snapshot needs a directory".into()));
                }
                parsed.snapshot = once(parsed.snapshot, PathBuf::from(dir), "--snapshot")?;
            }
            "--level-sheet" => {
                let dir = value("a directory")?;
                if dir.is_empty() {
                    return Err(ArgsError("--level-sheet needs a directory".into()));
                }
                parsed.level_sheet = once(parsed.level_sheet, PathBuf::from(dir), "--level-sheet")?;
            }
            _ => return Err(ArgsError(format!("unknown argument {flag:?}"))),
        }
    }
    Ok(parsed)
}

/// `Some(value)` the first time a flag is given; an error the second.
fn once<T>(seen: Option<T>, value: T, flag: &str) -> Result<Option<T>, ArgsError> {
    match seen {
        Some(_) => Err(ArgsError(format!("{flag} given twice"))),
        None => Ok(Some(value)),
    }
}

/// The page whose `--page` word is `word`, compared whole.
pub fn page_named(word: &str) -> Option<Page> {
    Page::ALL.into_iter().find(|page| page.slug() == word)
}

/// Every page word, for the usage line.
fn slugs() -> String {
    Page::ALL
        .iter()
        .map(|page| page.slug())
        .collect::<Vec<_>>()
        .join("|")
}

#[cfg(test)]
mod tests {
    use super::{Args, parse};
    use crate::page::Page;
    use std::path::PathBuf;

    /// A command line and what it parses to: the arguments, or the start of the error.
    type Case = (&'static [&'static str], Result<Args, &'static str>);

    fn args(page: Option<Page>, snapshot: Option<&str>) -> Args {
        Args {
            page,
            snapshot: snapshot.map(PathBuf::from),
            level_sheet: None,
        }
    }

    #[test]
    fn command_lines_parse_or_say_why_not() {
        let cases: Vec<Case> = vec![
            (&[], Ok(args(None, None))),
            (&["--page", "lists"], Ok(args(Some(Page::Lists), None))),
            (
                &["--page=motion-lab"],
                Ok(args(Some(Page::MotionLab), None)),
            ),
            (&["--snapshot", "out"], Ok(args(None, Some("out")))),
            (
                &["--snapshot=target/gallery", "--page", "matrix"],
                Ok(args(Some(Page::Matrix), Some("target/gallery"))),
            ),
            (
                &["--level-sheet", "out"],
                Ok(Args {
                    level_sheet: Some(PathBuf::from("out")),
                    ..args(None, None)
                }),
            ),
            (
                &["--level-sheet", ""],
                Err("--level-sheet needs a directory"),
            ),
            (&["--page"], Err("--page needs a page")),
            (&["--page", "motionlab"], Err("no page is called")),
            (&["--page", "Tokens"], Err("no page is called")),
            (&["--snapshot"], Err("--snapshot needs a directory")),
            (&["--snapshot="], Err("--snapshot needs a directory")),
            (
                &["--page", "type", "--page", "gaps"],
                Err("--page given twice"),
            ),
            (&["--verbose"], Err("unknown argument")),
            (&["tokens"], Err("unknown argument")),
        ];
        for (line, want) in cases {
            let got = parse(line.iter().map(|word| word.to_string()));
            match (&got, &want) {
                (Ok(got), Ok(want)) => assert_eq!(got, want, "{line:?}"),
                (Err(got), Err(start)) => {
                    assert!(got.0.starts_with(start), "{line:?}: {:?}", got.0)
                }
                _ => panic!("{line:?}: got {got:?}, want {want:?}"),
            }
        }
    }

    #[test]
    fn every_page_word_parses_back_to_its_page() {
        for page in Page::ALL {
            let got = parse(["--page".to_string(), page.slug().to_string()].into_iter());
            assert_eq!(got, Ok(args(Some(page), None)), "{page:?}");
        }
    }
}
