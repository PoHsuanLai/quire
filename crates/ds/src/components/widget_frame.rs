//! WidgetFrame: the card a widget is drawn on (design/04-COMPONENTS.md "Widgets"; design/20
//! section 1.14, design/22 section 3.20; sill FINDINGS Q182). The frame owns the corner, the
//! padding, the footprint and the title row, so a widget draws only its content and writes no
//! CSS for any of them.
//!
//! On the desktop the card is the `Widget` material's plate inside a transparent `Widget` scope,
//! as a notification's plate is: the plate paints the tint, hairline and drop, and the scope
//! carries the material's tokens to the widget's content. In the notification center it is a
//! tile on the center's own Popover, as a `ModuleTile` is. Both take their footprint from
//! `--widget-cell` and `--widget-gap` (`WidgetMetrics`).

use crate::components::text_runs::text;
use crate::components::widget_kind::{WidgetHost, WidgetSize, WidgetTitle};
use crate::components::widget_scope::use_frame_provider;
use crate::icon::render::{Glyph, IconSize};
use crate::material::Material;
use crate::root::chrome::RootChrome;
use crate::root::surface::Surface;
use dioxus::prelude::*;

/// `children` on a widget's card, `size` on the grid unit, for `host`. `title` draws a glyph and
/// a name above the content. `id` names the card for the layer's input and blur regions.
///
/// The frame provides its `size` to its content (`widget_scope`), so content that must fit
/// the frame, a `MonthGrid` at `MonthDensity::Auto`, fits itself without being told.
#[component]
pub fn WidgetFrame(
    #[props(default)] size: WidgetSize,
    #[props(default)] host: WidgetHost,
    #[props(default)] title: Option<WidgetTitle>,
    #[props(default)] id: Option<String>,
    children: Element,
) -> Element {
    use_frame_provider(size);
    let card = rsx! {
        div {
            class: "ds-widget",
            id,
            "data-size": size.slug(),
            "data-host": host.slug(),
            if let Some(title) = title {
                div { class: "ds-widget-title",
                    Glyph { icon: title.glyph, size: IconSize::Tiny }
                    span { class: "ds-widget-title-text", {text(&title.text)} }
                }
            }
            div { class: "ds-widget-body", {children} }
        }
    };
    match host {
        WidgetHost::Desktop => rsx! {
            Surface { material: Material::Widget, chrome: RootChrome::Transparent, {card} }
        },
        WidgetHost::Tile => card,
    }
}
