//! The gallery's command line: `ds-gallery [--page PAGE] [--typeface system|editorial]
//! [--snapshot DIR [--scale PERCENT]] [--level-sheet DIR] [--detail-frames DIR]
//! [--accent-sheet DIR] [--progress]`. A sheet is written only to its DIR; `--progress` also
//! copies it into the progress page's tracked shots.

use crate::page::Page;
use crate::progress_copy::ProgressCopy;
use ds::prelude::*;
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
    /// Render the accent candidates' contact sheets into this directory, then exit
    /// (design/03-COLOR.md section 20).
    pub accent_sheet: Option<PathBuf>,
    /// Render the small-state details' frames through each moment into this directory, then
    /// exit (design/26).
    pub detail_frames: Option<PathBuf>,
    /// With `--snapshot`, the device scale in percent (100 when absent; 100 to 300).
    pub scale: Option<u16>,
    /// The typeface the root speaks in (the settings default when absent).
    pub typeface: Option<Typeface>,
    /// Whether `--snapshot`, `--level-sheet` and `--accent-sheet` also refresh the progress
    /// page's tracked pictures (`--progress`; skipped when absent).
    pub progress: ProgressCopy,
}

/// A command line that is not one the gallery understands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgsError(pub String);

impl std::fmt::Display for ArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\nusage: ds-gallery [--page {}] [--snapshot DIR [--scale PERCENT]] [--level-sheet DIR] [--detail-frames DIR] [--accent-sheet DIR] [--progress]",
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
            "--accent-sheet" => {
                let dir = value("a directory")?;
                if dir.is_empty() {
                    return Err(ArgsError("--accent-sheet needs a directory".into()));
                }
                parsed.accent_sheet =
                    once(parsed.accent_sheet, PathBuf::from(dir), "--accent-sheet")?;
            }
            "--detail-frames" => {
                let dir = value("a directory")?;
                if dir.is_empty() {
                    return Err(ArgsError("--detail-frames needs a directory".into()));
                }
                parsed.detail_frames =
                    once(parsed.detail_frames, PathBuf::from(dir), "--detail-frames")?;
            }
            "--scale" => {
                let word = value("a percentage")?;
                let scale = word
                    .parse::<u16>()
                    .ok()
                    .filter(|scale| (100..=300).contains(scale))
                    .ok_or_else(|| ArgsError(format!("--scale takes 100 to 300, not {word:?}")))?;
                parsed.scale = once(parsed.scale, scale, "--scale")?;
            }
            "--typeface" => {
                let word = value("a typeface")?;
                let typeface = Typeface::parse(&word).ok_or_else(|| {
                    ArgsError(format!(
                        "--typeface takes system or editorial, not {word:?}"
                    ))
                })?;
                parsed.typeface = once(parsed.typeface, typeface, "--typeface")?;
            }
            "--progress" => {
                if inline.is_some() {
                    return Err(ArgsError("--progress takes no value".into()));
                }
                if parsed.progress == ProgressCopy::Refresh {
                    return Err(ArgsError("--progress given twice".into()));
                }
                parsed.progress = ProgressCopy::Refresh;
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
    Page::ALL.iter().copied().find(|page| page.slug() == word)
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
    use crate::progress_copy::ProgressCopy;
    use ds::prelude::*;
    use std::path::PathBuf;

    /// A command line and what it parses to: the arguments, or the start of the error.
    type Case = (&'static [&'static str], Result<Args, &'static str>);

    fn args(page: Option<Page>, snapshot: Option<&str>) -> Args {
        Args {
            page,
            snapshot: snapshot.map(PathBuf::from),
            level_sheet: None,
            accent_sheet: None,
            detail_frames: None,
            scale: None,
            typeface: None,
            progress: ProgressCopy::Skip,
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
                &["--snapshot", "out", "--progress"],
                Ok(Args {
                    progress: ProgressCopy::Refresh,
                    ..args(None, Some("out"))
                }),
            ),
            (
                &["--level-sheet", "out", "--progress"],
                Ok(Args {
                    level_sheet: Some(PathBuf::from("out")),
                    progress: ProgressCopy::Refresh,
                    ..args(None, None)
                }),
            ),
            (&["--progress=yes"], Err("--progress takes no value")),
            (&["--progress", "--progress"], Err("--progress given twice")),
            (
                &["--accent-sheet", "shots"],
                Ok(Args {
                    accent_sheet: Some(PathBuf::from("shots")),
                    ..args(None, None)
                }),
            ),
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
            (
                &["--detail-frames", "out"],
                Ok(Args {
                    detail_frames: Some(PathBuf::from("out")),
                    ..args(None, None)
                }),
            ),
            (&["--page"], Err("--page needs a page")),
            (&["--page", "motionlab"], Err("no page is called")),
            (&["--page", "Tokens"], Err("no page is called")),
            (&["--snapshot"], Err("--snapshot needs a directory")),
            (&["--snapshot="], Err("--snapshot needs a directory")),
            (
                &["--page", "type", "--page", "blitz-limits"],
                Err("--page given twice"),
            ),
            (
                &["--snapshot", "out", "--scale", "200"],
                Ok(Args {
                    scale: Some(200),
                    ..args(None, Some("out"))
                }),
            ),
            (
                &["--typeface", "editorial"],
                Ok(Args {
                    typeface: Some(Typeface::Editorial),
                    ..args(None, None)
                }),
            ),
            (&["--typeface", "serif"], Err("--typeface takes system")),
            (&["--scale", "50"], Err("--scale takes 100 to 300")),
            (&["--scale", "2x"], Err("--scale takes 100 to 300")),
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
        for page in Page::ALL.iter().copied() {
            let got = parse(["--page".to_string(), page.slug().to_string()].into_iter());
            assert_eq!(got, Ok(args(Some(page), None)), "{page:?}");
        }
    }
}
