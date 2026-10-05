//! The double tap (ux §4.2) and the hold (voice §4.1) as tables: each row is a state, an input
//! at a time, and the state, outputs and next wake that must follow.

use std::time::Duration;

use ds_core::machine::{Elapsed, Machine};
use ds_core::time::stamp::Stamp;

use super::{
    CancelCause, HoldIn, HoldKey, HoldOut, HoldParams, KeyEdge, Tap, TapIn, TapOut, TapParams,
};

/// `companion.double_tap_ms` at its default.
const TAP: TapParams = TapParams {
    window: Duration::from_millis(350),
};

/// `voice.hold_ms` and `voice.max_hold_s` at their defaults.
const HOLD: HoldParams = HoldParams {
    hold: Duration::from_millis(300),
    max: Duration::from_secs(60),
};

/// Name, state before, input, time, state after, outputs, next wake.
type Case<M, I, O> = (&'static str, M, I, u64, M, &'static [O], Option<Stamp>);

fn check<M>(cases: &[Case<M, M::In, M::Out>], params: &M::Params)
where
    M: Machine<Ctx = ()> + std::fmt::Debug,
    M::In: Copy,
    M::Out: PartialEq + std::fmt::Debug,
{
    for (name, from, input, at, state, outs, wake) in cases {
        let (next, out) = from.clone().step(*input, Stamp(*at), params, &());
        assert_eq!(next, *state, "{name}: state");
        assert_eq!(out.as_slice(), *outs, "{name}: outputs");
        assert_eq!(next.wake(), *wake, "{name}: wake");
    }
}

#[test]
fn the_double_tap_follows_its_table() {
    let armed = |t| Tap::Armed { until: Stamp(t) };
    #[rustfmt::skip]
    let cases: &[Case<Tap, TapIn, TapOut>] = &[
        ("a first tap arms the window",               Tap::Rest,  TapIn::Tap,     1000, armed(1350), &[],               Some(Stamp(1350))),
        ("a second tap inside the window summons",    armed(1350), TapIn::Tap,    1200, Tap::Rest,   &[TapOut::Summon], None),
        ("a second tap at the window's end summons",  armed(1350), TapIn::Tap,    1350, Tap::Rest,   &[TapOut::Summon], None),
        ("a tap past the window arms afresh",         armed(1350), TapIn::Tap,    1351, armed(1701), &[],               Some(Stamp(1701))),
        ("the window running out rests",              armed(1350), TapIn::Elapsed, 1350, Tap::Rest,  &[],               None),
        ("an early elapse changes nothing",           armed(1350), TapIn::Elapsed, 1349, armed(1350), &[],              Some(Stamp(1350))),
        ("an elapse at rest is nothing",              Tap::Rest,  TapIn::Elapsed, 500,  Tap::Rest,   &[],               None),
    ];
    check(cases, &TAP);
}

#[test]
fn the_hold_follows_its_table() {
    use KeyEdge::*;
    let edge = HoldIn::Edge;
    let down = |t| HoldKey::Down { until: Stamp(t) };
    let talking = |t| HoldKey::Talking { until: Stamp(t) };
    #[rustfmt::skip]
    let cases: &[Case<HoldKey, HoldIn, HoldOut>] = &[
        ("Command down alone starts the hold",           HoldKey::Rest,    edge(CommandDown), 1000, down(1300),      &[],               Some(Stamp(1300))),
        ("a short press is a tap",                       down(1300),       edge(CommandUp),   1100, HoldKey::Rest,   &[HoldOut::Tap],   None),
        ("a release at the deadline before its elapse is nothing", down(1300), edge(CommandUp), 1300, HoldKey::Rest, &[],               None),
        ("another key while down is a chord",            down(1300),       edge(OtherKey),    1100, HoldKey::Chorded, &[],              None),
        ("Escape while down is a chord",                 down(1300),       edge(Escape),      1100, HoldKey::Chorded, &[],              None),
        ("the pointer while down is a chord",            down(1300),       edge(Pointer),     1100, HoldKey::Chorded, &[],              None),
        ("key repeat while down changes nothing",        down(1300),       edge(CommandDown), 1100, down(1300),      &[],               Some(Stamp(1300))),
        ("an early elapse while down changes nothing",   down(1300),       HoldIn::Elapsed,   1299, down(1300),      &[],               Some(Stamp(1300))),
        ("held to the deadline starts talking",          down(1300),       HoldIn::Elapsed,   1300, talking(61300),  &[HoldOut::Start], Some(Stamp(61300))),
        ("a late elapse counts the talk from itself",    down(1300),       HoldIn::Elapsed,   1310, talking(61310),  &[HoldOut::Start], Some(Stamp(61310))),
        ("Command up ends the talk",                     talking(61300),   edge(CommandUp),   5000, HoldKey::Rest,   &[HoldOut::End],   None),
        ("another key cancels the talk",                 talking(61300),   edge(OtherKey),    5000, HoldKey::Chorded, &[HoldOut::Cancel(CancelCause::OtherInput)], None),
        ("Escape cancels the talk",                      talking(61300),   edge(Escape),      5000, HoldKey::Chorded, &[HoldOut::Cancel(CancelCause::Escape)],     None),
        ("the pointer cancels the talk",                 talking(61300),   edge(Pointer),     5000, HoldKey::Chorded, &[HoldOut::Cancel(CancelCause::OtherInput)], None),
        ("key repeat while talking changes nothing",     talking(61300),   edge(CommandDown), 5000, talking(61300),  &[],               Some(Stamp(61300))),
        ("an early elapse while talking changes nothing", talking(61300),  HoldIn::Elapsed,   61299, talking(61300), &[],               Some(Stamp(61300))),
        ("the longest hold ends the talk",               talking(61300),   HoldIn::Elapsed,   61300, HoldKey::Chorded, &[HoldOut::End], None),
        ("Command up after a chord rests",               HoldKey::Chorded, edge(CommandUp),   6000, HoldKey::Rest,   &[],               None),
        ("another key in a chord changes nothing",       HoldKey::Chorded, edge(OtherKey),    6000, HoldKey::Chorded, &[],              None),
        ("Command up at rest is nothing",                HoldKey::Rest,    edge(CommandUp),   500,  HoldKey::Rest,   &[],               None),
        ("another key at rest is nothing",               HoldKey::Rest,    edge(OtherKey),    500,  HoldKey::Rest,   &[],               None),
        ("an elapse at rest is nothing",                 HoldKey::Rest,    HoldIn::Elapsed,   500,  HoldKey::Rest,   &[],               None),
    ];
    check(cases, &HOLD);
}

/// The hold feeds the double tap: two short lone presses inside the window summon, and a chord
/// between them does not count as a tap.
#[test]
fn two_lone_presses_summon_and_a_chord_does_not() {
    /// Every edge at its time, through the hold and then the double tap.
    fn summons(edges: &[(u64, KeyEdge)]) -> usize {
        let (_, _, count) = edges.iter().fold(
            (HoldKey::Rest, Tap::Rest, 0),
            |(hold, tap, count), &(at, edge)| {
                let (hold, outs) = hold.step(HoldIn::Edge(edge), Stamp(at), &HOLD, &());
                let (tap, summoned) = outs.into_iter().filter(|o| *o == HoldOut::Tap).fold(
                    (tap, 0),
                    |(tap, n), _| {
                        let (tap, outs) = tap.step(TapIn::Tap, Stamp(at), &TAP, &());
                        (tap, n + outs.len())
                    },
                );
                (hold, tap, count + summoned)
            },
        );
        count
    }
    use KeyEdge::*;
    let double = [
        (0, CommandDown),
        (80, CommandUp),
        (200, CommandDown),
        (260, CommandUp),
    ];
    let chord_between = [
        (0, CommandDown),
        (80, CommandUp),
        (200, CommandDown),
        (220, OtherKey),
        (260, CommandUp),
    ];
    assert_eq!(summons(&double), 1, "two lone presses 200 ms apart");
    assert_eq!(summons(&chord_between), 0, "the second press was Command-C");
}

#[test]
fn elapsed_converts_to_each_machines_elapse() {
    assert_eq!(TapIn::from(Elapsed), TapIn::Elapsed);
    assert_eq!(HoldIn::from(Elapsed), HoldIn::Elapsed);
}
