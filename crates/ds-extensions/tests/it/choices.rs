//! Choosing among Extensions: the menu's words, what a stored choice means, and the look's text.

use ds_extensions::look::stylesheet_for;
use ds_extensions::{Capability, ExtensionId, Search, choices_for, chosen};
use ds_settings::quire_schema;
use ds_settings::schema::{Control, KeyKind, KeyPath, Provided};

use crate::support::{ICONS, Scratch, install_entries, install_look, write};

const PAPER: &str = "org.example.Paper";
const QUARTZ: &str = "org.example.Quartz";

fn found_in(scratch: &Scratch) -> ds_extensions::Found {
    let person = scratch.root("home");
    install_look(&person, QUARTZ, "Quartz", "1.0.0");
    let paper = install_look(&person, PAPER, "Paper", "1.0.0");
    write(&paper.join("look.css"), ".ds { --paper: ivory; }\n");
    let icons = install_entries(&person, "org.example.Pictures", ICONS);
    write(&icons.join("file-icons").join("text.png"), "png");
    Search::new(&person, &[]).scan(&[])
}

#[test]
fn the_choices_are_standard_first_then_the_usable_extensions_of_the_capability() {
    let scratch = Scratch::new();
    let found = found_in(&scratch);

    let looks = found.resolve(Capability::Look);
    let choices = choices_for(Capability::Look, looks.registry.installed());
    assert_eq!(choices.words(), ["standard", PAPER, QUARTZ]);
    assert_eq!(choices.labels().of("standard"), Some("Standard"));
    assert_eq!(choices.labels().of(PAPER), Some("Paper"));
    assert!(choices.offers(QUARTZ) && !choices.offers("standard"));

    let icons = found.resolve(Capability::FileIcons);
    let choices = choices_for(Capability::FileIcons, icons.registry.installed());
    assert_eq!(choices.words(), ["standard", "org.example.Pictures"]);
    // Nothing installed: the one choice is the standard one.
    let none = choices_for(Capability::Look, &[]);
    assert_eq!(none.words(), ["standard"]);
}

#[test]
fn the_filled_key_is_a_menu_of_those_words() {
    let scratch = Scratch::new();
    let looks = found_in(&scratch).resolve(Capability::Look);
    let choices = choices_for(Capability::Look, looks.registry.installed());
    let key = quire_schema()
        .key
        .into_iter()
        .find(|key| key.path.0 == "appearance.look")
        .expect("appearance.look");

    let filled = choices.fill(&key);

    assert_eq!(filled.control(), Control::Menu);
    assert_eq!(
        filled.kind,
        KeyKind::Menu {
            variants: vec!["standard".to_owned(), PAPER.to_owned(), QUARTZ.to_owned()]
        }
    );
    assert_eq!(filled.labels.of(QUARTZ), Some("Quartz"));
    assert_eq!(filled.default, key.default);
}

#[test]
fn a_stored_choice_that_is_not_on_offer_reads_as_standard_and_is_reported() {
    let scratch = Scratch::new();
    let looks = found_in(&scratch).resolve(Capability::Look);
    let choices = choices_for(Capability::Look, looks.registry.installed());

    // stored, the Extension used, whether it is reported
    let cases: [(&str, Option<&str>, bool); 5] = [
        ("standard", None, false),
        ("", None, false),
        (PAPER, Some(PAPER), false),
        ("org.example.Removed", None, true),
        ("org.example.Pictures", None, true),
    ];
    for (stored, want, stale) in cases {
        let got = chosen(&Provided::new(stored), &choices);
        assert_eq!(
            got.id,
            want.map(|id| ExtensionId(id.to_owned())),
            "{stored:?}"
        );
        assert_eq!(got.stale.is_some(), stale, "{stored:?}");
    }

    let report = chosen(&Provided::new("org.example.Removed"), &choices)
        .stale
        .expect("reported")
        .invalid_key(&KeyPath("appearance.look".to_owned()));
    assert_eq!(report.path, "appearance.look");
    assert!(
        report.reason.contains("Standard is used"),
        "{}",
        report.reason
    );
}

#[test]
fn a_turned_off_look_is_not_on_offer_so_the_choice_reads_as_standard() {
    let scratch = Scratch::new();
    let person = scratch.root("home");
    install_look(&person, PAPER, "Paper", "1.0.0");
    let off = [ExtensionId(PAPER.to_owned())];

    let looks = Search::new(&person, &[])
        .scan(&off)
        .resolve(Capability::Look);
    let choices = choices_for(Capability::Look, looks.registry.installed());

    assert_eq!(choices.words(), ["standard"]);
    assert!(chosen(&Provided::new(PAPER), &choices).stale.is_some());
}

#[test]
fn the_chosen_looks_stylesheet_is_read_with_its_folder() {
    let scratch = Scratch::new();
    let looks = found_in(&scratch).resolve(Capability::Look);
    let installed = &looks.registry;

    let sheet = stylesheet_for(&Provided::new(PAPER), installed).expect("Paper is installed");
    assert_eq!(sheet.text, ".ds { --paper: ivory; }\n");
    assert!(sheet.folder.ends_with(PAPER), "{:?}", sheet.folder);

    assert_eq!(stylesheet_for(&Provided::standard(), installed), None);
    assert_eq!(
        stylesheet_for(&Provided::new("org.example.Removed"), installed),
        None
    );
    // A set of file icons is not a look.
    assert_eq!(
        stylesheet_for(&Provided::new("org.example.Pictures"), installed),
        None
    );
}

/// The looks quire ships, as their folders sit in the repo (`extensions/<id>/`).
const SHIPPED: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../extensions");

#[test]
fn the_shipped_editorial_look_installs_as_a_usable_look() {
    const EDITORIAL: &str = "org.quire.look.Editorial";
    let scratch = Scratch::new();
    let system = scratch.root("system");
    let folder = system.join("quire").join("extensions").join(EDITORIAL);
    for file in ["extension.toml", "look.css"] {
        let text = std::fs::read_to_string(format!("{SHIPPED}/{EDITORIAL}/{file}"))
            .unwrap_or_else(|e| panic!("{file}: {e}"));
        write(&folder.join(file), &text);
    }
    let person = scratch.root("home");
    let looks = Search::new(&person, &[system])
        .scan(&[])
        .resolve(Capability::Look);

    let choices = choices_for(Capability::Look, looks.registry.installed());
    assert_eq!(choices.words(), ["standard", EDITORIAL]);
    assert_eq!(choices.labels().of(EDITORIAL), Some("Editorial"));
    let sheet =
        stylesheet_for(&Provided::new(EDITORIAL), &looks.registry).expect("Editorial reads");
    assert!(
        sheet.text.contains("--font-ui: \"Karla\""),
        "{}",
        sheet.text
    );
}
