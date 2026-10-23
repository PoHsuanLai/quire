//! "Your changes": the person's own Extension that holds one picture per file type.

use std::fs;
use std::path::Path;

use ds_extensions::{
    Capability, ChangeError, Removal, Search, YOUR_CHANGES_ID, YOUR_CHANGES_NAME, YourChanges,
    choices_for,
};
use ds_helpers::mime::{IconName, MimeDb, MimeLayer, MimeType};

use crate::support::{ICONS, Scratch, install_entries, write};

const SVG: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\"><circle r=\"4\"/></svg>";
const PNG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

fn icon(mime: &str) -> IconName {
    MimeDb::default().icon_name(&MimeType::new(mime).expect("a type"))
}

fn picture(scratch: &Scratch, name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let path = scratch.path().join(name);
    fs::write(&path, bytes).expect("a picture");
    path
}

#[test]
fn setting_a_picture_copies_it_under_the_icon_name_and_writes_the_manifest() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    let changes = YourChanges::in_root(&person);
    let mine = picture(&scratch, "mine.svg", SVG.as_bytes());

    changes.set(&icon("application/pdf"), &mine).expect("sets");

    let copied = changes.folder().join("file-icons/application-pdf.svg");
    assert_eq!(fs::read_to_string(copied).expect("copied"), SVG);
    // The Extension is found by the same scan as any other, under its name.
    let found = Search::new(&person, &[]).scan(&[]);
    let resolved = found.resolve(Capability::FileIcons);
    let installed = resolved.registry.installed();
    assert_eq!(installed.len(), 1);
    assert_eq!(installed[0].manifest.id.as_str(), YOUR_CHANGES_ID);
    assert_eq!(installed[0].manifest.name, YOUR_CHANGES_NAME);
    assert!(resolved.problems().is_empty());
}

#[test]
fn a_new_picture_for_a_type_replaces_the_old_one_in_either_format() {
    let scratch = Scratch::new();
    let changes = YourChanges::in_root(&scratch.root("home"));
    let pdf = icon("application/pdf");
    changes
        .set(&pdf, &picture(&scratch, "a.svg", SVG.as_bytes()))
        .expect("sets");
    changes
        .set(&pdf, &picture(&scratch, "b.png", &PNG))
        .expect("replaces");

    let dir = changes.folder().join("file-icons");
    assert!(!dir.join("application-pdf.svg").exists());
    assert!(dir.join("application-pdf.png").exists());
    assert_eq!(changes.changed().expect("lists"), ["application-pdf"]);
}

#[test]
fn clearing_a_change_takes_its_picture_away_and_a_type_with_none_is_left_alone() {
    let scratch = Scratch::new();
    let changes = YourChanges::in_root(&scratch.root("home"));
    let pdf = icon("application/pdf");
    let zip = icon("application/zip");
    changes
        .set(&pdf, &picture(&scratch, "a.svg", SVG.as_bytes()))
        .expect("sets");
    changes
        .set(&zip, &picture(&scratch, "b.svg", SVG.as_bytes()))
        .expect("sets");

    changes.clear(&pdf).expect("clears");
    changes.clear(&icon("image/png")).expect("nothing to clear");

    assert_eq!(changes.changed().expect("lists"), ["application-zip"]);
}

#[test]
fn it_cannot_be_removed_while_it_holds_a_change() {
    let scratch = Scratch::new();
    let changes = YourChanges::in_root(&scratch.root("home"));
    // Nothing set yet: nothing to lose.
    assert_eq!(changes.removal(), Removal::Allowed);
    let pdf = icon("application/pdf");
    changes
        .set(&pdf, &picture(&scratch, "a.svg", SVG.as_bytes()))
        .expect("sets");
    assert_eq!(changes.removal(), Removal::Blocked);
    changes.clear(&pdf).expect("clears");
    assert_eq!(changes.removal(), Removal::Allowed);
}

#[test]
fn a_file_that_is_not_a_picture_is_refused_and_nothing_is_written() {
    let scratch = Scratch::new();
    let changes = YourChanges::in_root(&scratch.root("home"));
    let pdf = icon("application/pdf");
    let refused = [
        picture(&scratch, "a.txt", b"text"),
        picture(&scratch, "b.png", b"not a png"),
        picture(&scratch, "c.svg", b"<svg><script>alert(1)</script></svg>"),
        picture(
            &scratch,
            "d.svg",
            b"<svg><image href=\"http://x/y.png\"/></svg>",
        ),
        picture(&scratch, "e.svg", b"just text"),
    ];
    for path in &refused {
        let error = changes.set(&pdf, path).expect_err("refused");
        assert!(matches!(error, ChangeError::NotAPicture { .. }), "{path:?}");
    }
    assert!(!changes.folder().exists());
}

#[test]
fn a_missing_picture_and_an_icon_name_that_cannot_be_a_file_name_are_errors() {
    let scratch = Scratch::new();
    let changes = YourChanges::in_root(&scratch.root("home"));
    let mine = picture(&scratch, "mine.svg", SVG.as_bytes());

    let missing = changes.set(&icon("application/pdf"), Path::new("/nonexistent/x.svg"));
    assert!(matches!(missing, Err(ChangeError::Read(_))));

    let layer = MimeLayer {
        icons: "application/x-evil:../evil\n",
        ..MimeLayer::default()
    };
    let db = MimeDb::from_layers(&[layer]);
    let evil = db.icon_name(&MimeType::new("application/x-evil").expect("a type"));
    assert!(matches!(
        changes.set(&evil, &mine),
        Err(ChangeError::BadName { .. })
    ));
}

#[test]
fn it_is_applied_on_top_and_never_offered_as_a_choice() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    install_entries(&person, "org.example.Paper", ICONS);
    write(
        &person.join("quire/extensions/org.example.Paper/file-icons/image.svg"),
        SVG,
    );
    let changes = YourChanges::in_root(&person);
    changes
        .set(
            &icon("application/pdf"),
            &picture(&scratch, "mine.svg", SVG.as_bytes()),
        )
        .expect("sets");

    let resolved = Search::new(&person, &[])
        .scan(&[])
        .resolve(Capability::FileIcons);
    let choices = choices_for(Capability::FileIcons, resolved.registry.installed());

    assert_eq!(resolved.registry.installed().len(), 2);
    assert_eq!(choices.words(), ["standard", "org.example.Paper"]);
}
