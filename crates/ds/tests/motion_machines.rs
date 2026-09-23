//! The pure motion machines as tables: the roster (enter, leave, heal and their timings), the
//! hover intent machine (design/06-INTERACTIONS.md section 3), drag and the pull tab
//! (sections 6 and 9.2), and the pulse's A/B alternation (design/05-MOTION.md section 9).

use ds::components::vocab::StaggerIndex;
use ds::motion::drag::fraction_along;
use ds::overlay::{Pull, PullTab, TabArm};
use ds::{
    Anim, Drag, DragPhase, Emphasis, Exit, Fraction, HoverEvent, HoverIntent, HoverWarmth,
    IntentEffect, IntentPhase, MotionLevel, Point, Presence, Px, Rect, RosterState, RowPitch, Size,
    settle, use_pulse,
};
use std::time::{Duration, Instant};

const PITCH: RowPitch = RowPitch(Px(79.0));

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

// ---- roster --------------------------------------------------------------------------------

/// A row as a test writes it: key and presence.
type Row = (&'static str, Presence);

fn rows(state: &RosterState<&'static str>) -> Vec<Row> {
    state
        .entries()
        .iter()
        .map(|e| (e.key, e.presence))
        .collect()
}

fn indices(state: &RosterState<&'static str>) -> Vec<u8> {
    state.entries().iter().map(|e| e.index.get()).collect()
}

fn healing(d: usize) -> Presence {
    Presence::Healing {
        dy: PITCH.0,
        d: StaggerIndex::new(d),
    }
}

/// A roster over `keys` whose entrance has finished.
fn at_rest(keys: &[&'static str]) -> RosterState<&'static str> {
    RosterState::first_show(keys, PITCH).rest()
}

#[test]
fn first_show_enters_every_row_staggered_and_capped_at_12() {
    let keys: Vec<&'static str> = vec![
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n",
    ];
    let state = RosterState::first_show(&keys, PITCH);
    assert!(
        state
            .entries()
            .iter()
            .all(|e| e.presence == Presence::Entering),
        "every row enters"
    );
    assert_eq!(
        indices(&state),
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 12]
    );
    let rested = state.rest();
    assert!(
        rested
            .entries()
            .iter()
            .all(|e| e.presence == Presence::Present),
        "rest marks every entering row present"
    );
}

#[test]
fn reconcile_orders_and_states() {
    use Presence::{Entering, Leaving, Present};
    struct Case {
        name: &'static str,
        state: RosterState<&'static str>,
        keys: &'static [&'static str],
        expect: Vec<Row>,
        expect_index: Vec<u8>,
    }
    let cases = [
        Case {
            name: "a new key enters where the consumer put it",
            state: at_rest(&["a", "b", "c"]),
            keys: &["a", "x", "b", "c"],
            expect: vec![
                ("a", Present),
                ("x", Entering),
                ("b", Present),
                ("c", Present),
            ],
            expect_index: vec![0, 0, 1, 2],
        },
        Case {
            name: "two arrivals stagger among themselves",
            state: at_rest(&["a"]),
            keys: &["x", "a", "y"],
            expect: vec![("x", Entering), ("a", Present), ("y", Entering)],
            expect_index: vec![0, 0, 1],
        },
        Case {
            name: "a key removed without an exit drops at once",
            state: at_rest(&["a", "b", "c"]),
            keys: &["a", "c"],
            expect: vec![("a", Present), ("c", Present)],
            expect_index: vec![0, 2],
        },
        Case {
            name: "a leaving key the consumer dropped stays in place until it settles",
            state: at_rest(&["a", "b", "c"])
                .leave(&"b", Exit::Fold, Emphasis::Plain)
                .0,
            keys: &["a", "c"],
            expect: vec![("a", Present), ("b", Leaving(Exit::Fold)), ("c", Present)],
            expect_index: vec![0, 1, 2],
        },
        Case {
            name: "a leaving row stays after its predecessor, before an arrival",
            state: at_rest(&["a", "b", "c"])
                .leave(&"c", Exit::Curl, Emphasis::Plain)
                .0,
            keys: &["a", "b", "z"],
            expect: vec![
                ("a", Present),
                ("b", Present),
                ("c", Leaving(Exit::Curl)),
                ("z", Entering),
            ],
            expect_index: vec![0, 1, 2, 0],
        },
        Case {
            name: "a leaving row whose predecessor went silently follows the row before that",
            state: at_rest(&["a", "b", "c", "d"])
                .leave(&"c", Exit::Fold, Emphasis::Plain)
                .0,
            keys: &["a", "d"],
            expect: vec![("a", Present), ("c", Leaving(Exit::Fold)), ("d", Present)],
            expect_index: vec![0, 2, 3],
        },
        Case {
            name: "a leaving first row stays first",
            state: at_rest(&["a", "b"])
                .leave(&"a", Exit::Fold, Emphasis::Plain)
                .0,
            keys: &["x", "b"],
            expect: vec![("a", Leaving(Exit::Fold)), ("x", Entering), ("b", Present)],
            expect_index: vec![0, 0, 1],
        },
        Case {
            name: "a leaving key still listed keeps leaving",
            state: at_rest(&["a", "b"])
                .leave(&"a", Exit::Crumple, Emphasis::Plain)
                .0,
            keys: &["a", "b"],
            expect: vec![("a", Leaving(Exit::Crumple)), ("b", Present)],
            expect_index: vec![0, 1],
        },
    ];
    for case in cases {
        let got = case.state.reconcile(case.keys);
        assert_eq!(rows(&got), case.expect, "{}", case.name);
        assert_eq!(indices(&got), case.expect_index, "{}: stagger", case.name);
    }
}

#[test]
fn leave_names_the_animation_to_settle() {
    let cases = [
        (Exit::Fold, Emphasis::Plain, Anim::Fold),
        (Exit::Fold, Emphasis::Strong, Anim::FoldHeavy),
        (Exit::Curl, Emphasis::Plain, Anim::Curl),
        // Wave 2 integration: every row exit has its heavy variant for an unread row.
        (Exit::Curl, Emphasis::Strong, Anim::CurlHeavy),
        (Exit::Crumple, Emphasis::Plain, Anim::Crumple),
        (Exit::Crumple, Emphasis::Strong, Anim::CrumpleHeavy),
        // A Today entry's exit has no heavy variant.
        (Exit::TabOut, Emphasis::Plain, Anim::TabOut),
        (Exit::TabOut, Emphasis::Strong, Anim::TabOut),
    ];
    for (exit, emphasis, want) in cases {
        let (state, anim) = at_rest(&["a", "b"]).leave(&"a", exit, emphasis);
        assert_eq!(anim, want, "{exit:?} {emphasis:?}");
        assert_eq!(
            rows(&state),
            vec![("a", Presence::Leaving(exit)), ("b", Presence::Present)],
            "{exit:?} {emphasis:?}: only the key leaves"
        );
    }
}

#[test]
fn settling_an_exit_heals_the_rows_below_in_order() {
    use Presence::{Leaving, Present};
    struct Case {
        name: &'static str,
        state: RosterState<&'static str>,
        settled: &'static str,
        expect: Vec<Row>,
    }
    let five = || at_rest(&["a", "b", "c", "d", "e"]);
    let cases = [
        Case {
            name: "rows below heal from one pitch down, 0, 1, 2 steps late; rows above rest",
            state: five().leave(&"b", Exit::Fold, Emphasis::Plain).0,
            settled: "b",
            expect: vec![
                ("a", Present),
                ("c", healing(0)),
                ("d", healing(1)),
                ("e", healing(2)),
            ],
        },
        Case {
            name: "the last row leaving heals nothing",
            state: five().leave(&"e", Exit::Curl, Emphasis::Plain).0,
            settled: "e",
            expect: vec![
                ("a", Present),
                ("b", Present),
                ("c", Present),
                ("d", Present),
            ],
        },
        Case {
            name: "a leaving row below keeps leaving and is not counted",
            state: five()
                .leave(&"b", Exit::Fold, Emphasis::Plain)
                .0
                .leave(&"c", Exit::Fold, Emphasis::Plain)
                .0,
            settled: "b",
            expect: vec![
                ("a", Present),
                ("c", Leaving(Exit::Fold)),
                ("d", healing(0)),
                ("e", healing(1)),
            ],
        },
        Case {
            name: "a key that is not leaving settles nothing",
            state: five(),
            settled: "c",
            expect: vec![
                ("a", Present),
                ("b", Present),
                ("c", Present),
                ("d", Present),
                ("e", Present),
            ],
        },
    ];
    for case in cases {
        let got = case.state.settled(&case.settled);
        assert_eq!(rows(&got), case.expect, "{}", case.name);
        let rested = got.rest();
        assert!(
            rested
                .entries()
                .iter()
                .all(|e| !matches!(e.presence, Presence::Healing { .. })),
            "{}: rest ends every heal",
            case.name
        );
    }
}

#[test]
fn heal_index_saturates_at_12() {
    let keys: Vec<&'static str> = vec![
        "x", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n",
    ];
    let state = at_rest(&keys)
        .leave(&"x", Exit::Fold, Emphasis::Plain)
        .0
        .settled(&"x");
    let ds: Vec<u8> = state
        .entries()
        .iter()
        .map(|e| match e.presence {
            Presence::Healing { d, .. } => d.get(),
            other => panic!("{}: {other:?}", e.key),
        })
        .collect();
    assert_eq!(ds, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 12]);
}

#[test]
fn roster_timings_follow_settle() {
    // design/05-MOTION.md section 7.1, Post Standard.
    let cases = [
        ("fold", Anim::Fold, 0, 454),
        ("unread fold", Anim::FoldHeavy, 0, 517),
        ("curl", Anim::Curl, 0, 594),
        ("heal, first row below", Anim::Heal, 0, 284),
        ("heal, third row below", Anim::Heal, 2, 320),
        ("heal, capped", Anim::Heal, 40, 250 + 18 * 12 + 34),
        ("rise, capped stagger", Anim::Rise, 40, 596),
    ];
    for (name, anim, index, want) in cases {
        assert_eq!(
            settle(anim, MotionLevel::Standard, StaggerIndex::new(index)),
            ms(want),
            "{name}"
        );
    }
    assert_eq!(
        settle(Anim::Fold, MotionLevel::Reduced, StaggerIndex::new(0)),
        ms(94),
        "reduced"
    );
}

// ---- hover intent --------------------------------------------------------------------------

/// A phase as a test writes it, with times in ms after the sequence starts.
#[derive(Debug, Clone, Copy)]
enum Ph {
    Idle,
    Pending(u8, u64),
    Open(u8),
    Closing(u8, u64),
}

fn phase(ph: Ph, t0: Instant) -> IntentPhase<u8> {
    match ph {
        Ph::Idle => IntentPhase::Idle,
        Ph::Pending(key, due) => IntentPhase::Pending {
            key,
            due: t0 + ms(due),
        },
        Ph::Open(key) => IntentPhase::Open { key },
        Ph::Closing(key, due) => IntentPhase::Closing {
            key,
            due: t0 + ms(due),
        },
    }
}

type Step = (u64, HoverEvent<u8>, Ph, IntentEffect<u8>);

fn open_after(n: u64) -> IntentEffect<u8> {
    IntentEffect::StartOpen { after: ms(n) }
}

fn close_after(n: u64) -> IntentEffect<u8> {
    IntentEffect::StartClose { after: ms(n) }
}

/// Opens card 1 cold: over at 0, the timer fires at 450.
fn opened() -> Vec<Step> {
    vec![
        (0, HoverEvent::Over(1), Ph::Pending(1, 450), open_after(450)),
        (450, HoverEvent::OpenDue, Ph::Open(1), IntentEffect::Open(1)),
    ]
}

/// Opens card 1, then closes it: out at 500, closed at 650, warm until 1050.
fn closed() -> Vec<Step> {
    let mut steps = opened();
    steps.extend([
        (500, HoverEvent::Out, Ph::Closing(1, 650), close_after(150)),
        (650, HoverEvent::CloseDue, Ph::Idle, IntentEffect::Close(1)),
    ]);
    steps
}

fn then(mut steps: Vec<Step>, more: impl IntoIterator<Item = Step>) -> Vec<Step> {
    steps.extend(more);
    steps
}

#[test]
fn hover_intent_sequences() {
    use HoverEvent::*;
    let cases: Vec<(&str, Vec<Step>)> = vec![
        ("cold: waits 450 ms, then opens", opened()),
        (
            "an early open timer is stale and ignored",
            vec![
                (0, Over(1), Ph::Pending(1, 450), open_after(450)),
                (449, OpenDue, Ph::Pending(1, 450), IntentEffect::None),
            ],
        ),
        (
            "moving within the pending target does not restart the wait",
            vec![
                (0, Over(1), Ph::Pending(1, 450), open_after(450)),
                (100, Over(1), Ph::Pending(1, 450), IntentEffect::None),
            ],
        ),
        (
            "leaving before the card opens cancels it",
            vec![
                (0, Over(1), Ph::Pending(1, 450), open_after(450)),
                (200, Out, Ph::Idle, IntentEffect::CancelOpen),
                (450, OpenDue, Ph::Idle, IntentEffect::None),
            ],
        ),
        (
            "a suppressed target changes nothing",
            vec![
                (0, OverSuppressed, Ph::Idle, IntentEffect::None),
                (10, Over(1), Ph::Pending(1, 460), open_after(450)),
                (100, OverSuppressed, Ph::Pending(1, 460), IntentEffect::None),
            ],
        ),
        ("out closes after 150 ms and the hub is warm", closed()),
        (
            "warm: the next card opens at once",
            then(
                closed(),
                [
                    (800, Over(2), Ph::Pending(2, 800), open_after(0)),
                    (800, OpenDue, Ph::Open(2), IntentEffect::Open(2)),
                ],
            ),
        ),
        (
            "warm ends 400 ms after the close",
            then(
                closed(),
                [(1050, Over(2), Ph::Pending(2, 1500), open_after(450))],
            ),
        ),
        (
            "a new key while open replaces the card instantly",
            then(
                opened(),
                [
                    (600, Over(2), Ph::Pending(2, 600), open_after(0)),
                    (600, OpenDue, Ph::Open(2), IntentEffect::Open(2)),
                ],
            ),
        ),
        (
            "a new key while closing replaces the card instantly",
            then(
                opened(),
                [
                    (500, Out, Ph::Closing(1, 650), close_after(150)),
                    (560, Over(2), Ph::Pending(2, 560), open_after(0)),
                    (650, CloseDue, Ph::Pending(2, 560), IntentEffect::None),
                ],
            ),
        ),
        (
            "back onto the same target while closing keeps it open",
            then(
                opened(),
                [
                    (500, Out, Ph::Closing(1, 650), close_after(150)),
                    (550, Over(1), Ph::Open(1), IntentEffect::CancelClose),
                    (650, CloseDue, Ph::Open(1), IntentEffect::None),
                ],
            ),
        ),
        (
            "over the open card's own target changes nothing",
            then(opened(), [(600, Over(1), Ph::Open(1), IntentEffect::None)]),
        ),
        (
            "into the card and out again",
            then(
                opened(),
                [
                    (500, Out, Ph::Closing(1, 650), close_after(150)),
                    (560, EnterCard, Ph::Open(1), IntentEffect::CancelClose),
                    (700, LeaveCard, Ph::Closing(1, 850), close_after(150)),
                    (850, CloseDue, Ph::Idle, IntentEffect::Close(1)),
                ],
            ),
        ),
        (
            "a stale close timer after re-closing waits for the new deadline",
            then(
                opened(),
                [
                    (500, Out, Ph::Closing(1, 650), close_after(150)),
                    (550, EnterCard, Ph::Open(1), IntentEffect::CancelClose),
                    (600, LeaveCard, Ph::Closing(1, 750), close_after(150)),
                    (650, CloseDue, Ph::Closing(1, 750), IntentEffect::None),
                    (750, CloseDue, Ph::Idle, IntentEffect::Close(1)),
                ],
            ),
        ),
        (
            "a click in the list removes the card without warmth",
            then(
                opened(),
                [
                    (600, ClickInList, Ph::Idle, IntentEffect::Remove(1)),
                    (700, Over(2), Ph::Pending(2, 1150), open_after(450)),
                ],
            ),
        ),
        (
            "a click in the list removes a closing card too",
            then(
                opened(),
                [
                    (500, Out, Ph::Closing(1, 650), close_after(150)),
                    (550, ClickInList, Ph::Idle, IntentEffect::Remove(1)),
                ],
            ),
        ),
        (
            "a click in the list cancels a pending card",
            vec![
                (0, Over(1), Ph::Pending(1, 450), open_after(450)),
                (100, ClickInList, Ph::Idle, IntentEffect::CancelOpen),
            ],
        ),
        (
            "space turns the open card into a peek and stays warm",
            then(
                opened(),
                [
                    (600, SpaceKey, Ph::Idle, IntentEffect::Peek(1)),
                    (700, Over(2), Ph::Pending(2, 700), open_after(0)),
                ],
            ),
        ),
        (
            "space with no open card does nothing",
            vec![(0, SpaceKey, Ph::Idle, IntentEffect::None)],
        ),
    ];
    for (name, steps) in cases {
        let t0 = Instant::now();
        let mut machine = HoverIntent::default();
        for (n, (at, event, want_phase, want_effect)) in steps.into_iter().enumerate() {
            let (next, effect) = machine.step(event.clone(), t0 + ms(at));
            assert_eq!(
                effect, want_effect,
                "{name}, step {n} ({event:?} at {at} ms): effect"
            );
            assert_eq!(
                next.phase(),
                &phase(want_phase, t0),
                "{name}, step {n} ({event:?} at {at} ms): phase"
            );
            machine = next;
        }
    }
}

#[test]
fn warmth_follows_the_card_and_the_warm_window() {
    let t0 = Instant::now();
    let mut machine = HoverIntent::default();
    let mut warmth = Vec::new();
    for (at, event) in [
        (0, HoverEvent::Over(1)),
        (450, HoverEvent::OpenDue),
        (500, HoverEvent::Out),
        (650, HoverEvent::CloseDue),
    ] {
        machine = machine.step(event, t0 + ms(at)).0;
        warmth.push(machine.warmth(t0 + ms(at)));
    }
    warmth.push(machine.warmth(t0 + ms(1049)));
    warmth.push(machine.warmth(t0 + ms(1050)));
    use HoverWarmth::{Cold, Warm};
    assert_eq!(warmth, vec![Cold, Warm, Warm, Warm, Warm, Cold]);
}

// ---- drag ----------------------------------------------------------------------------------

fn pt(x: f32, y: f32) -> Point {
    Point { x: Px(x), y: Px(y) }
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect {
        origin: pt(x, y),
        size: Size {
            width: Px(w),
            height: Px(h),
        },
    }
}

#[test]
fn drag_starts_after_8px_manhattan() {
    let cases = [
        ("still", pt(0.0, 0.0), false),
        ("3 + 4 = 7", pt(3.0, 4.0), false),
        ("0 + 7.9", pt(0.0, 7.9), false),
        ("4 + 4 = 8", pt(4.0, 4.0), true),
        ("-5 + 3 = 8", pt(-5.0, 3.0), true),
        ("8 left", pt(-8.0, 0.0), true),
    ];
    for (name, to, live) in cases {
        let drag = Drag::new(Px(8.0)).down(7u32, pt(0.0, 0.0)).moved(to);
        let want = if live {
            DragPhase::Live {
                key: 7,
                at: to,
                target: None,
            }
        } else {
            DragPhase::Pending {
                key: 7,
                from: pt(0.0, 0.0),
            }
        };
        assert_eq!(drag.phase(), &want, "{name}");
    }
}

#[test]
fn drag_hits_targets_and_drops() {
    let targets = vec![rect(100.0, 0.0, 50.0, 20.0), rect(100.0, 20.0, 50.0, 20.0)];
    let live = |to| {
        Drag::new(Px(8.0))
            .with_targets(targets.clone())
            .down("row", pt(0.0, 0.0))
            .moved(to)
    };
    let cases = [
        ("over the second target", pt(120.0, 25.0), Some(("row", 1))),
        (
            "on the first target's top-left edge",
            pt(100.0, 0.0),
            Some(("row", 0)),
        ),
        ("just past the right edge", pt(150.0, 5.0), None),
        ("nowhere", pt(20.0, 20.0), None),
    ];
    for (name, to, want) in cases {
        let (after, dropped) = live(to).up();
        assert_eq!(dropped, want, "{name}");
        assert_eq!(after.phase(), &DragPhase::Idle, "{name}: back to idle");
    }
    let (_, pending_up) = Drag::new(Px(8.0))
        .with_targets(targets.clone())
        .down("row", pt(120.0, 25.0))
        .up();
    assert_eq!(
        pending_up, None,
        "a press that never moved is a click, not a drop"
    );

    let retargeted = live(pt(120.0, 25.0)).with_targets(vec![rect(110.0, 20.0, 20.0, 10.0)]);
    assert_eq!(
        retargeted.phase(),
        &DragPhase::Live {
            key: "row",
            at: pt(120.0, 25.0),
            target: Some(0)
        },
        "new targets are hit-tested at once"
    );
}

#[test]
fn pull_tab_clamps_arms_and_undoes() {
    struct Case {
        name: &'static str,
        moves: &'static [f32],
        dx: f32,
        arm: TabArm,
        release: Pull,
        then_click: Pull,
    }
    let cases = [
        Case {
            name: "pulled past 46 arms, and the release undoes",
            moves: &[20.0, 50.0],
            dx: 50.0,
            arm: TabArm::Armed,
            release: Pull::Undo,
            then_click: Pull::Hold,
        },
        Case {
            name: "exactly 46 is not armed",
            moves: &[46.0],
            dx: 46.0,
            arm: TabArm::Disarmed,
            release: Pull::Hold,
            then_click: Pull::Hold,
        },
        Case {
            name: "pulled far right clamps to 78",
            moves: &[200.0],
            dx: 78.0,
            arm: TabArm::Armed,
            release: Pull::Undo,
            then_click: Pull::Hold,
        },
        Case {
            name: "pulled left clamps to -6",
            moves: &[-50.0],
            dx: -6.0,
            arm: TabArm::Disarmed,
            release: Pull::Hold,
            then_click: Pull::Hold,
        },
        Case {
            name: "pulled out and back: released disarmed, and the click is not a tap",
            moves: &[60.0, 10.0],
            dx: 10.0,
            arm: TabArm::Disarmed,
            release: Pull::Hold,
            then_click: Pull::Hold,
        },
        Case {
            name: "a press that barely moved: the click is a tap and undoes",
            moves: &[2.0],
            dx: 2.0,
            arm: TabArm::Disarmed,
            release: Pull::Hold,
            then_click: Pull::Undo,
        },
        Case {
            name: "a plain click",
            moves: &[],
            dx: 0.0,
            arm: TabArm::Disarmed,
            release: Pull::Hold,
            then_click: Pull::Undo,
        },
    ];
    for case in cases {
        let x0 = 300.0;
        let tab = case
            .moves
            .iter()
            .fold(PullTab::default().down(Px(x0)), |tab, dx| {
                tab.moved(Px(x0 + dx))
            });
        assert_eq!(tab.dx(), Px(case.dx), "{}: dx", case.name);
        assert_eq!(tab.arm(), case.arm, "{}: arm", case.name);
        let (released, pull) = tab.up();
        assert_eq!(pull, case.release, "{}: release", case.name);
        assert_eq!(released.dx(), Px(0.0), "{}: springs back", case.name);
        assert_eq!(released.arm(), TabArm::Disarmed, "{}: un-armed", case.name);
        let (_, click) = released.click();
        assert_eq!(click, case.then_click, "{}: click", case.name);
    }
}

#[test]
fn slider_fraction_along_the_track() {
    let track = rect(100.0, 0.0, 200.0, 10.0);
    let cases = [
        ("left end", 100.0, 0, 0),
        ("middle", 200.0, 0, 500),
        ("right end", 300.0, 0, 1000),
        ("before the track", 50.0, 0, 0),
        ("past the track", 350.0, 0, 1000),
        ("snapped down to a quarter", 160.0, 250, 250),
        ("snapped up to a half", 190.0, 250, 500),
        (
            "a step that does not divide 1000 stays inside",
            300.0,
            300,
            900,
        ),
    ];
    for (name, x, step, want) in cases {
        assert_eq!(
            fraction_along(track, Px(x), Fraction(step)),
            Fraction(want),
            "{name}"
        );
    }
    assert_eq!(
        fraction_along(rect(0.0, 0.0, 0.0, 0.0), Px(5.0), Fraction(0)),
        Fraction(0),
        "a zero-width track"
    );
}

// ---- pulse ---------------------------------------------------------------------------------

#[test]
fn pulse_alternates_aliases() {
    use dioxus::core::VirtualDom;
    use dioxus::prelude::*;
    use std::cell::Cell;

    thread_local! {
        static PULSE: Cell<Option<ds::Pulse>> = const { Cell::new(None) };
    }

    #[allow(non_snake_case)]
    fn App() -> Element {
        let pulse = use_pulse(Anim::Gulp);
        PULSE.set(Some(pulse));
        rsx! { div {} }
    }

    let mut dom = VirtualDom::new(App);
    dom.rebuild_in_place();
    let pulse = PULSE.get().expect("the component ran");
    dom.in_runtime(|| {
        assert_eq!(pulse.attrs(), None, "at rest until fired");
        let mut seen = Vec::new();
        for _ in 0..4 {
            pulse.fire();
            seen.push(pulse.attrs());
        }
        let a = Some(("a-gulp".to_owned(), "a"));
        let b = Some(("a-gulp".to_owned(), "b"));
        assert_eq!(seen, vec![a.clone(), b.clone(), a, b]);
    });
}

// ---- the roster hook's timers --------------------------------------------------------------

mod hook {
    use super::{PITCH, ms};
    use dioxus::core::{NoOpMutations, VirtualDom};
    use dioxus::prelude::*;
    use ds::{
        Accent, BlurState, Emphasis, Env, Exit, InputModality, Material, MotionLevel, Presence,
        Resolved, Roster, Scheme, use_roster,
    };
    use std::cell::Cell;
    use std::future::Future;
    use std::pin::pin;
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};
    use std::thread::{self, Thread};
    use std::time::{Duration, Instant};

    thread_local! {
        static ROSTER: Cell<Option<Roster<&'static str>>> = const { Cell::new(None) };
    }

    #[allow(non_snake_case)]
    fn List() -> Element {
        use_context_provider(|| {
            let resolved = Resolved {
                scheme: Scheme::Light,
                accent: Accent::Postmark,
                motion: MotionLevel::Standard,
            };
            Signal::new(Env {
                resolved,
                scheme: resolved.scheme,
                material: Material::Window,
                blur: BlurState::Unavailable,
                modality: InputModality::Pointer,
            })
        });
        let roster = use_roster(vec!["a", "b", "c", "d"], PITCH);
        ROSTER.set(Some(roster));
        rsx! {
            for entry in roster.entries() {
                p { key: "{entry.key}", "{entry.key}" }
            }
        }
    }

    struct Unpark(Thread);

    impl Wake for Unpark {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }

    /// Let the dom's timers run for `span`, rendering whatever they dirty.
    fn run_for(dom: &mut VirtualDom, span: Duration) {
        let end = Instant::now() + span;
        let waker = Waker::from(Arc::new(Unpark(thread::current())));
        let mut cx = Context::from_waker(&waker);
        while Instant::now() < end {
            let ready = {
                let mut work = pin!(dom.wait_for_work());
                loop {
                    if let Poll::Ready(()) = work.as_mut().poll(&mut cx) {
                        break true;
                    }
                    let now = Instant::now();
                    if now >= end {
                        break false;
                    }
                    thread::park_timeout(end - now);
                }
            };
            if ready {
                dom.render_immediate(&mut NoOpMutations);
            }
        }
    }

    fn presences(dom: &VirtualDom, roster: Roster<&'static str>) -> Vec<(&'static str, Presence)> {
        dom.in_runtime(|| {
            roster
                .entries()
                .into_iter()
                .map(|e| (e.key, e.presence))
                .collect()
        })
    }

    #[test]
    fn roster_hook_drops_the_row_and_heals_on_settle() {
        use Presence::{Entering, Healing, Leaving, Present};
        let mut dom = VirtualDom::new(List);
        dom.rebuild_in_place();
        let roster = ROSTER.get().expect("the list rendered");
        assert!(
            presences(&dom, roster).iter().all(|(_, p)| *p == Entering),
            "first show enters"
        );
        // settle(RowIn, 3) is the longest entrance: 420 + 3 x 26 + 34 at Standard.
        run_for(&mut dom, ms(700));
        assert!(
            presences(&dom, roster).iter().all(|(_, p)| *p == Present),
            "entrances settle: {:?}",
            presences(&dom, roster)
        );
        dom.in_runtime(|| roster.leave("b", Exit::Fold, Emphasis::Plain));
        assert_eq!(
            presences(&dom, roster)[1],
            ("b", Leaving(Exit::Fold)),
            "leaving at once"
        );
        run_for(&mut dom, ms(300));
        assert_eq!(presences(&dom, roster).len(), 4, "still there mid-exit");
        run_for(&mut dom, ms(250));
        let healed = presences(&dom, roster);
        assert_eq!(
            healed.iter().map(|(k, _)| *k).collect::<Vec<_>>(),
            vec!["a", "c", "d"],
            "dropped after settle(Fold)"
        );
        assert!(
            matches!(healed[1].1, Healing { .. }) && matches!(healed[2].1, Healing { .. }),
            "the rows below heal: {healed:?}"
        );
        run_for(&mut dom, ms(400));
        assert!(
            presences(&dom, roster).iter().all(|(_, p)| *p == Present),
            "heals settle"
        );
    }
}
