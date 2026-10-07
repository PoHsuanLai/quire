//! The solid forms of the icons a symbol effect animates by part (design/35-SYMBOL-EFFECTS.md):
//! the shapes that stay and one list of shapes per [`super::parts::Part`], in the order
//! [`Icon::parts`] lists them. An icon whose whole drawing is its one part is its
//! [`Icon::solid_shapes`]; the rest are drawn here (design/08-ICONS.md section 1.2.1).

use super::Icon;
use super::shape::Shape;
use super::solid_fan::{SPEAKER, WAVE_1, WAVE_2, WIFI_DOT, WIFI_LARGE, WIFI_MID, WIFI_SMALL};

/// A solid icon split for animation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolidParts {
    /// The shapes in no part: they never move.
    pub still: Vec<Shape>,
    /// The shapes of each part, in the order of [`Icon::parts`].
    pub parts: Vec<Vec<Shape>>,
}

/// A trash can's lid (the rule and the handle) and its body.
const TRASH_LID: &[Shape] = &[
    Shape::Solid("M4.5 4h15a1.5 1.5 0 0 1 0 3h-15a1.5 1.5 0 0 1 0-3z"),
    Shape::Solid("M9.5 1.5h5a1 1 0 0 1 1 1V4h-7V2.5a1 1 0 0 1 1-1z"),
];
const TRASH_BODY: &[Shape] = &[Shape::Solid(
    "M5.5 8.5h13l-.9 11.4A2.5 2.5 0 0 1 15.1 22H8.9a2.5 2.5 0 0 1-2.5-2.1z",
)];

/// An envelope's flap, a triangle, and the body under it, a clear gap between.
const MAIL_FLAP: &[Shape] = &[Shape::Solid(
    "M2 6.5A2.5 2.5 0 0 1 4.5 4h15A2.5 2.5 0 0 1 22 6.5L12 13.5z",
)];
const MAIL_BODY: &[Shape] = &[Shape::Solid(
    "M2 8.6L12 15.6L22 8.6V17.5A2.5 2.5 0 0 1 19.5 20h-15A2.5 2.5 0 0 1 2 17.5z",
)];

/// A padlock's shackle and its body.
const LOCK_SHACKLE: &[Shape] = &[Shape::Solid(
    "M6 11V7a6 6 0 0 1 12 0v4h-2V7a4 4 0 0 0-8 0v4z",
)];
const LOCK_BODY: &[Shape] = &[Shape::Solid(
    "M5.5 11h13a2.5 2.5 0 0 1 2.5 2.5v6a2.5 2.5 0 0 1-2.5 2.5h-13A2.5 2.5 0 0 1 3 19.5v-6A2.5 2.5 0 0 1 5.5 11z",
)];

/// A battery's three cells, left to right, inside the empty body.
const CELLS: [Shape; 3] = [
    Shape::Rect {
        x: "3.6",
        y: "8.2",
        width: "4",
        height: "7.6",
        rx: "0.8",
    },
    Shape::Rect {
        x: "8.5",
        y: "8.2",
        width: "4",
        height: "7.6",
        rx: "0.8",
    },
    Shape::Rect {
        x: "13.4",
        y: "8.2",
        width: "4",
        height: "7.6",
        rx: "0.8",
    },
];

fn of(shapes: &[Shape]) -> Vec<Shape> {
    shapes.to_vec()
}

impl Icon {
    /// The solid form of an icon in [`Icon::ANIMATED_BY_PART`], split like its [`Icon::parts`];
    /// `None` for any other.
    pub fn solid_parts(self) -> Option<SolidParts> {
        self.parts()?;
        let split = |still: &[Shape], parts: &[&[Shape]]| SolidParts {
            still: of(still),
            parts: parts.iter().map(|part| of(part)).collect(),
        };
        Some(match self {
            Icon::Trash => split(TRASH_BODY, &[TRASH_LID]),
            Icon::Mail => split(MAIL_BODY, &[MAIL_FLAP]),
            Icon::Lock => split(LOCK_BODY, &[LOCK_SHACKLE]),
            Icon::Wifi => split(
                &[],
                &[&[WIFI_DOT], &[WIFI_SMALL], &[WIFI_MID], &[WIFI_LARGE]],
            ),
            Icon::Volume2 => split(&[SPEAKER], &[&[WAVE_1], &[WAVE_2]]),
            Icon::BatteryFull => SolidParts {
                still: Icon::Battery.solid_shapes().to_vec(),
                parts: CELLS.iter().map(|cell| vec![cell.clone()]).collect(),
            },
            // The whole drawing is the one part.
            whole => SolidParts {
                still: Vec::new(),
                parts: vec![whole.solid_shapes().to_vec()],
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_animated_icon_has_a_solid_split_with_one_list_per_part() {
        for &icon in Icon::ANIMATED_BY_PART {
            let solid = icon
                .solid_parts()
                .unwrap_or_else(|| panic!("{icon:?}: no solid parts"));
            let count = icon.parts().map_or(0, |parts| parts.parts.len());
            assert_eq!(solid.parts.len(), count, "{icon:?}");
            assert!(solid.parts.iter().all(|part| !part.is_empty()), "{icon:?}");
        }
        assert_eq!(Icon::Check.solid_parts(), None);
    }
}
