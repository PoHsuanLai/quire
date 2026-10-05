use super::{
    Click, Sample, Speed, SwipeEffect, SwipeInput as I, SwipeLook, SwipeMetrics, SwipeState,
    release_speed, shaped,
};
use ds_core::geometry::units::Px;
use ds_core::machine::{Elapsed, Machine};
use ds_core::time::stamp::Stamp;

fn at(ms: u64) -> Stamp {
    Stamp(ms)
}

/// One input at a time in ms.
type Timed = (u64, I);

/// Run `inputs` from rest; the state and every effect along the way.
fn run(inputs: &[Timed]) -> (SwipeState, Vec<SwipeEffect>) {
    let metrics = SwipeMetrics::default();
    inputs.iter().fold(
        (SwipeState::default(), Vec::new()),
        |(state, mut effects), &(ms, input)| {
            let (next, out) = state.step(input, at(ms), &metrics, &());
            effects.extend(out);
            (next, effects)
        },
    )
}

#[test]
fn a_drag_follows_right_and_a_quarter_left() {
    #[rustfmt::skip]
    let cases: &[(f32, f32)] = &[
        (130.0, 30.0),
        (100.0, 0.0),
        (60.0, -10.0),
        (20.0, -20.0),
        (100.5, 0.5),
    ];
    for &(to, want) in cases {
        let (state, _) = run(&[(0, I::Down { x: Px(100.0) }), (200, I::Move { x: Px(to) })]);
        assert_eq!(state.offset(), Px(want), "to {to}");
        assert_eq!(state.look(), SwipeLook::Live, "to {to}");
    }
}

/// A release case: its name, the inputs, and the look, offset and last effect they end with.
type ReleaseCase<'a> = (&'a str, &'a [Timed], SwipeLook, Px, Option<SwipeEffect>);

#[test]
fn a_release_springs_back_under_both_thresholds_and_flies_out_past_either() {
    #[rustfmt::skip]
    let cases: &[ReleaseCase<'_>] = &[
        // Slowly to 40 px: back to its place.
        ("slow and short", &[
            (0, I::Down { x: Px(0.0) }),
            (100, I::Move { x: Px(20.0) }),
            (200, I::Move { x: Px(40.0) }),
            (210, I::Up),
        ], SwipeLook::Rest, Px(0.0), None),
        // Slowly past 80 px: dismissed from where it is.
        ("slow and far", &[
            (0, I::Down { x: Px(0.0) }),
            (200, I::Move { x: Px(50.0) }),
            (400, I::Move { x: Px(90.0) }),
            (410, I::Up),
        ], SwipeLook::Gone, Px(90.0), Some(SwipeEffect::Dismiss)),
        // A flick: 30 px in 20 ms is 1500 px/s.
        ("a flick", &[
            (0, I::Down { x: Px(0.0) }),
            (10, I::Move { x: Px(10.0) }),
            (30, I::Move { x: Px(40.0) }),
            (35, I::Up),
        ], SwipeLook::Gone, Px(40.0), Some(SwipeEffect::Dismiss)),
        // The same flick, held still for 150 ms before the release: no fling.
        ("a flick held", &[
            (0, I::Down { x: Px(0.0) }),
            (10, I::Move { x: Px(10.0) }),
            (30, I::Move { x: Px(40.0) }),
            (180, I::Up),
        ], SwipeLook::Rest, Px(0.0), None),
        // A fast flick leftwards never dismisses.
        ("left", &[
            (0, I::Down { x: Px(200.0) }),
            (10, I::Move { x: Px(100.0) }),
            (20, I::Move { x: Px(0.0) }),
            (21, I::Up),
        ], SwipeLook::Rest, Px(0.0), None),
    ];
    for (name, inputs, look, offset, effect) in cases {
        let (state, effects) = run(inputs);
        assert_eq!(state.look(), *look, "{name}");
        assert_eq!(state.offset(), *offset, "{name}");
        assert_eq!(effects.last(), effect.as_ref(), "{name}");
    }
}

/// A scroll case: its name, the inputs, and the look, offset and effects they end with.
type ScrollCase<'a> = (&'a str, &'a [Timed], SwipeLook, Px, &'a [SwipeEffect]);

#[test]
fn a_scroll_is_summed_and_decided_when_it_goes_quiet() {
    let scroll = |dx: f32, dy: f32| I::Scroll {
        dx: Px(dx),
        dy: Px(dy),
    };
    #[rustfmt::skip]
    let cases: &[ScrollCase<'_>] = &[
        ("far", &[
            (0, scroll(30.0, 0.0)),
            (40, scroll(30.0, 2.0)),
            (80, scroll(30.0, 0.0)),
            (80 + QUIET, I::Elapsed),
        ], SwipeLook::Gone, Px(90.0), &[SwipeEffect::Dismiss]),
        ("short", &[
            (0, scroll(30.0, 0.0)),
            (40, scroll(20.0, 0.0)),
            (40 + QUIET, I::Elapsed),
        ], SwipeLook::Rest, Px(0.0), &[]),
        // A wake before the quiet spell is over decides nothing.
        ("early", &[
            (0, scroll(100.0, 0.0)),
            (QUIET - 1, I::Elapsed),
        ], SwipeLook::Live, Px(100.0), &[]),
        // Mostly vertical deltas are the list's to scroll: they move nothing.
        ("vertical", &[
            (0, scroll(4.0, 30.0)),
            (10, scroll(-4.0, 30.0)),
        ], SwipeLook::Rest, Px(0.0), &[]),
        // Leftwards, damped like a drag.
        ("left", &[(0, scroll(-40.0, 0.0))], SwipeLook::Live, Px(-10.0), &[]),
        // Gone, it takes nothing more.
        ("after", &[
            (0, scroll(100.0, 0.0)),
            (QUIET, I::Elapsed),
            (QUIET + 10, scroll(10.0, 0.0)),
            (QUIET + 20, I::Down { x: Px(0.0) }),
        ], SwipeLook::Gone, Px(100.0), &[SwipeEffect::Dismiss]),
    ];
    for (name, inputs, look, offset, effects) in cases {
        let (state, seen) = run(inputs);
        assert_eq!(state.look(), *look, "{name}");
        assert_eq!(state.offset(), *offset, "{name}");
        assert_eq!(seen.as_slice(), *effects, "{name}");
    }
}

/// The quiet spell, in ms (`DelayToken::SwipeQuiet`).
const QUIET: u64 = 120;

#[test]
fn a_scroll_wakes_the_machine_a_quiet_spell_after_the_last_delta() {
    let metrics = SwipeMetrics::default();
    let scrolled = |ms| {
        SwipeState::default()
            .step(
                I::Scroll {
                    dx: Px(10.0),
                    dy: Px(0.0),
                },
                at(ms),
                &metrics,
                &(),
            )
            .0
    };
    assert_eq!(scrolled(0).wake(), Some(at(QUIET)));
    assert_eq!(scrolled(500).wake(), Some(at(500 + QUIET)));
    assert_eq!(SwipeState::default().wake(), None, "at rest: no timer");
    let (decided, _) = scrolled(0).step(I::from(Elapsed), at(QUIET), &metrics, &());
    assert_eq!(decided.wake(), None);
}

#[test]
fn a_drag_swallows_the_click_that_ends_it_and_a_press_does_not() {
    #[rustfmt::skip]
    let cases: &[(&str, &[Timed], Click)] = &[
        ("a press", &[
            (0, I::Down { x: Px(10.0) }),
            (10, I::Move { x: Px(11.0) }),
            (20, I::Up),
        ], Click::Passes),
        ("a drag back to where it began", &[
            (0, I::Down { x: Px(10.0) }),
            (100, I::Move { x: Px(40.0) }),
            (300, I::Move { x: Px(10.0) }),
            (400, I::Up),
        ], Click::Swallowed),
    ];
    for (name, inputs, want) in cases {
        let (state, _) = run(inputs);
        assert_eq!(state.click(), *want, "{name}");
        let (after, _) = state.step(I::Clicked, at(500), &SwipeMetrics::default(), &());
        assert_eq!(after.click(), Click::Passes, "{name}: one click only");
    }
}

#[test]
fn the_release_speed_is_the_last_two_moves() {
    let sample = |x: f32, ms: u64| Sample {
        x: Px(x),
        at: at(ms),
    };
    #[rustfmt::skip]
    let cases = [
        (sample(40.0, 30), Some(sample(10.0, 10)), at(35), Speed(1500.0)),
        (sample(40.0, 30), None, at(35), Speed(0.0)),
        (sample(40.0, 30), Some(sample(10.0, 10)), at(200), Speed(0.0)),
        (sample(40.0, 30), Some(sample(10.0, 30)), at(30), Speed(0.0)),
    ];
    for (last, before, released, want) in cases {
        assert_eq!(
            release_speed(last, before, released),
            want,
            "{last:?} {before:?}"
        );
    }
    assert_eq!(shaped(Px(-8.0), SwipeMetrics::default()), Px(-2.0));
}
