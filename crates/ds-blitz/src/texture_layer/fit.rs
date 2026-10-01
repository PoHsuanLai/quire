//! Where a texture's shown part lands in a layer's box: the pure arithmetic behind
//! [`TextureFit`]. A draw is the whole texture drawn with `transform` (its top-left texel at the
//! origin, one texel one unit before the transform) and clipped to `clip`, so a crop is the
//! texture shifted and scaled until the source rectangle sits where it should, with everything
//! outside it clipped away. The renderer's texture paint always draws the whole texture; this is
//! how a source rectangle and a fit reach it without a change to the renderer.

use super::model::{TexelRect, Texels, TextureFit};
use peniko::kurbo::{Affine, Rect};

/// The most tiles one layer draws under [`TextureFit::Tile`]: a one-pixel source in a window
/// would otherwise record millions of draws. Tiles past it are left out, row by row from the
/// bottom.
pub(crate) const MAX_TILES: usize = 4096;

/// One draw of the texture: where it goes, and the part of the box it may touch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Draw {
    /// Texel space to box space (device pixels from the box's top-left).
    pub(crate) transform: Affine,
    /// What this draw may paint: the placed source rectangle cut to the box.
    pub(crate) clip: Rect,
}

/// The part of a `width` by `height` texture that `source` names (the whole texture for
/// `None`), or `None` where nothing of it is inside the texture.
pub(crate) fn visible_source(
    source: Option<TexelRect>,
    width: Texels,
    height: Texels,
) -> Option<TexelRect> {
    let Some(rect) = source else {
        return Some(TexelRect::whole(width, height)).filter(|_| width.0 > 0 && height.0 > 0);
    };
    let right = rect.x.0.saturating_add(rect.width.0).min(width.0);
    let bottom = rect.y.0.saturating_add(rect.height.0).min(height.0);
    (rect.x.0 < right && rect.y.0 < bottom).then(|| TexelRect {
        x: rect.x,
        y: rect.y,
        width: Texels(right - rect.x.0),
        height: Texels(bottom - rect.y.0),
    })
}

/// The draws that put `source` into a box of `area` device pixels, `(width, height)`.
pub(crate) fn place(fit: TextureFit, source: TexelRect, area: (f64, f64)) -> Vec<Draw> {
    let (box_w, box_h) = area;
    let (src_w, src_h) = (f64::from(source.width.0), f64::from(source.height.0));
    if src_w <= 0.0 || src_h <= 0.0 || box_w <= 0.0 || box_h <= 0.0 {
        return Vec::new();
    }
    let (across, down) = (box_w / src_w, box_h / src_h);
    let (scale_x, scale_y) = match fit {
        TextureFit::Contain => (across.min(down), across.min(down)),
        TextureFit::Cover => (across.max(down), across.max(down)),
        TextureFit::Fill => (across, down),
        TextureFit::Actual | TextureFit::Tile => (1.0, 1.0),
    };
    let (dest_w, dest_h) = (src_w * scale_x, src_h * scale_y);
    let bounds = Rect::new(0.0, 0.0, box_w, box_h);
    let draw = |left: f64, top: f64| Draw {
        transform: Affine::new([
            scale_x,
            0.0,
            0.0,
            scale_y,
            left - f64::from(source.x.0) * scale_x,
            top - f64::from(source.y.0) * scale_y,
        ]),
        clip: Rect::new(left, top, left + dest_w, top + dest_h).intersect(bounds),
    };
    match fit {
        TextureFit::Tile => {
            let columns = (box_w / dest_w).ceil() as usize;
            let rows = (box_h / dest_h).ceil() as usize;
            (0..rows)
                .flat_map(|row| (0..columns).map(move |column| (column, row)))
                .take(MAX_TILES)
                .map(|(column, row)| draw(column as f64 * dest_w, row as f64 * dest_h))
                .collect()
        }
        TextureFit::Actual => {
            // Whole device pixels, so one texel is one pixel and nothing is resampled.
            vec![draw(
                ((box_w - dest_w) / 2.0).floor(),
                ((box_h - dest_h) / 2.0).floor(),
            )]
        }
        TextureFit::Contain | TextureFit::Cover | TextureFit::Fill => {
            vec![draw((box_w - dest_w) / 2.0, (box_h - dest_h) / 2.0)]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Draw, MAX_TILES, TexelRect, Texels, TextureFit, place, visible_source};
    use peniko::kurbo::{Affine, Rect};

    fn one(sx: f64, sy: f64, tx: f64, ty: f64, clip: Rect) -> Vec<Draw> {
        vec![Draw {
            transform: Affine::new([sx, 0.0, 0.0, sy, tx, ty]),
            clip,
        }]
    }

    #[test]
    fn each_fit_places_the_source_in_the_box() {
        let wide = TexelRect::new(0, 0, 100, 50);
        let cases: Vec<(&str, TextureFit, TexelRect, (f64, f64), Vec<Draw>)> = vec![
            (
                "contain letterboxes a wide picture",
                TextureFit::Contain,
                wide,
                (200.0, 200.0),
                one(2.0, 2.0, 0.0, 50.0, Rect::new(0.0, 50.0, 200.0, 150.0)),
            ),
            (
                "cover overhangs and the box cuts it",
                TextureFit::Cover,
                wide,
                (200.0, 200.0),
                one(4.0, 4.0, -100.0, 0.0, Rect::new(0.0, 0.0, 200.0, 200.0)),
            ),
            (
                "fill stretches each axis",
                TextureFit::Fill,
                wide,
                (200.0, 200.0),
                one(2.0, 4.0, 0.0, 0.0, Rect::new(0.0, 0.0, 200.0, 200.0)),
            ),
            (
                "actual centres on whole pixels",
                TextureFit::Actual,
                wide,
                (201.0, 200.0),
                one(1.0, 1.0, 50.0, 75.0, Rect::new(50.0, 75.0, 150.0, 125.0)),
            ),
            (
                "actual larger than the box is cut",
                TextureFit::Actual,
                wide,
                (60.0, 20.0),
                one(1.0, 1.0, -20.0, -15.0, Rect::new(0.0, 0.0, 60.0, 20.0)),
            ),
            (
                "a crop scales the source rectangle and clips the rest away",
                TextureFit::Contain,
                TexelRect::new(10, 20, 40, 20),
                (80.0, 80.0),
                one(2.0, 2.0, -20.0, -20.0, Rect::new(0.0, 20.0, 80.0, 60.0)),
            ),
        ];
        for (name, fit, source, area, want) in cases {
            assert_eq!(place(fit, source, area), want, "{name}");
        }
    }

    #[test]
    fn tiles_repeat_from_the_corner_and_the_box_cuts_the_last() {
        let tiles = place(TextureFit::Tile, TexelRect::new(0, 0, 4, 2), (10.0, 3.0));
        let clips: Vec<Rect> = tiles.iter().map(|draw| draw.clip).collect();
        let want = [
            Rect::new(0.0, 0.0, 4.0, 2.0),
            Rect::new(4.0, 0.0, 8.0, 2.0),
            Rect::new(8.0, 0.0, 10.0, 2.0),
            Rect::new(0.0, 2.0, 4.0, 3.0),
            Rect::new(4.0, 2.0, 8.0, 3.0),
            Rect::new(8.0, 2.0, 10.0, 3.0),
        ];
        assert_eq!(clips, want);
        let origins: Vec<(f64, f64)> = tiles
            .iter()
            .map(|draw| {
                let [_, _, _, _, tx, ty] = draw.transform.as_coeffs();
                (tx, ty)
            })
            .collect();
        assert_eq!(origins[1], (4.0, 0.0));
        assert_eq!(origins[3], (0.0, 2.0));
    }

    #[test]
    fn a_tiny_source_is_capped_at_the_most_tiles() {
        let tiles = place(TextureFit::Tile, TexelRect::new(0, 0, 1, 1), (800.0, 600.0));
        assert_eq!(tiles.len(), MAX_TILES);
    }

    #[test]
    fn nothing_to_draw_places_nothing() {
        let cases = [
            ("an empty source", TexelRect::new(0, 0, 0, 5), (10.0, 10.0)),
            ("an empty box", TexelRect::new(0, 0, 5, 5), (0.0, 10.0)),
        ];
        for (name, source, area) in cases {
            assert!(place(TextureFit::Fill, source, area).is_empty(), "{name}");
        }
    }

    #[test]
    fn a_source_is_cut_to_the_texture() {
        let (w, h) = (Texels(100), Texels(50));
        let cases = [
            (
                "none is the whole texture",
                None,
                Some(TexelRect::new(0, 0, 100, 50)),
            ),
            (
                "inside stays",
                Some(TexelRect::new(10, 10, 20, 20)),
                Some(TexelRect::new(10, 10, 20, 20)),
            ),
            (
                "overhang is cut",
                Some(TexelRect::new(90, 40, 50, 50)),
                Some(TexelRect::new(90, 40, 10, 10)),
            ),
            (
                "outside is nothing",
                Some(TexelRect::new(100, 0, 5, 5)),
                None,
            ),
            ("empty is nothing", Some(TexelRect::new(5, 5, 0, 5)), None),
        ];
        for (name, source, want) in cases {
            assert_eq!(visible_source(source, w, h), want, "{name}");
        }
        assert_eq!(
            visible_source(None, Texels(0), Texels(5)),
            None,
            "an empty texture"
        );
    }
}
