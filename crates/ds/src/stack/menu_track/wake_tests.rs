//! The tracker's wake and its cursor as tables: when it next wants to be stepped, and where the
//! keyboard cursor stands.

use super::tests::{
    Event, Key, Script, at, click_open, inert, item, parent, path, pt, run, session, submenu_open,
    submenu_open_to,
};
use crate::stack::menu_track::types::{ItemPath, MenuPhase, MenuTrack};
use ds_core::machine::{Elapsed, Machine};
use ds_core::time::stamp::Stamp;

/// The machine's wake after a script.
fn wake_of(phase: MenuPhase<Key>) -> Option<Stamp> {
    MenuTrack { phase }.wake()
}

#[test]
fn the_wake_is_the_submenu_delay_then_the_triangle_timeout_then_nothing() {
    // Resting at 100 on a parent item: the delay ends at 300. Placed at 310, the triangle waits
    // for a move inside it; a move at 320 sets the timeout 300 ms on; a move at 400 pushes it on.
    // A tick at the wake settles each. Name, script, the wake after it.
    let inside_at = |ms| (ms, Event::Move(pt(130.0, 130.0), item(3)));
    let cases: Vec<(&str, Script, Option<Stamp>)> = vec![
        ("closed", vec![], None),
        ("open, nothing pending", click_open(), None),
        (
            "resting on a parent item",
            [
                click_open(),
                vec![(100, Event::Move(pt(20.0, 60.0), parent(2)))],
            ]
            .concat(),
            Some(at(300)),
        ),
        (
            "resting on the parent item again keeps the first deadline",
            [
                click_open(),
                vec![
                    (100, Event::Move(pt(20.0, 60.0), parent(2))),
                    (150, Event::Move(pt(21.0, 60.0), parent(2))),
                ],
            ]
            .concat(),
            Some(at(300)),
        ),
        (
            "moving to a leaf drops the pending submenu",
            [
                click_open(),
                vec![
                    (100, Event::Move(pt(20.0, 60.0), parent(2))),
                    (150, Event::Move(pt(20.0, 90.0), item(3))),
                ],
            ]
            .concat(),
            None,
        ),
        ("the delay ended: open", submenu_open_to(300), None),
        (
            "placed: no timeout until the pointer moves inside",
            submenu_open(),
            None,
        ),
        (
            "a move inside the triangle arms the timeout",
            [submenu_open(), vec![inside_at(320)]].concat(),
            Some(at(620)),
        ),
        (
            "a later move inside pushes it out",
            [submenu_open(), vec![inside_at(320), inside_at(400)]].concat(),
            Some(at(700)),
        ),
        (
            "the timeout ended: the item under the pointer won",
            [submenu_open(), vec![inside_at(320), (620, Event::Tick)]].concat(),
            None,
        ),
        (
            "a tick before the deadline changes nothing",
            [
                submenu_open(),
                vec![inside_at(320), (500, Event::from(Elapsed))],
            ]
            .concat(),
            Some(at(620)),
        ),
    ];
    for (name, script, want) in cases {
        let (phase, _) = if script.is_empty() {
            (MenuPhase::Closed, Vec::new())
        } else {
            run(script)
        };
        assert_eq!(wake_of(phase), want, "{name}");
    }
}

#[test]
fn the_cursor_stays_on_the_last_highlighted_item_while_the_pointer_is_over_something_inert() {
    // Name, script, the highlight after it, the cursor after it.
    let cases: Vec<(&str, Script, Option<ItemPath>, Option<ItemPath>)> = vec![
        ("nothing yet", click_open(), None, None),
        (
            "on an item",
            [
                click_open(),
                vec![(100, Event::Move(pt(20.0, 60.0), item(2)))],
            ]
            .concat(),
            Some(path(2)),
            Some(path(2)),
        ),
        (
            "then over a rule: the highlight goes, the cursor stays",
            [
                click_open(),
                vec![
                    (100, Event::Move(pt(20.0, 60.0), item(2))),
                    (150, Event::Move(pt(20.0, 80.0), inert(3))),
                ],
            ]
            .concat(),
            None,
            Some(path(2)),
        ),
        (
            "the keyboard moves it",
            [click_open(), vec![(100, Event::Select(path(4)))]].concat(),
            Some(path(4)),
            Some(path(4)),
        ),
    ];
    for (name, script, hot, cursor) in cases {
        let (phase, _) = run(script);
        assert_eq!(session(&phase).hot, hot, "{name}: highlight");
        assert_eq!(session(&phase).cursor, cursor, "{name}: cursor");
    }
}
