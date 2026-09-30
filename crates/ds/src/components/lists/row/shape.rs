//! A row's shape: what a `Row` draws beyond its title, detail and accessory. `Plain` is an
//! ordinary row; `File` and `Clip` are the launcher's file and clipboard results (design/30
//! section 2.4, absorbed from the menu's row shapes). Drawn by `shape_view`.

use crate::components::content::image_source::{ImageSize, ImageSource};
use ds_core::word::Word;

/// How near a Today tab is to going (design/30 section 2.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Expiry {
    /// Plenty of time left.
    #[default]
    Later,
    /// About to expire: the row is drawn quieter.
    Soon,
}

/// How a row draws.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum RowShape {
    /// Leading element, title, detail, accessory.
    #[default]
    Plain,
    /// A file: its thumbnail in the tile (else the row's tile, a mime glyph), the name as the
    /// title, the parent folder (`~/Documents`) as the second line in place of the detail, and
    /// when it was modified as trailing data, before the row's trail.
    File {
        /// A picture of the file, drawn in the tile; `None` keeps the row's own tile.
        thumb: Option<ImageSource>,
        /// The parent folder, as the person reads it.
        location: String,
        /// When it was last changed, already worded ("Yesterday", "12:04").
        modified: String,
    },
    /// A Today tab: a temporary row of a source list that expires (design/30 section 2.11). The
    /// title is its page, and how long it has left is trailing data, before the row's trail.
    Today {
        /// What is left, already worded ("2 h", "12 min").
        left: String,
        /// Whether it is about to go.
        expiry: Expiry,
    },
    /// A clipboard entry: its text in the code face (clipped to `lines`) or its picture, and its
    /// age as trailing data, before the row's trail.
    Clip {
        /// What was copied.
        body: ClipBody,
        /// How long ago, already worded ("2 min").
        age: String,
    },
}

/// What a clipboard entry holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipBody {
    /// Text: drawn in `--font-code` in place of the title, at most `lines` lines (1 to 4).
    Text {
        /// The text, or its start.
        excerpt: String,
        /// How many lines it may take.
        lines: u8,
    },
    /// A picture: drawn under the title, 44 px tall at its own aspect (at most 200 wide).
    Image {
        /// Its pixels.
        src: ImageSource,
        /// Its size; only the ratio is read.
        size: ImageSize,
    },
}

impl RowShape {
    /// A file's thumbnail, when it has one: it takes the leading element's place.
    pub(crate) fn thumb_src(&self) -> Option<String> {
        match self {
            RowShape::File {
                thumb: Some(thumb), ..
            } => Some(thumb.0.clone()),
            RowShape::Plain
            | RowShape::File { .. }
            | RowShape::Today { .. }
            | RowShape::Clip { .. } => None,
        }
    }
}

/// The tallest a clipboard picture draws in a row.
pub(crate) const CLIP_IMAGE_HEIGHT: f32 = 44.0;
/// The widest it draws.
pub(crate) const CLIP_IMAGE_WIDTH: f32 = 200.0;

/// A clipboard text's line budget, kept to 1..=4 so a row never towers over the list.
pub(crate) fn clip_lines(lines: u8) -> u8 {
    lines.clamp(1, 4)
}

/// A clipboard picture's box in a row: 44 tall at `size`'s aspect, narrowed to 200 wide (and
/// shortened with it) when the picture is wider than that. A size with no area is square.
pub(crate) fn clip_box(size: ImageSize) -> (f32, f32) {
    let ratio = match (size.width, size.height) {
        (0, _) | (_, 0) => 1.0,
        (width, height) => width as f32 / height as f32,
    };
    let width = CLIP_IMAGE_HEIGHT * ratio;
    if width <= CLIP_IMAGE_WIDTH {
        (width, CLIP_IMAGE_HEIGHT)
    } else {
        (CLIP_IMAGE_WIDTH, CLIP_IMAGE_WIDTH / ratio)
    }
}

#[cfg(test)]
mod tests {
    use super::{clip_box, clip_lines};
    use crate::components::content::image_source::ImageSize;

    #[test]
    fn a_clip_picture_is_44_tall_and_at_most_200_wide() {
        // (width, height, drawn width, drawn height)
        const CASES: &[(u32, u32, f32, f32)] = &[
            (100, 100, 44.0, 44.0),
            (1600, 900, 78.22, 44.0),
            (1000, 100, 200.0, 20.0),
            (0, 50, 44.0, 44.0),
        ];
        for &(width, height, w, h) in CASES {
            let (got_w, got_h) = clip_box(ImageSize { width, height });
            assert!(
                (got_w - w).abs() < 0.01 && (got_h - h).abs() < 0.01,
                "{width}x{height}: {got_w}x{got_h}"
            );
        }
    }

    #[test]
    fn a_clip_text_takes_one_to_four_lines() {
        const CASES: &[(u8, u8)] = &[(0, 1), (1, 1), (2, 2), (4, 4), (9, 4)];
        for &(asked, want) in CASES {
            assert_eq!(clip_lines(asked), want, "{asked}");
        }
    }
}
