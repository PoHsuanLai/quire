//! Where text is inked: the glyph runs of a parley layout as rectangles in the document, each
//! with the element whose style it carries (Blitz tags every run with its span's node id).
//!
//! Blitz keeps no rectangle for a text node, only the inline root's layout. A run's left edge
//! is its offset and its right edge adds its advance, so the rectangle is the advance box of
//! the run, a hair wider than the ink (a glyph's side bearing). A run of spaces is not ink.

use crate::inset::bounds::Bounds;
use blitz_dom::NodeId;
use blitz_dom::node::TextBrush;
use parley::{Layout, PositionedLayoutItem};

/// One run of text.
pub(crate) struct Run {
    /// The element whose style the run carries.
    pub element: NodeId,
    /// Where it is inked.
    pub bounds: Bounds,
    /// Its characters.
    pub text: String,
}

/// The runs of `layout`, whose clusters index into `text`, with the layout's content box at
/// (`x`, `y`) in the document.
pub(crate) fn runs(layout: &Layout<TextBrush>, text: &str, x: f64, y: f64) -> Vec<Run> {
    let scale = f64::from(layout.scale());
    let mut found = Vec::new();
    for line in layout.lines() {
        let metrics = line.metrics();
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };
            let words = text.get(glyph_run.run().text_range()).unwrap_or("");
            if words.trim().is_empty() {
                continue;
            }
            let left = f64::from(glyph_run.offset());
            let right = left + f64::from(glyph_run.advance());
            let (top, bottom) = (
                f64::from(metrics.block_min_coord),
                f64::from(metrics.block_max_coord),
            );
            found.push(Run {
                element: glyph_run.style().brush.id,
                bounds: Bounds {
                    left: (x + left / scale) as f32,
                    top: (y + top / scale) as f32,
                    right: (x + right / scale) as f32,
                    bottom: (y + bottom / scale) as f32,
                },
                text: words.trim().to_owned(),
            });
        }
    }
    found
}
