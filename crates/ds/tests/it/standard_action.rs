//! The standard shortcut table (design/27-HIG-PARITY.md section 6.2): chordkit's conventions on
//! this desktop, drawn in the Mac's order, and the refusals `Shortcut::custom` gets from
//! chordkit's registration. chordkit's own tests cover that no two actions share a chord.

use ds::prelude::*;
use ds_core::standard_action::{Conflict, SpaceNumber, StandardAction};

#[test]
fn every_action_has_keys_on_this_desktop_in_the_macs_order() {
    for action in StandardAction::all() {
        let shortcut = Shortcut::standard(action);
        assert!(!shortcut.0.is_empty(), "{action:?} has keys");
        assert_eq!(
            shortcut.keys(),
            shortcut.0,
            "{action:?} is written in order"
        );
        assert!(shortcut.default_chord().is_some(), "{action:?} is a chord");
    }
}

/// What `Shortcut::custom` says about a combination.
#[derive(Debug, PartialEq)]
enum Verdict {
    Free,
    /// A standard action has it in every app.
    Standard(StandardAction),
    /// The desktop keeps it for itself.
    Kept,
}

fn verdict(keys: Vec<ShortcutKey>) -> Verdict {
    match Shortcut::custom(keys) {
        Ok(_) => Verdict::Free,
        Err(Conflict::Standard { action, .. }) => Verdict::Standard(action),
        Err(_) => Verdict::Kept,
    }
}

#[test]
fn custom_refuses_a_taken_combination_in_any_order_or_case() {
    let cases: Vec<(Vec<ShortcutKey>, Verdict)> = vec![
        (
            vec![ShortcutKey::Super, ShortcutKey::Char('S')],
            Verdict::Standard(StandardAction::Save),
        ),
        (
            vec![
                ShortcutKey::Char('z'),
                ShortcutKey::Super,
                ShortcutKey::Shift,
            ],
            Verdict::Standard(StandardAction::Redo),
        ),
        (
            vec![
                ShortcutKey::Super,
                ShortcutKey::Ctrl,
                ShortcutKey::Char('s'),
            ],
            Verdict::Standard(StandardAction::ToggleSidebar),
        ),
        (
            vec![ShortcutKey::Super, ShortcutKey::Char('t')],
            Verdict::Standard(StandardAction::ShowFonts),
        ),
        // The desktop's own keys are kept whoever asks.
        (vec![ShortcutKey::Super, ShortcutKey::Space], Verdict::Kept),
        (
            vec![ShortcutKey::Ctrl, ShortcutKey::Char('3')],
            Verdict::Kept,
        ),
        // The command menu's key, the launcher's actions menu and an app's own: free.
        (
            vec![ShortcutKey::Super, ShortcutKey::Char('k')],
            Verdict::Free,
        ),
        (
            vec![ShortcutKey::Super, ShortcutKey::Char('1')],
            Verdict::Free,
        ),
        (vec![ShortcutKey::Enter], Verdict::Free),
        (
            vec![ShortcutKey::Super, ShortcutKey::Backspace],
            Verdict::Free,
        ),
    ];
    for (keys, want) in cases {
        assert_eq!(verdict(keys.clone()), want, "{keys:?}");
    }
}

/// sill's launcher row chords are standard: Quick Look, Reveal and Copy.
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
        (StandardAction::SwitchToSpace(three()), "⌃3"),
    ];
    for (action, glyphs) in cases {
        assert_eq!(Shortcut::standard(action).glyphs(), glyphs, "{action:?}");
    }
    let given_backwards = Shortcut(vec![
        ShortcutKey::Super,
        ShortcutKey::Shift,
        ShortcutKey::Alt,
        ShortcutKey::Ctrl,
        ShortcutKey::Char('x'),
    ]);
    assert_eq!(given_backwards.glyphs(), "⌃⌥⇧⌘X");
    assert_eq!(
        Shortcut::custom([ShortcutKey::Char('k'), ShortcutKey::Super])
            .map(|shortcut| shortcut.glyphs()),
        Ok("⌘K".to_owned())
    );
}

fn three() -> SpaceNumber {
    SpaceNumber::new(3).expect("3 is a space")
}

#[test]
fn a_space_number_is_one_to_nine() {
    assert_eq!(SpaceNumber::new(0), None);
    assert_eq!(SpaceNumber::new(10), None);
    assert_eq!(SpaceNumber::new(9).map(SpaceNumber::get), Some(9));
}
