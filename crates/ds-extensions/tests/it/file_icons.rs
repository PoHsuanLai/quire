//! The file-icon resolver: Your changes, then the chosen pack, then the standard set; the lookup
//! order inside a pack; and what a stored choice that is gone or off means.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ds_extensions::{
    ExtensionId, FileFacts, FileIcons, FolderFacts, IconPick, Search, YourChanges,
};
use ds_helpers::mime::{FileName, MimeDb, MimeLayer, MimeType, NameGuess};
use ds_settings::schema::Provided;
use ds_style::icon::file_kind::{ExtLabel, FileKind, FolderKind};

use crate::support::{ICONS, Scratch, install_entries};

const PAPER: &str = "org.example.Paper";
const DOCX: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
const SVG: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>";
/// The PNG signature: the pack lookup reads names, not pictures.
const PNG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// A small database: PDF, a Word file (a ZIP by parent, a document by generic icon), a PNG and a
/// type nobody lists.
fn db() -> Arc<MimeDb> {
    let globs2 = format!(
        "50:application/pdf:*.pdf\n50:{DOCX}:*.docx\n50:image/png:*.png\n\
         50:application/x-foo:*.foo\n"
    );
    let subclasses = format!("{DOCX} application/zip\n");
    let generic_icons = format!("{DOCX}:x-office-document\n");
    Arc::new(MimeDb::from_layers(&[MimeLayer {
        globs2: &globs2,
        subclasses: &subclasses,
        generic_icons: &generic_icons,
        ..MimeLayer::default()
    }]))
}

/// What `icons` draws for `name`, typed by the database.
fn pick(icons: &FileIcons, db: &MimeDb, name: &str, folder: FolderFacts) -> IconPick {
    let file = FileName::new(name).expect("a file name");
    let mime = match db.by_name(&file) {
        NameGuess::Type(mime) => mime,
        _ => MimeType::new("application/octet-stream").expect("a type"),
    };
    icons.of(&FileFacts {
        mime: &mime,
        name: &file,
        folder,
    })
}

fn file(icons: &FileIcons, db: &MimeDb, name: &str) -> IconPick {
    pick(icons, db, name, FolderFacts::NotAFolder)
}

fn artwork(path: &Path) -> IconPick {
    IconPick::Artwork(fs::canonicalize(path).expect("the picture is there"))
}

/// A pack with these picture files (name, bytes).
fn install_pack(root: &Path, id: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let folder = install_entries(root, id, ICONS);
    let dir = folder.join("file-icons");
    fs::create_dir_all(&dir).expect("the pictures folder");
    for (name, bytes) in files {
        fs::write(dir.join(name), bytes).expect("a picture");
    }
    dir
}

fn built(person: &Path, stored: &str, off: &[ExtensionId]) -> ds_extensions::Built {
    let found = Search::new(person, &[]).scan(off);
    FileIcons::build(db(), &found, &Provided::new(stored))
}

#[test]
fn the_standard_set_is_the_kind_of_the_type_and_the_extension_label() {
    let db = db();
    let icons = FileIcons::standard(db.clone());
    let want = |kind, label: &str| IconPick::Standard {
        kind,
        label: ExtLabel::from_extension(label),
    };
    assert_eq!(file(&icons, &db, "Report.pdf"), want(FileKind::Pdf, "pdf"));
    // A ZIP by parent and a document by generic icon: the generic icon wins.
    assert_eq!(
        file(&icons, &db, "Letter.docx"),
        want(FileKind::Document, "docx")
    );
    assert_eq!(file(&icons, &db, "thing.foo"), want(FileKind::Other, "foo"));
    assert_eq!(
        pick(
            &icons,
            &db,
            "Downloads",
            FolderFacts::Folder(FolderKind::Downloads)
        ),
        IconPick::Folder(FolderKind::Downloads)
    );
}

#[test]
fn a_pack_is_asked_by_icon_name_then_generic_icon_then_parents_then_major_type_then_other() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    let svg = SVG.as_bytes();
    let dir = install_pack(
        &person,
        PAPER,
        &[
            ("application-pdf.svg", svg),
            ("x-office-document.svg", svg),
            // A parent the Word file has: the generic icon is asked before it.
            ("application-zip.svg", svg),
            ("image.svg", svg),
            ("other.svg", svg),
            ("folder.svg", svg),
            ("folder-downloads.svg", svg),
        ],
    );
    let db = db();
    let icons = built(&person, PAPER, &[]).icons;

    let drew = |name: &str| file(&icons, &db, name);
    assert_eq!(drew("a.pdf"), artwork(&dir.join("application-pdf.svg")));
    assert_eq!(drew("a.docx"), artwork(&dir.join("x-office-document.svg")));
    // `image-png` and `image-x-generic` are not in the pack; the major type is.
    assert_eq!(drew("a.png"), artwork(&dir.join("image.svg")));
    assert_eq!(drew("a.foo"), artwork(&dir.join("other.svg")));
    let folder = |kind| pick(&icons, &db, "f", FolderFacts::Folder(kind));
    assert_eq!(
        folder(FolderKind::Downloads),
        artwork(&dir.join("folder-downloads.svg"))
    );
    assert_eq!(folder(FolderKind::Music), artwork(&dir.join("folder.svg")));
}

#[test]
fn what_a_pack_lacks_keeps_the_standard_icon_and_a_vector_beats_a_bitmap() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    let dir = install_pack(
        &person,
        PAPER,
        &[
            ("application-pdf.png", &PNG[..]),
            ("application-pdf.svg", SVG.as_bytes()),
            ("notes.txt", &b"not a picture"[..]),
        ],
    );
    let db = db();
    let icons = built(&person, PAPER, &[]).icons;

    assert_eq!(
        file(&icons, &db, "a.pdf"),
        artwork(&dir.join("application-pdf.svg"))
    );
    assert_eq!(
        file(&icons, &db, "a.png"),
        IconPick::Standard {
            kind: FileKind::Image,
            label: ExtLabel::from_extension("png")
        }
    );
    assert_eq!(
        pick(&icons, &db, "d", FolderFacts::Folder(FolderKind::Plain)),
        IconPick::Folder(FolderKind::Plain)
    );
}

#[test]
fn your_changes_beat_the_chosen_pack_for_the_type_they_name_and_nothing_else() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    let svg = SVG.as_bytes();
    let dir = install_pack(
        &person,
        PAPER,
        &[("application-pdf.svg", svg), ("image.svg", svg)],
    );
    let db = db();
    let mime = MimeType::new("application/pdf").expect("a type");
    let mine = scratch.path().join("mine.svg");
    fs::write(&mine, SVG).expect("a picture");
    let changes = YourChanges::in_root(&person);
    changes.set(&db.icon_name(&mime), &mine).expect("sets");

    let icons = built(&person, PAPER, &[]).icons;

    assert_eq!(
        file(&icons, &db, "a.pdf"),
        artwork(&changes.folder().join("file-icons/application-pdf.svg"))
    );
    assert_eq!(file(&icons, &db, "a.png"), artwork(&dir.join("image.svg")));
    // With Standard chosen, the change is still there.
    let standard = built(&person, "standard", &[]).icons;
    assert_eq!(
        file(&standard, &db, "a.pdf"),
        artwork(&changes.folder().join("file-icons/application-pdf.svg"))
    );
}

#[test]
fn only_the_persons_own_copy_of_your_changes_counts() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    let system = scratch.root("usr");
    install_pack(
        &system,
        "user.YourChanges",
        &[("application-pdf.svg", SVG.as_bytes())],
    );
    let found = Search::new(&person, &[system]).scan(&[]);
    let icons = FileIcons::build(db(), &found, &Provided::standard()).icons;

    assert!(matches!(
        file(&icons, &db(), "a.pdf"),
        IconPick::Standard { .. }
    ));
}

#[test]
fn a_stored_pack_that_is_gone_reads_as_standard_and_is_reported() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    install_pack(&person, PAPER, &[("application-pdf.svg", SVG.as_bytes())]);

    let gone = built(&person, "org.example.Gone", &[]);

    assert_eq!(
        gone.stale.as_ref().map(|stale| stale.stored.as_str()),
        Some("org.example.Gone")
    );
    assert!(gone.off.is_none());
    assert!(matches!(
        file(&gone.icons, &db(), "a.pdf"),
        IconPick::Standard { .. }
    ));
}

#[test]
fn a_pack_that_is_turned_off_is_standard_but_not_missing() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    install_pack(&person, PAPER, &[("application-pdf.svg", SVG.as_bytes())]);

    let off = built(&person, PAPER, &[ExtensionId(PAPER.to_owned())]);

    assert!(off.stale.is_some());
    assert_eq!(off.off.as_ref().map(|off| off.id.0.as_str()), Some(PAPER));
    assert!(matches!(
        file(&off.icons, &db(), "a.pdf"),
        IconPick::Standard { .. }
    ));
    // On again, and nothing is reported.
    let on = built(&person, PAPER, &[]);
    assert!(on.stale.is_none() && on.off.is_none());
    assert!(matches!(
        file(&on.icons, &db(), "a.pdf"),
        IconPick::Artwork(_)
    ));
}

#[test]
fn a_picture_over_the_cap_or_a_link_out_of_the_folder_is_left_out() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    let big = vec![b'x'; usize::try_from(ds_extensions::PICTURE_CAP_BYTES).expect("fits") + 1];
    let dir = install_pack(&person, PAPER, &[("application-pdf.svg", big.as_slice())]);
    let outside = scratch.path().join("outside.svg");
    fs::write(&outside, SVG).expect("a picture");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, dir.join("image.svg")).expect("a link");
    let db = db();
    let icons = built(&person, PAPER, &[]).icons;

    assert!(matches!(
        file(&icons, &db, "a.pdf"),
        IconPick::Standard { .. }
    ));
    #[cfg(unix)]
    assert!(matches!(
        file(&icons, &db, "a.png"),
        IconPick::Standard { .. }
    ));
}
