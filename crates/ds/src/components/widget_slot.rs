//! WidgetSlotGuide: where a dragged widget will land (sill Q431; design/23-WIDGETS.md section
//! 9.8). While a host drags a desktop widget, it draws this at the snap cell under the pointer:
//! the card's footprint for `size`, a quiet translucent plate in the card's own corner, faded in
//! as the drag reaches the cell. The host places it; quire draws it, so a shell draws nothing of
//! its own for the guide.

use crate::components::widget_kind::{WidgetHost, WidgetSize};
use crate::motion::Anim;
use dioxus::prelude::*;

/// `div.ds-widget-slot[data-size][data-host]`: the footprint of a `size` widget in `host`.
/// Decorative (`aria-hidden`): the drag's own announcement says where it goes.
#[component]
pub fn WidgetSlotGuide(
    #[props(default)] size: WidgetSize,
    #[props(default)] host: WidgetHost,
) -> Element {
    rsx! {
        div {
            class: "ds-widget-slot {Anim::Fade.class()}",
            "data-size": size.slug(),
            "data-host": host.slug(),
            "aria-hidden": "true",
        }
    }
}
