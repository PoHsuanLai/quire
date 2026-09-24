//! Where a menu's element is drawn (mailo gaps 4): a placed popover surface in the overlay,
//! on the layer stack, or bare rows in its caller's flow. Split from `menu`.

use crate::components::flow::Flow;
use crate::components::menu_kind::MenuKind;
use crate::components::popover::{Dismiss, Float, Stacking, position_style};
use crate::geometry::{Anchor, Point};

/// A floating menu is a layer that Escape and an outside press close; an inline one is part of
/// its caller's card and stays off the stack.
pub(crate) fn stacking(flow: Flow) -> Stacking {
    match flow {
        Flow::Floating => Stacking::Layer(Dismiss::EscAndOutside),
        Flow::Inline => Stacking::Passive,
    }
}

/// How the menu's element is drawn: a placed popover surface, or bare rows in the flow.
pub(crate) struct Surface {
    pub class: &'static str,
    pub elevation: Option<&'static str>,
    pub layer: Option<&'static str>,
    pub style: Option<String>,
}

impl Surface {
    pub(crate) fn of(flow: Flow, float: Float, anchor: &Anchor, kind: MenuKind) -> Self {
        match flow {
            Flow::Floating => {
                let at = float
                    .anchor_rect(anchor)
                    .map(|rect| kind.placement(rect))
                    .map_or(Point::default(), |(rect, want, gap)| {
                        float.origin(Some(rect), want, gap)
                    });
                Surface {
                    class: "ds-popover ds-menu",
                    elevation: Some("pop"),
                    layer: Some("menu"),
                    style: Some(position_style(at)),
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
