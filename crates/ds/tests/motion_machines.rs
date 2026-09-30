//! The pure motion machines as tables: the hover intent machine (design/06-INTERACTIONS.md
//! section 3), drag and the pull tab (sections 6 and 9.2).

use ds::motion::fraction_along;
use ds::prelude::*;
use ds_motion::drag::{DRAG_THRESHOLD, Drag, DragPhase, WINDOW_DRAG_THRESHOLD};
use ds_motion::hover_intent::{
    HoverEvent, HoverIntent, HoverProfile, HoverWarmth, IntentEffect, IntentPhase,
};
use std::time::{Duration, Instant};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
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

/// Opens card 1 cold: over at 0, the timer fires at 500.
fn opened() -> Vec<Step> {
    vec![
        (
            0,
            HoverEvent::Over(1, HoverProfile::Card),
            Ph::Pending(1, 500),
            open_after(500),
        ),
        (500, HoverEvent::OpenDue, Ph::Open(1), IntentEffect::Open(1)),
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
        ("cold: waits 500 ms, then opens", opened()),
        (
            "an early open timer is stale and ignored",
            vec![
                (
                    0,
                    Over(1, HoverProfile::Card),
                    Ph::Pending(1, 500),
                    open_after(500),
                ),
                (499, OpenDue, Ph::Pending(1, 500), IntentEffect::None),
            ],
        ),
        (
            "moving within the pending target does not restart the wait",
            vec![
                (
                    0,
                    Over(1, HoverProfile::Card),
                    Ph::Pending(1, 500),
                    open_after(500),
                ),
                (
                    100,
                    Over(1, HoverProfile::Card),
                    Ph::Pending(1, 500),
                    IntentEffect::None,
                ),
            ],
        ),
        (
            "leaving before the card opens cancels it",
            vec![
                (
                    0,
                    Over(1, HoverProfile::Card),
                    Ph::Pending(1, 500),
                    open_after(500),
                ),
                (200, Out, Ph::Idle, IntentEffect::CancelOpen),
                (500, OpenDue, Ph::Idle, IntentEffect::None),
            ],
        ),
        (
            "a suppressed target changes nothing",
            vec![
                (0, OverSuppressed, Ph::Idle, IntentEffect::None),
                (
                    10,
                    Over(1, HoverProfile::Card),
                    Ph::Pending(1, 510),
                    open_after(500),
                ),
                (100, OverSuppressed, Ph::Pending(1, 510), IntentEffect::None),
            ],
        ),
        ("out closes after 150 ms and the hub is warm", closed()),
        (
            "warm: the next card opens at once",
            then(
                closed(),
                [
                    (
                        800,
                        Over(2, HoverProfile::Card),
                        Ph::Pending(2, 800),
                        open_after(0),
                    ),
                    (800, OpenDue, Ph::Open(2), IntentEffect::Open(2)),
                ],
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
                    open_after(500),
                )],
            ),
        ),
        (
            "a new key while open replaces the card instantly",
            then(
                opened(),
                [
                    (
                        600,
                        Over(2, HoverProfile::Card),
                        Ph::Pending(2, 600),
                        open_after(0),
                    ),
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
                    (
                        560,
                        Over(2, HoverProfile::Card),
                        Ph::Pending(2, 560),
                        open_after(0),
                    ),
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
                    (
                        550,
                        Over(1, HoverProfile::Card),
                        Ph::Open(1),
                        IntentEffect::CancelClose,
                    ),
                    (650, CloseDue, Ph::Open(1), IntentEffect::None),
                ],
            ),
        ),
        (
            "over the open card's own target changes nothing",
            then(
                opened(),
                [(
                    600,
                    Over(1, HoverProfile::Card),
                    Ph::Open(1),
                    IntentEffect::None,
                )],
            ),
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
                    (
                        700,
                        Over(2, HoverProfile::Card),
                        Ph::Pending(2, 1200),
                        open_after(500),
                    ),
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
                (
                    0,
                    Over(1, HoverProfile::Card),
                    Ph::Pending(1, 500),
                    open_after(500),
                ),
                (100, ClickInList, Ph::Idle, IntentEffect::CancelOpen),
            ],
        ),
        (
            "space turns the open card into a peek and stays warm",
            then(
                opened(),
                [
                    (600, SpaceKey, Ph::Idle, IntentEffect::Peek(1)),
                    (
                        700,
                        Over(2, HoverProfile::Card),
                        Ph::Pending(2, 700),
                        open_after(0),
                    ),
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
        (0, HoverEvent::Over(1, HoverProfile::Card)),
        (500, HoverEvent::OpenDue),
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
    let t0 = Instant::now();
    // (profile, open delay, close delay)
    const CASES: &[(HoverProfile, u64, u64)] = &[
        (HoverProfile::Tip, 1000, 0),
        (HoverProfile::Card, 500, 150),
        (HoverProfile::Label, 100, 0),
    ];
    for &(profile, open, close) in CASES {
        let machine = HoverIntent::default();
        let (machine, effect) = machine.step(HoverEvent::Over(1u8, profile), t0);
        assert_eq!(
            effect,
            IntentEffect::StartOpen { after: ms(open) },
            "{profile:?} opens after {open} ms"
        );
        let (machine, _) = machine.step(HoverEvent::OpenDue, t0 + ms(open));
        let (_, effect) = machine.step(HoverEvent::Out, t0 + ms(open + 10));
        assert_eq!(
            effect,
            IntentEffect::StartClose { after: ms(close) },
            "{profile:?} closes after {close} ms"
        );
    }
}
