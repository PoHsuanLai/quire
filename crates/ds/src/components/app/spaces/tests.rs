use super::chord::SwitchChord;
use super::menu_link::{DesktopSpace, LinkChange, Linking};
use super::menu_pick::{SpacePick, Then, apply, delete_words, rows};
use super::open_menu::Showing;
use crate::components::menus::item::item::MenuItem;
use dioxus::prelude::{Key, Modifiers};
use ds_style::appearance::theme::Theme;
use ds_style::space::look::{CardAccent, SpaceLook};

fn key(text: &str) -> Key {
    Key::Character(text.to_string())
}

#[test]
fn only_the_chords_modifier_and_a_digit_from_one_to_nine_switches() {
    // (chord, key, modifiers, the number asked for)
    let cases = [
        (SwitchChord::Command, key("1"), Modifiers::META, Some(1)),
        (SwitchChord::Command, key("9"), Modifiers::META, Some(9)),
        (SwitchChord::Command, key("0"), Modifiers::META, None),
        (SwitchChord::Command, key("1"), Modifiers::CONTROL, None),
        (
            SwitchChord::Command,
            key("1"),
            Modifiers::META | Modifiers::SHIFT,
            None,
        ),
        (SwitchChord::Command, key("1"), Modifiers::empty(), None),
        (SwitchChord::Command, key("a"), Modifiers::META, None),
        (SwitchChord::Command, key("12"), Modifiers::META, None),
        (SwitchChord::Command, Key::Enter, Modifiers::META, None),
        (SwitchChord::Control, key("3"), Modifiers::CONTROL, Some(3)),
        (SwitchChord::Control, key("3"), Modifiers::META, None),
    ];
    for (chord, pressed, modifiers, want) in cases {
        let got = chord
            .pressed(&pressed, modifiers)
            .map(|number| number.get());
        assert_eq!(got, want, "{chord:?} {pressed:?} {modifiers:?}");
    }
}

#[test]
fn the_hint_beside_a_dot_is_the_chord_and_its_place() {
    // (chord, index, glyphs)
    const CASES: &[(SwitchChord, usize, &str)] = &[
        (SwitchChord::Command, 0, "\u{2318}1"),
        (SwitchChord::Command, 8, "\u{2318}9"),
        (SwitchChord::Command, 9, ""),
        (SwitchChord::Control, 2, "\u{2303}3"),
    ];
    for &(chord, index, glyphs) in CASES {
        assert_eq!(chord.shortcut(index).glyphs(), glyphs, "{chord:?} {index}");
    }
}

fn titles(items: &[MenuItem<SpacePick<u8>>]) -> Vec<String> {
    items
        .iter()
        .map(|item| match item {
            MenuItem::Item { title, .. } | MenuItem::Submenu { title, .. } => title.clone(),
            MenuItem::Separator => "-".to_owned(),
            MenuItem::Header(title) => title.clone(),
            MenuItem::Info { title, .. } => title.clone(),
        })
        .collect()
}

#[test]
fn the_menu_lists_the_rows_in_the_agreed_order_and_delete_only_beside_another_space() {
    let slot = vec![MenuItem::new(7u8, "Mine")];
    let look = SpaceLook::default();
    let many = titles(&rows(&look, 2, slot.clone(), Linking::Off));
    assert_eq!(
        many,
        [
            "Rename\u{2026}",
            "Colour\u{2026}",
            "Appearance",
            "Accent Inside the Card",
            "Mine",
            "-",
            "New Space",
            "Delete Space\u{2026}"
        ]
    );
    let one = titles(&rows(&look, 1, slot, Linking::Off));
    assert_eq!(one.last().map(String::as_str), Some("New Space"));
    assert!(!one.iter().any(|title| title.starts_with("Delete")));
}

#[test]
fn the_app_slot_keeps_its_own_values() {
    let slot = vec![MenuItem::Submenu {
        title: "Accounts".into(),
        image: None,
        availability: ds_core::vocab::Availability::Enabled,
        children: vec![MenuItem::new(3u8, "Ada")],
    }];
    let items = rows(&SpaceLook::default(), 2, slot, Linking::Off);
    let MenuItem::Submenu { children, .. } = &items[4] else {
        panic!("the slot is the fifth row");
    };
    assert_eq!(
        children[0],
        MenuItem::new(SpacePick::App(3u8), "Ada"),
        "a pick from the slot reaches the app wrapped"
    );
}

#[test]
fn a_pick_opens_a_part_changes_the_look_or_is_the_apps() {
    let look = SpaceLook::default();
    let dark = Then::<u8>::Kept(SpaceLook {
        theme: Theme::Dark,
        ..look.clone()
    });
    let hue = Then::<u8>::Kept(SpaceLook {
        card_accent: CardAccent::SpaceHue,
        ..look.clone()
    });
    // (pick, what it asks)
    let cases: Vec<(SpacePick<u8>, Then<u8>)> = vec![
        (SpacePick::Rename, Then::Open(Showing::Rename)),
        (SpacePick::Colour, Then::Open(Showing::Colour)),
        (SpacePick::Delete, Then::Open(Showing::Delete)),
        (SpacePick::New, Then::NewSpace),
        (SpacePick::Theme(Theme::Dark), dark),
        (SpacePick::Accent(CardAccent::SpaceHue), hue),
        (SpacePick::App(4), Then::App(4)),
    ];
    for (pick, want) in cases {
        assert_eq!(apply(pick.clone(), &look), want, "{pick:?}");
    }
}

#[test]
fn the_delete_question_names_the_space_and_what_is_kept() {
    // (name, title)
    const CASES: &[(&str, &str)] = &[
        ("Work", "Delete \u{201c}Work\u{201d}?"),
        ("  Home ", "Delete \u{201c}Home\u{201d}?"),
        ("", "Delete this Space?"),
        ("   ", "Delete this Space?"),
    ];
    for &(name, title) in CASES {
        let (said, body) = delete_words(name, "Your mail");
        assert_eq!(said, title);
        assert!(body.starts_with("Your mail is not affected"), "{body}");
    }
}

fn desktop() -> Vec<DesktopSpace> {
    vec![
        DesktopSpace {
            id: "work".into(),
            name: "Work".into(),
        },
        DesktopSpace {
            id: "home".into(),
            name: "Home".into(),
        },
    ]
}

#[test]
fn link_rows_list_the_desktop_spaces_and_unlink_shows_only_while_linked() {
    let desktop = desktop();
    let unlinked = rows::<u8>(
        &SpaceLook::default(),
        2,
        Vec::new(),
        Linking::Offered {
            current: None,
            desktop: &desktop,
        },
    );
    assert_eq!(
        titles(&unlinked),
        [
            "Rename\u{2026}",
            "Colour\u{2026}",
            "Appearance",
            "Accent Inside the Card",
            "Link to\u{2026}",
            "-",
            "New Space",
            "Delete Space\u{2026}"
        ]
    );
    let MenuItem::Submenu { children, .. } = &unlinked[4] else {
        panic!("Link to... is a submenu");
    };
    assert_eq!(
        titles(children),
        ["Work", "Home", "-", "New Desktop Space\u{2026}"]
    );
    let linked = rows::<u8>(
        &SpaceLook::default(),
        2,
        Vec::new(),
        Linking::Offered {
            current: Some("home"),
            desktop: &desktop,
        },
    );
    assert_eq!(titles(&linked)[5], "Unlink");
    let MenuItem::Submenu { children, .. } = &linked[4] else {
        panic!("Link to... is a submenu");
    };
    let checks: Vec<ds_core::vocab::Check> = children
        .iter()
        .filter_map(|item| match item {
            MenuItem::Item { check, .. } => Some(check.unwrap_or(ds_core::vocab::Check::Off)),
            _ => None,
        })
        .collect();
    use ds_core::vocab::Check::{Off, On};
    assert_eq!(
        checks,
        [Off, On, Off],
        "the linked Space is the one checked"
    );
}

#[test]
fn a_menu_with_no_desktop_has_no_link_rows() {
    let off = titles(&rows::<u8>(
        &SpaceLook::default(),
        2,
        Vec::new(),
        Linking::Off,
    ));
    assert!(
        !off.iter()
            .any(|title| title.contains("Link") || title == "Unlink"),
        "{off:?}"
    );
}

#[test]
fn link_picks_become_link_changes() {
    let look = SpaceLook::default();
    let table: [(SpacePick<u8>, Then<u8>); 3] = [
        (
            SpacePick::Link("work".into()),
            Then::Link(LinkChange::Linked("work".into())),
        ),
        (SpacePick::Unlink, Then::Link(LinkChange::Unlinked)),
        (
            SpacePick::NewDesktopSpace,
            Then::Link(LinkChange::NewDesktopSpace),
        ),
    ];
    for (pick, want) in table {
        assert_eq!(apply(pick.clone(), &look), want, "{pick:?}");
    }
}
