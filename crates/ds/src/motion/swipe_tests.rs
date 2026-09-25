use super::{
    Click, Sample, Speed, Stamp, SwipeEffect, SwipeInput as I, SwipeLook, SwipeMetrics, SwipeState,
    release_speed, shaped,
};
use crate::geometry::Px;
use std::time::Duration;

fn at(ms: u64) -> Stamp {
    Stamp(Duration::from_millis(ms))
}

/// Run `inputs` from rest; the state and every effect along the way.
fn run(inputs: &[I]) -> (SwipeState, Vec<SwipeEffect>) {
    let metrics = SwipeMetrics::default();
    inputs.iter().fold(
        (SwipeState::default(), Vec::new()),
        |(state, mut effects), &input| {
            let (next, effect) = state.step(input, metrics);
            effects.push(effect);
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
        let (state, _) = run(&[
            I::Down {
                x: Px(100.0),
                at: at(0),
            },
            I::Move {
                x: Px(to),
                at: at(200),
            },
        ]);
        assert_eq!(state.offset(), Px(want), "to {to}");
        assert_eq!(state.look(), SwipeLook::Live, "to {to}");
    }
}

#[test]
fn a_release_springs_back_under_both_thresholds_and_flies_out_past_either() {
    #[rustfmt::skip]
    let cases: &[(&str, &[I], SwipeLook, Px, SwipeEffect)] = &[
        // Slowly to 40 px: back to its place.
        ("slow and short", &[
            I::Down { x: Px(0.0), at: at(0) },
            I::Move { x: Px(20.0), at: at(100) },
            I::Move { x: Px(40.0), at: at(200) },
            I::Up { at: at(210) },
        ], SwipeLook::Rest, Px(0.0), SwipeEffect::None),
        // Slowly past 80 px: dismissed from where it is.
        ("slow and far", &[
            I::Down { x: Px(0.0), at: at(0) },
            I::Move { x: Px(50.0), at: at(200) },
            I::Move { x: Px(90.0), at: at(400) },
            I::Up { at: at(410) },
        ], SwipeLook::Gone, Px(90.0), SwipeEffect::Dismiss),
        // A flick: 30 px in 20 ms is 1500 px/s.
        ("a flick", &[
            I::Down { x: Px(0.0), at: at(0) },
            I::Move { x: Px(10.0), at: at(10) },
            I::Move { x: Px(40.0), at: at(30) },
            I::Up { at: at(35) },
        ], SwipeLook::Gone, Px(40.0), SwipeEffect::Dismiss),
        // The same flick, held still for 150 ms before the release: no fling.
        ("a flick held", &[
            I::Down { x: Px(0.0), at: at(0) },
            I::Move { x: Px(10.0), at: at(10) },
            I::Move { x: Px(40.0), at: at(30) },
            I::Up { at: at(180) },
        ], SwipeLook::Rest, Px(0.0), SwipeEffect::None),
        // A fast flick leftwards never dismisses.
        ("left", &[
            I::Down { x: Px(200.0), at: at(0) },
            I::Move { x: Px(100.0), at: at(10) },
            I::Move { x: Px(0.0), at: at(20) },
            I::Up { at: at(21) },
        ], SwipeLook::Rest, Px(0.0), SwipeEffect::None),
    ];
    for (name, inputs, look, offset, effect) in cases {
        let (state, effects) = run(inputs);
        assert_eq!(state.look(), *look, "{name}");
        assert_eq!(state.offset(), *offset, "{name}");
        assert_eq!(effects.last(), Some(effect), "{name}");
    }
}

#[test]
fn a_scroll_is_summed_and_decided_when_it_goes_quiet() {
    #[rustfmt::skip]
    let cases: &[(&str, &[I], SwipeLook, Px, &[SwipeEffect])] = &[
        ("far", &[
            I::Scroll { dx: Px(30.0), dy: Px(0.0) },
            I::Scroll { dx: Px(30.0), dy: Px(2.0) },
            I::Scroll { dx: Px(30.0), dy: Px(0.0) },
            I::Quiet,
        ], SwipeLook::Gone, Px(90.0), &[SwipeEffect::ArmQuiet, SwipeEffect::ArmQuiet, SwipeEffect::ArmQuiet, SwipeEffect::Dismiss]),
        ("short", &[
            I::Scroll { dx: Px(30.0), dy: Px(0.0) },
            I::Scroll { dx: Px(20.0), dy: Px(0.0) },
            I::Quiet,
        ], SwipeLook::Rest, Px(0.0), &[SwipeEffect::ArmQuiet, SwipeEffect::ArmQuiet, SwipeEffect::None]),
        // Mostly vertical deltas are the list's to scroll: they move nothing.
        ("vertical", &[
            I::Scroll { dx: Px(4.0), dy: Px(30.0) },
            I::Scroll { dx: Px(-4.0), dy: Px(30.0) },
        ], SwipeLook::Rest, Px(0.0), &[SwipeEffect::None, SwipeEffect::None]),
        // Leftwards, damped like a drag.
        ("left", &[
            I::Scroll { dx: Px(-40.0), dy: Px(0.0) },
        ], SwipeLook::Live, Px(-10.0), &[SwipeEffect::ArmQuiet]),
        // Gone, it takes nothing more.
        ("after", &[
            I::Scroll { dx: Px(100.0), dy: Px(0.0) },
            I::Quiet,
            I::Scroll { dx: Px(10.0), dy: Px(0.0) },
            I::Down { x: Px(0.0), at: at(0) },
        ], SwipeLook::Gone, Px(100.0), &[SwipeEffect::ArmQuiet, SwipeEffect::Dismiss, SwipeEffect::None, SwipeEffect::None]),
    ];
    for (name, inputs, look, offset, effects) in cases {
        let (state, seen) = run(inputs);
        assert_eq!(state.look(), *look, "{name}");
        assert_eq!(state.offset(), *offset, "{name}");
        assert_eq!(seen.as_slice(), *effects, "{name}");
    }
}

#[test]
fn a_drag_swallows_the_click_that_ends_it_and_a_press_does_not() {
    #[rustfmt::skip]
    let cases: &[(&str, &[I], Click)] = &[
        ("a press", &[
            I::Down { x: Px(10.0), at: at(0) },
            I::Move { x: Px(11.0), at: at(10) },
            I::Up { at: at(20) },
        ], Click::Passes),
        ("a drag back to where it began", &[
            I::Down { x: Px(10.0), at: at(0) },
            I::Move { x: Px(40.0), at: at(100) },
            I::Move { x: Px(10.0), at: at(300) },
            I::Up { at: at(400) },
        ], Click::Swallowed),
    ];
    for (name, inputs, want) in cases {
        let (state, _) = run(inputs);
        assert_eq!(state.click(), *want, "{name}");
        assert_eq!(
            state.clicked().click(),
            Click::Passes,
            "{name}: one click only"
        );
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
