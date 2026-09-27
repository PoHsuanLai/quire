//! Whether a sheet also refreshes the progress page's tracked pictures
//! (`tools/progress/shots/…`). A render goes only to the directory it was given unless
//! `--progress` asks for the copy (design/29-SIZING.md section 2: a scratch `--snapshot DIR`
//! used to overwrite tracked PNGs as a side effect).

use std::path::{Path, PathBuf};

/// Whether a render is also copied to the progress page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProgressCopy {
    /// Write only to the directory asked for.
    #[default]
    Skip,
    /// Also copy each picture into the progress page's shots (`--progress`).
    Refresh,
}

impl ProgressCopy {
    /// The directories a picture is written to: `dir`, then `progress` when refreshing.
    pub fn places(self, dir: &Path, progress: PathBuf) -> Vec<PathBuf> {
        match self {
            ProgressCopy::Skip => vec![dir.to_path_buf()],
            ProgressCopy::Refresh => vec![dir.to_path_buf(), progress],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProgressCopy;
    use std::path::{Path, PathBuf};

    #[test]
    fn only_an_explicit_refresh_reaches_the_progress_page() {
        let dir = Path::new("scratch/out");
        let progress = PathBuf::from("tools/progress/shots/gallery");
        let cases = [
            (ProgressCopy::Skip, vec![dir.to_path_buf()]),
            (
                ProgressCopy::Refresh,
                vec![dir.to_path_buf(), progress.clone()],
            ),
        ];
        for (copy, want) in cases {
            assert_eq!(copy.places(dir, progress.clone()), want, "{copy:?}");
        }
        assert_eq!(ProgressCopy::default(), ProgressCopy::Skip);
    }
}
