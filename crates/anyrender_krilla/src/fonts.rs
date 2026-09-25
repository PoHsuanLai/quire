//! Faces as krilla holds them, parsed once per document: the face at the instance the layout
//! used, its reverse cmap for runs without text, and its vertical metrics for placing a glyph's
//! box.

use crate::cmap;
use crate::instance::{self, AxisValue};
use anyrender::{Glyph, NormalizedCoord};
use krilla::text::{Font, Tag};
use peniko::FontData;
use skrifa::instance::{LocationRef, Size};
use skrifa::{FontRef, GlyphId, MetadataProvider as _};
use std::collections::HashMap;
use std::sync::Arc;

/// One face at one instance, ready to draw with.
pub(crate) struct Face {
    /// What krilla embeds and subsets.
    pub(crate) font: Font,
    /// Each glyph's preferred character, for runs whose text is unknown.
    pub(crate) chars: HashMap<u32, char>,
    /// The face's bytes and index, for advances.
    data: FontData,
    /// The normalized coordinates the layout used, for advances at that instance.
    coords: Vec<NormalizedCoord>,
    /// Ascent and descent (negative below the baseline) per unit of font size.
    pub(crate) ascent: f32,
    pub(crate) descent: f32,
}

impl Face {
    /// Each of `glyphs`' advance at `size`, at this instance; half an em for a glyph the face
    /// does not know.
    pub(crate) fn advances(&self, glyphs: &[Glyph], size: f32) -> Vec<f32> {
        let fallback = size / 2.0;
        let Ok(font) = FontRef::from_index(self.data.data.data(), self.data.index) else {
            return vec![fallback; glyphs.len()];
        };
        let location: Vec<_> = self
            .coords
            .iter()
            .map(|&coord| skrifa::instance::NormalizedCoord::from_bits(coord))
            .collect();
        let metrics = font.glyph_metrics(Size::new(size), LocationRef::new(&location));
        glyphs
            .iter()
            .map(|glyph| {
                metrics
                    .advance_width(GlyphId::new(glyph.id))
                    .unwrap_or(fallback)
            })
            .collect()
    }
}

/// Faces by blob, index and instance.
#[derive(Default)]
pub(crate) struct Faces {
    faces: HashMap<(u64, u32, Vec<NormalizedCoord>), Option<Face>>,
}

impl Faces {
    /// The face `font` at `coords`, parsed on first use; `None` for bytes neither skrifa nor
    /// krilla can read (the run is then not drawn).
    pub(crate) fn get(&mut self, font: &FontData, coords: &[NormalizedCoord]) -> Option<&Face> {
        self.faces
            .entry((font.data.id(), font.index, coords.to_vec()))
            .or_insert_with(|| load(font, coords))
            .as_ref()
    }

    /// How many faces (at distinct instances) were loaded.
    pub(crate) fn len(&self) -> usize {
        self.faces.len()
    }
}

fn load(font: &FontData, coords: &[NormalizedCoord]) -> Option<Face> {
    let parsed = FontRef::from_index(font.data.data(), font.index).ok()?;
    let values: Vec<(Tag, f32)> = instance::user_values(&parsed, coords)
        .into_iter()
        .map(|AxisValue { tag, value }| (Tag::new(&tag), value))
        .collect();
    let bytes: Arc<dyn AsRef<[u8]> + Send + Sync> = Arc::new(font.data.clone());
    let krilla_font = Font::new_variable(bytes.into(), font.index, &values)?;
    let metrics = parsed.metrics(Size::unscaled(), LocationRef::default());
    let per_em = f32::from(metrics.units_per_em.max(1));
    Some(Face {
        font: krilla_font,
        chars: cmap::reverse(&parsed),
        data: font.clone(),
        coords: coords.to_vec(),
        ascent: metrics.ascent / per_em,
        descent: metrics.descent / per_em,
    })
}
