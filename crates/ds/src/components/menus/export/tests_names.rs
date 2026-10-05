//! The names around the tree: key chords, bus addresses and standard command ids.

use super::*;
use crate::components::menus::menu_bar::BarCommand;
use ds_core::command::CommandId;
use ds_core::standard_action::StandardAction;
use ds_core::vocab::{Shortcut, ShortcutKey};

#[test]
fn a_chord_round_trips_for_every_key() {
    let keys = [
        ShortcutKey::Space,
        ShortcutKey::Enter,
        ShortcutKey::Escape,
        ShortcutKey::Tab,
        ShortcutKey::Backspace,
        ShortcutKey::Up,
        ShortcutKey::Down,
        ShortcutKey::Left,
        ShortcutKey::Right,
        ShortcutKey::Home,
        ShortcutKey::End,
        ShortcutKey::Delete,
        ShortcutKey::PageUp,
        ShortcutKey::PageDown,
        ShortcutKey::Insert,
        ShortcutKey::ContextMenu,
        ShortcutKey::Char('z'),
        ShortcutKey::Char(','),
        ShortcutKey::Char('+'),
    ];
    for key in keys {
        let shortcut = Shortcut(vec![ShortcutKey::Super, ShortcutKey::Shift, key]);
        let chord = Chord::of(&shortcut);
        assert_eq!(chord.shortcut(), Ok(Shortcut(shortcut.keys())), "{key:?}");
    }
    assert_eq!(
        Chord::of(&Shortcut::standard(StandardAction::Redo)),
        Chord(vec!["Shift".to_owned(), "Super".to_owned(), "Z".to_owned()])
    );
    assert_eq!(
        Chord(vec!["Hyper".to_owned()]).shortcut(),
        Err(ChordError("Hyper".to_owned()))
    );
}

#[test]
fn an_app_id_makes_a_bus_name() {
    let cases: &[(&str, Result<&str, AddressError>)] = &[
        ("dev.mailo.Mailo", Ok("org.quire.AppMenu.dev.mailo.Mailo")),
        ("org.quire.Detent", Ok("org.quire.AppMenu.org.quire.Detent")),
        ("Alacritty", Ok("org.quire.AppMenu.Alacritty")),
        (
            "com.1password.app",
            Ok("org.quire.AppMenu.com._1password.app"),
        ),
        ("my app.x", Ok("org.quire.AppMenu.my_app.x")),
        ("a..b", Ok("org.quire.AppMenu.a._.b")),
        ("", Err(AddressError::Empty)),
    ];
    for (id, want) in cases {
        let got = AppMenuAddress::for_app_id(id).map(|address| address.service);
        assert_eq!(got, want.clone().map(str::to_owned), "{id}");
    }
    let long = "a".repeat(300);
    assert_eq!(
        AppMenuAddress::for_app_id(&long),
        Err(AddressError::TooLong(long))
    );
    assert_eq!(
        AppMenuAddress::for_app_id("x").map(|address| address.path),
        Ok("/org/quire/AppMenu".to_owned())
    );
}

#[test]
fn standard_ids_are_snake_case_words() {
    let cases: &[(BarCommand, &str)] = &[
        (BarCommand::About, "bar.about"),
        (BarCommand::HelpSearch, "bar.help_search"),
        (BarCommand::Standard(StandardAction::Undo), "standard.undo"),
        (
            BarCommand::Standard(StandardAction::PasteAndMatchStyle),
            "standard.paste_and_match_style",
        ),
    ];
    for (command, want) in cases {
        assert_eq!(command.id(), CommandId((*want).to_owned()));
    }
}
