//! What a test reads off an element's computed style: its text colour and its fill, as
//! `getComputedStyle` would give them (animated values included, at the harness's animation
//! time), the laid-out box of its `::before` (a thumb drawn as a pseudo-element has no
//! selector of its own), and where that box is painted through its transforms.

use crate::driver::{DocQuery, first};
use crate::harness::Harness;
use blitz_dom::{BaseDocument, NodeId};
use blitz_kit::paint_rect::painted_rect as painted_bounds_of;
use ds::prelude::*;

/// A computed colour in sRGB, each channel and alpha in 0..=1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Srgba(pub [f32; 4]);

impl Srgba {
    /// This colour painted over `ground`, which is taken as opaque: what the eye sees.
    pub fn over(self, ground: Srgba) -> Srgba {
        let [r, g, b, a] = self.0;
        let [gr, gg, gb, _] = ground.0;
        Srgba([
            r * a + gr * (1.0 - a),
            g * a + gg * (1.0 - a),
            b * a + gb * (1.0 - a),
            1.0,
        ])
    }

    /// WCAG relative luminance (alpha ignored).
    pub fn luminance(self) -> f32 {
        let linear = |c: f32| {
            if c <= 0.039_28 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let [r, g, b, _] = self.0;
        0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
    }

    /// The WCAG contrast ratio between two opaque colours, 1 to 21.
    pub fn contrast(self, other: Srgba) -> f32 {
        let (a, b) = (self.luminance(), other.luminance());
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }
}

impl Harness {
    /// Where the part is painted: the border-box rect of the first element matching `selector`
    /// (or of its `::before`) carried through the part's own `transform` and every transformed
    /// ancestor's (`blitz_kit::paint_rect`): a box that slides by `translateX` is read where it
    /// appears. [`Query::rect`](crate::Query::rect) reads the layout box with transforms left out.
    pub fn painted_rect(&self, selector: &str, part: Part) -> Option<Rect> {
        self.with_doc(|doc| {
            let node = part.node(doc, first(doc, selector)?)?;
            let painted = painted_bounds_of(doc, node)?;
            Some(Rect {
                origin: Point {
                    x: Px(painted.x as f32),
                    y: Px(painted.y as f32),
                },
                size: Size {
                    width: Px(painted.width as f32),
                    height: Px(painted.height as f32),
                },
            })
        })
    }
}

/// Which box of an element a probe reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Part {
    /// The element itself.
    Element,
    /// Its `::before` pseudo-element.
    Before,
}

impl Part {
    pub(crate) fn node(self, doc: &BaseDocument, element: NodeId) -> Option<NodeId> {
        match self {
            Part::Element => Some(element),
            Part::Before => doc.get_node(element)?.before(),
        }
    }
}
