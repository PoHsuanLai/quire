//! The fallback: the PDF in a temporary file, opened in the system's viewer.

use super::{PrintError, PrintOutcome};
use std::fs::{DirBuilder, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// The folder under the person's runtime directory (private to them, cleared at logout) that
/// holds the PDFs handed to a viewer.
const STAGING: &str = "quire-print";

/// How long a staged PDF is kept. The viewer may still be reading it after `xdg-open` returns,
/// so the file cannot be removed at once; one day is far past any print, and the next print
/// sweeps what is older.
const KEEP: Duration = Duration::from_secs(24 * 60 * 60);

/// Write `pdf` to a private file named after `title` and open it.
///
/// The file is created exclusively and readable by the person only, in their runtime directory
/// when there is one (else the system temporary directory), so a printed letter is not left
/// world-readable and a link planted at a guessed name is refused instead of followed.
pub(super) fn open(pdf: &[u8], title: &str) -> Result<PrintOutcome, PrintError> {
    let dir = staging_dir();
    private_dir(&dir)?;
    sweep(&dir, SystemTime::now());
    let path = dir.join(file_name(title, &unique()));
    create_private(&path, pdf)?;
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

/// `$XDG_RUNTIME_DIR/quire-print` when the runtime directory is an absolute directory, else the
/// system temporary directory.
fn staging_dir() -> PathBuf {
    match std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|dir| dir.is_absolute() && dir.is_dir())
    {
        Some(runtime) => runtime.join(STAGING),
        None => std::env::temp_dir(),
    }
}

/// `dir`, made if it is missing, readable by the person only.
fn private_dir(dir: &Path) -> std::io::Result<()> {
    let mut builder = DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(dir)
}

/// Create `path`, which must not exist, readable by the person only, holding `bytes`.
fn create_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file: File = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

/// Remove the PDFs in the staging folder that are older than [`KEEP`] at `now`. Only the
/// staging folder is swept: the shared temporary directory is not ours to clean.
fn sweep(dir: &Path, now: SystemTime) {
    if dir.file_name() != Some(std::ffi::OsStr::new(STAGING)) {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let stale = entry
            .metadata()
            .ok()
            .filter(std::fs::Metadata::is_file)
            .and_then(|meta| meta.modified().ok())
            .and_then(|modified| now.duration_since(modified).ok())
            .is_some_and(|age| age > KEEP);
        if stale && path.extension().is_some_and(|ext| ext == "pdf") {
            // Best effort: a file that stays is only clutter in a private folder.
            let _ = std::fs::remove_file(&path);
        }
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
fn file_name(title: &str, unique: &str) -> PathBuf {
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
    use super::{KEEP, STAGING, create_private, file_name, private_dir, sweep};
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ds-blitz-print-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join(STAGING)
    }

    #[cfg(unix)]
    #[test]
    fn a_staged_pdf_is_private_and_never_replaces_a_file_or_follows_a_link() {
        use std::os::unix::fs::PermissionsExt;
        let dir = scratch("private");
        private_dir(&dir).expect("dir");
        let mode =
            |path: &PathBuf| std::fs::metadata(path).expect("meta").permissions().mode() & 0o777;
        assert_eq!(mode(&dir), 0o700);
        let file = dir.join("a.pdf");
        create_private(&file, b"%PDF").expect("created");
        assert_eq!(mode(&file), 0o600);
        // The name is taken: refused, and the first file is intact.
        assert!(create_private(&file, b"other").is_err());
        assert_eq!(std::fs::read(&file).expect("read"), b"%PDF");
        // A link planted at the name is refused, not written through.
        let victim = dir.join("victim");
        std::fs::write(&victim, "mine").expect("victim");
        std::os::unix::fs::symlink(&victim, dir.join("b.pdf")).expect("link");
        assert!(create_private(&dir.join("b.pdf"), b"x").is_err());
        assert_eq!(std::fs::read(&victim).expect("read"), b"mine");
        let _ = std::fs::remove_dir_all(dir.parent().expect("parent"));
    }

    #[test]
    fn only_old_pdfs_in_the_staging_folder_are_swept() {
        let dir = scratch("sweep");
        private_dir(&dir).expect("dir");
        create_private(&dir.join("a.pdf"), b"1").expect("pdf");
        create_private(&dir.join("note.txt"), b"3").expect("note");
        // Two days from now, the file made just now is old; only a PDF is swept.
        sweep(&dir, SystemTime::now() + KEEP + KEEP);
        assert!(!dir.join("a.pdf").exists());
        assert!(dir.join("note.txt").exists());
        // Now: nothing is stale.
        create_private(&dir.join("fresh.pdf"), b"4").expect("fresh");
        sweep(&dir, SystemTime::now());
        assert!(dir.join("fresh.pdf").exists());
        // A folder that is not the staging folder is left alone.
        let shared = dir.parent().expect("parent").to_path_buf();
        create_private(&shared.join("keep.pdf"), b"5").expect("keep");
        sweep(&shared, SystemTime::now() + KEEP + KEEP);
        assert!(shared.join("keep.pdf").exists());
        let _ = std::fs::remove_dir_all(&shared);
    }

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
            assert_eq!(file_name(title, "7"), PathBuf::from(want), "{title:?}");
        }
    }
}
