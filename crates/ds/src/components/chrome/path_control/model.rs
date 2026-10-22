//! What a `PathControl` shows: a path as data, and the pure rule of which segments stay on the
//! band when the path is long.

use crate::components::menus::item::item::{AfterPick, MenuImage, MenuItem};
use ds_core::vocab::{Availability, DropState};
use ds_style::icon::Icon;
use std::ops::Range;

/// The most segments a band shows before the middle ones collapse, unless the caller says
/// otherwise.
pub const DEFAULT_ROOM: usize = 5;

/// The fewest segments a collapsed band keeps: the first, the collapse mark and the last.
const LEAST_ROOM: usize = 3;

/// One folder of a path: a glyph and a name. A `PathControl` draws a row of them, and a
/// `Toolbar` title menu lists them (the folders enclosing the one the title names).
#[derive(Debug, Clone, PartialEq)]
pub struct PathSegment {
    /// Its name.
    pub label: String,
    /// Its glyph.
    pub icon: Icon,
    /// Where the segment stands in a drag the owner is driving, as a `Row`'s `RowState::drop`
    /// is driven. Only a `PathControl` reads it.
    pub drop: DropState,
}

impl PathSegment {
    /// A segment taking no drop.
    pub fn new(label: impl Into<String>, icon: Icon) -> Self {
        PathSegment {
            label: label.into(),
            icon,
            drop: DropState::Idle,
        }
    }

    /// The same segment in `drop`'s place in a drag.
    pub fn with_drop(self, drop: DropState) -> Self {
        PathSegment { drop, ..self }
    }
}

/// One place on the band.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathPart {
    /// The segment at this index.
    Segment(usize),
    /// The segments at these indexes, collapsed behind one mark that opens them as a menu.
    Collapsed(Range<usize>),
}

/// What stays on the band for a path of `len` segments when it has room for `room` places (the
/// collapse mark takes one). A path that fits shows every segment. A longer one keeps the first
/// segment, the collapse mark, and the last `room - 2` segments, so the root and the folder the
/// path ends in are always there. `room` below three counts as three.
pub fn parts(len: usize, room: usize) -> Vec<PathPart> {
    let room = room.max(LEAST_ROOM);
    if len <= room {
        return (0..len).map(PathPart::Segment).collect();
    }
    let tail = room - 2;
    let first_kept = len - tail;
    std::iter::once(PathPart::Segment(0))
        .chain(std::iter::once(PathPart::Collapsed(1..first_kept)))
        .chain((first_kept..len).map(PathPart::Segment))
        .collect()
}

/// Menu rows for `segments`, each yielding its index in the whole path (`first` is the index of
/// the slice's first segment).
pub(crate) fn rows(segments: &[PathSegment], first: usize) -> Vec<MenuItem<usize>> {
    segments
        .iter()
        .enumerate()
        .map(|(at, segment)| MenuItem::Item {
            value: first + at,
            title: segment.label.clone(),
            image: Some(MenuImage::Icon(segment.icon)),
            key: None,
            hint: None,
            check: None,
            availability: Availability::Enabled,
            after: AfterPick::Close,
            text: Default::default(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{PathPart, parts};

    fn segments(at: &[usize]) -> Vec<PathPart> {
        at.iter().copied().map(PathPart::Segment).collect()
    }

    #[test]
    fn a_path_that_fits_keeps_every_segment() {
        assert_eq!(parts(0, 5), segments(&[]));
        assert_eq!(parts(3, 5), segments(&[0, 1, 2]));
        assert_eq!(parts(5, 5), segments(&[0, 1, 2, 3, 4]));
    }

    #[test]
    fn a_long_path_keeps_its_root_and_its_last_folders() {
        let mut want = segments(&[0]);
        want.push(PathPart::Collapsed(1..5));
        want.extend(segments(&[5, 6, 7]));
        assert_eq!(parts(8, 5), want);
    }

    #[test]
    fn a_tight_band_still_keeps_the_root_and_the_last_folder() {
        let mut want = segments(&[0]);
        want.push(PathPart::Collapsed(1..5));
        want.extend(segments(&[5]));
        assert_eq!(parts(6, 1), want);
    }
}
