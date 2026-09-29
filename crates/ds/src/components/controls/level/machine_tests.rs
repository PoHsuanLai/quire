use super::*;
use crate::core::geometry::units::{Point, Size};

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

fn pressing(x: f32) -> Hold {
    Hold::Pressing { x: Px(x) }
}

fn held() -> Hold {
    Hold::Held { track: track() }
}

#[test]
fn press_drag_release() {
    use crate::components::controls::level::machine::LevelInput as I;
    #[rustfmt::skip]
    let cases = [
        // The press holds at once; its measurement jumps the level to the pointer.
        (Hold::Idle, 400, I::Down { x: Px(150.0) }, pressing(150.0), None),
        (pressing(150.0), 400, I::Measured { x: Px(150.0), track: track() }, held(), Some(250)),
        // A move before the measurement is kept, and the measurement goes to it.
        (pressing(150.0), 400, I::Move { x: Px(250.0) }, pressing(250.0), None),
        (pressing(250.0), 400, I::Measured { x: Px(150.0), track: track() }, held(), Some(750)),
        // A click let go before the measurement landed sets the level and holds nothing.
        (Hold::Idle, 400, I::Measured { x: Px(150.0), track: track() }, Hold::Idle, Some(250)),
        // Moves follow 1:1 while held.
        (held(), 250, I::Move { x: Px(250.0) }, held(), Some(750)),
        (held(), 750, I::Move { x: Px(250.0) }, held(), None),
        // A move with nothing held is nothing.
        (Hold::Idle, 500, I::Move { x: Px(250.0) }, Hold::Idle, None),
        // Release lets go and reports nothing new.
        (held(), 750, I::Up, Hold::Idle, None),
    ];
    for (from, value, input, want, reported) in cases {
        let got = step(from, Fraction(value), input);
        assert_eq!(got.state, want, "{input:?}");
        assert_eq!(got.value, reported.map(Fraction), "{input:?}");
    }
}

#[test]
fn keys_step_by_sixteenths_and_shift_by_sixty_fourths() {
    use crate::components::controls::level::machine::KeyStep::{Coarse, Fine};
    use crate::components::controls::level::machine::Nudge::{Down, Up};
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
    let stepped = step(Hold::Idle, Fraction(500), key);
    assert_eq!(stepped.value, Some(Fraction(563)));
    let at_top = step(Hold::Idle, Fraction(1000), key);
    assert_eq!(at_top.value, None, "nothing to report at the top");
}
