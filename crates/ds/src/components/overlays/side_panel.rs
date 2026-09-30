//! SidePanel: a full-height panel at the right edge of its root, for the notification center
//! (design/30 section 2.5, design/20 section 1.6: "center: `Overlay` panel anchored right",
//! material `Popover`). Shell only; macOS has no counterpart but Notification Center's own side
//! sheet.
//!
//! Its own component rather than a `Sheet` attachment: a sheet slides from the top and joins the
//! overlay's layer stack, while the center is a non-modal edge panel in the Popover material on a
//! surface of its own, with its showing and its exit handed to the host exactly as the OSD's are
//! (`shown`, `on_hidden`).
//! Shown, it slides in from past the right edge over `--t-move --e-out`; hidden, it slides back
//! out over `--t-quick --e-exit` and `on_hidden` runs once the exit has settled, when the host
//! unmaps the surface; shown again while it leaves, it is present at once.
//! It paints its material from inside a transparent scope of that material, so the root it sits
//! in may be any; the scope fills the root, since Blitz places an absolutely positioned box
//! against its parent. It needs a root with a height (`Ds { extent: RootExtent::Viewport }`).

use crate::root::common::Common;
use crate::root::surface::ClassedScope;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::vocab::Shown;
use ds_core::word::Word;
use ds_motion::anim::Anim;
use ds_motion::presence::{
    Exit,
    spec::PresenceSpec,
    use_presence::{Presented, use_presence},
};
use ds_style::appearance::material::Material;

/// A panel at the right edge, `width` wide (`notifications.center_width_px`, 384), in `material`
/// (Popover, design/20 section 1.6). `header` is an optional row above the body. `onclose` hears
/// Escape inside the panel. `common` goes on the panel; its `aria_label` names it in place of
/// `label`.
#[component]
pub fn SidePanel(
    label: String,
    shown: Shown,
    #[props(default)] on_hidden: EventHandler<()>,
    #[props(default)] onclose: Option<EventHandler<()>>,
    #[props(default = Px(384.0))] width: Px,
    #[props(default = Material::Popover)] material: Material,
    #[props(default)] header: Option<Element>,
    #[props(default)] common: Common,
    children: Element,
) -> Element {
    let Presented { presence, alias } = use_presence(
        shown,
        PresenceSpec {
            enter: Anim::PanelIn,
            exit: Exit::PanelOut,
        },
        Some(on_hidden),
    );
    let close = onclose.unwrap_or_default();
    let class = common.class("ds-side-panel");
    let data = common.data_attributes();
    let name = common.aria_label.clone().unwrap_or(label);
    rsx! {
        ClassedScope { material, class: "ds-side-panel-scope",
            div {
                class: "ds-side-panel-stage",
                "data-shown": presence.shown().slug(),
                div {
                    id: common.id.clone(),
                    class,
                    role: "complementary",
                    "aria-label": "{name}",
                    "data-presence": presence.drawn_slug(),
                    "data-pulse": alias.slug(),
                    "data-overscroll": "band",
                    style: "width:{width.0}px",
                    onmounted: move |event| common.mounted(event),
                    onkeydown: move |event| {
                        if event.key() == Key::Escape {
                            event.stop_propagation();
                            close.call(());
                        }
                    },
                    ..data,
                    if let Some(header) = header {
                        div { class: "ds-side-panel-header", {header} }
                    }
                    div { class: "ds-side-panel-body", {children} }
                }
            }
        }
    }
}
