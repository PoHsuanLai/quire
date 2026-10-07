//! Where a menu's element is drawn: a placed popover surface in the overlay,
//! on the layer stack, or bare rows in its caller's flow. Split from `menu`.

use crate::components::menus::menu::hung::Hung;
use crate::components::menus::menu::placement::MenuPlacement;
use crate::components::overlays::flow::Flow;
use crate::components::overlays::popover::{Float, Stacking, position_style};
use crate::host::measure::Anchor;
use ds_core::geometry::units::Point;
use ds_core::vocab::Dismiss;

/// A floating menu is a layer that Escape and an outside press close; an inline one is part of
/// its caller's card and stays off the stack.
pub(crate) fn stacking(flow: Flow) -> Stacking {
    match flow {
        Flow::Floating => Stacking::Layer(Dismiss::Transient),
        Flow::Inline => Stacking::Passive,
    }
}

/// The style of a menu whose anchor is not measured yet.
const UNPLACED: &str = ";visibility:hidden;pointer-events:none";

/// How the menu's element is drawn: a placed popover surface, or bare rows in the flow.
pub(crate) struct Surface {
    pub class: &'static str,
    pub elevation: Option<&'static str>,
    pub layer: Option<&'static str>,
    pub style: Option<String>,
}

impl Surface {
    pub(crate) fn of(
        flow: Flow,
        float: Float,
        anchor: &Anchor,
        placement: MenuPlacement,
        hung: Hung,
    ) -> Self {
        match flow {
            Flow::Floating => {
                let field = float.anchor_rect(anchor);
                let at = field
                    .map(|rect| placement.placement(rect))
                    .map_or(Point::default(), |(rect, want, gap)| {
                        float.origin(Some(rect), want, gap)
                    });
                let width = hung.width_style(field).unwrap_or_default();
                // Until the anchor is measured the menu has no place: it is laid out (so it can
                // be measured) but not drawn, and a press cannot land on it.
                let waiting = field.map_or(UNPLACED, |_| "");
                Surface {
                    class: "ds-popover ds-menu",
                    elevation: Some("pop"),
                    layer: Some("menu"),
                    style: Some(position_style(at) + &width + waiting),
                }
            }
            Flow::Inline => Surface {
                class: "ds-menu",
                elevation: None,
                layer: None,
                style: None,
            },
        }
    }
}
