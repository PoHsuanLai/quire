//! Panel: a full-height panel at an edge of its root, for the notification center (design/20
//! section 1.6: "center: `Overlay` panel anchored right", material `Popover`).
//!
//!
//! Its own component rather than a `Sheet` placement: a sheet is a modal dialog in the Sheet
//! material that joins the overlay's layer stack over a scrim, while the center is a
//! non-modal edge panel in the Popover material on a surface of its own, with its showing and
//! its exit handed to the host exactly as the OSD's are (`shown`, `on_hidden`). Sharing Sheet
//! would have carried the layer stack, the centring stage and the modal scrim into a surface
//! that wants none of them by default.
//! Mounted shown, it slides in from past the right edge (`Anim::PanelIn`, `--t-move --e-out`);
//! at the bottom edge (Edit Widgets) it arrives as a sheet does (`peek-in`).
//! Every change after that is driven motion (design/05 section 14): hidden, a spring in
//! Rust slides it back out past the edge and `on_hidden` runs once the spring rests, when the
//! host unmaps the surface; shown again while it leaves, it turns back from where it is.
//! It paints its material from inside a transparent scope of that material, so the root it sits
//! in may be any; the scope fills the root, since Blitz places an absolutely positioned box
//! against its parent. It needs a root with a height (`Ds { extent: RootExtent::Viewport }`).

use crate::components::overlays::scrim::{ScrimLook, scrim_button_as};
use crate::components::overlays::scrim_strength::ScrimStrength;
use ds_motion::anim::Anim;
use ds_motion::presence::spring::use_spring_presence;
use crate::root::surface::ClassedScope;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::vocab::Shown;
use ds_core::word::Word;
use ds_style::appearance::material::Material;

/// Which edge a panel stands at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum PanelEdge {
    /// The right edge, where macOS keeps Notification Center: full height, `width` wide.
    #[default]
    Right,
    /// The bottom edge, as the reference's Edit Widgets sheet: centred, `width` wide
    /// at most (less `--s-8` at each side), `height` tall but never more than half the root, so
    /// the top of the desktop, where widgets land, stays in view. It arrives and leaves as a
    /// sheet does (`peek-in`, then the sheet's spring), not by sliding off an edge.
    Bottom,
}

impl PanelEdge {
    /// The keyframe its first showing plays: the edge panel's slide, or the sheet's `peek-in`
    /// at `--t-move --e-out` (no spring: opening is not contact, design/05 principle 2).
    fn entrance(self) -> Anim {
        match self {
            PanelEdge::Right => Anim::PanelIn,
            PanelEdge::Bottom => Anim::PeekFullIn,
        }
    }

    /// Its extent as an inline style: the width at the right; the width and the height at the
    /// bottom.
    fn extent(self, width: Px, height: Px) -> String {
        match self {
            PanelEdge::Right => format!("width:{}px;", width.0),
            PanelEdge::Bottom => format!("width:{}px;height:{}px;", width.0, height.0),
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
/// (Popover, design/20 section 1.6). At `PanelEdge::Bottom` it is `height` tall (440 by
/// default), capped at half the root. `onclose` hears Escape inside the panel and a press on its
/// scrim.
#[component]
pub fn Panel(
    label: String,
    shown: Shown,
    #[props(default)] on_hidden: EventHandler<()>,
    #[props(default)] onclose: Option<EventHandler<()>>,
    #[props(default = Px(384.0))] width: Px,
    #[props(default)] edge: PanelEdge,
    #[props(default = Px(440.0))] height: Px,
    #[props(default = Material::Popover)] material: Material,
    #[props(default)] scrim: PanelScrim,
    children: Element,
) -> Element {
    let showing = use_spring_presence(Some(shown), Some(on_hidden), edge.entrance());
    let close = onclose.unwrap_or_default();
    let dim = match scrim {
        PanelScrim::Dim(strength) => Some(ScrimLook {
            presence: showing.leaving().then_some("leaving"),
            strength,
        }),
        PanelScrim::None => None,
    };
    let drawn = if showing.drawn() {
        Shown::Visible
    } else {
        Shown::Hidden
    };
    rsx! {
        ClassedScope { material, class: "ds-panel-scope",
            div {
                class: "ds-panel-stage",
                "data-shown": drawn.slug(),
                "data-edge": edge.slug(),
                if let Some(look) = dim {
                    {scrim_button_as(&format!("Close {label}"), look, || true, close)}
                }
                div {
                    class: "ds-panel",
                    role: "complementary",
                    "aria-label": "{label}",
                    "data-presence": showing.drawn().then(|| showing.slug()),
                    "data-drive": showing.drive(),
                    "data-overscroll": "band",
                    style: "{edge.extent(width, height)}{showing.style()}",
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
