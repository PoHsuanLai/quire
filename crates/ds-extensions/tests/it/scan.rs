//! Finding Extensions: every copy unranked, the turned-off apart, other owners' words left alone.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use bayonet::registry::Origin;
use ds_extensions::{Capability, ExtensionId, Provision, Search};

use crate::support::{LOOK, Scratch, install, install_look, manifest, write};

const PAPER: &str = "org.example.Paper";

fn ids(found: &ds_extensions::Found) -> Vec<(String, Origin)> {
    found
        .candidates
        .iter()
        .map(|candidate| (candidate.manifest.id.as_str().to_owned(), candidate.origin))
        .collect()
}

#[test]
fn every_copy_of_an_id_comes_back_unranked_with_where_it_was_found() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    let system = [scratch.root("share-a"), scratch.root("share-b")];
    install_look(&person, PAPER, "Paper", "1.0.0");
    install_look(&system[0], PAPER, "Paper", "2.0.0");
    install_look(&system[1], PAPER, "Paper", "1.5.0");

    let found = Search::new(&person, &system).scan(&[]);

    assert_eq!(
        ids(&found),
        [
            (PAPER.to_owned(), Origin::User),
            (PAPER.to_owned(), Origin::System),
            (PAPER.to_owned(), Origin::System),
        ]
    );
    let versions: Vec<String> = found
        .candidates
        .iter()
        .filter_map(|candidate| candidate.manifest.version.as_ref().map(ToString::to_string))
        .collect();
    assert_eq!(versions, ["1.0.0", "2.0.0", "1.5.0"]);
    assert!(found.off.is_empty() && found.rejected.is_empty());
}

#[test]
fn ranking_after_the_checks_puts_the_persons_copy_over_the_systems_then_the_newer_version() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    let system = [scratch.root("share-a"), scratch.root("share-b")];
    install_look(&system[0], PAPER, "Paper", "1.5.0");
    install_look(&system[1], PAPER, "Paper", "2.0.0");

    let versions = |person: &PathBuf| {
        let resolved = Search::new(person, &system)
            .scan(&[])
            .resolve(Capability::Look);
        resolved
            .registry
            .installed()
            .iter()
            .map(|installed| {
                let version = installed.manifest.version.as_ref();
                (installed.origin, version.map(ToString::to_string))
            })
            .collect::<Vec<_>>()
    };

    // Among the system's copies the newer wins.
    assert_eq!(
        versions(&person),
        [(Origin::System, Some("2.0.0".to_owned()))]
    );
    // The person's older copy still beats a newer system one.
    install_look(&person, PAPER, "Paper", "1.0.0");
    assert_eq!(
        versions(&person),
        [(Origin::User, Some("1.0.0".to_owned()))]
    );
}

#[test]
fn a_turned_off_id_is_apart_with_its_name_and_hides_from_the_ranking() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    install_look(&person, PAPER, "Paper", "1.0.0");
    install_look(&person, "org.example.Quartz", "Quartz", "1.0.0");
    let off = [
        ExtensionId(PAPER.to_owned()),
        ExtensionId("org.example.Gone".to_owned()),
    ];

    let found = Search::new(&person, &[]).scan(&off);

    assert_eq!(
        ids(&found),
        [("org.example.Quartz".to_owned(), Origin::User)]
    );
    // Only an installed Extension is listed as off; a stale id in the file is not.
    assert_eq!(found.off.len(), 1);
    assert_eq!(found.off[0].id, ExtensionId(PAPER.to_owned()));
    assert_eq!(found.off[0].name, "Paper");
    assert_eq!(found.off_serving("look").count(), 1);
    assert_eq!(found.off_serving("thumbnail").count(), 0);
    // It is off, so it is not in the choices.
    let resolved = found.resolve(Capability::Look);
    let usable: Vec<&str> = resolved
        .registry
        .installed()
        .iter()
        .map(|installed| installed.manifest.id.as_str())
        .collect();
    assert_eq!(usable, ["org.example.Quartz"]);
}

#[test]
fn another_owners_word_is_kept_raw_and_skipped_not_refused() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    let heif = "org.quire.anyview.Heif";
    let program = "[program]\npath = \"/usr/libexec/anyview/heif\"\n\n";
    let entry = "[[provides]]\ncapability = \"thumbnail\"\nmimes = [\"image/heic\"]\n";
    let thumbnail = format!("{program}{entry}");
    install(&person, heif, &manifest(heif, "HEIF", "1.0.0", &thumbnail));
    // One Extension with a word of ours and a word of someone else's.
    let mixed = format!("{LOOK}\n[[provides]]\ncapability = \"widget\"\nsize = 2\n");
    let both = "org.example.Both";
    let folder = install(&person, both, &manifest(both, "Both", "1.0.0", &mixed));
    write(&folder.join("look.css"), ".ds {}\n");

    let found = Search::new(&person, &[]).scan(&[]);

    // The scan keeps both, entries raw.
    assert_eq!(found.candidates.len(), 2);
    assert!(found.rejected.is_empty(), "{:?}", found.rejected);
    // Our words see only the Extension that has one, and the other owner's entry does not fail it.
    let resolved = found.resolve(Capability::Look);
    assert!(resolved.problems().is_empty());
    let usable: Vec<&str> = resolved
        .registry
        .installed()
        .iter()
        .map(|installed| installed.manifest.id.as_str())
        .collect();
    assert_eq!(usable, [both]);
    // The entry set aside is reported as skipped, in the shape another owner relies on.
    let raw = found
        .candidates
        .iter()
        .find(|candidate| candidate.manifest.id.as_str() == both)
        .cloned()
        .expect("the mixed Extension was scanned");
    let read = bayonet::reparse::<Provision>(raw).expect("our words are readable");
    let skipped: Vec<&str> = read.skipped.iter().map(|s| s.capability.as_str()).collect();
    assert_eq!(skipped, ["widget"]);
    assert_eq!(read.candidate.map(|c| c.manifest.provides.len()), Some(1));
}

#[test]
fn a_folder_not_named_for_its_id_is_rejected_not_scanned() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    let text = manifest(PAPER, "Paper", "1.0.0", LOOK);
    install(&person, "org.example.Other", &text);

    let found = Search::new(&person, &[]).scan(&[]);

    assert!(found.candidates.is_empty());
    assert_eq!(found.rejected.len(), 1);
}

#[test]
fn the_roots_follow_the_xdg_rules() {
    let os = |text: &'static str| Some(OsStr::new(text));
    let cases: [(&str, Search, Search); 3] = [
        (
            "variables win",
            Search::from_dirs(os("/data"), os("/a:/b"), os("/home/ada")),
            Search::new(
                Path::new("/data"),
                &[PathBuf::from("/a"), PathBuf::from("/b")],
            ),
        ),
        (
            "home and the default dirs",
            Search::from_dirs(None, None, os("/home/ada")),
            Search::new(
                Path::new("/home/ada/.local/share"),
                &[
                    PathBuf::from("/usr/local/share"),
                    PathBuf::from("/usr/share"),
                ],
            ),
        ),
        (
            "a relative data home is ignored",
            Search::from_dirs(os("data"), os("/a"), os("/home/ada")),
            Search::new(Path::new("/home/ada/.local/share"), &[PathBuf::from("/a")]),
        ),
    ];
    for (name, got, want) in cases {
        assert_eq!(got, want, "{name}");
    }
    // Nowhere to put a personal root: the system's alone, and nothing is read from the
    // working directory.
    let homeless = Search::from_dirs(None, os("/nonexistent-ds-extensions-root"), None);
    assert!(homeless.scan(&[]).candidates.is_empty());
}
