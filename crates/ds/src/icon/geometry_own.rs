//! quire's own glyphs: composed on Lucide's grid (24, a 2 px round stroke, no fills) where Lucide
//! has no mark for the thing, so they sit in a row of Lucide glyphs without standing out.
//!
//! `Switches`, the control center's: two toggle tracks with their knobs at
//! opposite ends, the mark of a panel of switches (macOS draws its control center the same way;
//! nothing here is copied from its symbol). Built from Lucide `toggle-left`/`toggle-right`'s
//! parts (a pill track and a round knob inside it), stacked as `sliders-horizontal` stacks its
//! rails:
//!
//! - Each track is a pill 20 x 8 (`rx` 4) from x 2 to 22, the full width Lucide's own toggles
//!   use. The two sit at y 2-10 and 14-22, so with the stroke their outer edges run 1-11 and
//!   13-23: Lucide's outermost reach (a 10-radius circle's), and 2 px of clear space between them
//!   at every size down to 16 px.
//! - Each knob is a dot of radius 1 (4 px across with the stroke) centred in its track's end cap
//!   (x 6 on the top track, x 18 on the bottom), so a clear pixel rings it inside the track's
//!   6 px inner height. Lucide's toggle knob is a ring; at this track height a ring would touch
//!   the track, so the knob is drawn as Lucide draws its dots (a stroked point).
//! - Top knob left (off), bottom knob right (on): the pair reads as switches, not as one toggle
//!   drawn twice.
//!
//! `MoonFilled` (design/26): Lucide `moon`'s own outline, filled as well as stroked
//! (`Shape::Solid`), so it has exactly the outline moon's silhouette and a Focus disc can morph
//! from one to the other with nothing but the fill arriving.

use super::shape::Shape;

/// Lucide `moon`, filled.
pub(super) const MOON_FILLED: &[Shape] = &[Shape::Solid(
    "M20.985 12.486a9 9 0 1 1-9.473-9.472c.405-.022.617.46.402.803a6 6 0 0 0 8.268 8.268c.344-.215.825-.004.803.401",
)];

/// Two toggles, knobs at opposite ends.
pub(super) const SWITCHES: &[Shape] = &[
    Shape::Rect {
        x: "2",
        y: "2",
        width: "20",
        height: "8",
        rx: "4",
    },
    Shape::Circle {
        cx: "6",
        cy: "6",
        r: "1",
    },
    Shape::Rect {
        x: "2",
        y: "14",
        width: "20",
        height: "8",
        rx: "4",
    },
    Shape::Circle {
        cx: "18",
        cy: "18",
        r: "1",
    },
];
