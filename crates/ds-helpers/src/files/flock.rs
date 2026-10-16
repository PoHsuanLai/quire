//! A lock between programs: an `flock` on a sidecar file beside the one being changed.

use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;
use std::time::Duration;

/// The longest a writer waits for another program's turn. A turn is one read and one rename, so
/// a few seconds means the holder is wedged; the waiter gives up instead of wedging too.
const LOCK_WAIT: Duration = Duration::from_secs(5);

/// How often a waiting writer asks again.
#[cfg(unix)]
const LOCK_POLL: Duration = Duration::from_millis(10);

/// The exclusive lock on one sidecar file, held for as long as this value lives (closing the file
/// releases it, also when the process dies).
#[derive(Debug)]
pub(super) struct Flock(File);

impl Flock {
    /// Create the sidecar at `path` if it is not there and take its exclusive lock, waiting up to
    /// [`LOCK_WAIT`]. The sidecar stays on disk afterwards: removing it would let a second writer
    /// lock a new file while the first still holds the old one.
    pub(super) fn exclusive(path: &Path) -> io::Result<Flock> {
        let file = open(path)?;
        wait(&file)?;
        Ok(Flock(file))
    }
}

fn open(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

#[cfg(unix)]
fn wait(file: &File) -> io::Result<()> {
    use rustix::fs::{FlockOperation, flock};
    use rustix::io::Errno;
    use std::time::Instant;

    let give_up = Instant::now() + LOCK_WAIT;
    loop {
        match flock(file, FlockOperation::NonBlockingLockExclusive) {
            Ok(()) => return Ok(()),
            Err(Errno::WOULDBLOCK | Errno::INTR) if Instant::now() < give_up => {
                std::thread::sleep(LOCK_POLL);
            }
            Err(Errno::WOULDBLOCK) => return Err(io::ErrorKind::TimedOut.into()),
            Err(other) => return Err(other.into()),
        }
    }
}

#[cfg(not(unix))]
fn wait(file: &File) -> io::Result<()> {
    // No `flock` here: writers in one process still take turns (see `lock`).
    let _ = (file, LOCK_WAIT);
    Ok(())
}
