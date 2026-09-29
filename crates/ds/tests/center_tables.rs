//! design/26: the control center modules' moment tables as data (catalogue 5.2.2, 5.2.3,
//! 5.2.8, 5.2.10), and the pieces the modules draw with: the filled moon.

use ds::detail::{EventStamp, Moment, first_table, moment_table};
use ds::{Icon, RowPhase, Shape};

const ONE: EventStamp = EventStamp(1);
const TWO: EventStamp = EventStamp(2);

#[test]
fn the_row_table() {
    use RowPhase::{Failed, Pending, Rest, Succeeded};
    moment_table(&[
        // Joining, connecting, switching.
        (Rest, Pending(ONE), Moment::Pending),
        (Failed(ONE), Pending(TWO), Moment::Pending),
        (Pending(ONE), Pending(TWO), Moment::Pending),
        // The operation the row watched landed.
        (Pending(ONE), Succeeded(ONE), Moment::Success),
        // A success nobody watched start is no ceremony (R1).
        (Rest, Succeeded(ONE), Moment::Change),
        (Succeeded(ONE), Succeeded(TWO), Moment::Change),
        // Failure: a new stamp each time.
        (Pending(ONE), Failed(TWO), Moment::Failure),
        (Failed(ONE), Failed(TWO), Moment::Failure),
        (Succeeded(ONE), Rest, Moment::Change),
        (Pending(ONE), Rest, Moment::Change),
    ]);
    first_table(&[
        (Rest, Moment::Rest),
        (Pending(ONE), Moment::Pending),
        (Succeeded(ONE), Moment::Rest),
        (Failed(ONE), Moment::Rest),
    ]);
}

#[test]
fn the_filled_moon_is_the_outline_moon_filled() {
    let [Shape::Path(outline)] = Icon::Moon.shapes() else {
        panic!("the moon is one path");
    };
    assert_eq!(Icon::MoonFilled.shapes(), &[Shape::Solid(outline)][..]);
    assert!(Icon::SHELL.contains(&Icon::MoonFilled) && Icon::ALL.contains(&Icon::MoonFilled));
}
