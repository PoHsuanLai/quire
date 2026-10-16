//! A file that is there and cannot be used belongs to the person: hostile, huge, FIFO, directory
//! and unreadable input is never taken for a missing file and never written over, and a save that
//! must happen keeps the old file under a `.bad` name first. Writers racing each other never tear
//! a file.

use crate::support::{Scratch, make_fifo, make_readable, make_unreadable, names};

use ds_settings::{
    AppName, AppearanceFile, ConfigRoot, Fixup, Origin, Percent, Saved, SpacesLoad, SpacesStorage,
};
use ds_style::space::list::{Epoch, NoPark, Spaces, Today};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Barrier};

const FILE: &str = "appearance.toml";

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
struct Note {
    #[serde(default)]
    folder: String,
}

type Set = Spaces<Note, String>;

fn first() -> Set {
    Spaces::first_run(
        [Note { folder: "a".into() }, Note { folder: "b".into() }],
        Note::default,
    )
}

fn kept(_: &mut Set) -> Fixup {
    Fixup::Kept
}

fn storage(scratch: &Scratch) -> SpacesStorage {
    SpacesStorage::new(
        &ConfigRoot::Scratch(scratch.root().to_path_buf()),
        AppName("notes"),
    )
}

fn spaces_dir(scratch: &Scratch) -> PathBuf {
    let dir = scratch.root().join("notes");
    std::fs::create_dir_all(&dir).expect("dir");
    dir
}

fn state_dir(scratch: &Scratch) -> PathBuf {
    let dir = ConfigRoot::Scratch(scratch.root().to_path_buf())
        .state_dir(AppName("notes"))
        .expect("state dir");
    std::fs::create_dir_all(&dir).expect("dir");
    dir
}

/// Boot `storage` and check the damaged file at `path` was reported, left byte for byte as it
/// was, and that the first save keeps it under a `.bad` name before writing the new file.
fn assert_kept_then_saved(scratch: &Scratch, original: &[u8], what: &str) {
    let disk = storage(scratch);
    let path = spaces_dir(scratch).join("spaces.json");
    let (spaces, origin) = disk.boot_spaces(|_| {}, first, kept);
    let Origin::Damaged(damaged) = origin else {
        panic!("{what}: {origin:?}");
    };
    assert_eq!(damaged.path, path, "{what}");
    assert_eq!(spaces.count(), 2, "{what}: runs on the first-run Spaces");
    assert_eq!(
        std::fs::read(&path).expect("still there"),
        original,
        "{what}"
    );
    assert_eq!(names(&spaces_dir(scratch)), ["spaces.json"], "{what}");

    let Saved { kept_at, .. } = disk.save_spaces(&spaces).expect("saves");
    let aside = kept_at.unwrap_or_else(|| panic!("{what}: nothing was kept"));
    assert!(
        aside
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("spaces.json.bad-")),
        "{what}: {aside:?}"
    );
    assert_eq!(std::fs::read(&aside).expect("kept"), original, "{what}");
    assert!(
        matches!(disk.load_spaces::<Note, String>(|_| {}), SpacesLoad::Stored(ref stored) if *stored == spaces),
        "{what}: the new file reads back"
    );
}

#[test]
fn a_damaged_spaces_file_survives_a_boot_and_is_kept_by_the_first_save() {
    const CASES: &[(&str, &[u8])] = &[
        ("not json", b"{ nope"),
        ("empty", b""),
        ("an array", b"[]"),
        ("no spaces", br#"{"spaces":[]}"#),
        ("a wrong type", br#"{"spaces":"x"}"#),
        ("a nul", b"\0"),
        ("not utf-8", &[0xff, 0xfe, 0x00]),
    ];
    for &(what, bytes) in CASES {
        let scratch = Scratch::new();
        std::fs::write(spaces_dir(&scratch).join("spaces.json"), bytes).expect("write");
        assert_kept_then_saved(&scratch, bytes, what);
    }
}

#[test]
fn a_good_spaces_file_is_never_moved_aside() {
    let scratch = Scratch::new();
    let disk = storage(&scratch);
    let (spaces, _) = disk.boot_spaces(|_| {}, first, kept);
    let saved = disk.save_spaces(&spaces).expect("saves");
    assert_eq!(saved.kept_at, None);
    assert_eq!(names(&spaces_dir(&scratch)), ["spaces.json"]);
}

#[test]
fn a_huge_spaces_file_is_damaged_not_read_and_not_overwritten() {
    let scratch = Scratch::new();
    let path = spaces_dir(&scratch).join("spaces.json");
    std::fs::File::create(&path)
        .expect("file")
        .set_len(17 * 1024 * 1024)
        .expect("sparse");
    let (_, origin) = storage(&scratch).boot_spaces(|_| {}, first, kept);
    let Origin::Damaged(damaged) = origin else {
        panic!("{origin:?}");
    };
    assert!(damaged.reason.contains("too large"), "{}", damaged.reason);
    assert_eq!(
        std::fs::metadata(&path).expect("meta").len(),
        17 * 1024 * 1024
    );
}

#[test]
fn a_fifo_or_a_directory_where_spaces_json_belongs_is_damaged_and_never_blocks() {
    let scratch = Scratch::new();
    let path = spaces_dir(&scratch).join("spaces.json");
    if make_fifo(&path) {
        let (_, origin) = storage(&scratch).boot_spaces(|_| {}, first, kept);
        assert!(matches!(origin, Origin::Damaged(_)), "{origin:?}");
        assert!(!path.is_file(), "the FIFO was not replaced");
    }
    let scratch = Scratch::new();
    std::fs::create_dir_all(spaces_dir(&scratch).join("spaces.json")).expect("dir");
    let (_, origin) = storage(&scratch).boot_spaces(|_| {}, first, kept);
    assert!(matches!(origin, Origin::Damaged(_)), "{origin:?}");
    assert!(spaces_dir(&scratch).join("spaces.json").is_dir());
}

#[test]
fn an_unreadable_spaces_file_is_damaged_not_a_first_run() {
    let scratch = Scratch::new();
    let path = spaces_dir(&scratch).join("spaces.json");
    let disk = storage(&scratch);
    let (spaces, _) = disk.boot_spaces(|_| {}, first, kept);
    let before = std::fs::read(&path).expect("written");
    if !make_unreadable(&path) {
        return;
    }
    let (again, origin) = disk.boot_spaces(
        |_| {},
        || -> Set { Spaces::first_run([Note::default()], Note::default) },
        kept,
    );
    assert!(matches!(origin, Origin::Damaged(_)), "{origin:?}");
    assert_eq!(
        again.count(),
        1,
        "the in-memory first run, not the stored two"
    );
    make_readable(&path);
    assert_eq!(std::fs::read(&path).expect("untouched"), before);
    assert_eq!(names(&spaces_dir(&scratch)), ["spaces.json"]);
    assert_eq!(spaces.count(), 2);
}

#[test]
fn a_damaged_today_file_is_kept_by_the_first_save() {
    let scratch = Scratch::new();
    let disk = storage(&scratch);
    let path = state_dir(&scratch).join("today.json");
    std::fs::write(&path, "{ half").expect("write");
    let booted: Today<u32, NoPark> = disk.boot_today(Epoch(0));
    assert!(booted.entries.is_empty());
    assert_eq!(std::fs::read(&path).expect("untouched"), b"{ half");
    let saved = disk.save_today(&booted).expect("saves");
    let aside = saved.kept_at.expect("the old file was kept");
    assert_eq!(std::fs::read(aside).expect("kept"), b"{ half");
}

#[test]
fn writers_racing_for_spaces_json_never_tear_it() {
    const WRITERS: usize = 6;
    let scratch = Scratch::new();
    let disk = Arc::new(storage(&scratch));
    let start = Arc::new(Barrier::new(WRITERS));
    let mut docs: Vec<Set> = Vec::new();
    for n in 0..WRITERS {
        let mut spaces = first();
        spaces.rename(spaces.current().id, format!("writer {n}"));
        docs.push(spaces);
    }
    let handles: Vec<_> = docs
        .iter()
        .cloned()
        .map(|spaces| {
            let (disk, start) = (Arc::clone(&disk), Arc::clone(&start));
            std::thread::spawn(move || {
                start.wait();
                for _ in 0..25 {
                    disk.save_spaces(&spaces).expect("saves");
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("writer");
    }
    let SpacesLoad::Stored(last) = disk.load_spaces::<Note, String>(|_| {}) else {
        panic!("the file is torn");
    };
    assert!(docs.contains(&last), "{last:?}");
    assert_eq!(names(&spaces_dir(&scratch)), ["spaces.json"]);
}

// ---- settings files (`Store`) ----

fn themed(alpha: u8) -> AppearanceFile {
    let mut file = AppearanceFile::default();
    file.appearance.material_tint_alpha = Percent(alpha);
    file
}

#[test]
fn a_huge_settings_file_is_refused_whole_and_left_alone() {
    let scratch = Scratch::new();
    std::fs::create_dir_all(scratch.app_dir()).expect("dir");
    let path = scratch.app_dir().join(FILE);
    std::fs::File::create(&path)
        .expect("file")
        .set_len(2 * 1024 * 1024)
        .expect("sparse");
    let loaded = scratch.store().load::<AppearanceFile>();
    assert_eq!(loaded.value, AppearanceFile::default());
    assert_eq!(loaded.invalid.len(), 1, "{:?}", loaded.invalid);
    assert!(
        loaded.invalid[0].reason.contains("too large"),
        "{:?}",
        loaded.invalid
    );
    assert_eq!(
        std::fs::metadata(&path).expect("meta").len(),
        2 * 1024 * 1024
    );
}

#[test]
fn a_settings_file_with_a_hostile_number_of_keys_is_refused_whole() {
    let scratch = Scratch::new();
    std::fs::create_dir_all(scratch.app_dir()).expect("dir");
    let text: String = (0..6_000).map(|n| format!("key{n} = 1\n")).collect();
    std::fs::write(scratch.app_dir().join(FILE), text).expect("write");
    let started = std::time::Instant::now();
    let loaded = scratch.store().load::<AppearanceFile>();
    assert!(
        started.elapsed().as_secs() < 5,
        "the reader must not go quadratic"
    );
    assert_eq!(loaded.value, AppearanceFile::default());
    assert_eq!(loaded.invalid.len(), 1, "{:?}", loaded.invalid);
    assert!(loaded.unknown.is_empty());
}

#[test]
fn a_fifo_where_a_settings_file_belongs_is_reported_and_never_blocks() {
    let scratch = Scratch::new();
    std::fs::create_dir_all(scratch.app_dir()).expect("dir");
    if !make_fifo(&scratch.app_dir().join(FILE)) {
        return;
    }
    let loaded = scratch.store().load::<AppearanceFile>();
    assert_eq!(loaded.invalid.len(), 1, "{:?}", loaded.invalid);
}

#[test]
fn an_unreadable_settings_file_is_not_a_first_run_and_is_kept_by_the_next_save() {
    let scratch = Scratch::new();
    let store = scratch.store();
    store.save(&themed(33)).expect("saves");
    let path = scratch.app_dir().join(FILE);
    let before = std::fs::read(&path).expect("read");
    if !make_unreadable(&path) {
        return;
    }
    let loaded = store.load::<AppearanceFile>();
    assert_eq!(loaded.invalid.len(), 1, "{:?}", loaded.invalid);
    assert!(
        loaded.invalid[0].reason.contains("cannot be read"),
        "{:?}",
        loaded.invalid
    );
    let saved = store.save(&themed(44)).expect("saves");
    let aside = saved.kept_at.expect("the old file was kept");
    make_readable(&aside);
    assert_eq!(std::fs::read(&aside).expect("kept"), before);
    assert_eq!(store.load::<AppearanceFile>().value, themed(44));
}

#[test]
fn a_broken_settings_file_is_kept_when_a_save_replaces_it() {
    let scratch = Scratch::new();
    std::fs::create_dir_all(scratch.app_dir()).expect("dir");
    std::fs::write(scratch.app_dir().join(FILE), "not toml [[[ = =").expect("write");
    let saved = scratch.store().save(&themed(20)).expect("saves");
    let aside = saved.kept_at.expect("the old file was kept");
    assert_eq!(std::fs::read(aside).expect("kept"), b"not toml [[[ = =");
    // The kept file, the new one and the lock's sidecar.
    assert_eq!(names(&scratch.app_dir()).len(), 3);
    // A readable file with a bad value is not damaged: the save replaces it in place.
    std::fs::write(
        scratch.app_dir().join(FILE),
        "[appearance]\ntheme = \"mauve\"\n",
    )
    .expect("write");
    assert_eq!(
        scratch.store().save(&themed(21)).expect("saves").kept_at,
        None
    );
}

#[test]
fn a_save_keeps_the_files_mode_and_writes_through_a_symlink() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = Scratch::new();
    std::fs::create_dir_all(scratch.app_dir()).expect("dir");
    let real = scratch.root().join("dotfiles-appearance.toml");
    std::fs::write(&real, "").expect("real");
    std::fs::set_permissions(&real, std::fs::Permissions::from_mode(0o640)).expect("chmod");
    let link = scratch.app_dir().join(FILE);
    std::os::unix::fs::symlink(&real, &link).expect("link");
    scratch.store().save(&themed(55)).expect("saves");
    assert!(
        std::fs::symlink_metadata(&link)
            .expect("meta")
            .file_type()
            .is_symlink(),
        "the link stays a link"
    );
    assert_eq!(scratch.store().load::<AppearanceFile>().value, themed(55));
    let mode = std::fs::metadata(&real).expect("meta").permissions().mode() & 0o7777;
    assert_eq!(mode, 0o640);
}

#[test]
fn a_temporary_file_left_by_a_crash_does_not_touch_the_settings_file() {
    let scratch = Scratch::new();
    let store = scratch.store();
    store.save(&themed(70)).expect("saves");
    std::fs::write(
        scratch.app_dir().join(".appearance.toml.1.0.tmp"),
        "[appear",
    )
    .expect("leftover");
    assert_eq!(store.load::<AppearanceFile>().value, themed(70));
    store.save(&themed(71)).expect("saves");
    assert_eq!(store.load::<AppearanceFile>().value, themed(71));
}

#[test]
fn writers_racing_for_a_settings_file_never_tear_it() {
    const WRITERS: u8 = 8;
    let scratch = Scratch::new();
    let store = Arc::new(scratch.store());
    let start = Arc::new(Barrier::new(usize::from(WRITERS)));
    let handles: Vec<_> = (0..WRITERS)
        .map(|n| {
            let (store, start) = (Arc::clone(&store), Arc::clone(&start));
            std::thread::spawn(move || {
                start.wait();
                for _ in 0..30 {
                    store.save(&themed(10 + n)).expect("saves");
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("writer");
    }
    let loaded = store.load::<AppearanceFile>();
    assert!(
        loaded.invalid.is_empty() && loaded.unknown.is_empty(),
        "{loaded:?}"
    );
    assert!(
        (0..WRITERS).any(|n| loaded.value == themed(10 + n)),
        "{:?}",
        loaded.value
    );
    assert_eq!(names(&scratch.app_dir()), [".appearance.toml.lock", FILE]);
}
