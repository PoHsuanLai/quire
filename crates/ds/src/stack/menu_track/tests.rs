//! design/13 section 13.4's table and section 13.8 tests 1-5 as input sequences (ported from
//! sill's `bar/menu_track/tests.rs`).

use crate::stack::menu_track::triangle::{inside, shielded};
use crate::stack::menu_track::types::{
    Branch, ItemPath, MenuAnim, MenuDirection, MenuKey, MenuPhase, MenuTarget, MenuTiming,
    MenuTrack, MenuTrackEffect, MenuTrackEvent, Pickable, SafeTriangle, Session, ShownBy, Submenu,
};
use ds_core::geometry::units::{Point, Px};
use ds_core::machine::{Elapsed, Machine};
use ds_core::time::stamp::Stamp;
use ds_core::vocab::PressPhase;
use std::time::Duration;

/// A bar title, as the caller keys its menus.
type Key = u32;
type Event = MenuTrackEvent<Key>;
type Effect = MenuTrackEffect<Key>;
type Target = MenuTarget<Key>;
/// One scripted input sequence: `(ms, event)` pairs.
type Script = Vec<(u64, Event)>;

const A: Key = 1;
const B: Key = 2;

fn at(ms: u64) -> Stamp {
    Stamp(ms)
}

/// `track` after `event` at `ms`, with the default timings.
fn step(track: MenuTrack<Key>, event: Event, ms: u64) -> (MenuTrack<Key>, Vec<Effect>) {
    track.step(event, at(ms), &MenuTiming::default(), &())
}

fn path(i: u16) -> ItemPath {
    ItemPath(vec![i])
}

fn target(i: u16, pick: Pickable, branch: Branch) -> Target {
    MenuTarget::Item {
        path: path(i),
        pick,
        branch,
    }
}

fn item(i: u16) -> Target {
    target(i, Pickable::Enabled, Branch::Leaf)
}

fn parent(i: u16) -> Target {
    target(i, Pickable::Enabled, Branch::Submenu)
}

fn inert(i: u16) -> Target {
    target(i, Pickable::Inert, Branch::Leaf)
}

fn pt(x: f32, y: f32) -> Point {
    Point { x: Px(x), y: Px(y) }
}

/// Feed `(ms, event)` pairs from closed; the final phase and every effect, in order.
fn run(script: Script) -> (MenuPhase<Key>, Vec<Effect>) {
    let start = (MenuTrack::closed(), Vec::new());
    let (track, effects) = script
        .into_iter()
        .fold(start, |(track, mut all), (ms, event)| {
            let (track, effects) = step(track, event, ms);
            all.extend(effects);
            (track, all)
        });
    (track.phase, effects)
}

fn session(phase: &MenuPhase<Key>) -> &Session<Key> {
    match phase {
        MenuPhase::Tracking(session) => session,
        MenuPhase::Closed => panic!("expected an open menu"),
    }
}

/// Press A and release on its title: click mode.
fn click_open() -> Script {
    vec![
        (0, Event::PressTitle(A)),
        (60, Event::Release(MenuTarget::Title(A))),
    ]
}

#[test]
fn open_on_press_click_mode_and_toggle() {
    // Section 13.8 test 1.
    let (phase, effects) = run(vec![(0, Event::PressTitle(A))]);
    assert_eq!(effects, vec![Effect::Open(A, MenuAnim::Pop)]);
    assert_eq!(session(&phase).held, PressPhase::Pressed);
    let (phase, _) = run(click_open());
    assert_eq!(
        session(&phase).held,
        PressPhase::Idle,
        "release on the title leaves it open"
    );
    let (phase, effects) = run([click_open(), vec![(500, Event::PressTitle(A))]].concat());
    assert_eq!(phase, MenuPhase::Closed);
    assert_eq!(effects.last(), Some(&Effect::Close(MenuAnim::Fade)));
}

#[test]
fn press_drag_release_picks_or_closes() {
    // Section 13.8 test 2 and the table's Release rows.
    let press_move = |target: Target| {
        vec![
            (0, Event::PressTitle(A)),
            (40, Event::Move(pt(20.0, 60.0), target.clone())),
            (80, Event::Release(target)),
        ]
    };
    let cases: Vec<(&str, Script, Vec<Effect>)> = vec![
        (
            "release on an enabled item picks it once",
            press_move(item(3)),
            vec![Effect::Close(MenuAnim::Fade), Effect::Pick(path(3))],
        ),
        (
            "release on a separator closes without picking",
            press_move(inert(2)),
            vec![Effect::Close(MenuAnim::Fade)],
        ),
        (
            "release outside after entering closes",
            vec![
                (0, Event::PressTitle(A)),
                (40, Event::Move(pt(20.0, 60.0), item(1))),
                (80, Event::Release(MenuTarget::Outside)),
            ],
            vec![Effect::Close(MenuAnim::Fade)],
        ),
        (
            "release outside before entering closes",
            vec![
                (0, Event::PressTitle(A)),
                (80, Event::Release(MenuTarget::Outside)),
            ],
            vec![Effect::Close(MenuAnim::Fade)],
        ),
        (
            "in click mode a later release on an item picks it",
            [click_open(), vec![(400, Event::Release(item(4)))]].concat(),
            vec![Effect::Close(MenuAnim::Fade), Effect::Pick(path(4))],
        ),
        (
            "an outside press closes",
            vec![(0, Event::PressTitle(A)), (60, Event::OutsidePress)],
            vec![Effect::Close(MenuAnim::Fade)],
        ),
    ];
    for (name, script, want) in cases {
        let (phase, effects) = run(script);
        assert_eq!(phase, MenuPhase::Closed, "{name}");
        let tail: Vec<Effect> = effects
            .into_iter()
            .filter(|e| matches!(e, Effect::Close(_) | Effect::Pick(_)))
            .collect();
        assert_eq!(tail, want, "{name}");
    }
}

#[test]
fn in_click_mode_a_release_elsewhere_keeps_the_menu() {
    let (phase, effects) = run([click_open(), vec![(300, Event::Release(inert(1)))]].concat());
    assert_eq!(session(&phase).menu, A);
    assert_eq!(effects, vec![Effect::Open(A, MenuAnim::Pop)]);
}

#[test]
fn hover_switch_swaps_menus_in_one_step_without_animation() {
    // Section 13.8 test 3: one Switch, never a Close then an Open.
    let (phase, effects) = run([
        click_open(),
        vec![(300, Event::Move(pt(120.0, 10.0), MenuTarget::Title(B)))],
    ]
    .concat());
    assert_eq!(
        effects,
        vec![Effect::Open(A, MenuAnim::Pop), Effect::Check(B)]
    );
    let session = session(&phase);
    assert_eq!((session.menu, session.held), (B, PressPhase::Idle));
}

/// One row of the click-after-switch table: its name, the script, and the open menu with how it
/// was shown and whether its press is still down (`None` when the tracker closed).
type SwitchCase = (&'static str, Script, Option<(Key, ShownBy, PressPhase)>);

/// With A open in click mode and the pointer switched to B (its title shows B, hover-shown).
fn switched_to_b() -> Script {
    [
        click_open(),
        vec![(300, Event::Move(pt(120.0, 10.0), MenuTarget::Title(B)))],
    ]
    .concat()
}

#[test]
fn a_press_on_a_title_the_pointer_switched_to_keeps_its_menu() {
    // With A open, the pointer enters B (the hover switch shows B), then presses B. That press
    // is the click that opens B; only a press on a title already pressed closes.
    let cases: Vec<SwitchCase> = vec![
        (
            "switched to by hover",
            switched_to_b(),
            Some((B, ShownBy::Hover, PressPhase::Idle)),
        ),
        (
            "a press on the switched title keeps it, held",
            [switched_to_b(), vec![(400, Event::PressTitle(B))]].concat(),
            Some((B, ShownBy::Press, PressPhase::Pressed)),
        ),
        (
            "released there: click mode",
            [
                switched_to_b(),
                vec![
                    (400, Event::PressTitle(B)),
                    (460, Event::Release(MenuTarget::Title(B))),
                ],
            ]
            .concat(),
            Some((B, ShownBy::Press, PressPhase::Idle)),
        ),
        (
            "a second press closes it",
            [
                switched_to_b(),
                vec![
                    (400, Event::PressTitle(B)),
                    (460, Event::Release(MenuTarget::Title(B))),
                    (900, Event::PressTitle(B)),
                ],
            ]
            .concat(),
            None,
        ),
        (
            "moving inside B's menu keeps it hover-shown",
            [
                switched_to_b(),
                vec![(400, Event::Move(pt(130.0, 60.0), item(1)))],
            ]
            .concat(),
            Some((B, ShownBy::Hover, PressPhase::Idle)),
        ),
        (
            "a press-drag onto B released on its title makes it B's own",
            vec![
                (0, Event::PressTitle(A)),
                (100, Event::Move(pt(120.0, 10.0), MenuTarget::Title(B))),
                (200, Event::Release(MenuTarget::Title(B))),
            ],
            Some((B, ShownBy::Press, PressPhase::Idle)),
        ),
        (
            "a press on A's own title still closes A",
            [click_open(), vec![(500, Event::PressTitle(A))]].concat(),
            None,
        ),
        (
            "a press on another title opens it as pressed",
            [click_open(), vec![(300, Event::PressTitle(B))]].concat(),
            Some((B, ShownBy::Press, PressPhase::Pressed)),
        ),
    ];
    for (name, script, want) in cases {
        let (phase, effects) = run(script);
        let got = match &phase {
            MenuPhase::Closed => None,
            MenuPhase::Tracking(session) => Some((session.menu, session.shown, session.held)),
        };
        assert_eq!(got, want, "{name}");
        assert!(
            !effects
                .iter()
                .any(|effect| matches!(effect, Effect::Open(menu, _) if *menu == B)),
            "{name}: B is swapped in, never opened afresh"
        );
    }
}

#[test]
fn moving_over_the_open_menus_own_title_changes_nothing() {
    let (phase, effects) = run([
        click_open(),
        vec![(300, Event::Move(pt(10.0, 10.0), MenuTarget::Title(A)))],
    ]
    .concat());
    assert_eq!(session(&phase).menu, A);
    assert_eq!(effects, vec![Effect::Open(A, MenuAnim::Pop)]);
}

#[test]
fn a_submenu_opens_after_the_delay_or_at_once_on_right_arrow() {
    // Section 13.8 test 4.
    let rest = [
        click_open(),
        vec![(100, Event::Move(pt(20.0, 60.0), parent(2)))],
    ]
    .concat();
    let (phase, effects) = run(rest.clone());
    assert_eq!(effects.last(), Some(&Effect::Prepare(path(2))));
    assert_eq!(
        wake_of(phase),
        Some(at(300)),
        "the delay's end is the machine's wake"
    );
    let (phase, effects) = run([rest.clone(), vec![(290, Event::Tick)]].concat());
    assert!(
        !effects.contains(&Effect::OpenSub(path(2))),
        "not at 190 ms"
    );
    assert!(matches!(session(&phase).sub, Submenu::Pending { .. }));
    let (_, effects) = run([rest.clone(), vec![(300, Event::Tick)]].concat());
    assert_eq!(effects.last(), Some(&Effect::OpenSub(path(2))));
    let (_, effects) = run([rest, vec![(150, Event::Key(MenuKey::Right))]].concat());
    assert_eq!(effects.last(), Some(&Effect::OpenSub(path(2))));
}

#[test]
fn the_timings_come_from_the_caller() {
    let timing = MenuTiming {
        submenu_delay: Duration::from_millis(50),
        triangle_timeout: Duration::from_millis(300),
    };
    let step = |track: MenuTrack<Key>, event, ms| track.step(event, at(ms), &timing, &());
    let (track, _) = step(MenuTrack::closed(), Event::PressTitle(A), 0);
    let (track, effects) = step(track, Event::Move(pt(20.0, 60.0), parent(2)), 100);
    assert_eq!(effects.last(), Some(&Effect::Prepare(path(2))));
    assert_eq!(track.wake(), Some(at(150)));
}

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

/// Click-open A, rest on parent item 2 until its submenu opens at `ms`.
fn submenu_open_to(ms: u64) -> Script {
    [
        click_open(),
        vec![
            (100, Event::Move(pt(100.0, 100.0), parent(2))),
            (ms, Event::Tick),
        ],
    ]
    .concat()
}

/// Click-open A, rest on parent item 2 until its submenu opens, and place the submenu.
fn submenu_open() -> Script {
    [
        submenu_open_to(300),
        vec![(
            310,
            Event::SubPlaced {
                top: pt(200.0, 95.0),
                bottom: pt(200.0, 300.0),
            },
        )],
    ]
    .concat()
}

#[test]
fn the_safe_triangle_keeps_the_submenu_and_times_out() {
    // Section 13.8 test 5: a diagonal toward the submenu crossing two other items keeps it.
    let (phase, effects) = run([
        submenu_open(),
        vec![
            (320, Event::Move(pt(130.0, 130.0), item(3))),
            (330, Event::Move(pt(160.0, 170.0), item(4))),
        ],
    ]
    .concat());
    assert!(!effects.contains(&Effect::CloseSub), "{effects:?}");
    let s = session(&phase);
    assert_eq!(s.hot, Some(path(2)));
    assert!(matches!(s.sub, Submenu::Open { .. }));
    // Resting inside the triangle for 300 ms selects what is under the pointer.
    let (phase, effects) = run([
        submenu_open(),
        vec![
            (320, Event::Move(pt(130.0, 130.0), item(3))),
            (620, Event::Tick),
        ],
    ]
    .concat());
    assert!(effects.contains(&Effect::CloseSub), "{effects:?}");
    assert_eq!(session(&phase).hot, Some(path(3)));
    // A tick before the timeout changes nothing.
    let (phase, _) = run([
        submenu_open(),
        vec![
            (320, Event::Move(pt(130.0, 130.0), item(3))),
            (610, Event::Tick),
        ],
    ]
    .concat());
    assert_eq!(session(&phase).hot, Some(path(2)));
    // Leaving the triangle onto another item closes the submenu at once.
    let (_, effects) = run([
        submenu_open(),
        vec![(320, Event::Move(pt(100.0, 20.0), item(0)))],
    ]
    .concat());
    assert!(effects.contains(&Effect::CloseSub), "{effects:?}");
}

#[test]
fn keys_close_a_level_cross_menus_and_pick() {
    let open = [
        click_open(),
        vec![(100, Event::Move(pt(20.0, 60.0), item(1)))],
    ]
    .concat();
    let (phase, effects) = run([open.clone(), vec![(200, Event::Key(MenuKey::Escape))]].concat());
    assert_eq!(phase, MenuPhase::Closed);
    assert_eq!(effects.last(), Some(&Effect::Close(MenuAnim::Fade)));
    let cases = [
        (MenuKey::Left, MenuDirection::Left),
        (MenuKey::Right, MenuDirection::Right),
    ];
    for (key, side) in cases {
        let (_, effects) = run([open.clone(), vec![(200, Event::Key(key))]].concat());
        assert_eq!(effects.last(), Some(&Effect::Adjacent(side)), "{key:?}");
    }
    let (phase, effects) = run([open, vec![(200, Event::Key(MenuKey::Enter))]].concat());
    assert_eq!(phase, MenuPhase::Closed);
    assert_eq!(
        effects[effects.len() - 2..],
        [Effect::Close(MenuAnim::Fade), Effect::Pick(path(1))]
    );
}

#[test]
fn escape_with_a_submenu_open_closes_only_the_submenu() {
    let (phase, effects) = run([
        click_open(),
        vec![
            (100, Event::Move(pt(20.0, 60.0), parent(2))),
            (150, Event::Key(MenuKey::Right)),
            (200, Event::Key(MenuKey::Escape)),
        ],
    ]
    .concat());
    assert_eq!(effects.last(), Some(&Effect::CloseSub));
    assert_eq!(session(&phase).sub, Submenu::None);
}

#[test]
fn a_closed_tracker_ignores_everything_but_a_press_on_a_title() {
    let events = [
        Event::Release(item(1)),
        Event::Move(pt(1.0, 1.0), item(1)),
        Event::Key(MenuKey::Escape),
        Event::OutsidePress,
        Event::Tick,
    ];
    for event in events {
        let (phase, effects) = run(vec![(0, event.clone())]);
        assert_eq!(phase, MenuPhase::Closed, "{event:?}");
        assert!(effects.is_empty(), "{event:?}");
    }
}

#[test]
fn the_triangle_test_is_a_sign_test_either_winding() {
    let (a, b, c) = (pt(0.0, 0.0), pt(100.0, -10.0), pt(100.0, 110.0));
    const CASES: &[((f32, f32), bool)] = &[
        ((50.0, 50.0), true),
        ((10.0, 0.0), true),
        ((100.0, 50.0), true),
        ((50.0, -20.0), false),
        ((120.0, 50.0), false),
        ((-1.0, 0.0), false),
    ];
    for &((x, y), want) in CASES {
        assert_eq!(inside(pt(x, y), a, b, c), want, "({x}, {y})");
        assert_eq!(inside(pt(x, y), a, c, b), want, "({x}, {y}) reversed");
    }
    let guard = SafeTriangle {
        from: pt(0.0, 50.0),
        top: pt(100.0, 0.0),
        bottom: pt(100.0, 100.0),
        timeout: None,
    };
    assert!(
        shielded(&guard, pt(100.0, -3.0)),
        "the corners are inflated 4 px"
    );
    assert!(!shielded(&guard, pt(100.0, -5.0)));
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
