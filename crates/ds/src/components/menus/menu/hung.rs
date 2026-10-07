//! A menu that hangs from a field: the suggestions under a search field. The field's presses
//! count as inside the menu, so a press on it neither closes the menu nor moves the keyboard, and
//! the menu is as wide as the field or wider.

use crate::components::overlays::catcher::Catcher;
use crate::components::overlays::popover::Float;
use crate::host::measure::Anchor;
use ds_core::geometry::units::{Px, Rect};

/// How wide a menu hung from a field is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelWidth {
    /// Exactly the field's width.
    SameAsField,
    /// As wide as its rows need, but at least this wide and at least the field's width.
    AtLeast(Px),
}

/// Whether a menu hangs from its anchor, a field.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Hung {
    /// A menu of its own: an outside press anywhere closes it, the anchor's included.
    #[default]
    Free,
    /// Hung from the anchor, a field: presses on it are inside, and the width follows it.
    FromField(PanelWidth),
}

impl Hung {
    /// What the outside-press catcher leaves alone: the field's rect, and until it is laid out the
    /// catcher waits rather than cover the field.
    pub(crate) fn catcher(self, float: Float, anchor: &Anchor) -> Catcher {
        match self {
            Hung::Free => Catcher::Whole,
            Hung::FromField(_) => float
                .within(anchor)
                .map_or(Catcher::Pending, Catcher::Around),
        }
    }

    /// The `width` and `min-width` the surface is given against `field`: none for a free menu.
    pub(crate) fn width_style(self, field: Option<Rect>) -> Option<String> {
        let (Hung::FromField(width), Some(field)) = (self, field) else {
            return None;
        };
        let wide = field.size.width.0;
        Some(match width {
            PanelWidth::SameAsField => format!(";width:{wide}px;min-width:{wide}px"),
            PanelWidth::AtLeast(least) => format!(";min-width:{}px", wide.max(least.0)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Hung, PanelWidth};
    use ds_core::geometry::units::{Point, Px, Rect, Size};

    fn field(width: f32) -> Option<Rect> {
        Some(Rect {
            origin: Point::default(),
            size: Size {
                width: Px(width),
                height: Px(28.0),
            },
        })
    }

    #[test]
    fn a_hung_menu_follows_the_fields_width() {
        // (hung, field, want)
        #[rustfmt::skip]
        let cases: &[(Hung, Option<f32>, Option<&str>)] = &[
            (Hung::Free, Some(240.0), None),
            (Hung::FromField(PanelWidth::SameAsField), Some(240.0), Some(";width:240px;min-width:240px")),
            (Hung::FromField(PanelWidth::AtLeast(Px(320.0))), Some(240.0), Some(";min-width:320px")),
            (Hung::FromField(PanelWidth::AtLeast(Px(200.0))), Some(240.0), Some(";min-width:240px")),
            (Hung::FromField(PanelWidth::SameAsField), None, None),
        ];
        for &(hung, width, want) in cases {
            assert_eq!(
                hung.width_style(width.and_then(field)).as_deref(),
                want,
                "{hung:?} {width:?}"
            );
        }
    }
}
