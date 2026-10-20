//! A glyph run in a bitmap-only colour face, drawn into the PDF as its pictures.
//!
//! Where the system has no vector colour emoji face, `quire_emoji` takes a bitmap one: Apple
//! Color Emoji (`sbix`) on a Mac, Noto Color Emoji (`CBDT`) on Ubuntu and Arch. Such a face has
//! no `glyf`, `CFF` or `CFF2` outlines, so the PDF writer cannot embed it as a font (the
//! subsetter has nothing to subset), and every printout with an emoji in it failed. Each run in a
//! face like that is replaced here, before the page is written, by one image fill per glyph: the
//! glyph's PNG from the strike nearest the run's size, placed as the window's renderer places it
//! (the strike's bearings, scaled from its ppem to the run's size). The pictures carry no text,
//! so an emoji is not in the printout's selectable text. A run in a face with neither outlines nor
//! pictures draws nothing rather than failing the printout.

use anyrender::recording::{FillCommand, GlyphRunCommand, RenderCommand};
use anyrender::{Paint, Scene};
use peniko::kurbo::{Affine, Rect, Shape, Vec2};
use peniko::{Blob, Fill, ImageAlphaType, ImageBrush, ImageData, ImageFormat};
use quire_emoji::table_listed;
use skrifa::bitmap::{BitmapData, BitmapFormat, BitmapStrikes, Origin};
use skrifa::instance::{LocationRef, Size};
use skrifa::{FontRef, GlyphId, MetadataProvider};
use std::sync::Arc;

/// The tables a PDF font can be embedded from.
const OUTLINES: [[u8; 4]; 3] = [*b"glyf", *b"CFF ", *b"CFF2"];

/// What CoreText adds above an Apple Color Emoji glyph that says nothing of its own, in font
/// units (the window's renderer, glifo, does the same).
const SBIX_LIFT: f32 = 100.0;

/// `scene` with each glyph run in a face without outlines drawn as its glyphs' pictures.
pub(crate) fn as_pictures(scene: &mut Scene) {
    let commands = std::mem::take(&mut scene.commands);
    scene.commands = commands
        .into_iter()
        .flat_map(|command| match command {
            RenderCommand::GlyphRun(run) if !has_outlines(&run) => pictures(&run),
            command => vec![command],
        })
        .collect();
}

fn has_outlines(run: &GlyphRunCommand) -> bool {
    let data = run.font_data.data.data();
    OUTLINES
        .iter()
        .any(|tag| table_listed(data, run.font_data.index, *tag))
}

/// One image fill per glyph of `run` that its face has a picture for.
fn pictures(run: &GlyphRunCommand) -> Vec<RenderCommand> {
    let Ok(font) = FontRef::from_index(run.font_data.data.data(), run.font_data.index) else {
        return Vec::new();
    };
    let strikes = BitmapStrikes::new(&font);
    let upem = font
        .metrics(Size::unscaled(), LocationRef::default())
        .units_per_em;
    let units = run.font_size / f32::from(upem.max(1));
    let slant = run.glyph_transform.unwrap_or(Affine::IDENTITY);
    run.glyphs
        .iter()
        .filter_map(|glyph| {
            let bitmap =
                strikes.glyph_for_size(Size::new(run.font_size), GlyphId::new(glyph.id))?;
            let BitmapData::Png(png) = bitmap.data else {
                return None;
            };
            let image = decode(png)?;
            let (width, height) = (f64::from(image.width), f64::from(image.height));
            // The picture's corner in the strike's pixels, y down from the baseline.
            let top = match bitmap.placement_origin {
                Origin::TopLeft => -f64::from(bitmap.inner_bearing_y),
                Origin::BottomLeft => -f64::from(bitmap.inner_bearing_y) - height,
            };
            let lift = if strikes.format() == Some(BitmapFormat::Sbix) && bitmap.bearing_y == 0.0 {
                SBIX_LIFT * units
            } else {
                0.0
            };
            let transform = run.transform
                * Affine::translate((f64::from(glyph.x), f64::from(glyph.y) - f64::from(lift)))
                * slant
                * Affine::scale_non_uniform(
                    f64::from(run.font_size / bitmap.ppem_x),
                    f64::from(run.font_size / bitmap.ppem_y),
                )
                * Affine::translate(Vec2::new(f64::from(bitmap.inner_bearing_x), top));
            Some(RenderCommand::Fill(FillCommand {
                fill: Fill::NonZero,
                transform,
                brush: Paint::Image(ImageBrush::new(image)),
                brush_transform: None,
                shape: Rect::new(0.0, 0.0, width, height).to_path(0.1),
            }))
        })
        .collect()
}

/// A PNG's pixels as straight-alpha RGBA.
fn decode(png: &[u8]) -> Option<ImageData> {
    let pixels = image::load_from_memory_with_format(png, image::ImageFormat::Png)
        .ok()?
        .to_rgba8();
    let (width, height) = pixels.dimensions();
    Some(ImageData {
        data: Blob::new(Arc::new(pixels.into_raw())),
        format: ImageFormat::Rgba8,
        alpha_type: ImageAlphaType::Alpha,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::as_pictures;
    use anyrender::recording::{GlyphRunCommand, RenderCommand};
    use anyrender::{Glyph, Paint, Scene};
    use pdfrum_anyrender::{GlyphArea, Page, Sources, write};
    use peniko::kurbo::{Affine, Point, Rect, Shape, Size};
    use peniko::{Blob, Color, FontData, Style};
    use std::sync::Arc;

    /// One glyph, U+1F600, as an 8 px red square at 16 ppem in `CBDT` and nothing else: the
    /// shape of Ubuntu's Noto Color Emoji (`tests/fixtures/bitmap-emoji.py` writes it).
    const BITMAP_EMOJI: &[u8] = include_bytes!("../../tests/fixtures/bitmap-emoji.ttf");

    /// A scene drawing glyph 1 of `face` at 32 px with its baseline at (10, 40).
    fn scene(face: &'static [u8]) -> Scene {
        let mut scene = Scene::new();
        scene
            .commands
            .push(RenderCommand::GlyphRun(GlyphRunCommand {
                font_data: FontData::new(Blob::new(Arc::new(face)), 0),
                font_size: 32.0,
                hint: false,
                normalized_coords: Vec::new(),
                embolden: Default::default(),
                style: Style::Fill(peniko::Fill::NonZero),
                brush: Paint::Solid(Color::BLACK),
                brush_alpha: 1.0,
                transform: Affine::translate((10.0, 40.0)),
                glyph_transform: None,
                glyphs: vec![Glyph {
                    id: 1,
                    x: 0.0,
                    y: 0.0,
                }],
            }));
        scene
    }

    fn written(scene: Scene) -> Result<Vec<u8>, String> {
        let page = Page {
            size: Size::new(200.0, 200.0),
            scene,
            placement: Affine::IDENTITY,
            clip: Rect::new(0.0, 0.0, 200.0, 200.0),
            area: GlyphArea::Within(Rect::new(0.0, 0.0, 200.0, 200.0)),
        };
        write(&[page], &Sources::default())
            .map(|written| written.bytes)
            .map_err(|error| error.to_string())
    }

    #[test]
    fn a_bitmap_only_face_cannot_be_written_as_a_font() {
        // What every printout with an emoji in it met where the emoji face is a bitmap one.
        let error = written(scene(BITMAP_EMOJI)).expect_err("a face with no outlines was embedded");
        assert!(error.contains("font"), "{error}");
    }

    #[test]
    fn a_bitmap_only_glyph_is_written_as_its_picture_on_the_strikes_bearings() {
        let mut scene = scene(BITMAP_EMOJI);
        as_pictures(&mut scene);
        let [RenderCommand::Fill(fill)] = scene.commands.as_slice() else {
            panic!("not one fill: {:?}", scene.commands);
        };
        assert!(matches!(&fill.brush, Paint::Image(image) if image.image.width == 8));
        // 8 px at 16 ppem drawn at 32 px is 16 px square, its top 16 px above the baseline.
        let bounds = fill
            .transform
            .transform_rect_bbox(fill.shape.bounding_box());
        let near = |a: Point, b: Point| (a - b).hypot() < 1e-6;
        assert!(near(bounds.origin(), Point::new(10.0, 24.0)), "{bounds:?}");
        assert!((bounds.width() - 16.0).abs() < 1e-6, "{bounds:?}");
        let bytes = written(scene).expect("the picture is written");
        assert!(
            bytes.windows(6).any(|w| w == b"/Image"),
            "no image in the PDF"
        );
    }

    #[test]
    fn runs_in_faces_with_outlines_are_left_to_the_font_writer() {
        let mut scene = scene(ds::style::fonts::FACES[0].bytes);
        let before = scene.commands.clone();
        as_pictures(&mut scene);
        assert_eq!(scene.commands, before);
    }
}
