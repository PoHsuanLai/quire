//! The corner's table (design/13 §13.3.12: dwell 150 ms, re-arm 500 ms).

use std::time::Duration;

use super::{CornerIn, CornerMachine, CornerOut, CornerParams, Inside, Phase, Rearm, Token, step};

const PLAIN: CornerParams = CornerParams {
    dwell: Duration::from_millis(150),
    rearm: Duration::from_millis(500),
};

const DWELL: Duration = Duration::from_millis(150);
const REARM: Duration = Duration::from_millis(500);

fn at(phase: Phase, next: u32) -> CornerMachine {
    CornerMachine {
        phase,
        next: Token(next),
    }
}

#[test]
#[ignore = "S0-QB-F"]
fn the_corner_follows_its_table() {
    use CornerIn::*;
    let spent = |pointer, rearm| Phase::Spent { pointer, rearm };
    let cases: Vec<(&str, CornerMachine, CornerIn, CornerMachine, Vec<CornerOut>)> = vec![
        (
            "entering an armed corner starts the dwell",
            at(Phase::Armed, 0),
            Enter,
            at(Phase::Dwelling(Token(0)), 1),
            vec![CornerOut::StartDwell(Token(0), DWELL)],
        ),
        (
            "leaving while dwelling re-arms",
            at(Phase::Dwelling(Token(0)), 1),
            Leave,
            at(Phase::Armed, 1),
            vec![],
        ),
        (
            "a stale dwell does nothing",
            at(Phase::Armed, 1),
            DwellElapsed(Token(0)),
            at(Phase::Armed, 1),
            vec![],
        ),
        (
            "the dwell running out fires and starts the re-arm",
            at(Phase::Dwelling(Token(0)), 1),
            DwellElapsed(Token(0)),
            at(spent(Inside::In, Rearm::Running(Token(1))), 2),
            vec![CornerOut::Fire, CornerOut::StartRearm(Token(1), REARM)],
        ),
        (
            "the re-arm running out with the pointer in only marks it over",
            at(spent(Inside::In, Rearm::Running(Token(1))), 2),
            RearmElapsed(Token(1)),
            at(spent(Inside::In, Rearm::Over), 2),
            vec![],
        ),
        (
            "leaving after the re-arm is over arms the corner",
            at(spent(Inside::In, Rearm::Over), 2),
            Leave,
            at(Phase::Armed, 2),
            vec![],
        ),
        (
            "leaving while the re-arm runs only notes the pointer is out",
            at(spent(Inside::In, Rearm::Running(Token(1))), 2),
            Leave,
            at(spent(Inside::Out, Rearm::Running(Token(1))), 2),
            vec![],
        ),
        (
            "entering while spent only notes the pointer is in",
            at(spent(Inside::Out, Rearm::Running(Token(1))), 2),
            Enter,
            at(spent(Inside::In, Rearm::Running(Token(1))), 2),
            vec![],
        ),
        (
            "the re-arm running out with the pointer out arms the corner",
            at(spent(Inside::Out, Rearm::Running(Token(1))), 2),
            RearmElapsed(Token(1)),
            at(Phase::Armed, 2),
            vec![],
        ),
    ];
    for (name, from, input, want, outs) in cases {
        assert_eq!(step(from, input, PLAIN), (want, outs), "{name}");
    }
}
