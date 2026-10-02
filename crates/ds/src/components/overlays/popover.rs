//! Popover: the shared floating surface under menus, hover cards, tooltips and the
//! palette. Placement in Rust; Esc and outside click through the layer stack
//! (design/04-COMPONENTS.md section 21).
//!
//! Every floating component goes through [`use_float`]: it registers its surface with the
//! `OverlayHost` at the end of `.ds` (design/05-MOTION.md section 9 rule 10), places it with
//! [`place`] against its anchor and the measured overlay bounds, and joins the `LayerStack` so
//! one Escape or one outside click closes the topmost layer only (design/06-INTERACTIONS.md
//! sections 5 and 18).

use crate::host::measure::follow_rect;
use crate::host::measure::{Anchor, MountedRef, RectProbe};
use crate::root::common::Common;
use crate::stack::host::{OverlayId, Overlays, use_overlays};
use crate::stack::layer_stack::{Dismissal, LayerId, LayerStack};
use dioxus::core::{current_scope_id, queue_effect};
use dioxus::prelude::*;
use ds_core::geometry::{
    placement::{Placed, Placement, Side, place},
    units::{Point, Px, Rect, Size},
};
use ds_core::vocab::{Dismiss, Shown};
use ds_core::word::Word;
use ds_motion::anim::Anim;
use ds_motion::presence::{
    Exit,
    spec::PresenceSpec,
    use_presence::{Presented, use_presence},
};
use ds_style::tokens::layer::ZLayer;

/// Whether a floating surface joins the layer stack: hover cards and tooltips never take
/// Escape or a click, so they stay off it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stacking {
    /// A layer: Escape and outside clicks are routed through the stack.
    Layer(Dismiss),
    /// A passive surface: the pointer passes around it.
    Passive,
}

/// `data-layer` on the overlay wrapper and the popover: the `--z-*` layer it floats on.
pub(crate) fn layer_slug(layer: ZLayer) -> &'static str {
    match layer {
        ZLayer::Scene => "scene",
        ZLayer::Grain => "grain",
        ZLayer::Raise => "raise",
        ZLayer::LinkPill => "link-pill",
        ZLayer::Toast => "toast",
        ZLayer::SendPill => "send-pill",
        ZLayer::Scrim => "scrim",
        ZLayer::Peek => "peek",
        ZLayer::FocusPage => "focus-page",
        ZLayer::Edge => "edge",
        ZLayer::SidePeek => "side-peek",
        ZLayer::Palette => "palette",
        ZLayer::Card => "card",
        ZLayer::Menu => "menu",
        ZLayer::Drag => "drag",
    }
}

/// One floating surface's registration: its overlay slot, its layer, and the two rects its
/// placement needs (the overlay bounds and its own size).
#[derive(Clone, Copy)]
pub(crate) struct Float {
    overlays: Overlays,
    overlay: OverlayId,
    layer_id: LayerId,
    layer: ZLayer,
    stacking: Stacking,
    stack: Option<Signal<LayerStack>>,
    bounds: RectProbe,
    surface: RectProbe,
    anchor: Signal<Option<Rect>>,
    asked: CopyValue<Option<MountedRef>>,
}

/// Bounds used before the overlay wrapper has been measured: the window's top-left corner and
/// no far edge, so the first frame (and a server render) places against the anchor alone.
const UNMEASURED: Rect = Rect {
    origin: Point {
        x: Px(0.0),
        y: Px(0.0),
    },
    size: Size {
        width: Px(1.0e6),
        height: Px(1.0e6),
    },
};

/// Register a floating surface on `layer`. The calling scope owns it: it is shown from the
/// next render on and hidden, and removed from the stack, when the scope drops.
pub(crate) fn use_float(layer: ZLayer, stacking: Stacking) -> Float {
    let overlays = use_overlays();
    let key = use_hook(|| u32::try_from(current_scope_id().0).unwrap_or(u32::MAX));
    let stack = try_use_context::<Signal<LayerStack>>();
    let float = Float {
        overlays,
        overlay: OverlayId(key),
        layer_id: LayerId(key),
        layer,
        stacking,
        stack,
        bounds: crate::host::measure::use_rect(),
        surface: crate::host::measure::use_rect(),
        anchor: use_signal(|| None),
        asked: use_hook(|| CopyValue::new(None)),
    };
    use_hook(move || {
        if let (Stacking::Layer(dismiss), Some(stack)) = (stacking, stack) {
            queue_effect(move || {
                let mut stack = stack;
                let next = stack.peek().clone().push(LayerId(key), dismiss);
                stack.set(next);
            });
        }
    });
    use_drop(move || {
        overlays.hide(OverlayId(key));
        if let Some(mut stack) = stack {
            let next = stack.peek().clone().remove(LayerId(key));
            if *stack.peek() != next {
                stack.set(next);
            }
        }
    });
    float
}

impl Float {
    /// The surface's measured rect, for its `onmounted`.
    pub(crate) fn surface(&self) -> RectProbe {
        self.surface
    }

    /// The anchor as a rect: a point is a zero-size rect, a mounted element is read after
    /// layout (spike S9) and is `None` until then.
    pub(crate) fn anchor_rect(&self, anchor: &Anchor) -> Option<Rect> {
        match anchor {
            Anchor::Point(point) => Some(Rect {
                origin: *point,
                size: Size::default(),
            }),
            Anchor::Rect(rect) => Some(*rect),
            Anchor::Mounted(element) => {
                self.measure(element.clone());
                (self.anchor)()
            }
        }
    }

    /// Read a mounted anchor once layout has run.
    fn measure(&self, element: MountedRef) {
        let slot = self.anchor;
        let mut asked = self.asked;
        if asked.peek().as_ref() == Some(&element) {
            return;
        }
        asked.set(Some(element.clone()));
        let bounds = self.bounds;
        spawn(async move {
            let mut slot = slot;
            follow_rect(&element.0, |rect| slot.set(Some(rect))).await;
            // The bounds were read once, when the overlay mounted; the anchor has settled since,
            // so read them again to place the surface against the same layout.
            bounds.reread().await;
        });
    }

    /// `anchor`'s rect in the overlay's own coordinates (relative to the overlay bounds, which
    /// are the window's unless the host insets them), for a surface that fills or hangs from an
    /// element. `None` until the anchor has been laid out.
    pub(crate) fn within(&self, anchor: &Anchor) -> Option<Rect> {
        let rect = self.anchor_rect(anchor)?;
        let bounds = self
            .bounds
            .rect()
            .map_or(Point::default(), |bounds| bounds.origin);
        Some(Rect {
            origin: Point {
                x: rect.origin.x - bounds.x,
                y: rect.origin.y - bounds.y,
            },
            size: rect.size,
        })
    }

    /// Where the surface goes, relative to the overlay bounds: `place()` against the measured
    /// bounds and the surface's own size, or against the anchor alone before either is known.
    pub(crate) fn origin(&self, anchor: Option<Rect>, want: Placement, gap: Px) -> Point {
        self.placing(anchor, want, gap)
            .map_or(Point::default(), |placed| placed.origin)
    }

    /// [`Float::origin`] with the side the surface landed on, for a surface that draws an arrow
    /// toward its anchor. `None` before the anchor is known.
    pub(crate) fn placing(&self, anchor: Option<Rect>, want: Placement, gap: Px) -> Option<Placed> {
        let anchor = anchor?;
        let bounds = self.bounds.rect().unwrap_or(UNMEASURED);
        let content = self
            .surface
            .rect()
            .map(|rect| rect.size)
            .unwrap_or_default();
        Some(place(anchor, content, bounds, want, gap))
    }

    /// Where the surface sits in client coordinates, once the bounds and its own size are
    /// measured: `origin`'s placement offset by the bounds' corner. `None` before that.
    pub(crate) fn placed(&self, anchor: Option<Rect>, want: Placement, gap: Px) -> Option<Rect> {
        let bounds = self.bounds.rect()?;
        let size = self.surface.rect()?.size;
        let at = self.origin(anchor, want, gap);
        Some(Rect {
            origin: Point {
                x: bounds.origin.x + at.x,
                y: bounds.origin.y + at.y,
            },
            size,
        })
    }

    /// Whether Escape closes this surface now: it is the topmost layer and takes Escape.
    pub(crate) fn takes_escape(&self) -> bool {
        self.stack
            .is_some_and(|stack| stack.peek().escape() == Dismissal::Close(self.layer_id))
    }

    /// Show `surface` in the overlay host from the next render on, inside the full-bounds
    /// wrapper that measures the bounds and, for a layer closed by an outside click, catches
    /// that click.
    pub(crate) fn show(&self, surface: Element, onclose: EventHandler<()>) {
        let float = *self;
        let catches = matches!(self.stacking, Stacking::Layer(Dismiss::Transient));
        let bounds = self.bounds;
        let content = rsx! {
            div {
                class: "ds-overlay",
                "data-layer": layer_slug(self.layer),
                onmounted: move |event| bounds.on_mounted(event),
                if catches {
                    div {
                        class: "ds-overlay-catch",
                        onpointerdown: move |_| {
                            if float.outside_click_closes() {
                                onclose.call(());
                            }
                        },
                    }
                }
                {surface}
            }
        };
        let overlays = self.overlays;
        let (id, layer) = (self.overlay, self.layer);
        queue_effect(move || overlays.show(id, layer, content));
    }

    /// Leave the layer stack while the surface is kept mounted but hidden (a palette with
    /// `shown: Hidden`): it takes no Escape and no outside click until it [`Float::rejoin`]s.
    /// Call it from an effect or a handler.
    pub(crate) fn withdraw(&self) {
        if let Some(stack) = self.stack {
            let next = stack.peek().clone().remove(self.layer_id);
            if *stack.peek() != next {
                let _ = ds_style::task::try_set(stack, next);
            }
        }
    }

    /// Join the layer stack again, on top, as a hidden surface is shown. Call it from an effect
    /// or a handler.
    pub(crate) fn rejoin(&self) {
        if let (Stacking::Layer(dismiss), Some(stack)) = (self.stacking, self.stack) {
            let next = stack.peek().clone().push(self.layer_id, dismiss);
            let _ = ds_style::task::try_set(stack, next);
        }
    }

    /// Whether this surface is the topmost open layer: a scrim or backdrop it draws itself
    /// closes it only then.
    pub(crate) fn is_top(&self) -> bool {
        self.stack
            .is_none_or(|stack| stack.peek().top() == Some(self.layer_id))
    }

    /// Whether a click outside every surface closes this one: it is the topmost layer.
    fn outside_click_closes(&self) -> bool {
        self.stack
            .is_some_and(|stack| stack.peek().outside_click() == Dismissal::Close(self.layer_id))
    }
}

/// `left` and `top` for a placed surface.
pub(crate) fn position_style(at: Point) -> String {
    format!("left:{}px;top:{}px", px(at.x), px(at.y))
}

/// A length as CSS writes it, to a tenth of a pixel: `12`, `12.5`.
fn px(length: Px) -> String {
    let rounded = (length.0 * 10.0).round() / 10.0;
    format!("{rounded}")
}

/// Close on Escape when this surface is the topmost layer that takes it; stop the key there so
/// one Escape closes one layer (design/06-INTERACTIONS.md section 18).
pub(crate) fn escape_closes(float: Float, event: &KeyboardEvent, onclose: EventHandler<()>) {
    if event.key() == Key::Escape && float.takes_escape() {
        event.stop_propagation();
        event.prevent_default();
        onclose.call(());
    }
}

/// Whether a popover points at its anchor (`NSPopover`'s arrow, design/30 section 2.5): an app
/// popover anchored to a control does; a shell popover hung from the bar or Control Center does
/// not (design/27 5.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Arrow {
    /// The 34 x 8 arrow on the side facing the anchor.
    Arrow,
    /// No arrow: the popover hangs from its anchor.
    #[default]
    None,
}

/// How far the arrow stands out of the popover, and so how much further from its anchor the
/// popover sits.
const ARROW_HEIGHT: Px = Px(8.0);

/// How far the arrow's centre stays from a corner of the popover.
const ARROW_INSET: Px = Px(24.0);

/// The arrow's `data-side` and `left`/`top`: on the side of the popover that faces the anchor,
/// centred on the anchor's middle, kept `ARROW_INSET` from the corners.
fn arrow_at(side: Side, anchor: Rect, at: Point, size: Size) -> (&'static str, String) {
    let clamp = |along: f32, extent: f32| {
        along.clamp(ARROW_INSET.0, (extent - ARROW_INSET.0).max(ARROW_INSET.0))
    };
    match side {
        Side::Bottom | Side::Top => {
            let x = anchor.origin.x.0 + anchor.size.width.0 / 2.0 - at.x.0;
            let word = if side == Side::Bottom {
                "top"
            } else {
                "bottom"
            };
            (word, format!("left:{}px", clamp(x, size.width.0)))
        }
        Side::Left | Side::Right => {
            let y = anchor.origin.y.0 + anchor.size.height.0 / 2.0 - at.y.0;
            let word = if side == Side::Right { "left" } else { "right" };
            (word, format!("top:{}px", clamp(y, size.height.0)))
        }
    }
}

/// A floating surface anchored to an element, a rect or a point (`NSPopover`).
///
/// `dismiss` says what closes it: [`Dismiss::Transient`] (the default) Escape and a click
/// outside, [`Dismiss::Semitransient`] Escape only, [`Dismiss::Manual`] neither. It fades in over
/// `--t-quick` and, closed by either, fades out before `onclose` runs. `arrow` draws the 34 x 8
/// arrow toward the anchor. `common` puts the consumer's `id`, `data-*` and classes on the
/// surface, and its `aria_label` names it.
#[component]
pub fn Popover(
    anchor: Anchor,
    placement: Placement,
    gap: Px,
    #[props(default)] arrow: Arrow,
    #[props(default)] dismiss: Dismiss,
    onclose: EventHandler<()>,
    #[props(default)] common: Common,
    children: Element,
) -> Element {
    let layer = ZLayer::Menu;
    let float = use_float(layer, Stacking::Layer(dismiss));
    let gap = match arrow {
        Arrow::Arrow => gap + ARROW_HEIGHT,
        Arrow::None => gap,
    };
    let anchor_rect = float.anchor_rect(&anchor);
    let placed = float.placing(anchor_rect, placement, gap);
    let at = placed.map_or(Point::default(), |placed| placed.origin);
    let probe = float.surface();
    // Escape and an outside click fade the popover out, then close it (`Exit::Fade`,
    // `--t-quick`).
    let mut shown = use_signal(|| Shown::Visible);
    let Presented { presence, alias } = use_presence(
        shown(),
        PresenceSpec {
            enter: Anim::PaletteFade,
            exit: Exit::Fade,
        },
        Some(onclose),
    );
    let close = EventHandler::new(move |()| {
        if *shown.peek() == Shown::Visible {
            shown.set(Shown::Hidden);
        }
    });
    let pointer = match (arrow, placed, anchor_rect, probe.rect()) {
        (Arrow::Arrow, Some(placed), Some(anchor), Some(size)) => {
            Some(arrow_at(placed.side, anchor, placed.origin, size.size))
        }
        _ => None,
    };
    let class = common.class("ds-popover");
    let data = common.data_attributes();
    if presence.drawn_slug().is_none() {
        float.show(rsx! {}, close);
        return rsx! {};
    }
    float.show(
        rsx! {
            div {
                id: common.id.clone(),
                class,
                "aria-label": common.aria_label.clone(),
                "data-dismiss": dismiss.slug(),
                "data-layer": layer_slug(layer),
                "data-presence": presence.drawn_slug(),
                "data-pulse": alias.slug(),
                style: if placed.is_some() && probe.rect().is_some_and(|rect| rect.size.width.0 > 0.0) {
                    position_style(at)
                } else {
                    format!("{};visibility:hidden", position_style(at))
                },
                onmounted: move |event| {
                    common.mounted(event.clone());
                    probe.on_mounted(event);
                },
                onkeydown: move |event| escape_closes(float, &event, close),
                ..data,
                if let Some((side, along)) = pointer {
                    span { class: "ds-popover-arrow", "data-side": side, style: along }
                }
                div { class: "ds-popover-body", {children} }
            }
        },
        close,
    );
    rsx! {}
}
