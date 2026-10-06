//! The moving parts of an icon, as data beside its geometry (design/35-SYMBOL-EFFECTS.md).
//!
//! Lucide draws an icon as a list of shapes. Where a symbol animates one piece of it (a trash
//! can's lid, a bell's swing, a lock's shackle), the icon is annotated here with the indices of
//! the shapes that form the piece and the point it turns about. The drawing is Lucide's,
//! untouched; the annotation only says which of its shapes to move. A [`PartGesture::Layers`]
//! icon (Wi-Fi, volume, a battery) lists its layers in the order they step.

use super::Icon;
use ds_core::word::Word;

/// A length on the 24 grid, in tenths of a unit: `Tenths(125)` is 12.5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Tenths(pub i32);

/// A point on the 24 grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pivot {
    /// Across.
    pub x: Tenths,
    /// Down.
    pub y: Tenths,
}

/// What an icon's part does when its symbol effect plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum PartGesture {
    /// The part tips up about its hinge and settles back: a lid, a shackle.
    Lift,
    /// The part swings from its pivot, each swing smaller: a bell.
    Ring,
    /// The part flips open about its edge and closes: a mail flap.
    Flap,
    /// The part leans open about its corner: a folder.
    Open,
    /// The part swells and flashes its fill: a star, a heart.
    Pop,
    /// The part turns once about its centre: a refresh arrow pair.
    Turn,
    /// The parts light one after another, in the order listed: bars, waves, cells.
    Layers,
}

/// One moving part: the shapes that form it and the point it moves about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Part {
    /// Indices into [`Icon::shapes`].
    pub shapes: &'static [usize],
    /// The point the part turns or scales about.
    pub pivot: Pivot,
}

/// The annotation of one icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Parts {
    /// What the parts do.
    pub gesture: PartGesture,
    /// The parts; the shapes that are in none stay still.
    pub parts: &'static [Part],
}

const fn pivot(x: i32, y: i32) -> Pivot {
    Pivot {
        x: Tenths(x),
        y: Tenths(y),
    }
}

const fn part(shapes: &'static [usize], at: Pivot) -> Part {
    Part { shapes, pivot: at }
}

const CENTRE: Pivot = pivot(120, 120);

/// Trash: the lid (the rule and the handle) tips up about its left end.
const TRASH: Parts = Parts {
    gesture: PartGesture::Lift,
    parts: &[part(&[0, 2], pivot(50, 60))],
};
/// Bell: the whole bell swings from its top.
const BELL: Parts = Parts {
    gesture: PartGesture::Ring,
    parts: &[part(&[0, 1], pivot(120, 30))],
};
/// Mail: the flap (the V) flips about the envelope's top edge.
const MAIL: Parts = Parts {
    gesture: PartGesture::Flap,
    parts: &[part(&[1], pivot(120, 70))],
};
/// Folder: the outline leans open about its bottom-left corner.
const FOLDER: Parts = Parts {
    gesture: PartGesture::Open,
    parts: &[part(&[0], pivot(20, 200))],
};
/// Lock: the shackle lifts about its right foot.
const LOCK: Parts = Parts {
    gesture: PartGesture::Lift,
    parts: &[part(&[1], pivot(170, 110))],
};
/// Star: the outline swells about its centre.
const STAR: Parts = Parts {
    gesture: PartGesture::Pop,
    parts: &[part(&[0], CENTRE)],
};
/// Heart: the outline swells about its centre.
const HEART: Parts = Parts {
    gesture: PartGesture::Pop,
    parts: &[part(&[0], pivot(120, 130))],
};
/// Refresh: both arrows turn together about the centre.
const REFRESH: Parts = Parts {
    gesture: PartGesture::Turn,
    parts: &[part(&[0, 1, 2, 3], CENTRE)],
};
/// Wi-Fi: the dot, then the small, middle and large arc.
const WIFI: Parts = Parts {
    gesture: PartGesture::Layers,
    parts: &[
        part(&[0], CENTRE),
        part(&[3], CENTRE),
        part(&[2], CENTRE),
        part(&[1], CENTRE),
    ],
};
/// Volume: the small wave, then the large one; the speaker stays.
const VOLUME: Parts = Parts {
    gesture: PartGesture::Layers,
    parts: &[part(&[1], CENTRE), part(&[2], CENTRE)],
};
/// A full battery: its three cells, left to right.
const BATTERY: Parts = Parts {
    gesture: PartGesture::Layers,
    parts: &[part(&[3], CENTRE), part(&[0], CENTRE), part(&[1], CENTRE)],
};

impl Icon {
    /// The annotation of the icon's moving parts, for the icons a symbol effect animates by part.
    pub fn parts(self) -> Option<&'static Parts> {
        match self {
            Icon::Trash => Some(&TRASH),
            Icon::Bell => Some(&BELL),
            Icon::Mail => Some(&MAIL),
            Icon::Folder => Some(&FOLDER),
            Icon::Lock => Some(&LOCK),
            Icon::Star => Some(&STAR),
            Icon::Heart => Some(&HEART),
            Icon::Refresh => Some(&REFRESH),
            Icon::Wifi => Some(&WIFI),
            Icon::Volume2 => Some(&VOLUME),
            Icon::BatteryFull => Some(&BATTERY),
            _ => None,
        }
    }

    /// Every icon that has an annotation.
    pub const ANIMATED_BY_PART: &[Icon] = &[
        Icon::Trash,
        Icon::Bell,
        Icon::Mail,
        Icon::Folder,
        Icon::Lock,
        Icon::Star,
        Icon::Heart,
        Icon::Refresh,
        Icon::Wifi,
        Icon::Volume2,
        Icon::BatteryFull,
    ];
}

#[cfg(test)]
mod tests {
    use super::{PartGesture, Tenths};
    use crate::icon::Icon;

    #[test]
    fn every_part_names_shapes_the_icon_has_and_no_shape_twice() {
        for &icon in Icon::ANIMATED_BY_PART {
            let parts = icon.parts().unwrap_or_else(|| panic!("{icon:?}: no parts"));
            let count = icon.shapes().len();
            let mut named: Vec<usize> =
                parts.parts.iter().flat_map(|p| p.shapes).copied().collect();
            assert!(!named.is_empty(), "{icon:?}: a part with no shape");
            assert!(parts.parts.iter().all(|p| !p.shapes.is_empty()), "{icon:?}");
            assert!(
                named.iter().all(|&shape| shape < count),
                "{icon:?}: {named:?} of {count}"
            );
            named.sort_unstable();
            let before = named.len();
            named.dedup();
            assert_eq!(named.len(), before, "{icon:?}: a shape in two parts");
        }
    }

    #[test]
    fn a_pivot_is_on_the_grid_and_the_table_lists_exactly_the_annotated_icons() {
        for &icon in Icon::ALL {
            assert_eq!(
                icon.parts().is_some(),
                Icon::ANIMATED_BY_PART.contains(&icon),
                "{icon:?}"
            );
        }
        for &icon in Icon::ANIMATED_BY_PART {
            for part in icon.parts().map_or(&[][..], |p| p.parts) {
                for at in [part.pivot.x, part.pivot.y] {
                    assert!((Tenths(0)..=Tenths(240)).contains(&at), "{icon:?}: {at:?}");
                }
            }
        }
    }

    #[test]
    fn layers_are_listed_in_the_order_they_step() {
        // Wi-Fi lights its dot, then the small, middle and large arcs: shape 0, 3, 2, 1.
        let wifi = Icon::Wifi.parts().map(|p| (p.gesture, p.parts));
        let order: Vec<usize> = wifi
            .iter()
            .flat_map(|(_, parts)| parts.iter().map(|p| p.shapes[0]))
            .collect();
        assert_eq!(order, [0, 3, 2, 1]);
        assert_eq!(wifi.map(|(g, _)| g), Some(PartGesture::Layers));
    }
}
