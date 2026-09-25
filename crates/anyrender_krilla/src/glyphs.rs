//! Glyph runs as PDF text: which glyphs a page keeps, the text each carries, and the krilla
//! glyphs that draw them in the embedded face.

use crate::fonts::Face;
use crate::geometry;
use crate::paint::Ink;
use crate::run_text::{GlyphSource, RunKey, RunText};
use crate::sources::Sources;
use anyrender::Glyph;
use krilla::geom::Point as KrillaPoint;
use krilla::paint::{Fill, FillRule, LineCap, LineJoin, Stroke};
use krilla::surface::Surface;
use krilla::text::{GlyphId, KrillaGlyph};
use peniko::FontData;
use peniko::kurbo::{Affine, Point, Rect, Vec2};

/// Where a scene's glyphs may land, in the scene's own coordinates (before whatever transform
/// the surface already has).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GlyphArea {
    /// Every glyph is drawn.
    #[default]
    Anywhere,
    /// A glyph is drawn only if the centre of its box (its advance across, ascent to descent
    /// down) lies inside: a page's clip hides what falls outside it, but a PDF's text layer
    /// does not, so a glyph under the clip would still be found, selected and copied from the
    /// wrong page. The centre decides so each glyph belongs to exactly one of two pages that
    /// share an edge.
    Within(Rect),
}

impl GlyphArea {
    fn holds(self, point: Point) -> Keep {
        match self {
            GlyphArea::Anywhere => Keep::Yes,
            GlyphArea::Within(rect) => {
                let inside = point.x >= rect.x0
                    && point.x < rect.x1
                    && point.y >= rect.y0
                    && point.y < rect.y1;
                if inside { Keep::Yes } else { Keep::No }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Keep {
    Yes,
    No,
}

/// What a scene did with the glyphs it was given.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GlyphTally {
    /// Glyphs written to the page.
    pub drawn: usize,
    /// Glyphs outside the [`GlyphArea`], not written.
    pub dropped: usize,
    /// Written glyphs whose text came from the caller's [`RunTexts`](crate::RunTexts).
    pub layout_text: usize,
    /// Written glyphs whose text was read back from the font's cmap.
    pub cmap_text: usize,
}

/// One `draw_glyphs` call, owned.
pub(crate) struct Run<'a> {
    pub(crate) font: &'a FontData,
    pub(crate) size: f32,
    pub(crate) transform: Affine,
    pub(crate) glyph_transform: Option<Affine>,
    pub(crate) embolden: Vec2,
    pub(crate) glyphs: Vec<Glyph>,
}

/// Where the text of a run came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Origin {
    Layout,
    Cmap,
}

/// Draw the glyphs of `run` that `area` keeps, with their text, in `face`.
pub(crate) fn draw(
    surface: &mut Surface<'_>,
    face: &Face,
    sources: &Sources,
    area: GlyphArea,
    tally: &mut GlyphTally,
    run: Run<'_>,
    ink: Ink,
) {
    if run.glyphs.is_empty() || run.size <= 0.0 {
        return;
    }
    let (text, origin) = text_of(face, sources, &run);
    let advances = face.advances(&run.glyphs, run.size);
    let kept: Vec<usize> = (0..run.glyphs.len())
        .filter(|&index| {
            let centre = centre(face, run.glyphs[index], advances[index], run.size);
            area.holds(run.transform * centre) == Keep::Yes
        })
        .collect();
    tally.dropped += run.glyphs.len() - kept.len();
    let Some(&first) = kept.first() else {
        return;
    };
    tally.drawn += kept.len();
    match origin {
        Origin::Layout => tally.layout_text += kept.len(),
        Origin::Cmap => tally.cmap_text += kept.len(),
    }
    let text = text.select(&kept);
    let start = run.glyphs[first];
    let glyphs = krilla_glyphs(&run, &advances, &kept, &text);
    surface.push_transform(&geometry::transform(
        run.transform * slant(run.glyph_transform, start.y),
    ));
    surface.set_stroke(embolden_stroke(run.embolden, &ink));
    surface.set_fill(Some(Fill {
        paint: ink.paint,
        opacity: ink.opacity,
        rule: FillRule::NonZero,
    }));
    surface.draw_glyphs(
        KrillaPoint::from_xy(start.x, start.y),
        &glyphs,
        face.font.clone(),
        text.text(),
        run.size,
        false,
    );
    surface.set_stroke(None);
    surface.pop();
}

/// The run's text from the caller if it has it for exactly these glyphs, else from the cmap.
fn text_of(face: &Face, sources: &Sources, run: &Run<'_>) -> (RunText, Origin) {
    let key = RunKey::new(run.font, run.size, run.glyphs.iter().copied());
    match sources.texts.get(&key) {
        Some(text) if text.len() == run.glyphs.len() => (text.clone(), Origin::Layout),
        _ => {
            let chars: Vec<String> = run
                .glyphs
                .iter()
                .map(|glyph| {
                    face.chars
                        .get(&glyph.id)
                        .map(char::to_string)
                        .unwrap_or_default()
                })
                .collect();
            let text =
                RunText::from_glyphs(chars.iter().enumerate().map(|(index, text)| GlyphSource {
                    cluster: index,
                    text,
                }));
            (text, Origin::Cmap)
        }
    }
}

/// The centre of `glyph`'s box, in run space: half its advance across, midway between ascent
/// and descent down from the baseline.
fn centre(face: &Face, glyph: Glyph, advance: f32, size: f32) -> Point {
    Point::new(
        f64::from(glyph.x + advance / 2.0),
        f64::from(glyph.y - (face.ascent + face.descent) / 2.0 * size),
    )
}

/// The kept glyphs positioned as krilla wants: advances and offsets in ems, each glyph landing
/// exactly where the layout put it.
fn krilla_glyphs(
    run: &Run<'_>,
    advances: &[f32],
    kept: &[usize],
    text: &RunText,
) -> Vec<KrillaGlyph> {
    let start = run.glyphs[kept[0]];
    let mut pen = start.x;
    kept.iter()
        .enumerate()
        .map(|(position, &index)| {
            let glyph = run.glyphs[index];
            let next = kept
                .get(position + 1)
                .map(|&next| run.glyphs[next])
                .filter(|next| (next.y - glyph.y).abs() < 0.01 && next.x >= glyph.x);
            let next_x = next.map_or(glyph.x + advances[index], |next| next.x);
            let made = KrillaGlyph::new(
                GlyphId::new(glyph.id),
                (next_x - glyph.x) / run.size,
                (glyph.x - pen) / run.size,
                (glyph.y - start.y) / run.size,
                0.0,
                text.clusters()[position].clone(),
                None,
            );
            pen = next_x;
            made
        })
        .collect()
}

/// A synthetic oblique (the glyph transform a renderer applies to each outline) as one
/// transform about the run's baseline, which leaves every glyph origin in place.
fn slant(glyph_transform: Option<Affine>, baseline: f32) -> Affine {
    glyph_transform.map_or(Affine::IDENTITY, |slant| {
        let baseline = f64::from(baseline);
        Affine::translate((0.0, baseline)) * slant * Affine::translate((0.0, -baseline))
    })
}

/// A synthetic bold as a stroke of the fill's paint, so the text stays text (render mode
/// fill-then-stroke) rather than becoming outlines.
fn embolden_stroke(embolden: Vec2, ink: &Ink) -> Option<Stroke> {
    let width = (embolden.x + embolden.y) as f32;
    (width > 0.0).then(|| Stroke {
        paint: ink.paint.clone(),
        width,
        miter_limit: 4.0,
        line_cap: LineCap::Butt,
        line_join: LineJoin::Round,
        opacity: ink.opacity,
        dash: None,
    })
}

#[cfg(test)]
mod tests {
    use super::{GlyphArea, Keep, slant};
    use peniko::kurbo::{Affine, Point, Rect};

    #[test]
    fn the_area_keeps_its_top_edge_and_not_its_bottom() {
        let area = GlyphArea::Within(Rect::new(0.0, 0.0, 100.0, 50.0));
        const CASES: &[((f64, f64), Keep)] = &[
            ((10.0, 0.0), Keep::Yes),
            ((10.0, 49.9), Keep::Yes),
            ((10.0, 50.0), Keep::No),
            ((-0.1, 10.0), Keep::No),
            ((100.0, 10.0), Keep::No),
        ];
        for &((x, y), want) in CASES {
            assert_eq!(area.holds(Point::new(x, y)), want, "({x}, {y})");
        }
        assert_eq!(GlyphArea::Anywhere.holds(Point::new(1e9, -1e9)), Keep::Yes);
    }

    #[test]
    fn a_slant_leaves_the_baseline_in_place() {
        let skew = slant(Some(Affine::skew(0.25, 0.0)), 20.0);
        assert_eq!(skew * Point::new(7.0, 20.0), Point::new(7.0, 20.0));
        assert_ne!(skew * Point::new(7.0, 10.0), Point::new(7.0, 10.0));
    }
}
