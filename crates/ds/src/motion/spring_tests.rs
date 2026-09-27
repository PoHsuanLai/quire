use super::{Leg, Millis, Ratio, Spring, SpringPhase, State};
use std::time::Duration;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn leg(damping: Ratio, from: f64, velocity: f64, to: f64) -> Leg {
    Leg {
        start: State {
            position: from,
            velocity,
        },
        target: to,
        spring: Spring::new(damping, Millis(450)),
    }
}

/// The first millisecond at which the leg has come to rest.
fn rests_at(leg: Leg) -> u64 {
    (0..5_000)
        .find(|&t| leg.phase(ms(t), 1.0) == SpringPhase::Rest)
        .unwrap_or(u64::MAX)
}

#[test]
fn a_critical_spring_arrives_without_overshoot_and_rests() {
    let leg = leg(Ratio::CRITICAL, 0.0, 0.0, 100.0);
    let peak = (0..2_000)
        .map(|t| leg.at(ms(t)).position)
        .fold(f64::MIN, f64::max);
    assert!(peak <= 100.0 + 1e-9, "peak {peak}");
    let rest = rests_at(leg);
    assert!((300..1_200).contains(&rest), "rests at {rest} ms");
    assert!((leg.at(ms(rest)).position - 100.0).abs() < 0.25);
}

#[test]
fn a_momentum_spring_overshoots_a_little_and_still_rests() {
    let leg = leg(Ratio::MOMENTUM, 0.0, 0.0, 100.0);
    let peak = (0..2_000)
        .map(|t| leg.at(ms(t)).position)
        .fold(f64::MIN, f64::max);
    assert!((100.5..106.0).contains(&peak), "peak {peak}");
    assert!(rests_at(leg) < 1_500, "rests at {}", rests_at(leg));
}

#[test]
fn a_standing_leg_rests_at_once_and_a_moving_one_starts_where_it_is() {
    let still = Leg::still(42.0, Spring::new(Ratio::CRITICAL, Millis(300)));
    assert_eq!(still.phase(Duration::ZERO, 1.0), SpringPhase::Rest);
    let thrown = leg(Ratio::CRITICAL, 10.0, 800.0, 10.0);
    assert_eq!(
        thrown.at(Duration::ZERO),
        State {
            position: 10.0,
            velocity: 800.0
        }
    );
    assert_eq!(
        thrown.phase(Duration::ZERO, 1.0),
        SpringPhase::Moving,
        "at its target but moving"
    );
}

#[test]
fn a_retarget_at_forty_percent_keeps_position_and_velocity() {
    let out = leg(Ratio::CRITICAL, 0.0, 0.0, 100.0);
    let t = (0..2_000)
        .find(|&t| out.at(ms(t)).position >= 40.0)
        .expect("reaches 40 %");
    let at = out.at(ms(t));
    let back = Leg {
        start: at,
        target: -50.0,
        spring: out.spring,
    };
    let first = back.at(Duration::ZERO);
    assert!(
        (first.position - at.position).abs() < 1e-9,
        "position jumped: {first:?} {at:?}"
    );
    assert!(
        (first.velocity - at.velocity).abs() < 1e-9,
        "velocity jumped: {first:?} {at:?}"
    );
    assert!(at.velocity > 100.0, "it was moving: {}", at.velocity);
    // A tenth of a millisecond on, the velocity has changed by far less than 5 % of itself:
    // the new target changes the acceleration, never the speed at once.
    let next = back.at(Duration::from_micros(100));
    let change = (next.velocity - at.velocity).abs() / at.velocity.abs();
    assert!(change < 0.05, "velocity jumped by {:.1} %", change * 100.0);
    let moved = next.position - at.position;
    assert!(
        (moved - at.velocity / 10_000.0).abs() < 0.01,
        "moved {moved}"
    );
}

#[test]
fn every_spring_rests_within_a_second_and_a_half() {
    for damping in [Ratio::CRITICAL, Ratio::MOMENTUM] {
        for response in [Millis(300), Millis(450)] {
            for (from, v, to) in [
                (0.0, 0.0, 100.0),
                (0.0, 1500.0, 100.0),
                (300.0, -2000.0, 0.0),
            ] {
                let leg = Leg {
                    start: State {
                        position: from,
                        velocity: v,
                    },
                    target: to,
                    spring: Spring::new(damping, response),
                };
                let rest = rests_at(leg);
                assert!(
                    rest < 1_500,
                    "{damping:?} {response:?} {from}->{to} at {v}: {rest} ms"
                );
            }
        }
    }
}
