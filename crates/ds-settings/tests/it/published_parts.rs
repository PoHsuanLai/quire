//! An app's published style parts as data (`ds_settings::published_parts`): written, found, and
//! held to the app's own prefix.

use ds_settings::{PublishedParts, discover_parts, parts_dirs_from};

use crate::support::Scratch;

fn classes(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

#[test]
fn only_the_apps_own_prefix_is_kept_on_write_and_on_read() {
    let parts = PublishedParts::new(
        "sill",
        classes(&["sill-bar", "ds-row", "mailo-list", "sill-bar-item"]),
    );
    assert_eq!(parts.classes, classes(&["sill-bar", "sill-bar-item"]));
    let text = "app = \"sill\"\nclasses = [\"sill-bar\", \"ds-chip\"]\n";
    let read = PublishedParts::from_toml(text).expect("parses");
    assert_eq!(read.classes, classes(&["sill-bar"]));
    assert_eq!(PublishedParts::from_toml(&parts.to_toml()), Ok(parts));
}

#[test]
fn every_app_is_found_once_and_a_misnamed_file_is_skipped() {
    let scratch = Scratch::new();
    let system = scratch.root().join("system");
    let person = scratch.root().join("person");
    PublishedParts::new("sill", classes(&["sill-bar"]))
        .write_to(&system)
        .expect("writes");
    PublishedParts::new("files", classes(&["files-path"]))
        .write_to(&system)
        .expect("writes");
    // The person's directory comes last: it may add an app, never replace an installed one.
    PublishedParts::new("sill", classes(&["sill-dock"]))
        .write_to(&person)
        .expect("writes");
    PublishedParts::new("temor", classes(&["temor-tab"]))
        .write_to(&person)
        .expect("writes");
    std::fs::write(person.join("wrong.toml"), "app = \"mailo\"\nclasses = []\n").expect("writes");
    let found = discover_parts(&[system, person]);
    let apps: Vec<(&str, &[String])> = found
        .iter()
        .map(|parts| (parts.app.as_str(), parts.classes.as_slice()))
        .collect();
    assert_eq!(
        apps,
        [
            ("files", &classes(&["files-path"])[..]),
            ("sill", &classes(&["sill-bar"])[..]),
            ("temor", &classes(&["temor-tab"])[..]),
        ]
    );
}

#[test]
fn the_directories_are_the_system_ones_then_the_persons() {
    let dirs = parts_dirs_from(
        Some("/home/p/.local/share".into()),
        Some("/usr/local/share:/usr/share".into()),
        None,
    );
    assert_eq!(
        dirs,
        [
            "/usr/local/share/quire/selectors",
            "/usr/share/quire/selectors",
            "/home/p/.local/share/quire/selectors",
        ]
        .map(std::path::PathBuf::from)
    );
}
