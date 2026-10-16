//! The capped reader and the durable atomic writer on scratch directories: hostile, huge, FIFO
//! and unreadable input, a write that fails or is cut short never tearing the target, and
//! writers racing for one path.

use ds_helpers::{
    NewFileMode, ReadCap, ReadError, UnixSeconds, move_aside, read_capped, read_text_capped,
    write_atomic,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Barrier};
use std::time::Duration;

/// One test's own directory under the system temp dir, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("ds-helpers-files-{}-{n}-{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("scratch dir");
        Scratch(path)
    }

    fn file(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    /// Every entry in the directory, by name.
    fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(&self.0)
            .expect("list")
            .map(|entry| {
                entry
                    .expect("entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();
        names
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const SMALL: ReadCap = ReadCap::bytes(8);

#[test]
fn a_missing_file_is_not_found_and_nothing_else_is() {
    let scratch = Scratch::new("missing");
    let error = read_capped(&scratch.file("none"), SMALL).expect_err("absent");
    assert!(error.is_missing(), "{error}");

    // A directory, a loop of links and a file under a file all exist in some sense: none is
    // "missing".
    fs::create_dir(scratch.file("dir")).expect("dir");
    std::os::unix::fs::symlink(scratch.file("loop"), scratch.file("loop")).expect("link");
    fs::write(scratch.file("plain"), "x").expect("file");
    for name in ["dir", "loop", "plain/under"] {
        let error = read_capped(&scratch.file(name), SMALL).expect_err(name);
        assert!(!error.is_missing(), "{name}: {error}");
    }
}

#[test]
fn a_file_at_the_cap_is_read_and_one_byte_over_is_refused() {
    let scratch = Scratch::new("cap");
    fs::write(scratch.file("at"), "12345678").expect("file");
    fs::write(scratch.file("over"), "123456789").expect("file");
    assert_eq!(
        read_capped(&scratch.file("at"), SMALL).expect("at the cap"),
        b"12345678"
    );
    assert!(matches!(
        read_capped(&scratch.file("over"), SMALL),
        Err(ReadError::TooBig { cap, .. }) if cap == SMALL
    ));
}

#[test]
fn a_huge_file_is_refused_without_being_read() {
    let scratch = Scratch::new("huge");
    // A sparse file: a terabyte long, no blocks. Reading it whole would never finish.
    let file = fs::File::create(scratch.file("huge")).expect("file");
    file.set_len(1 << 40).expect("sparse");
    let result = read_capped(&scratch.file("huge"), ReadCap::mib(1));
    assert!(
        matches!(result, Err(ReadError::TooBig { .. })),
        "{result:?}"
    );
}

#[test]
fn a_fifo_and_a_device_are_refused_instead_of_blocking() {
    let scratch = Scratch::new("fifo");
    let made = std::process::Command::new("mkfifo")
        .arg(scratch.file("pipe"))
        .status();
    if made.is_ok_and(|status| status.success()) {
        // No writer ever opens it: a blocking open would hang the test.
        let result = read_capped(&scratch.file("pipe"), SMALL);
        assert!(
            matches!(result, Err(ReadError::NotRegular { .. })),
            "{result:?}"
        );
    }
    let device = read_capped(Path::new("/dev/zero"), ReadCap::mib(1));
    assert!(
        matches!(device, Err(ReadError::NotRegular { .. })),
        "{device:?}"
    );
}

#[test]
fn text_that_is_not_utf8_is_told_apart() {
    let scratch = Scratch::new("text");
    fs::write(scratch.file("bytes"), [0xff, 0xfe, 0x00]).expect("file");
    fs::write(scratch.file("words"), "héllo").expect("file");
    assert!(matches!(
        read_text_capped(&scratch.file("bytes"), SMALL),
        Err(ReadError::NotText { .. })
    ));
    assert_eq!(
        read_text_capped(&scratch.file("words"), SMALL).expect("text"),
        "héllo"
    );
}

#[test]
fn a_file_the_system_refuses_is_an_io_error_never_missing() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = Scratch::new("locked");
    let path = scratch.file("locked");
    fs::write(&path, "secret").expect("file");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("chmod");
    if fs::File::open(&path).is_ok() {
        // Running as a user the mode does not bind (root): nothing to prove here.
        return;
    }
    let error = read_capped(&path, SMALL).expect_err("unreadable");
    assert!(matches!(error, ReadError::Io { .. }), "{error:?}");
    assert!(!error.is_missing());
}

#[test]
fn a_new_file_is_private_when_asked_and_an_existing_one_keeps_its_mode() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = Scratch::new("mode");
    let mode = |name: &str| {
        fs::metadata(scratch.file(name))
            .expect("meta")
            .permissions()
            .mode()
            & 0o7777
    };
    write_atomic(&scratch.file("private"), b"a", NewFileMode::OwnerOnly).expect("write");
    assert_eq!(mode("private"), 0o600);

    fs::write(scratch.file("shared"), "old").expect("file");
    fs::set_permissions(scratch.file("shared"), fs::Permissions::from_mode(0o640)).expect("chmod");
    write_atomic(&scratch.file("shared"), b"new", NewFileMode::OwnerOnly).expect("write");
    assert_eq!(mode("shared"), 0o640);
    assert_eq!(fs::read(scratch.file("shared")).expect("read"), b"new");
}

#[test]
fn a_symlinked_file_is_written_through_and_stays_a_link() {
    let scratch = Scratch::new("link");
    fs::create_dir(scratch.file("dotfiles")).expect("dir");
    fs::write(scratch.file("dotfiles/real"), "old").expect("file");
    std::os::unix::fs::symlink("dotfiles/real", scratch.file("settings")).expect("link");
    write_atomic(&scratch.file("settings"), b"new", NewFileMode::Umask).expect("write");
    let meta = fs::symlink_metadata(scratch.file("settings")).expect("meta");
    assert!(meta.file_type().is_symlink());
    assert_eq!(
        fs::read(scratch.file("dotfiles/real")).expect("read"),
        b"new"
    );
    assert!(
        !scratch.names().iter().any(|name| name.ends_with(".tmp")),
        "{:?}",
        scratch.names()
    );
}

#[test]
fn a_write_replaces_the_inode_so_a_reader_of_the_old_file_never_sees_a_mixture() {
    let scratch = Scratch::new("inode");
    let path = scratch.file("doc");
    fs::write(&path, "old old old").expect("file");
    let mut old = fs::File::open(&path).expect("open");
    write_atomic(&path, b"new", NewFileMode::Umask).expect("write");
    let mut seen = String::new();
    std::io::Read::read_to_string(&mut old, &mut seen).expect("read");
    assert_eq!(seen, "old old old");
    assert_eq!(fs::read(&path).expect("read"), b"new");
}

#[test]
fn a_failed_write_leaves_the_target_alone_and_cleans_its_temporary_file() {
    let scratch = Scratch::new("fail");
    // The target is a non-empty directory: the rename must fail after the bytes were written.
    fs::create_dir(scratch.file("target")).expect("dir");
    fs::write(scratch.file("target/keep"), "keep").expect("file");
    let error = write_atomic(&scratch.file("target"), b"new", NewFileMode::Umask);
    assert!(error.is_err());
    assert_eq!(scratch.names(), ["target"]);
    assert_eq!(
        fs::read(scratch.file("target/keep")).expect("read"),
        b"keep"
    );
}

#[test]
fn a_temporary_file_left_by_a_crash_does_not_touch_the_target_or_block_the_next_write() {
    let scratch = Scratch::new("crash");
    let path = scratch.file("doc");
    write_atomic(&path, b"good", NewFileMode::Umask).expect("write");
    // What a process killed between the temporary write and the rename leaves behind.
    fs::write(scratch.file(".doc.1.0.tmp"), "half writ").expect("leftover");
    assert_eq!(fs::read(&path).expect("read"), b"good");
    write_atomic(&path, b"better", NewFileMode::Umask).expect("write");
    assert_eq!(fs::read(&path).expect("read"), b"better");
}

#[test]
fn writers_racing_for_one_path_never_publish_a_torn_file() {
    const WRITERS: u8 = 8;
    const ROUNDS: u8 = 40;
    const LEN: usize = 16 * 1024;
    let scratch = Scratch::new("race");
    let path = Arc::new(scratch.file("doc"));
    write_atomic(&path, &[b'a'; LEN], NewFileMode::Umask).expect("write");
    let start = Arc::new(Barrier::new(usize::from(WRITERS) + 1));
    let writers: Vec<_> = (0..WRITERS)
        .map(|n| {
            let (path, start) = (Arc::clone(&path), Arc::clone(&start));
            std::thread::spawn(move || {
                start.wait();
                for _ in 0..ROUNDS {
                    write_atomic(&path, &[b'b' + n; LEN], NewFileMode::Umask).expect("write");
                }
            })
        })
        .collect();
    let reader = {
        let (path, start) = (Arc::clone(&path), Arc::clone(&start));
        std::thread::spawn(move || {
            start.wait();
            for _ in 0..200 {
                let bytes = read_capped(&path, ReadCap::mib(1)).expect("read");
                assert_eq!(bytes.len(), LEN, "a short file was published");
                assert!(
                    bytes.iter().all(|&b| b == bytes[0]),
                    "a mixed file was published"
                );
                std::thread::sleep(Duration::from_micros(200));
            }
        })
    };
    for writer in writers {
        writer.join().expect("writer");
    }
    reader.join().expect("reader");
    assert_eq!(scratch.names(), ["doc"]);
}

#[test]
fn a_damaged_file_is_moved_aside_under_a_timestamped_name_and_never_overwritten() {
    let scratch = Scratch::new("aside");
    let path = scratch.file("spaces.json");
    fs::write(&path, "{ half").expect("file");
    let first = move_aside(&path, UnixSeconds(1_700_000_000)).expect("aside");
    assert_eq!(first, scratch.file("spaces.json.bad-1700000000"));
    assert!(!path.exists());
    assert_eq!(fs::read(&first).expect("read"), b"{ half");

    // The same second again does not overwrite the first copy.
    fs::write(&path, "other").expect("file");
    let second = move_aside(&path, UnixSeconds(1_700_000_000)).expect("aside");
    assert_eq!(second, scratch.file("spaces.json.bad-1700000000-1"));
    assert_eq!(fs::read(&first).expect("read"), b"{ half");
}
