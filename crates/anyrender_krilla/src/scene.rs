//! The painter: anyrender's drawing calls onto one krilla surface (one PDF page, or any
//! krilla surface).

use crate::geometry;
use crate::glyphs::{self, GlyphArea, GlyphTally, Run};
use crate::image;
use crate::layer::{self, Pushes};
use crate::paint::{self, Ink};
use crate::resources::Resources;
use crate::sources::Sources;
use anyrender::{Filter, Glyph, NormalizedCoord, Paint, PaintRef, PaintScene, RenderContext};
use krilla::geom::Size;
use krilla::paint::{
    Fill as KrillaFill, FillRule, LineCap, LineJoin, Stroke as KrillaStroke, StrokeDash,
};
use krilla::surface::Surface;
use peniko::kurbo::{Affine, Cap, Join, Rect, Shape, Stroke, Vec2};
use peniko::{BlendMode, Color, Fill, FontData, ImageBrushRef, StyleRef};
use std::fmt;
use std::sync::Arc;

/// An anyrender scene that draws onto `surface`. Coordinates are the surface's current user
/// space: push the page's own transform (points per pixel, margins) before painting.
pub struct KrillaScene<'s, 'p> {
    surface: &'s mut Surface<'p>,
    resources: &'s mut Resources,
    sources: &'s Sources,
    area: GlyphArea,
    layers: Vec<Pushes>,
    tally: GlyphTally,
}

impl<'s, 'p> KrillaScene<'s, 'p> {
    /// A scene over `surface`, keeping faces and images in `resources` (shared by every page of
    /// the document), looking up text and image sources in `sources`, and drawing only the
    /// glyphs `area` keeps.
    pub fn new(
        surface: &'s mut Surface<'p>,
        resources: &'s mut Resources,
        sources: &'s Sources,
        area: GlyphArea,
    ) -> Self {
        KrillaScene {
            surface,
            resources,
            sources,
            area,
            layers: Vec::new(),
            tally: GlyphTally::default(),
        }
    }

    /// What happened to the glyphs painted so far.
    pub fn tally(&self) -> GlyphTally {
        self.tally
    }

    fn draw_image(
        &mut self,
        brush: ImageBrushRef<'_>,
        transform: Affine,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        let decoded = brush.image;
        let Some(image) = image::prepare(self.resources, &self.sources.images, decoded) else {
            return;
        };
        let (Some(size), Some(clip)) = (
            Size::from_wh(decoded.width as f32, decoded.height as f32),
            geometry::path(shape, transform),
        ) else {
            return;
        };
        self.surface.push_clip_path(&clip, &FillRule::NonZero);
        let faded = brush.sampler.alpha < 1.0;
        if faded {
            self.surface
                .push_opacity(paint::opacity(brush.sampler.alpha));
        }
        self.surface.push_transform(&geometry::transform(
            transform * brush_transform.unwrap_or_default(),
        ));
        self.surface.draw_image(image, size);
        self.surface.pop();
        if faded {
            self.surface.pop();
        }
        self.surface.pop();
    }
}

impl fmt::Debug for KrillaScene<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KrillaScene")
            .field("area", &self.area)
            .field("layers", &self.layers.len())
            .field("tally", &self.tally)
            .finish_non_exhaustive()
    }
}

impl RenderContext for KrillaScene<'_, '_> {}

impl PaintScene for KrillaScene<'_, '_> {
    /// Closes every layer still open; what was drawn stays drawn (a PDF page is append-only).
    fn reset(&mut self) {
        while !self.layers.is_empty() {
            self.pop_layer();
        }
    }

    fn push_layer(
        &mut self,
        blend: impl Into<BlendMode>,
        alpha: f32,
        transform: Affine,
        clip: &impl Shape,
        _filter: Option<Arc<Filter>>,
        _backdrop_filter: Option<Arc<Filter>>,
    ) {
        let mut pushes = Pushes::default();
        if let Some(clip) = geometry::path(clip, transform) {
            self.surface.push_clip_path(&clip, &FillRule::NonZero);
            pushes = pushes.and_one();
        }
        if let Some(mode) = layer::blend(blend.into()) {
            self.surface.push_blend_mode(mode);
            pushes = pushes.and_one();
        }
        if alpha < 1.0 {
            self.surface.push_opacity(paint::opacity(alpha));
            pushes = pushes.and_one();
        }
        self.layers.push(pushes);
    }

    fn push_clip_layer(&mut self, transform: Affine, clip: &impl Shape) {
        self.push_layer(BlendMode::default(), 1.0, transform, clip, None, None);
    }

    fn pop_layer(&mut self) {
        let Pushes(count) = self.layers.pop().unwrap_or_default();
        (0..count).for_each(|_| self.surface.pop());
    }

    fn stroke<'a>(
        &mut self,
        style: &Stroke,
        transform: Affine,
        brush: impl Into<PaintRef<'a>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        let (Some(Ink { paint, opacity }), Some(path)) = (
            paint::ink(brush.into(), 1.0, brush_transform),
            geometry::path(shape, Affine::IDENTITY),
        ) else {
            return;
        };
        // Stroked in the shape's own space so the width scales with the transform.
        self.surface.push_transform(&geometry::transform(transform));
        self.surface.set_fill(None);
        self.surface.set_stroke(Some(KrillaStroke {
            paint,
            width: style.width as f32,
            miter_limit: style.miter_limit as f32,
            line_cap: cap(style.start_cap),
            line_join: join(style.join),
            opacity,
            dash: (!style.dash_pattern.is_empty()).then(|| StrokeDash {
                array: style.dash_pattern.iter().map(|&dash| dash as f32).collect(),
                offset: style.dash_offset as f32,
            }),
        }));
        self.surface.draw_path(&path);
        self.surface.set_stroke(None);
        self.surface.pop();
    }

    fn fill<'a>(
        &mut self,
        style: Fill,
        transform: Affine,
        brush: impl Into<PaintRef<'a>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        let brush = brush.into();
        if let Paint::Image(image) = brush {
            self.draw_image(image, transform, brush_transform, shape);
            return;
        }
        let (Some(Ink { paint, opacity }), Some(path)) = (
            paint::ink(brush, 1.0, brush_transform),
            geometry::path(shape, Affine::IDENTITY),
        ) else {
            return;
        };
        self.surface.push_transform(&geometry::transform(transform));
        self.surface.set_stroke(None);
        self.surface.set_fill(Some(KrillaFill {
            paint,
            opacity,
            rule: match style {
                Fill::NonZero => FillRule::NonZero,
                Fill::EvenOdd => FillRule::EvenOdd,
            },
        }));
        self.surface.draw_path(&path);
        self.surface.pop();
    }

    fn draw_glyphs<'a, 's: 'a>(
        &'s mut self,
        font: &'a FontData,
        font_size: f32,
        _hint: bool,
        normalized_coords: &'a [NormalizedCoord],
        embolden: Vec2,
        _style: impl Into<StyleRef<'a>>,
        brush: impl Into<PaintRef<'a>>,
        brush_alpha: f32,
        transform: Affine,
        glyph_transform: Option<Affine>,
        glyphs: impl Iterator<Item = Glyph> + Clone,
    ) {
        let Some(ink) = paint::ink(brush.into(), brush_alpha, None) else {
            return;
        };
        let Some(face) = self.resources.faces.get(font, normalized_coords) else {
            return;
        };
        let run = Run {
            font,
            size: font_size,
            transform,
            glyph_transform,
            embolden,
            glyphs: glyphs.collect(),
        };
        glyphs::draw(
            self.surface,
            face,
            self.sources,
            self.area,
            &mut self.tally,
            run,
            ink,
        );
    }

    /// Dropped: a printout has no shadows, and a PDF has no blur to draw one with.
    fn draw_box_shadow(&mut self, _: Affine, _: Rect, _: Color, _: f64, _: f64) {}
}

fn cap(cap: Cap) -> LineCap {
    match cap {
        Cap::Butt => LineCap::Butt,
        Cap::Round => LineCap::Round,
        Cap::Square => LineCap::Square,
    }
}

fn join(join: Join) -> LineJoin {
    match join {
        Join::Bevel => LineJoin::Bevel,
        Join::Miter => LineJoin::Miter,
        Join::Round => LineJoin::Round,
    }
}
