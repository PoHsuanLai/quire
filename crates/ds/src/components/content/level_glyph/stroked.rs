//! The level glyph's stroked geometry on the 24 grid (design/08-ICONS.md section 1.2): Lucide's
//! 2 unit stroke with round caps and joins, as the Bluetooth glyph is. The speaker body is the
//! outline `Volume`'s, the sun is the outline `Sun`'s split into its disc and eight rays, and the
//! keyboard light is a keyboard under a rising half sun with five rays. The waves are three arcs
//! on one centre (11, 12) at radii 4.5, 7.5 and 10.5, 3 apart, the same spacing as Lucide's two.
//! The slash is not here: it stays the solid bar of `ds_style::icon::slash`.

use ds_style::icon::Icon;
use ds_style::icon::shape::Shape;

/// The speaker body, Lucide `volume`.
pub(super) fn body() -> &'static [Shape] {
    Icon::Volume.shapes()
}

/// The inner, middle and outer wave.
pub(super) const WAVE_1: &[Shape] = &[Shape::Path("M14.45 9.1a4.5 4.5 0 0 1 0 5.8")];
pub(super) const WAVE_2: &[Shape] = &[Shape::Path("M16.75 7.18a7.5 7.5 0 0 1 0 9.64")];
pub(super) const WAVE_3: &[Shape] = &[Shape::Path("M19.04 5.25a10.5 10.5 0 0 1 0 13.5")];

/// The sun's disc, Lucide `sun`'s circle.
pub(super) fn core() -> &'static [Shape] {
    &Icon::Sun.shapes()[..1]
}

/// The sun's eight rays, Lucide `sun`'s lines.
pub(super) fn rays() -> &'static [Shape] {
    &Icon::Sun.shapes()[1..]
}

/// The keyboard light's body: a rounded slab with four keys (stroked points) and the space bar.
pub(super) const KEYBOARD_BODY: &[Shape] = &[
    Shape::Rect {
        x: "2",
        y: "12",
        width: "20",
        height: "10",
        rx: "2",
    },
    Shape::Path("M6 16h.01"),
    Shape::Path("M10 16h.01"),
    Shape::Path("M14 16h.01"),
    Shape::Path("M18 16h.01"),
    Shape::Path("M8 19h8"),
];

/// The half sun rising from behind the keyboard, an arc of radius 2 on (12, 9).
pub(super) const KEYBOARD_CORE: &[Shape] = &[Shape::Path("M10 9a2 2 0 0 1 4 0")];

/// The keyboard light's five rays, short strokes clockwise from the top.
pub(super) const KEYBOARD_RAYS: &[Shape] = &[
    Shape::Path("M12 4.5V3"),
    Shape::Path("M15.18 5.82l1.06-1.06"),
    Shape::Path("M8.82 5.82L7.76 4.76"),
    Shape::Path("M16.5 9H18"),
    Shape::Path("M7.5 9H6"),
];
