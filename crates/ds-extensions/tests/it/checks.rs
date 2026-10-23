//! The checks before anything is used: paths stay in the folder, files stay under their caps, and
//! an Extension that fails is listed with a plain reason and never half-used.

use std::fs;

use bayonet::manifest::ManifestError;
use ds_extensions::{Capability, Fault, STYLESHEET_CAP_BYTES, Search, Trouble};

use crate::support::{
    ICONS, LOOK, Scratch, install, install_entries, install_look, manifest, write,
};

const ID: &str = "org.example.Paper";
const LOOK_WORDS: &str = "Some files in this Extension could not be read";
const ICON_WORDS: &str = "Some pictures in this Extension could not be read";

fn usable(found: &ds_extensions::Found, capability: Capability) -> Vec<String> {
    found
        .resolve(capability)
        .registry
        .installed()
        .iter()
        .map(|installed| installed.manifest.id.as_str().to_owned())
        .collect()
}

#[test]
fn a_stylesheet_that_climbs_out_of_the_folder_is_unreadable_and_listed_in_plain_words() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    write(&scratch.path().join("outside.css"), ".ds {}\n");
    let entry = LOOK.replace("look.css", "../../../../outside.css");
    install(&person, ID, &manifest(ID, "Paper", "1.0.0", &entry));

    let found = Search::new(&person, &[]).scan(&[]);
    let resolved = found.resolve(Capability::Look);

    assert!(usable(&found, Capability::Look).is_empty());
    assert_eq!(resolved.unreadable.len(), 1);
    assert!(matches!(
        resolved.unreadable[0].reason,
        ManifestError::PathEscapes { .. }
    ));
    let problems = resolved.problems();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].id.0, ID);
    assert_eq!(problems[0].trouble, Trouble::FilesUnreadable);
    assert_eq!(problems[0].words(), LOOK_WORDS);
}

#[test]
fn an_absolute_path_and_a_link_out_of_the_folder_are_refused_too() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    let outside = scratch.path().join("outside.css");
    write(&outside, ".ds {}\n");

    let absolute = LOOK.replace("look.css", &outside.display().to_string());
    install_entries(&person, "org.example.Absolute", &absolute);

    let linked = install_entries(&person, "org.example.Linked", LOOK);
    std::os::unix::fs::symlink(&outside, linked.join("look.css")).expect("a link");

    let found = Search::new(&person, &[]).scan(&[]);
    let resolved = found.resolve(Capability::Look);

    assert!(usable(&found, Capability::Look).is_empty());
    let named: Vec<String> = resolved.problems().iter().map(|p| p.id.0.clone()).collect();
    assert_eq!(named, ["org.example.Absolute", "org.example.Linked"]);
}

#[test]
fn a_stylesheet_over_its_cap_sets_the_extension_aside() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    let fits = install_look(&person, "org.example.Fits", "Fits", "1.0.0");
    let big = install_look(&person, "org.example.Big", "Big", "1.0.0");
    let size = usize::try_from(STYLESHEET_CAP_BYTES).expect("a cap that fits");
    write(&fits.join("look.css"), &" ".repeat(size));
    write(&big.join("look.css"), &" ".repeat(size + 1));

    let found = Search::new(&person, &[]).scan(&[]);
    let resolved = found.resolve(Capability::Look);

    assert_eq!(usable(&found, Capability::Look), ["org.example.Fits"]);
    let aside = resolved.registry.unusable();
    assert_eq!(aside.len(), 1);
    assert_eq!(aside[0].id.as_str(), "org.example.Big");
    assert!(matches!(
        aside[0].reason,
        ManifestError::Provision(Fault::TooBig { .. })
    ));
    assert_eq!(resolved.problems()[0].words(), LOOK_WORDS);
}

#[test]
fn a_missing_sheet_or_one_that_is_not_css_is_set_aside() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    install_entries(&person, "org.example.Missing", LOOK);
    let text = LOOK.replace("look.css", "look.txt");
    let plain = install_entries(&person, "org.example.Text", &text);
    write(&plain.join("look.txt"), ".ds {}\n");

    let found = Search::new(&person, &[]).scan(&[]);

    assert!(usable(&found, Capability::Look).is_empty());
    assert_eq!(found.resolve(Capability::Look).problems().len(), 2);
}

#[test]
fn an_entry_format_newer_than_this_crate_says_it_needs_a_newer_desktop() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    let text = manifest(ID, "Paper", "1.0.0", LOOK).replace("protocol = 1", "protocol = 2");
    let folder = install(&person, ID, &text);
    write(&folder.join("look.css"), ".ds {}\n");

    let resolved = Search::new(&person, &[])
        .scan(&[])
        .resolve(Capability::Look);

    let problems = resolved.problems();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].trouble, Trouble::NeedsNewer);
    assert_eq!(
        problems[0].words(),
        "This Extension needs a newer version of the desktop"
    );
}

#[test]
fn a_file_icons_folder_must_be_a_folder_inside_the_extension() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    let good = install_entries(&person, "org.example.Good", ICONS);
    write(&good.join("file-icons").join("text.png"), "png");
    install_entries(&person, "org.example.Absent", ICONS);
    let plain = install_entries(&person, "org.example.File", ICONS);
    write(&plain.join("file-icons"), "not a folder");
    let outside = scratch.root("outside");
    let linked = install_entries(&person, "org.example.Linked", ICONS);
    std::os::unix::fs::symlink(&outside, linked.join("file-icons")).expect("a link");

    let found = Search::new(&person, &[]).scan(&[]);
    let resolved = found.resolve(Capability::FileIcons);

    assert_eq!(usable(&found, Capability::FileIcons), ["org.example.Good"]);
    let dir = resolved.registry.installed()[0]
        .manifest
        .provides
        .iter()
        .find_map(|provision| provision.file_icons())
        .map(|icons| icons.dir().to_path_buf());
    assert_eq!(dir, Some(good.join("file-icons")));
    let named: Vec<String> = resolved.problems().iter().map(|p| p.id.0.clone()).collect();
    assert_eq!(
        named,
        [
            "org.example.Absent",
            "org.example.File",
            "org.example.Linked"
        ]
    );
    assert!(resolved.problems().iter().all(|p| p.words() == ICON_WORDS));
}

#[test]
fn an_extension_is_never_half_used() {
    // A good look beside file icons whose folder is missing: neither word is served.
    let scratch = Scratch::new();
    let person = scratch.root("home");
    let both = format!("{LOOK}\n{ICONS}");
    let folder = install(&person, ID, &manifest(ID, "Paper", "1.0.0", &both));
    write(&folder.join("look.css"), ".ds {}\n");
    assert!(fs::metadata(folder.join("file-icons")).is_err());

    let found = Search::new(&person, &[]).scan(&[]);

    assert!(usable(&found, Capability::Look).is_empty());
    assert!(usable(&found, Capability::FileIcons).is_empty());
    assert_eq!(found.resolve(Capability::Look).problems().len(), 1);
}
