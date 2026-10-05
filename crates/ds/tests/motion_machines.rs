//! The pure motion machines as tables: the hover intent machine (design/06-INTERACTIONS.md
//! section 3), drag and the pull tab (sections 6 and 9.2).

use ds::base::machine::Machine;
use ds::base::time::stamp::Stamp;
use ds::motion::drag::fraction_along;
use ds::prelude::*;
use ds_motion::drag::{DRAG_THRESHOLD, Drag, DragPhase, WINDOW_DRAG_THRESHOLD};
use ds_motion::hover_intent::{
    HoverEvent, HoverIntent, HoverProfile, HoverWarmth, IntentEffect, IntentPhase,
};

// ---- hover intent --------------------------------------------------------------------------

/// A phase as a test writes it, with times in ms after the sequence starts.
#[derive(Debug, Clone, Copy)]
enum Ph {
    Idle,
    Pending(u8, u64),
    Open(u8),
    Closing(u8, u64),
}

fn phase(ph: Ph) -> IntentPhase<u8> {
    match ph {
        Ph::Idle => IntentPhase::Idle,
        Ph::Pending(key, due) => IntentPhase::Pending {
            key,
            due: Stamp(due),
        },
        Ph::Open(key) => IntentPhase::Open { key },
        Ph::Closing(key, due) => IntentPhase::Closing {
            key,
            due: Stamp(due),
        },
    }
}

type Step = (u64, HoverEvent<u8>, Ph, Vec<IntentEffect<u8>>);

/// Opens card 1 cold: over at 0, the machine wakes at 500.
fn opened() -> Vec<Step> {
    vec![
        (
            0,
            HoverEvent::Over(1, HoverProfile::Card),
            Ph::Pending(1, 500),
            vec![],
        ),
        (
            500,
            HoverEvent::Elapsed,
            Ph::Open(1),
            vec![IntentEffect::Open(1)],
        ),
    ]
}

/// Opens card 1, then closes it: out at 500, closed at 650, warm until 1050.
fn closed() -> Vec<Step> {
    let mut steps = opened();
    steps.extend([
        (500, HoverEvent::Out, Ph::Closing(1, 650), vec![]),
        (
            650,
            HoverEvent::Elapsed,
            Ph::Idle,
            vec![IntentEffect::Close(1)],
        ),
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
        ("cold: waits 500 ms, then opens", opened()),
        (
            "an early open timer is stale and ignored",
            vec![
                (0, Over(1, HoverProfile::Card), Ph::Pending(1, 500), vec![]),
                (499, Elapsed, Ph::Pending(1, 500), vec![]),
            ],
        ),
        (
            "moving within the pending target does not restart the wait",
            vec![
                (0, Over(1, HoverProfile::Card), Ph::Pending(1, 500), vec![]),
                (
                    100,
                    Over(1, HoverProfile::Card),
                    Ph::Pending(1, 500),
                    vec![],
                ),
            ],
        ),
        (
            "leaving before the card opens cancels it",
            vec![
                (0, Over(1, HoverProfile::Card), Ph::Pending(1, 500), vec![]),
                (200, Out, Ph::Idle, vec![]),
                (500, Elapsed, Ph::Idle, vec![]),
            ],
        ),
        (
            "a suppressed target changes nothing",
            vec![
                (0, OverSuppressed, Ph::Idle, vec![]),
                (10, Over(1, HoverProfile::Card), Ph::Pending(1, 510), vec![]),
                (100, OverSuppressed, Ph::Pending(1, 510), vec![]),
            ],
        ),
        ("out closes after 150 ms and the hub is warm", closed()),
        (
            "warm: the next card opens at once",
            then(
                closed(),
                [(
                    800,
                    Over(2, HoverProfile::Card),
                    Ph::Open(2),
                    vec![IntentEffect::Open(2)],
                )],
            ),
        ),
        (
            "warm ends 400 ms after the close",
            then(
                closed(),
                [(
                    1050,
                    Over(2, HoverProfile::Card),
                    Ph::Pending(2, 1550),
                    vec![],
                )],
            ),
        ),
        (
            "a new key while open replaces the card instantly",
            then(
                opened(),
                [(
                    600,
                    Over(2, HoverProfile::Card),
                    Ph::Open(2),
                    vec![IntentEffect::Open(2)],
                )],
            ),
        ),
        (
            "a new key while closing replaces the card instantly",
            then(
                opened(),
                [
                    (500, Out, Ph::Closing(1, 650), vec![]),
                    (
                        560,
                        Over(2, HoverProfile::Card),
                        Ph::Open(2),
                        vec![IntentEffect::Open(2)],
                    ),
                    (650, Elapsed, Ph::Open(2), vec![]),
                ],
            ),
        ),
        (
            "back onto the same target while closing keeps it open",
            then(
                opened(),
                [
                    (500, Out, Ph::Closing(1, 650), vec![]),
                    (550, Over(1, HoverProfile::Card), Ph::Open(1), vec![]),
                    (650, Elapsed, Ph::Open(1), vec![]),
                ],
            ),
        ),
        (
            "over the open card's own target changes nothing",
            then(
                opened(),
                [(600, Over(1, HoverProfile::Card), Ph::Open(1), vec![])],
            ),
        ),
        (
            "into the card and out again",
            then(
                opened(),
                [
                    (500, Out, Ph::Closing(1, 650), vec![]),
                    (560, EnterCard, Ph::Open(1), vec![]),
                    (700, LeaveCard, Ph::Closing(1, 850), vec![]),
                    (850, Elapsed, Ph::Idle, vec![IntentEffect::Close(1)]),
                ],
            ),
        ),
        (
            "a stale close timer after re-closing waits for the new deadline",
            then(
                opened(),
                [
                    (500, Out, Ph::Closing(1, 650), vec![]),
                    (550, EnterCard, Ph::Open(1), vec![]),
                    (600, LeaveCard, Ph::Closing(1, 750), vec![]),
                    (650, Elapsed, Ph::Closing(1, 750), vec![]),
                    (750, Elapsed, Ph::Idle, vec![IntentEffect::Close(1)]),
                ],
            ),
        ),
        (
            "a click in the list removes the card without warmth",
            then(
                opened(),
                [
                    (600, ClickInList, Ph::Idle, vec![IntentEffect::Remove(1)]),
                    (
                        700,
                        Over(2, HoverProfile::Card),
                        Ph::Pending(2, 1200),
                        vec![],
                    ),
                ],
            ),
        ),
        (
            "a click in the list removes a closing card too",
            then(
                opened(),
                [
                    (500, Out, Ph::Closing(1, 650), vec![]),
                    (550, ClickInList, Ph::Idle, vec![IntentEffect::Remove(1)]),
                ],
            ),
        ),
        (
            "a click in the list cancels a pending card",
            vec![
                (0, Over(1, HoverProfile::Card), Ph::Pending(1, 500), vec![]),
                (100, ClickInList, Ph::Idle, vec![]),
            ],
        ),
        (
            "space turns the open card into a peek and stays warm",
            then(
                opened(),
                [
                    (600, SpaceKey, Ph::Idle, vec![IntentEffect::Peek(1)]),
                    (
                        700,
                        Over(2, HoverProfile::Card),
                        Ph::Open(2),
                        vec![IntentEffect::Open(2)],
                    ),
                ],
            ),
        ),
        (
            "space with no open card does nothing",
            vec![(0, SpaceKey, Ph::Idle, vec![])],
        ),
    ];
    for (name, steps) in cases {
        let mut machine = HoverIntent::default();
        for (n, (at, event, want_phase, want_effect)) in steps.into_iter().enumerate() {
            let (next, effect) = machine.step(event.clone(), Stamp(at), &(), &());
            assert_eq!(
                effect, want_effect,
                "{name}, step {n} ({event:?} at {at} ms): effect"
            );
            assert_eq!(
                next.phase(),
                &phase(want_phase),
                "{name}, step {n} ({event:?} at {at} ms): phase"
            );
            machine = next;
        }
    }
}

#[test]
fn warmth_follows_the_card_and_the_warm_window() {
    let mut machine = HoverIntent::default();
    let mut warmth = Vec::new();
    for (at, event) in [
        (0, HoverEvent::Over(1, HoverProfile::Card)),
        (500, HoverEvent::Elapsed),
        (500, HoverEvent::Out),
        (650, HoverEvent::Elapsed),
    ] {
        machine = machine.step(event, Stamp(at), &(), &()).0;
        warmth.push(machine.warmth(Stamp(at)));
    }
    warmth.push(machine.warmth(Stamp(1049)));
    warmth.push(machine.warmth(Stamp(1050)));
    use HoverWarmth::{Cold, Warm};
    assert_eq!(warmth, vec![Cold, Warm, Warm, Warm, Warm, Cold]);
}

#[test]
fn the_machine_wakes_for_the_open_and_the_close_and_rests_otherwise() {
    let step = |machine: HoverIntent<u8>, event, at| machine.step(event, Stamp(at), &(), &()).0;
    let idle = HoverIntent::default();
    assert_eq!(idle.wake(), None);
    let pending = step(idle, HoverEvent::Over(1, HoverProfile::Card), 0);
    assert_eq!(pending.wake(), Some(Stamp(500)));
    let open = step(pending, HoverEvent::Elapsed, 500);
    assert_eq!(open.wake(), None, "an open card needs no timer");
    let closing = step(open, HoverEvent::Out, 600);
    assert_eq!(closing.wake(), Some(Stamp(750)));
    assert_eq!(step(closing, HoverEvent::Elapsed, 750).wake(), None);
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
fn drag_starts_after_the_euclidean_threshold() {
    // (name, threshold, where the pointer went, whether the drag is live)
    let cases = [
        ("still", DRAG_THRESHOLD, pt(0.0, 0.0), false),
        ("2 + 2 is 2.83", DRAG_THRESHOLD, pt(2.0, 2.0), false),
        ("2.9 down", DRAG_THRESHOLD, pt(0.0, 2.9), false),
        ("3 right", DRAG_THRESHOLD, pt(3.0, 0.0), true),
        ("3 left", DRAG_THRESHOLD, pt(-3.0, 0.0), true),
        ("3 + 4 is 5", DRAG_THRESHOLD, pt(3.0, 4.0), true),
        (
            "a window at 3.9",
            WINDOW_DRAG_THRESHOLD,
            pt(0.0, 3.9),
            false,
        ),
        ("a window at 4", WINDOW_DRAG_THRESHOLD, pt(0.0, -4.0), true),
    ];
    for (name, threshold, to, live) in cases {
        let drag = Drag::new(threshold).down(7u32, pt(0.0, 0.0)).moved(to);
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

#[test]
fn each_hover_profile_waits_by_its_own_open_and_close() {
    // (profile, open delay, close delay)
    const CASES: &[(HoverProfile, u64, u64)] = &[
        (HoverProfile::Tip, 1000, 0),
        (HoverProfile::Card, 500, 150),
        (HoverProfile::Label, 100, 0),
    ];
    for &(profile, open, close) in CASES {
        let machine = HoverIntent::default();
        let (machine, _) = machine.step(HoverEvent::Over(1u8, profile), Stamp(0), &(), &());
        assert_eq!(
            machine.wake(),
            Some(Stamp(open)),
            "{profile:?} opens after {open} ms"
        );
        let (machine, _) = machine.step(HoverEvent::Elapsed, Stamp(open), &(), &());
        let (machine, _) = machine.step(HoverEvent::Out, Stamp(open + 10), &(), &());
        assert_eq!(
            machine.wake(),
            Some(Stamp(open + 10 + close)),
            "{profile:?} closes after {close} ms"
        );
    }
}
