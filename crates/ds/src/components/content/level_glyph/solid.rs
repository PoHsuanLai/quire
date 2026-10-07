//! The level glyph's solid geometry on the 24 grid (design/08-ICONS.md section 1.2): the sun's
//! core and eight rays, and the keyboard light's body, rising half sun and five rays. The speaker
//! body and its waves are `ds_style::icon::solid_fan`'s, the slash `ds_style::icon::slash`'s. A ray
//! is a capsule 2 wide with the length and place of Lucide `sun`'s stroked line.

use ds_style::icon::shape::Shape;

/// The sun's disc.
pub(super) const CORE: &[Shape] = &[Shape::Circle {
    cx: "12",
    cy: "12",
    r: "5",
}];

/// The sun's eight rays, clockwise from the top.
pub(super) const RAYS: &[Shape] = &[
    Shape::Solid("M11 2L11 4A1 1 0 0 0 13 4L13 2A1 1 0 0 0 11 2Z"),
    Shape::Solid("M11 20L11 22A1 1 0 0 0 13 22L13 20A1 1 0 0 0 11 20Z"),
    Shape::Solid("M4.22 5.64L5.63 7.05A1 1 0 0 0 7.05 5.63L5.64 4.22A1 1 0 0 0 4.22 5.64Z"),
    Shape::Solid(
        "M16.95 18.37L18.36 19.78A1 1 0 0 0 19.78 18.36L18.37 16.95A1 1 0 0 0 16.95 18.37Z",
    ),
    Shape::Solid("M2 13L4 13A1 1 0 0 0 4 11L2 11A1 1 0 0 0 2 13Z"),
    Shape::Solid("M20 13L22 13A1 1 0 0 0 22 11L20 11A1 1 0 0 0 20 13Z"),
    Shape::Solid("M5.63 16.95L4.22 18.36A1 1 0 0 0 5.64 19.78L7.05 18.37A1 1 0 0 0 5.63 16.95Z"),
    Shape::Solid("M18.36 4.22L16.95 5.63A1 1 0 0 0 18.37 7.05L19.78 5.64A1 1 0 0 0 18.36 4.22Z"),
];

/// The keyboard light's body: a rounded slab with four keys and the space bar cut out (the
/// glyph fills with `fill-rule="evenodd"`).
pub(super) const KEYBOARD_BODY: &[Shape] = &[Shape::Solid(
    "M4 12h16a2 2 0 0 1 2 2v6a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2v-6a2 2 0 0 1 2-2zM5 16a1 1 0 1 0 2 0a1 1 0 1 0-2 0zM9 16a1 1 0 1 0 2 0a1 1 0 1 0-2 0zM13 16a1 1 0 1 0 2 0a1 1 0 1 0-2 0zM17 16a1 1 0 1 0 2 0a1 1 0 1 0-2 0zM9.9 18.1h4.2a.9.9 0 0 1 0 1.8H9.9a.9.9 0 0 1 0-1.8z",
)];

/// The half sun rising from behind the keyboard, a half disc of radius 3 on (12, 9).
pub(super) const KEYBOARD_CORE: &[Shape] = &[Shape::Solid("M9 9a3 3 0 0 1 6 0z")];

/// The keyboard light's five rays.
pub(super) const KEYBOARD_RAYS: &[Shape] = &[
    Shape::Solid("M13 4.5L13 3A1 1 0 0 0 11 3L11 4.5A1 1 0 0 0 13 4.5Z"),
    Shape::Solid("M15.89 6.53L16.95 5.47A1 1 0 0 0 15.53 4.05L14.47 5.11A1 1 0 0 0 15.89 6.53Z"),
    Shape::Solid("M9.53 5.11L8.47 4.05A1 1 0 0 0 7.05 5.47L8.11 6.53A1 1 0 0 0 9.53 5.11Z"),
    Shape::Solid("M16.5 10L18 10A1 1 0 0 0 18 8L16.5 8A1 1 0 0 0 16.5 10Z"),
    Shape::Solid("M7.5 8L6 8A1 1 0 0 0 6 10L7.5 10A1 1 0 0 0 7.5 8Z"),
];
