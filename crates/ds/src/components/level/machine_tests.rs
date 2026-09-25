use super::*;
use crate::geometry::units::{Point, Size};

/// A 200 px track from x 100.
fn track() -> Rect {
    Rect {
        origin: Point {
            x: Px(100.0),
            y: Px(0.0),
        },
        size: Size {
            width: Px(200.0),
            height: Px(26.0),
        },
    }
}

fn pressing(x: f32) -> LevelState {
    LevelState {
        hold: Hold::Pressing { x: Px(x) },
        stretch: Stretch::None,
    }
}

fn held() -> LevelState {
    LevelState {
        hold: Hold::Held { track: track() },
        stretch: Stretch::None,
    }
}

#[test]
fn press_drag_release() {
    use LevelInput as I;
    #[rustfmt::skip]
    let cases = [
        // The press holds at once; its measurement jumps the level to the pointer.
        (LevelState::IDLE, 400, I::Down { x: Px(150.0) }, pressing(150.0), None),
        (pressing(150.0), 400, I::Measured { x: Px(150.0), track: track() }, held(), Some(250)),
        // A move before the measurement is kept, and the measurement goes to it.
        (pressing(150.0), 400, I::Move { x: Px(250.0) }, pressing(250.0), None),
        (pressing(250.0), 400, I::Measured { x: Px(150.0), track: track() }, held(), Some(750)),
        // A click let go before the measurement landed sets the level and holds nothing.
        (LevelState::IDLE, 400, I::Measured { x: Px(150.0), track: track() }, LevelState::IDLE, Some(250)),
        // Moves follow 1:1 while held.
        (held(), 250, I::Move { x: Px(250.0) }, held(), Some(750)),
        (held(), 750, I::Move { x: Px(250.0) }, held(), None),
        // A move with nothing held is nothing.
        (LevelState::IDLE, 500, I::Move { x: Px(250.0) }, LevelState::IDLE, None),
        // Release lets go and reports nothing new.
        (held(), 750, I::Up, LevelState::IDLE, None),
    ];
    for (from, value, input, want, reported) in cases {
        let got = step(from, Fraction(value), input, Rubber::On);
        assert_eq!(got.state, want, "{input:?}");
        assert_eq!(got.value, reported.map(Fraction), "{input:?}");
    }
}

#[test]
fn past_either_end_the_capsule_stretches_and_springs_back() {
    let over = step(
        held(),
        Fraction(1000),
        LevelInput::Move { x: Px(306.0) },
        Rubber::On,
    );
    assert_eq!(over.state.stretch, Stretch::End(Px(6.0 * 6.0 / 18.0)));
    assert_eq!(over.value, None, "the level stays at the end");
    let under = step(
        held(),
        Fraction(0),
        LevelInput::Move { x: Px(94.0) },
        Rubber::On,
    );
    assert_eq!(under.state.stretch, Stretch::Start(Px(2.0)));
    let far = stretch(track(), Px(10_000.0), Rubber::On);
    assert!(
        matches!(far, Stretch::End(Px(by)) if by < STRETCH_MAX.0 && by > 5.9),
        "{far:?}"
    );
    let released = step(over.state, Fraction(1000), LevelInput::Up, Rubber::On);
    assert_eq!(released.state.stretch, Stretch::None);
    // Reduced: no rubber band at all.
    let reduced = step(
        held(),
        Fraction(1000),
        LevelInput::Move { x: Px(330.0) },
        Rubber::Off,
    );
    assert_eq!(reduced.state.stretch, Stretch::None);
}

#[test]
fn keys_step_by_sixteenths_and_shift_by_sixty_fourths() {
    use KeyStep::{Coarse, Fine};
    use Nudge::{Down, Up};
    #[rustfmt::skip]
    let cases = [
        (500, Up, Coarse, 563),
        (500, Down, Coarse, 438),
        // Between two points: to the next one, not by a whole step.
        (510, Up, Coarse, 563),
        (510, Down, Coarse, 500),
        (500, Up, Fine, 516),
        (500, Down, Fine, 484),
        (1000, Up, Coarse, 1000),
        (0, Down, Coarse, 0),
        (990, Up, Coarse, 1000),
    ];
    for (value, nudge, size, want) in cases {
        assert_eq!(
            keyed(Fraction(value), nudge, size),
            Fraction(want),
            "{value} {nudge:?} {size:?}"
        );
    }
    let key = LevelInput::Key {
        nudge: Up,
        step: Coarse,
    };
    let stepped = step(LevelState::IDLE, Fraction(500), key, Rubber::On);
    assert_eq!(stepped.value, Some(Fraction(563)));
    let at_top = step(LevelState::IDLE, Fraction(1000), key, Rubber::On);
    assert_eq!(at_top.value, None, "nothing to report at the top");
}

#[test]
fn a_step_is_crossed_only_between_sixteenths() {
    assert_eq!(crossing(Fraction(500), Fraction(520)), Crossing::Within);
    assert_eq!(crossing(Fraction(500), Fraction(570)), Crossing::Crossed);
    assert_eq!(crossing(Fraction(990), Fraction(1000)), Crossing::Within);
    assert_eq!(step_of(Fraction(1000)), 15);
    assert_eq!(step_of(Fraction(0)), 0);
}
