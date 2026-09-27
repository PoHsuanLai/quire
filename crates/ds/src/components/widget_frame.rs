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
use crate::components::widget_exit::{CardPresence, use_card_exit};
use crate::components::widget_kind::{CardTint, Lift, WidgetHost, WidgetSize, WidgetTitle};
use crate::components::widget_scope::use_frame_provider;
use crate::icon::render::{Glyph, IconSize};
use crate::material::Material;
use crate::root::chrome::RootChrome;
use crate::root::surface::Surface;
use crate::widget::WidgetKind;
use dioxus::prelude::*;

/// `children` on a widget's card, `size` on the grid unit, for `host`, tinted with `tint` (the
/// Space's by default; a tile never lays a second gradient, `CardTint::on`). `kind` writes
/// `data-widget` (a `WidgetCard` passes its widget's kind). `lift` picks the card up while a
/// host moves it (sill Q430).
/// `title` draws a glyph and a name above the content (the Batteries and Clock widgets take
/// none: their content fills the card). `id` names the card for the layer's input and blur
/// regions.
///
/// The frame provides its `size` to its content (`widget_scope`), so content that must fit
/// the frame, a `MonthGrid` at `MonthDensity::Auto`, fits itself without being told.
///
/// `presence: CardPresence::Leaving` plays the card's exit (sill G423): it shrinks and fades
/// (`widget-out`, a fade alone under Reduced) and `on_gone` runs once at `settle(WidgetOut)`,
/// when the host stops drawing it. The host keeps the card until then.
#[component]
pub fn WidgetFrame(
    #[props(default)] size: WidgetSize,
    #[props(default)] host: WidgetHost,
    #[props(default)] tint: CardTint,
    #[props(default)] title: Option<WidgetTitle>,
    #[props(default)] id: Option<String>,
    #[props(default)] kind: Option<WidgetKind>,
    #[props(default)] lift: Lift,
    #[props(default)] presence: CardPresence,
    #[props(default)] on_gone: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    use_frame_provider(size);
    let motion = use_card_exit(presence, on_gone);
    let class = match motion.pulse() {
        Some(pulse) => format!("ds-widget {pulse}"),
        None => "ds-widget".to_owned(),
    };
    let tint = tint.on(host);
    let kind = kind.map(|kind| kind.as_str().to_owned());
    let card = rsx! {
        div {
            class,
            id,
            "data-size": size.slug(),
            "data-host": host.slug(),
            "data-tint": tint.slug(),
            "data-widget": kind,
            "data-lift": lift.slug(),
            "data-presence": motion.presence(),
            "data-pulse": motion.pulse().map(|_| "a"),
            if tint == CardTint::Space {
                div { class: "ds-frame" }
            }
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
