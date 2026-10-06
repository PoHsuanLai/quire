//! Where a part is in its gesture, as pure arithmetic a table pins (design/35-SYMBOL-EFFECTS.md
//! section 3): the pose of one part at a share of a cycle, in thousandths. Every gesture starts
//! and ends at rest, so a cycle that finishes leaves the icon as Lucide drew it.

use ds_style::icon::parts::{PartGesture, Tenths};
use ds_style::icon::posed::{PartPose, Thousandths};

/// The pose at `at` thousandths of a cycle, one stop of a gesture.
type Stop = (i32, PartPose);

const fn turned(at: i32, tenth_degrees: i32) -> Stop {
    (
        at,
        PartPose {
            rotate: Tenths(tenth_degrees),
            ..PartPose::REST
        },
    )
}

const REST: PartPose = PartPose::REST;

/// A lid or a shackle: tips up about its hinge and settles back.
const LIFT: &[Stop] = &[
    (0, REST),
    (
        350,
        PartPose {
            rotate: Tenths(-160),
            dy: Tenths(-10),
            ..REST
        },
    ),
    (1000, REST),
];

/// A bell: swings from its top, each swing smaller.
const RING: &[Stop] = &[
    (0, REST),
    turned(150, 120),
    turned(300, -100),
    turned(450, 70),
    turned(600, -40),
    turned(800, 15),
    (1000, REST),
];

/// A flap: flips about its edge to half open height and closes.
const FLAP: &[Stop] = &[
    (0, REST),
    (
        400,
        PartPose {
            scale_y: Thousandths(-500),
            ..REST
        },
    ),
    (1000, REST),
];

/// A folder: leans open about its corner and settles.
const OPEN: &[Stop] = &[(0, REST), turned(400, -40), (1000, REST)];

/// A star or a heart: swells and flashes its fill, then settles.
const POP: &[Stop] = &[
    (0, REST),
    (
        300,
        PartPose {
            scale_x: Thousandths(1250),
            scale_y: Thousandths(1250),
            fill: Thousandths(450),
            ..REST
        },
    ),
    (1000, REST),
];

/// A whole revolution; 360 degrees draws as rest.
const TURN: &[Stop] = &[(0, REST), turned(1000, 3600)];

/// How much of the cycle the layers spend dimming before the first lights.
const DIM_SHARE: i32 = 100;
/// How dim an unlit layer is.
const DIM: i32 = 300;
/// How much of the cycle one layer takes to come up.
const RISE_SHARE: i32 = 100;

/// The pose of part `index` of `count` at `at` thousandths of a cycle in `gesture`.
pub fn pose(gesture: PartGesture, index: usize, count: usize, at: Thousandths) -> PartPose {
    let at = at.0.clamp(0, 1000);
    match gesture {
        PartGesture::Lift => along(LIFT, at),
        PartGesture::Ring => along(RING, at),
        PartGesture::Flap => along(FLAP, at),
        PartGesture::Open => along(OPEN, at),
        PartGesture::Pop => along(POP, at),
        PartGesture::Turn => along(TURN, at),
        PartGesture::Layers => PartPose {
            opacity: Thousandths(layer_opacity(index, count, at)),
            ..REST
        },
    }
}

/// A layer's ink: all dim at first, then each comes up in its turn and stays.
fn layer_opacity(index: usize, count: usize, at: i32) -> i32 {
    let count = i32::try_from(count.max(1)).unwrap_or(1);
    let index = i32::try_from(index).unwrap_or(0);
    let window = (1000 - DIM_SHARE - RISE_SHARE) / count;
    let start = DIM_SHARE + index * window;
    if at < DIM_SHARE {
        return lerp(1000, DIM, at, DIM_SHARE);
    }
    lerp(DIM, 1000, (at - start).clamp(0, RISE_SHARE), RISE_SHARE)
}

fn lerp(from: i32, to: i32, along: i32, span: i32) -> i32 {
    from + (to - from) * along / span.max(1)
}

/// The pose between the stops that bracket `at`.
fn along(stops: &[Stop], at: i32) -> PartPose {
    let Some(after) = stops.iter().position(|(stop, _)| *stop >= at) else {
        return stops.last().map_or(REST, |(_, pose)| *pose);
    };
    let (to_at, to) = stops[after];
    let Some((from_at, from)) = after.checked_sub(1).map(|before| stops[before]) else {
        return to;
    };
    let span = to_at - from_at;
    let step = |from: i32, to: i32| lerp(from, to, at - from_at, span);
    PartPose {
        rotate: Tenths(step(from.rotate.0, to.rotate.0)),
        dx: Tenths(step(from.dx.0, to.dx.0)),
        dy: Tenths(step(from.dy.0, to.dy.0)),
        scale_x: Thousandths(step(from.scale_x.0, to.scale_x.0)),
        scale_y: Thousandths(step(from.scale_y.0, to.scale_y.0)),
        opacity: Thousandths(step(from.opacity.0, to.opacity.0)),
        fill: Thousandths(step(from.fill.0, to.fill.0)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(gesture: PartGesture, index: usize, count: usize, share: i32) -> PartPose {
        pose(gesture, index, count, Thousandths(share))
    }

    #[test]
    fn every_gesture_begins_at_rest_and_ends_looking_like_rest() {
        for gesture in [
            PartGesture::Lift,
            PartGesture::Ring,
            PartGesture::Flap,
            PartGesture::Open,
            PartGesture::Pop,
            PartGesture::Turn,
            PartGesture::Layers,
        ] {
            assert_eq!(at(gesture, 0, 3, 0), REST, "{gesture:?} at 0");
            let end = at(gesture, 0, 3, 1000);
            // A whole turn is a rest that is not equal to it.
            let turns = Tenths(end.rotate.0 % 3600);
            assert_eq!(
                PartPose {
                    rotate: turns,
                    ..end
                },
                REST,
                "{gesture:?} at 1000"
            );
        }
    }

    #[test]
    fn a_lid_tips_up_and_a_bell_swings_both_ways() {
        let lifted = at(PartGesture::Lift, 0, 1, 350);
        assert_eq!((lifted.rotate, lifted.dy), (Tenths(-160), Tenths(-10)));
        let halfway = at(PartGesture::Lift, 0, 1, 175);
        assert_eq!(halfway.rotate, Tenths(-80));
        assert!(at(PartGesture::Ring, 0, 1, 150).rotate > Tenths(0));
        assert!(at(PartGesture::Ring, 0, 1, 300).rotate < Tenths(0));
        assert_eq!(at(PartGesture::Flap, 0, 1, 400).scale_y, Thousandths(-500));
        assert_eq!(at(PartGesture::Pop, 0, 1, 300).fill, Thousandths(450));
    }

    #[test]
    fn layers_come_up_one_after_another_in_order() {
        // (share, ink of each of three layers).
        const CASES: &[(i32, [i32; 3])] = &[
            (0, [1000, 1000, 1000]),
            (100, [300, 300, 300]),
            (200, [1000, 300, 300]),
            (470, [1000, 1000, 300]),
            (750, [1000, 1000, 1000]),
            (1000, [1000, 1000, 1000]),
        ];
        for &(share, want) in CASES {
            let got: Vec<i32> = (0..3)
                .map(|index| at(PartGesture::Layers, index, 3, share).opacity.0)
                .collect();
            assert_eq!(got, want, "{share}");
        }
    }
}
