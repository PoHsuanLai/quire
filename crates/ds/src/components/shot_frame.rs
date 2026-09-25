//! Where a screenshot thumbnail's picture sits (design/04-COMPONENTS.md section 39): the card
//! is `width` across, the picture box inside a 4 px mat takes the picture's own ratio held
//! between 2:1 and 16:10, and the picture is fitted into that box, centred, so a picture outside
//! those ratios is letterboxed on the material. Computed here rather than left to
//! `object-fit`, so the box the host sizes its surface for is a number a test can read.

use crate::components::image_source::ImageSize;
use crate::geometry::{Point, Px, Rect, Size};

/// The mat between the card's edge and the picture box: `--s-4`, which the stylesheet's picture
/// radius subtracts from the card's.
pub(crate) const MAT: Px = Px(4.0);

/// The tallest picture box, as height over width: 16:10, the shape sill sizes the thumbnail's
/// surface for; a taller picture is pillarboxed.
const TALLEST: f32 = 10.0 / 16.0;

/// The widest picture box, as height over width: 2:1; a wider picture is letterboxed rather
/// than drawing a card too thin to press.
const WIDEST: f32 = 1.0 / 2.0;

/// A card's shape: its outer size, and the picture's rect inside it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ShotFrame {
    /// The card's outer size.
    pub card: Size,
    /// The picture, relative to the card's top-left corner.
    pub picture: Rect,
}

/// The card `width` across for a picture of `size`. A picture with no pixels fills the box.
pub(crate) fn shot_frame(width: Px, size: ImageSize) -> ShotFrame {
    let inner = Px((width.0 - 2.0 * MAT.0).max(0.0));
    let ratio = ratio(size);
    let box_height = Px(inner.0 * ratio.unwrap_or(TALLEST).clamp(WIDEST, TALLEST));
    let picture = fit(
        Size {
            width: inner,
            height: box_height,
        },
        ratio,
    );
    ShotFrame {
        card: Size {
            width,
            height: Px(box_height.0 + 2.0 * MAT.0),
        },
        picture: Rect {
            origin: Point {
                x: Px(MAT.0 + picture.origin.x.0),
                y: Px(MAT.0 + picture.origin.y.0),
            },
            size: picture.size,
        },
    }
}

/// Height over width, or `None` for a picture with no pixels.
fn ratio(size: ImageSize) -> Option<f32> {
    (size.width > 0 && size.height > 0).then(|| size.height as f32 / size.width as f32)
}

/// A picture of `ratio` fitted inside `room`, centred: the largest rect of that ratio that fits.
fn fit(room: Size, ratio: Option<f32>) -> Rect {
    let size = match ratio {
        None => room,
        Some(ratio) if room.width.0 * ratio <= room.height.0 => Size {
            width: room.width,
            height: Px(room.width.0 * ratio),
        },
        Some(ratio) => Size {
            width: Px(room.height.0 / ratio),
            height: room.height,
        },
    };
    Rect {
        origin: Point {
            x: Px((room.width.0 - size.width.0) / 2.0),
            y: Px((room.height.0 - size.height.0) / 2.0),
        },
        size,
    }
}

/// The picture's inline placement, in whole hundredths of a pixel.
pub(crate) fn picture_style(picture: Rect) -> String {
    format!(
        "left:{}px;top:{}px;width:{}px;height:{}px",
        round(picture.left()),
        round(picture.top()),
        round(picture.size.width),
        round(picture.size.height)
    )
}

fn round(px: Px) -> f32 {
    (px.0 * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::{MAT, picture_style, shot_frame};
    use crate::components::image_source::ImageSize;
    use crate::geometry::Px;
    use crate::tokens::SpacingToken;

    #[test]
    fn the_box_follows_the_picture_between_two_to_one_and_sixteen_to_ten() {
        // (width, picture, card height, picture's placement)
        #[rustfmt::skip]
        let cases = [
            // 16:9, inside the range: the box is the picture, no bars.
            (240.0, (1920, 1080), 138.5, "left:4px;top:4px;width:232px;height:130.5px"),
            // 16:10 exactly.
            (240.0, (1440, 900), 153.0, "left:4px;top:4px;width:232px;height:145px"),
            // Portrait: the box stops at 16:10 and the picture is pillarboxed.
            (240.0, (900, 1800), 153.0, "left:83.75px;top:4px;width:72.5px;height:145px"),
            // A panorama: the box stops at 2:1 and the picture is letterboxed.
            (240.0, (4000, 1000), 124.0, "left:4px;top:33px;width:232px;height:58px"),
            // No pixels: the picture fills a 16:10 box.
            (240.0, (0, 0), 153.0, "left:4px;top:4px;width:232px;height:145px"),
            // Another width scales the same way.
            (320.0, (1920, 1200), 203.0, "left:4px;top:4px;width:312px;height:195px"),
        ];
        for (width, (w, h), card, want) in cases {
            let frame = shot_frame(
                Px(width),
                ImageSize {
                    width: w,
                    height: h,
                },
            );
            assert_eq!(frame.card.width, Px(width), "{w}x{h}");
            assert!(
                (frame.card.height.0 - card).abs() < 0.01,
                "{w}x{h}: {frame:?}"
            );
            assert_eq!(picture_style(frame.picture), want, "{w}x{h}");
        }
    }

    #[test]
    fn the_mat_is_the_spacing_token_the_stylesheet_subtracts() {
        assert_eq!(u32::from(SpacingToken::S4.tenths()), (MAT.0 * 10.0) as u32);
    }
}
