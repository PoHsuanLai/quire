//! How big a collection's tiles are, and how many sit in a row.

use ds_core::geometry::units::Px;
use ds_core::word::Word;

/// The room under a thumbnail for its caption: one line of the small type plus the gaps around
/// it. A tile is `thumb + CAPTION_BAND` tall, and the stylesheet reads the same number from the
/// `--cv-pitch` the view writes, so the rows the view windows are the rows that are drawn.
const CAPTION_BAND: Px = Px(44.0);

/// The shape of the thumbnail's box: Finder's square, or the landscape 16:10 of a screen (a
/// wallpaper, a screenshot), which a square would letterbox to a strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum ThumbAspect {
    /// As tall as wide.
    #[default]
    Square,
    /// 16 wide to 10 tall.
    Landscape,
}

impl ThumbAspect {
    /// The box's height for a `width`.
    pub fn height(self, width: Px) -> Px {
        match self {
            ThumbAspect::Square => width,
            ThumbAspect::Landscape => Px((width.0 * 10.0 / 16.0).round()),
        }
    }
}

/// Whether the view scrolls itself or takes the room its rows need.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum CollectionExtent {
    /// The view fills the box its owner gives it and scrolls inside it, mounting only the rows
    /// near the viewport.
    #[default]
    Fill,
    /// The view is as tall as all its rows and does not scroll itself, for a page that scrolls
    /// (every tile is mounted, so it suits a few hundred tiles, not a folder of thousands).
    Content,
}

/// Finder's icon sizes: how large the thumbnail is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum CollectionSize {
    /// A 48 px thumbnail.
    Small,
    /// A 64 px thumbnail, Finder's default.
    #[default]
    Medium,
    /// A 96 px thumbnail.
    Large,
}

impl CollectionSize {
    /// The thumbnail's side.
    pub fn thumb(self) -> Px {
        match self {
            CollectionSize::Small => Px(48.0),
            CollectionSize::Medium => Px(64.0),
            CollectionSize::Large => Px(96.0),
        }
    }

    /// The tile's least width: the thumbnail and the room beside it for the caption, so
    /// neighbouring captions do not touch.
    pub fn tile(self) -> Px {
        Px(self.thumb().0 + 36.0)
    }

    /// The distance between one row's top and the next: the thumbnail and its caption band.
    pub fn pitch(self) -> Px {
        self.pitch_for(ThumbAspect::Square)
    }

    /// The row pitch when the thumbnail's box has the shape `aspect`.
    pub fn pitch_for(self, aspect: ThumbAspect) -> Px {
        Px(aspect.height(self.thumb()).0 + CAPTION_BAND.0)
    }

    /// How many characters of a caption fit on its line. Blitz has no `text-overflow` and quire
    /// has no text measure for a caption it has not laid out, so this is a budget of the small
    /// type's average advance (about 6.5 px) over the tile's width, not a measurement: a caption
    /// of narrow letters is cut a little early, one of wide letters a little late.
    pub fn caption_chars(self) -> usize {
        (self.tile().0 / 6.5).floor() as usize
    }

    /// The inline style that hands the stylesheet the size's three numbers, for a thumbnail box
    /// of the shape `aspect`; a landscape one adds its height as `--cv-thumb-h` (the square's is
    /// `--cv-thumb`).
    pub(crate) fn style_for(self, aspect: ThumbAspect) -> String {
        let base = format!(
            "--cv-thumb:{}px;--cv-tile:{}px;--cv-pitch:{}px",
            self.thumb().0,
            self.tile().0,
            self.pitch_for(aspect).0
        );
        match aspect {
            ThumbAspect::Square => base,
            ThumbAspect::Landscape => {
                format!("{base};--cv-thumb-h:{}px", aspect.height(self.thumb()).0)
            }
        }
    }
}

/// How many tiles sit in a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Flow {
    /// As many as the view's measured width holds at the size's tile width; one before the view
    /// has been measured.
    #[default]
    Fit,
    /// This many, whatever the width (at least one).
    Columns(u8),
}

impl Flow {
    /// The columns for a view `width` wide (unmeasured when `None`) of tiles `tile` wide.
    pub(crate) fn columns(self, width: Option<Px>, tile: Px) -> usize {
        match (self, width) {
            (Flow::Columns(count), _) => usize::from(count.max(1)),
            (Flow::Fit, Some(width)) if tile.0 > 0.0 && width.0.is_finite() => {
                ((width.0 / tile.0).floor() as usize).max(1)
            }
            (Flow::Fit, _) => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CollectionExtent, CollectionSize, Flow, ThumbAspect};
    use ds_core::geometry::units::Px;

    #[test]
    fn a_tile_is_its_thumbnail_and_a_caption_band_and_a_larger_size_is_larger_in_every_part() {
        // (size, thumb, tile, pitch)
        let cases = [
            (CollectionSize::Small, 48.0, 84.0, 92.0),
            (CollectionSize::Medium, 64.0, 100.0, 108.0),
            (CollectionSize::Large, 96.0, 132.0, 140.0),
        ];
        for (size, thumb, tile, pitch) in cases {
            assert_eq!(size.thumb(), Px(thumb), "{size:?} thumb");
            assert_eq!(size.tile(), Px(tile), "{size:?} tile");
            assert_eq!(size.pitch(), Px(pitch), "{size:?} pitch");
        }
        assert_eq!(
            CollectionSize::Medium.style_for(ThumbAspect::Square),
            "--cv-thumb:64px;--cv-tile:100px;--cv-pitch:108px"
        );
    }

    #[test]
    fn a_landscape_thumbnail_is_sixteen_to_ten_and_shortens_the_row_to_match() {
        let large = CollectionSize::Large;
        assert_eq!(ThumbAspect::Landscape.height(large.thumb()), Px(60.0));
        assert_eq!(ThumbAspect::Square.height(large.thumb()), Px(96.0));
        assert_eq!(large.pitch_for(ThumbAspect::Landscape), Px(104.0));
        assert_eq!(large.pitch_for(ThumbAspect::Square), large.pitch());
        assert_eq!(
            large.style_for(ThumbAspect::Landscape),
            "--cv-thumb:96px;--cv-tile:132px;--cv-pitch:104px;--cv-thumb-h:60px"
        );
        assert_eq!(CollectionExtent::default(), CollectionExtent::Fill);
    }

    #[test]
    fn the_columns_are_the_tiles_the_width_holds_and_at_least_one() {
        let tile = Px(100.0);
        // (flow, width, columns)
        let cases = [
            (Flow::Fit, Some(Px(450.0)), 4),
            (Flow::Fit, Some(Px(400.0)), 4),
            (Flow::Fit, Some(Px(399.0)), 3),
            (Flow::Fit, Some(Px(40.0)), 1),
            (Flow::Fit, Some(Px(f32::NAN)), 1),
            (Flow::Fit, None, 1),
            (Flow::Columns(5), Some(Px(100.0)), 5),
            (Flow::Columns(5), None, 5),
            (Flow::Columns(0), None, 1),
        ];
        for (flow, width, want) in cases {
            assert_eq!(flow.columns(width, tile), want, "{flow:?} in {width:?}");
        }
    }
}
