//! Panel: a full-height panel at an edge of its root, for the notification center (sill Q123;
//! design/20 section 1.6: "center: `Overlay` panel anchored right", material `Popover`).
//!
//! Its own component rather than a `Sheet` placement: a sheet is a modal dialog in the Sheet
//! material that joins the overlay's layer stack over a scrim, while the center is a
//! non-modal edge panel in the Popover material on a surface of its own, with its showing and
//! its exit handed to the host exactly as the OSD's are (`shown`, `on_hidden`). Sharing Sheet
//! would have carried the layer stack, the centring stage and the modal scrim into a surface
//! that wants none of them by default.
//!
//! Shown, it slides in from past the right edge (`Anim::PanelIn`, `--t-move --e-out`); hidden,
//! it slides back out (`Anim::PanelOut`, `--t-move --e-exit`) and `on_hidden` runs at
//! `settle(PanelOut)`, when the host unmaps the surface; shown again while it leaves, it stays.
//! It paints its material from inside a transparent scope of that material, so the root it sits
//! in may be any; the scope fills the root, since Blitz places an absolutely positioned box
//! against its parent. It needs a root with a height (`Ds { extent: RootExtent::Viewport }`).

use crate::components::scrim::{ScrimLook, scrim_button_as};
use crate::components::scrim_strength::ScrimStrength;
use crate::components::shown_phase::use_shown_phase;
use crate::components::tooltip::Shown;
use crate::geometry::Px;
use crate::material::Material;
use crate::motion::anim::Anim;
use crate::root::surface::ClassedScope;
use dioxus::prelude::*;

/// Which edge a panel stands at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PanelEdge {
    /// The right edge, where macOS keeps Notification Center.
    #[default]
    Right,
}

impl PanelEdge {
    /// The `data-edge` word.
    fn slug(self) -> &'static str {
        match self {
            PanelEdge::Right => "right",
        }
    }
}

/// Whether a panel dims what is behind it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PanelScrim {
    /// Nothing behind it is dimmed (Notification Center's own way): a press outside is the
    /// host's to hear.
    #[default]
    None,
    /// A scrim of this strength covers the root behind the panel and closes it on a press.
    Dim(ScrimStrength),
}

/// A panel at `edge`, `width` wide (`notifications.center_width_px`, 384), in `material`
/// (Popover, design/20 section 1.6). `onclose` hears Escape inside the panel and a press on its
/// scrim.
#[component]
pub fn Panel(
    label: String,
    shown: Shown,
    #[props(default)] on_hidden: EventHandler<()>,
    #[props(default)] onclose: Option<EventHandler<()>>,
    #[props(default = Px(384.0))] width: Px,
    #[props(default)] edge: PanelEdge,
    #[props(default = Material::Popover)] material: Material,
    #[props(default)] scrim: PanelScrim,
    children: Element,
) -> Element {
    let (phase, alias) = use_shown_phase(shown, on_hidden, Anim::PanelIn, Anim::PanelOut);
    let close = onclose.unwrap_or_default();
    let dim = match scrim {
        PanelScrim::Dim(strength) => Some(ScrimLook {
            presence: (phase.presence() == Some("leaving")).then_some("leaving"),
            strength,
        }),
        PanelScrim::None => None,
    };
    rsx! {
        ClassedScope { material, class: "ds-panel-scope",
            div {
                class: "ds-panel-stage",
                "data-shown": phase.shown().slug(),
                "data-edge": edge.slug(),
                if let Some(look) = dim {
                    {scrim_button_as(&format!("Close {label}"), look, || true, close)}
                }
                div {
                    class: "ds-panel",
                    role: "complementary",
                    "aria-label": "{label}",
                    "data-presence": phase.presence(),
                    "data-pulse": alias.slug(),
                    style: "width:{width.0}px",
                    onkeydown: move |event| {
                        if event.key() == Key::Escape {
                            event.stop_propagation();
                            close.call(());
                        }
                    },
                    {children}
                }
            }
        }
    }
}
