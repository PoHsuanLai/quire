//! The fallback: the PDF in a temporary file, opened in the system's viewer.

use super::{PrintError, PrintOutcome};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Write `pdf` to a temporary file named after `title` and open it.
pub(super) fn open(pdf: &[u8], title: &str) -> Result<PrintOutcome, PrintError> {
    let path = std::env::temp_dir().join(file_name(title, unique()));
    std::fs::write(&path, pdf)?;
    let status = opener(&path)
        .status()
        .map_err(|error| PrintError::NoViewer(error.to_string()))?;
    if status.success() {
        Ok(PrintOutcome::Opened(path))
    } else {
        Err(PrintError::NoViewer(format!(
            "the opener exited with {status}"
        )))
    }
}

/// The command that opens `path` in the viewer the system associates with PDFs.
fn opener(path: &Path) -> Command {
    let (program, lead): (&str, &[&str]) = if cfg!(target_os = "macos") {
        ("open", &[])
    } else if cfg!(target_os = "windows") {
        ("cmd", &["/C", "start", ""])
    } else {
        ("xdg-open", &[])
    };
    let mut command = Command::new(program);
    command.args(lead).arg(path);
    command
}

/// A number no other print of this process has used: the process id and the time.
fn unique() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos());
    format!("{}-{nanos}", std::process::id())
}

/// `title` made safe as a file name (letters, digits, `-` and `_` kept; everything else a
/// `-`; at most 60 characters), then `unique` and `.pdf`.
fn file_name(title: &str, unique: String) -> PathBuf {
    let stem: String = title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .take(60)
        .collect();
    let stem = stem.trim_matches('-');
    let stem = if stem.is_empty() { "print" } else { stem };
    PathBuf::from(format!("{stem}-{unique}.pdf"))
}

#[cfg(test)]
mod tests {
    use super::file_name;
    use std::path::PathBuf;

    const CASES: &[(&str, &str)] = &[
        ("Meeting notes", "Meeting-notes-7.pdf"),
        ("../../etc/passwd", "etc-passwd-7.pdf"),
        ("會議紀錄", "會議紀錄-7.pdf"),
        ("", "print-7.pdf"),
        ("///", "print-7.pdf"),
    ];

    #[test]
    fn titles_become_safe_file_names() {
        for &(title, want) in CASES {
            assert_eq!(
                file_name(title, "7".to_owned()),
                PathBuf::from(want),
                "{title:?}"
            );
        }
    }
}
