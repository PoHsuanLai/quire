//! The kit's two files through `SpacesStorage` on a scratch tree: a round trip, a corrupt file
//! falling back to the first run, the one-time import, the payload fix-up, and Today's prune.

use crate::support::Scratch;

use ds_settings::{AppName, ConfigRoot, Fixup, Origin, SpacesStorage};
use ds_style::space::list::{Epoch, IDLE, SpaceId, Spaces, Today};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
struct Note {
    #[serde(default)]
    folder: String,
}

type Set = Spaces<Note, String>;

fn storage(scratch: &Scratch) -> SpacesStorage {
    SpacesStorage::new(
        &ConfigRoot::Scratch(scratch.root().to_path_buf()),
        AppName("notes"),
    )
}

fn first() -> Set {
    Spaces::first_run(
        [Note { folder: "a".into() }, Note { folder: "b".into() }],
        Note::default,
    )
}

fn untouched(_: &mut Set) -> Fixup {
    Fixup::Kept
}

#[test]
fn a_first_run_is_written_and_read_back() {
    let scratch = Scratch::new();
    let disk = storage(&scratch);
    let (spaces, origin) = disk.boot_spaces(|_| {}, first, untouched);
    assert_eq!(origin, Origin::FirstRun);
    let (again, origin) = disk.boot_spaces(|_| {}, || panic!("stored"), untouched);
    assert_eq!(origin, Origin::Stored);
    assert_eq!(again, spaces);
}

#[test]
fn a_corrupt_or_empty_file_is_a_first_run_not_a_crash() {
    for text in ["{ nope", "", "[]", r#"{"spaces":[]}"#, "\u{0}"] {
        let scratch = Scratch::new();
        let disk = storage(&scratch);
        let dir = disk_dir(&scratch);
        std::fs::create_dir_all(&dir).expect("dir");
        std::fs::write(dir.join("spaces.json"), text).expect("write");
        let (spaces, origin) = disk.boot_spaces(|_| {}, first, untouched);
        assert_eq!(origin, Origin::FirstRun, "{text:?}");
        assert_eq!(spaces.count(), 2);
    }
}

fn disk_dir(scratch: &Scratch) -> std::path::PathBuf {
    ConfigRoot::Scratch(scratch.root().to_path_buf())
        .dir(AppName("notes"))
        .expect("scratch has a dir")
}

#[test]
fn the_raw_import_and_the_payload_fixup_run_and_a_change_is_saved() {
    let scratch = Scratch::new();
    let disk = storage(&scratch);
    let dir = disk_dir(&scratch);
    std::fs::create_dir_all(&dir).expect("dir");
    std::fs::write(
        dir.join("spaces.json"),
        r#"{"spaces":[{"name":"Old","folder":"x"}]}"#,
    )
    .expect("write");
    let import = |value: &mut serde_json::Value| {
        if let Some(space) = value["spaces"][0].as_object_mut() {
            space.entry("theme").or_insert_with(|| "dark".into());
        }
    };
    let fill = |spaces: &mut Set| {
        spaces.edit(SpaceId(0), |space| space.payload.folder.push('!'));
        Fixup::Changed
    };
    let (spaces, origin) = disk.boot_spaces(import, first, fill);
    assert_eq!(origin, Origin::Stored);
    assert_eq!(spaces.current().payload.folder, "x!");
    assert_eq!(
        spaces.current().look.theme,
        ds_style::appearance::theme::Theme::Dark
    );
    let saved: Set = disk.load_spaces(|_| {}).expect("written");
    assert_eq!(saved, spaces, "the fix-up was written back");
}

#[test]
fn today_lives_in_the_state_directory_and_boots_pruned() {
    let scratch = Scratch::new();
    let disk = storage(&scratch);
    let mut today: Today<u32> = Today::default();
    today.opened(SpaceId(0), 1, Epoch(0));
    today.opened(SpaceId(0), 2, Epoch(IDLE.as_secs() as i64));
    disk.save_today(&today).expect("saves");
    let state = ConfigRoot::Scratch(scratch.root().to_path_buf())
        .state_dir(AppName("notes"))
        .expect("dir");
    assert!(state.join("today.json").exists());
    assert!(!disk_dir(&scratch).join("today.json").exists());
    let booted: Today<u32> = disk.boot_today(Epoch(IDLE.as_secs() as i64 + 5));
    assert_eq!(booted.entries.len(), 1);
    let stored: Today<u32> = disk.load_today();
    assert_eq!(stored, booted, "the prune was written back");
    std::fs::write(state.join("today.json"), "garbage").expect("write");
    assert_eq!(
        disk.load_today::<u32, ds_style::space::list::NoPark>()
            .entries
            .len(),
        0
    );
}

#[test]
fn nowhere_to_write_keeps_the_spaces_for_the_session() {
    let disk = SpacesStorage::at(None, None);
    let (spaces, origin) = disk.boot_spaces(|_| {}, first, untouched);
    assert_eq!((spaces.count(), origin), (2, Origin::FirstRun));
    assert!(disk.save_spaces(&spaces).is_err());
}
