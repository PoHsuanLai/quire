//! design/13 section 13.4's table and section 13.8 tests 1-5 as input sequences (ported from
//! sill's `bar/menu_track/tests.rs`).

use super::{
    Branch, Held, ItemPath, MenuAnim, MenuDirection, MenuKey, MenuPhase, MenuTarget, MenuTiming,
    MenuTrack, MenuTrackEffect, MenuTrackEvent, Pickable, SafeTriangle, Session, Submenu, inside,
    shielded,
};
use crate::geometry::{Point, Px};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

/// A bar title, as the caller keys its menus.
type Key = u32;
type Event = MenuTrackEvent<Key>;
type Effect = MenuTrackEffect<Key>;
type Target = MenuTarget<Key>;
/// One scripted input sequence: `(ms, event)` pairs.
type Script = Vec<(u64, Event)>;

const A: Key = 1;
const B: Key = 2;

/// Every script's time zero: one instant, so a failure replays identically.
static ZERO: LazyLock<Instant> = LazyLock::new(Instant::now);

fn at(ms: u64) -> Instant {
    *ZERO + Duration::from_millis(ms)
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
    let start = (MenuTrack::new(MenuTiming::default()), Vec::new());
    let (track, effects) = script
        .into_iter()
        .fold(start, |(track, mut all), (ms, event)| {
            let (track, effects) = track.step(event, at(ms));
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
    assert_eq!(session(&phase).held, Held::Held);
    let (phase, _) = run(click_open());
    assert_eq!(
        session(&phase).held,
        Held::Released,
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
        vec![Effect::Open(A, MenuAnim::Pop), Effect::Switch(B)]
    );
    let session = session(&phase);
    assert_eq!((session.menu, session.held), (B, Held::Released));
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
    let (_, effects) = run(rest.clone());
    assert_eq!(effects.last(), Some(&Effect::RequestTick(at(300))));
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
    let track = MenuTrack::new(timing);
    let (track, _) = track.step(Event::PressTitle(A), at(0));
    let (_, effects) = track.step(Event::Move(pt(20.0, 60.0), parent(2)), at(100));
    assert_eq!(effects.last(), Some(&Effect::RequestTick(at(150))));
}

/// Click-open A, rest on parent item 2 until its submenu opens, and place the submenu.
fn submenu_open() -> Script {
    [
        click_open(),
        vec![
            (100, Event::Move(pt(100.0, 100.0), parent(2))),
            (300, Event::Tick),
            (
                310,
                Event::SubPlaced {
                    top: pt(200.0, 95.0),
                    bottom: pt(200.0, 300.0),
                },
            ),
        ],
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
        still_since: at(0),
    };
    assert!(
        shielded(&guard, pt(100.0, -3.0)),
        "the corners are inflated 4 px"
    );
    assert!(!shielded(&guard, pt(100.0, -5.0)));
}
