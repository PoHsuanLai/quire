//! The corner's table (design/13 §13.3.12: dwell 150 ms, re-arm 500 ms).

use std::time::Duration;

use ds_core::machine::Machine;
use ds_core::time::stamp::Stamp;

use super::{Corner, CornerIn, CornerOut, CornerParams, Inside, Rearm};

const PLAIN: CornerParams = CornerParams {
    dwell: Duration::from_millis(150),
    rearm: Duration::from_millis(500),
};

/// Name, state before, input, time, state after, outputs, next wake.
type Case = (
    &'static str,
    Corner,
    CornerIn,
    u64,
    Corner,
    Vec<CornerOut>,
    Option<Stamp>,
);

#[test]
fn the_corner_follows_its_table() {
    use CornerIn::*;
    let dwelling = |t| Corner::Dwelling { until: Stamp(t) };
    let running = |t| Rearm::Running { until: Stamp(t) };
    let spent = |pointer, rearm| Corner::Spent { pointer, rearm };
    let cases: Vec<Case> = vec![
        (
            "entering an armed corner starts the dwell",
            Corner::Armed,
            Enter,
            1000,
            dwelling(1150),
            vec![],
            Some(Stamp(1150)),
        ),
        (
            "leaving while dwelling re-arms and stops the timer",
            dwelling(1150),
            Leave,
            1100,
            Corner::Armed,
            vec![],
            None,
        ),
        (
            "a wake before the dwell is due does nothing",
            dwelling(1150),
            Elapsed,
            1149,
            dwelling(1150),
            vec![],
            Some(Stamp(1150)),
        ),
        (
            "an elapsed with nothing running does nothing",
            Corner::Armed,
            Elapsed,
            1200,
            Corner::Armed,
            vec![],
            None,
        ),
        (
            "the dwell running out fires and starts the re-arm",
            dwelling(1150),
            Elapsed,
            1150,
            spent(Inside::In, running(1650)),
            vec![CornerOut::Fire],
            Some(Stamp(1650)),
        ),
        (
            "the re-arm running out with the pointer in only marks it over",
            spent(Inside::In, running(1650)),
            Elapsed,
            1650,
            spent(Inside::In, Rearm::Over),
            vec![],
            None,
        ),
        (
            "leaving after the re-arm is over arms the corner",
            spent(Inside::In, Rearm::Over),
            Leave,
            1700,
            Corner::Armed,
            vec![],
            None,
        ),
        (
            "leaving while the re-arm runs only notes the pointer is out",
            spent(Inside::In, running(1650)),
            Leave,
            1300,
            spent(Inside::Out, running(1650)),
            vec![],
            Some(Stamp(1650)),
        ),
        (
            "entering while spent only notes the pointer is in",
            spent(Inside::Out, running(1650)),
            Enter,
            1400,
            spent(Inside::In, running(1650)),
            vec![],
            Some(Stamp(1650)),
        ),
        (
            "the re-arm running out with the pointer out arms the corner",
            spent(Inside::Out, running(1650)),
            Elapsed,
            1650,
            Corner::Armed,
            vec![],
            None,
        ),
        (
            "a wake before the re-arm is due does nothing",
            spent(Inside::Out, running(1650)),
            Elapsed,
            1649,
            spent(Inside::Out, running(1650)),
            vec![],
            Some(Stamp(1650)),
        ),
    ];
    for (name, from, input, at, want, outs, wake) in cases {
        let (next, out) = from.step(input, Stamp(at), &PLAIN, &());
        assert_eq!((next, out), (want, outs), "{name}");
        assert_eq!(next.wake(), wake, "{name}: wake");
    }
}

#[test]
fn a_second_dwell_after_a_full_cycle_fires_again() {
    let script = [
        (0, CornerIn::Enter),
        (150, CornerIn::Elapsed),
        (200, CornerIn::Leave),
        (650, CornerIn::Elapsed),
        (700, CornerIn::Enter),
        (850, CornerIn::Elapsed),
    ];
    let (_, fired) = script
        .iter()
        .fold((Corner::Armed, 0), |(state, fired), &(at, input)| {
            let (next, outs) = state.step(input, Stamp(at), &PLAIN, &());
            (next, fired + outs.len())
        });
    assert_eq!(fired, 2, "once per rest in the corner");
}
