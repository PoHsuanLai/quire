//! The standard shortcut table (design/27-HIG-PARITY.md section 6.2): every combination is
//! reserved once, drawn in the Mac's order, and refused to `Shortcut::custom`.

use ds::{Key, Reserved, Shortcut, SpaceNumber, StandardAction};

#[test]
fn no_two_actions_share_a_combination() {
    let mut seen: Vec<(Vec<Key>, StandardAction)> = Vec::new();
    for action in StandardAction::ALL {
        let keys = Shortcut::standard(action).keys();
        if let Some((_, owner)) = seen.iter().find(|(other, _)| *other == keys) {
            panic!("{action:?} and {owner:?} share {keys:?}");
        }
        seen.push((keys, action));
    }
}

#[test]
fn every_action_is_written_in_the_macs_order_and_owns_its_keys() {
    for action in StandardAction::ALL {
        let shortcut = Shortcut::standard(action);
        assert_eq!(
            shortcut.keys(),
            shortcut.0,
            "{action:?} is written in order"
        );
        assert_eq!(
            StandardAction::owning(&shortcut.0),
            Some(action),
            "{action:?}"
        );
        assert_eq!(
            Shortcut::custom(shortcut.0.clone()),
            Err(Reserved(action)),
            "{action:?} is refused to custom"
        );
    }
}

#[test]
fn custom_refuses_a_reserved_combination_in_any_order_or_case() {
    let cases: Vec<(Vec<Key>, Option<StandardAction>)> = vec![
        (vec![Key::Super, Key::Char('S')], Some(StandardAction::Save)),
        (
            vec![Key::Char('z'), Key::Super, Key::Shift],
            Some(StandardAction::Redo),
        ),
        (
            vec![Key::Super, Key::Ctrl, Key::Char('s')],
            Some(StandardAction::ToggleSidebar),
        ),
        (
            vec![Key::Super, Key::Char('t')],
            Some(StandardAction::ShowFonts),
        ),
        (vec![Key::Super, Key::Space], Some(StandardAction::Launcher)),
        (
            vec![Key::Ctrl, Key::Char('3')],
            SpaceNumber::new(3).map(StandardAction::SwitchToSpace),
        ),
        // The command menu's key, the launcher's actions menu and an app's own: free.
        (vec![Key::Super, Key::Char('k')], None),
        (vec![Key::Super, Key::Char('1')], None),
        (vec![Key::Enter], None),
        (vec![Key::Super, Key::Backspace], None),
    ];
    for (keys, owner) in cases {
        let got = Shortcut::custom(keys.clone());
        match owner {
            Some(action) => assert_eq!(got, Err(Reserved(action)), "{keys:?}"),
            None => assert!(got.is_ok(), "{keys:?}: {got:?}"),
        }
    }
}

/// sill's launcher row chords (Q298) are standard: Quick Look, Reveal and Copy.
#[test]
fn the_launcher_row_chords_are_standard() {
    let cases = [
        (StandardAction::QuickLook, "⌘Y"),
        (StandardAction::Reveal, "⌘R"),
        (StandardAction::Copy, "⌘C"),
    ];
    for (action, glyphs) in cases {
        assert_eq!(Shortcut::standard(action).glyphs(), glyphs, "{action:?}");
    }
}

#[test]
fn modifiers_draw_control_option_shift_command() {
    let cases = [
        (StandardAction::Redo, "⇧⌘Z"),
        (StandardAction::PasteAndMatchStyle, "⌥⇧⌘V"),
        (StandardAction::ToggleSidebar, "⌃⌘S"),
        (StandardAction::ForceQuit, "⌥⌘Esc"),
        (StandardAction::Help, "⌘?"),
    ];
    for (action, glyphs) in cases {
        assert_eq!(Shortcut::standard(action).glyphs(), glyphs, "{action:?}");
    }
    let given_backwards = Shortcut(vec![
        Key::Super,
        Key::Shift,
        Key::Alt,
        Key::Ctrl,
        Key::Char('x'),
    ]);
    assert_eq!(given_backwards.glyphs(), "⌃⌥⇧⌘X");
    assert_eq!(
        Shortcut::custom([Key::Char('k'), Key::Super]).map(|shortcut| shortcut.glyphs()),
        Ok("⌘K".to_owned())
    );
}

#[test]
fn a_space_number_is_one_to_nine() {
    assert_eq!(SpaceNumber::new(0), None);
    assert_eq!(SpaceNumber::new(10), None);
    assert_eq!(SpaceNumber::new(9).map(SpaceNumber::get), Some(9));
    assert_eq!(
        Reserved(StandardAction::Save).to_string(),
        "⌘S is reserved for Save"
    );
}
