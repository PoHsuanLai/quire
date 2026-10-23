//! Reading a file with a size cap, from a regular file only.

use crate::files::cap::ReadCap;
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;

/// Why a file was not read. [`ReadError::NotFound`] is the only variant that means the file is
/// absent; every other one means something is there that this read could not take, and a caller
/// must not treat it as absent (and so must not overwrite it with defaults).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ReadError {
    /// There is no file at the path.
    #[error("{path}: no such file")]
    NotFound {
        /// Which path.
        path: PathBuf,
    },
    /// The path is a directory, a FIFO, a device or a socket, not a regular file.
    #[error("{path}: not a regular file")]
    NotRegular {
        /// Which path.
        path: PathBuf,
    },
    /// The file is larger than the cap the caller chose.
    #[error("{path}: larger than {cap}")]
    TooBig {
        /// Which path.
        path: PathBuf,
        /// The cap it exceeded.
        cap: ReadCap,
    },
    /// The file is not UTF-8 (only [`read_text_capped`] reports this).
    #[error("{path}: not valid text: {source}")]
    NotText {
        /// Which path.
        path: PathBuf,
        /// Where the text stops being UTF-8.
        source: FromUtf8Error,
    },
    /// The file exists and the system refused the read (permissions, too many open files, a
    /// failing disk).
    #[error("{path}: {source}")]
    Io {
        /// Which path.
        path: PathBuf,
        /// Why.
        source: io::Error,
    },
}

impl ReadError {
    /// Whether the file is simply absent.
    pub fn is_missing(&self) -> bool {
        matches!(self, ReadError::NotFound { .. })
    }

    fn at(path: &Path, source: io::Error) -> Self {
        match source.kind() {
            io::ErrorKind::NotFound => ReadError::NotFound {
                path: path.to_path_buf(),
            },
            _ => ReadError::Io {
                path: path.to_path_buf(),
                source,
            },
        }
    }
}

/// `O_NONBLOCK` on the systems where the value is known, so opening a FIFO that nobody writes to
/// returns at once. Elsewhere 0: the check for a regular file before opening still catches a
/// FIFO that is there when the read starts.
#[cfg(unix)]
const NONBLOCK: i32 = if cfg!(any(target_os = "linux", target_os = "android")) {
    if cfg!(any(
        target_arch = "mips",
        target_arch = "mips64",
        target_arch = "sparc",
        target_arch = "sparc64"
    )) {
        0
    } else {
        0o4000
    }
} else if cfg!(any(
    target_vendor = "apple",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
)) {
    0x4
} else {
    0
};

fn open(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(NONBLOCK);
    }
    options.open(path)
}

/// The bytes of the regular file at `path`, at most `cap` of them.
///
/// The file is checked to be regular before it is opened and again once it is open (a path can be
/// swapped between the two), and it is opened without blocking, so a FIFO or a device at the path
/// is [`ReadError::NotRegular`] rather than a hang. A file that grows past the cap while being
/// read is refused too.
pub fn read_capped(path: &Path, cap: ReadCap) -> Result<Vec<u8>, ReadError> {
    let at = |source: io::Error| ReadError::at(path, source);
    let named = std::fs::metadata(path).map_err(at)?;
    if !named.is_file() {
        return Err(ReadError::NotRegular {
            path: path.to_path_buf(),
        });
    }
    let file = open(path).map_err(at)?;
    let opened = file.metadata().map_err(at)?;
    if !opened.is_file() {
        return Err(ReadError::NotRegular {
            path: path.to_path_buf(),
        });
    }
    let too_big = || ReadError::TooBig {
        path: path.to_path_buf(),
        cap,
    };
    if opened.len() > cap.limit() {
        return Err(too_big());
    }
    let mut bytes = Vec::new();
    file.take(cap.limit().saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(at)?;
    match u64::try_from(bytes.len()) {
        Ok(len) if len <= cap.limit() => Ok(bytes),
        _ => Err(too_big()),
    }
}

/// [`read_capped`] as UTF-8 text.
pub fn read_text_capped(path: &Path, cap: ReadCap) -> Result<String, ReadError> {
    String::from_utf8(read_capped(path, cap)?).map_err(|source| ReadError::NotText {
        path: path.to_path_buf(),
        source,
    })
}
