//! Where a picture sits inside its box, as inline CSS: a rect the caller has fitted (a screenshot's
//! card, a PDF page, a preview pane's media), written to whole hundredths of a pixel so the box a
//! host sizes for and the box the document lays out are the same number.

use crate::core::geometry::units::{Px, Rect};

/// The picture's inline placement, in whole hundredths of a pixel.
pub fn picture_style(picture: Rect) -> String {
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
    use super::picture_style;
    use crate::core::geometry::units::{Point, Px, Rect, Size};

    #[test]
    fn a_rect_is_written_to_hundredths_of_a_pixel() {
        let rect = |left, top, width, height| Rect {
            origin: Point {
                x: Px(left),
                y: Px(top),
            },
            size: Size {
                width: Px(width),
                height: Px(height),
            },
        };
        // (rect, its style)
        #[rustfmt::skip]
        let cases = [
            (rect(4.0, 4.0, 232.0, 130.5), "left:4px;top:4px;width:232px;height:130.5px"),
            (rect(83.75, 4.0, 72.5, 145.0), "left:83.75px;top:4px;width:72.5px;height:145px"),
            (rect(4.0, 33.0, 232.0, 58.0), "left:4px;top:33px;width:232px;height:58px"),
            (rect(0.004, 0.006, 10.123_4, 9.999_9), "left:0px;top:0.01px;width:10.12px;height:10px"),
        ];
        for (rect, want) in cases {
            assert_eq!(picture_style(rect), want, "{rect:?}");
        }
    }
}
