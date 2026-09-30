//! The page a PDF is printed on: its size, its margins, and the content box in between that
//! the document is laid out into.

use crate::pdf::error::PdfError;
use blitz_traits::shell::{ColorScheme, Viewport as BlitzViewport};

/// PDF points per CSS pixel: a CSS pixel is 1/96 inch, a point 1/72.
pub(crate) const PT_PER_PX: f32 = 0.75;

/// A length in PDF points (1/72 inch), the unit a PDF page is measured in.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Pt(pub f32);

impl Pt {
    /// `mm` millimetres.
    pub fn from_mm(mm: f32) -> Self {
        Pt(mm * 72.0 / 25.4)
    }

    /// `inches` inches.
    pub fn from_inches(inches: f32) -> Self {
        Pt(inches * 72.0)
    }

    /// The same length in CSS pixels.
    pub(crate) fn px(self) -> f32 {
        self.0 / PT_PER_PX
    }
}

/// The sheet a document is printed on.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PageSize {
    /// ISO A4, 210 x 297 mm.
    #[default]
    A4,
    /// US Letter, 8.5 x 11 in.
    Letter,
    /// Any other sheet, portrait or landscape as given.
    Custom {
        /// Across.
        width: Pt,
        /// Down.
        height: Pt,
    },
}

impl PageSize {
    /// Across and down, in points.
    pub fn dimensions(self) -> (Pt, Pt) {
        match self {
            PageSize::A4 => (Pt::from_mm(210.0), Pt::from_mm(297.0)),
            PageSize::Letter => (Pt::from_inches(8.5), Pt::from_inches(11.0)),
            PageSize::Custom { width, height } => (width, height),
        }
    }
}

/// The blank border of every page. The spec's margins are the only ones: `@page` is not read
/// (Blitz's style engine ignores it).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Margins {
    /// Above the content.
    pub top: Pt,
    /// Right of it.
    pub right: Pt,
    /// Below it.
    pub bottom: Pt,
    /// Left of it.
    pub left: Pt,
}

impl Margins {
    /// The same margin on all four sides.
    pub fn uniform(margin: Pt) -> Self {
        Margins {
            top: margin,
            right: margin,
            bottom: margin,
            left: margin,
        }
    }

    /// `vertical` above and below, `horizontal` left and right.
    pub fn symmetric(vertical: Pt, horizontal: Pt) -> Self {
        Margins {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

impl Default for Margins {
    /// 18 mm above and below, 16 mm at the sides: what mailo's printout has always asked
    /// `@page` for.
    fn default() -> Self {
        Margins::symmetric(Pt::from_mm(18.0), Pt::from_mm(16.0))
    }
}

/// How a document is put on paper.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PageSpec {
    /// The sheet.
    pub size: PageSize,
    /// Its blank border.
    pub margins: Margins,
}

/// The part of a page the document is laid out into, in CSS pixels, and where it sits on the
/// sheet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ContentBox {
    /// Across, whole pixels (the layout viewport's width).
    pub(crate) width: u32,
    /// Down: how much of the document one page holds.
    pub(crate) height: f32,
    /// The sheet's size, in points.
    pub(crate) sheet: (Pt, Pt),
    /// The content's top-left corner on the sheet, in points.
    pub(crate) origin: (Pt, Pt),
}

impl ContentBox {
    /// The content box of `spec`, or why it has none.
    pub(crate) fn of(spec: PageSpec) -> Result<Self, PdfError> {
        let (width, height) = spec.size.dimensions();
        let Margins {
            top,
            right,
            bottom,
            left,
        } = spec.margins;
        let across = Pt(width.0 - left.0 - right.0).px().floor();
        let down = Pt(height.0 - top.0 - bottom.0).px();
        if !(across >= 1.0 && down >= 1.0 && across.is_finite() && down.is_finite()) {
            return Err(PdfError::NoContentArea);
        }
        Ok(ContentBox {
            width: across as u32,
            height: down,
            sheet: (width, height),
            origin: (left, top),
        })
    }

    /// Blitz's viewport for laying the document out: one page wide and one page tall (so `vh`
    /// means a page), at scale 1 (a CSS pixel is a layout pixel), light.
    pub(crate) fn viewport(self) -> BlitzViewport {
        BlitzViewport::new(
            self.width,
            self.height.floor() as u32,
            1.0,
            ColorScheme::Light,
        )
    }

    /// The same box as a quire `Viewport`, for building a harness at page width.
    pub(crate) fn logical(self) -> crate::Viewport {
        crate::Viewport {
            width: self.width,
            height: self.height.floor() as u32,
            scale_percent: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ContentBox, Margins, PageSize, PageSpec, Pt};
    use crate::pdf::error::PdfError;

    #[test]
    fn a4_with_mailos_margins_is_672_px_across() {
        let content = ContentBox::of(PageSpec::default()).expect("A4 has room");
        assert_eq!(content.width, 672);
        assert!((content.height - 986.5).abs() < 0.1, "{}", content.height);
    }

    #[test]
    fn sizes_are_in_points() {
        const CASES: &[(PageSize, (f32, f32))] = &[
            (PageSize::A4, (595.28, 841.89)),
            (PageSize::Letter, (612.0, 792.0)),
            (
                PageSize::Custom {
                    width: Pt(100.0),
                    height: Pt(50.0),
                },
                (100.0, 50.0),
            ),
        ];
        for &(size, (width, height)) in CASES {
            let (w, h) = size.dimensions();
            assert!(
                (w.0 - width).abs() < 0.01 && (h.0 - height).abs() < 0.01,
                "{size:?}"
            );
        }
    }

    #[test]
    fn margins_that_meet_leave_no_content() {
        let spec = PageSpec {
            size: PageSize::Custom {
                width: Pt(100.0),
                height: Pt(100.0),
            },
            margins: Margins::uniform(Pt(50.0)),
        };
        assert!(matches!(ContentBox::of(spec), Err(PdfError::NoContentArea)));
    }
}
