//! Popover: the shared floating surface under menus, hover cards, tooltips, the bubble and the
//! palette. Placement in Rust; Esc and outside click through the layer stack
//! (design/04-COMPONENTS.md section 21).
//!
//! Every floating component goes through [`use_float`]: it registers its surface with the
//! `OverlayHost` at the end of `.ds` (design/05-MOTION.md section 9 rule 10), places it with
//! [`place`] against its anchor and the measured overlay bounds, and joins the `LayerStack` so
//! one Escape or one outside click closes the topmost layer only (design/06-INTERACTIONS.md
//! sections 5 and 18).

use crate::geometry::measure::client_rect;
use crate::geometry::{Anchor, MountedRef, Placement, Point, Px, Rect, RectProbe, Size, place};
use crate::motion::anim::Anim;
use crate::motion::timer::use_motion_timer;
use crate::overlay::host::{OverlayId, Overlays, use_overlays};
use crate::overlay::stack::{Dismissal, LayerId, LayerStack};
use crate::time::{FRAME_SLACK, sleep};
use crate::tokens::ZLayer;
use dioxus::core::{current_scope_id, queue_effect};
use dioxus::prelude::*;

/// Which surface a popover draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Elevation {
    /// Menus, hover cards: `--shadow-pop`.
    #[default]
    Pop,
    /// The selection bubble.
    Bubble,
    /// The palette, peek: `--shadow-sheet`.
    Sheet,
}

impl Elevation {
    /// The `data-elevation` word.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            Elevation::Pop => "pop",
            Elevation::Bubble => "bubble",
            Elevation::Sheet => "sheet",
        }
    }

    /// The layer a bare popover of this elevation floats on: menus pop, the bubble is the
    /// bubble, a sheet-elevated surface is the palette's.
    fn layer(self) -> ZLayer {
        match self {
            Elevation::Pop => ZLayer::Menu,
            Elevation::Bubble => ZLayer::Bubble,
            Elevation::Sheet => ZLayer::Palette,
        }
    }
}

/// What closes a popover.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Dismiss {
    /// Escape, or a click outside.
    #[default]
    EscAndOutside,
    /// Escape only.
    EscOnly,
    /// Only its owner.
    None,
}

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
        ZLayer::Bubble => "bubble",
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
        bounds: crate::geometry::use_rect(),
        surface: crate::geometry::use_rect(),
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
        spawn(async move {
            sleep(FRAME_SLACK).await;
            if let Some(rect) = client_rect(&element.0).await {
                let mut slot = slot;
                slot.set(Some(rect));
            }
        });
    }

    /// Where the surface goes, relative to the overlay bounds: `place()` against the measured
    /// bounds and the surface's own size, or against the anchor alone before either is known.
    pub(crate) fn origin(&self, anchor: Option<Rect>, want: Placement, gap: Px) -> Point {
        let Some(anchor) = anchor else {
            return Point::default();
        };
        let bounds = self.bounds.rect().unwrap_or(UNMEASURED);
        let content = self
            .surface
            .rect()
            .map(|rect| rect.size)
            .unwrap_or_default();
        place(anchor, content, bounds, want, gap).origin
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
        let catches = matches!(self.stacking, Stacking::Layer(Dismiss::EscAndOutside));
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
                let _ = crate::task::try_set(stack, next);
            }
        }
    }

    /// Join the layer stack again, on top, as a hidden surface is shown. Call it from an effect
    /// or a handler.
    pub(crate) fn rejoin(&self) {
        if let (Stacking::Layer(dismiss), Some(stack)) = (self.stacking, self.stack) {
            let next = stack.peek().clone().push(self.layer_id, dismiss);
            let _ = crate::task::try_set(stack, next);
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

/// Whether a popover is on its way out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Leaving {
    Staying,
    Fading,
}

/// A floating surface. Escape or a click outside fades it out over `--t-quick` before
/// `onclose` runs.
#[component]
pub fn Popover(
    anchor: Anchor,
    placement: Placement,
    gap: Px,
    #[props(default)] elevation: Elevation,
    #[props(default)] dismiss: Dismiss,
    onclose: EventHandler<()>,
    children: Element,
) -> Element {
    let layer = elevation.layer();
    let float = use_float(layer, Stacking::Layer(dismiss));
    let at = float.origin(float.anchor_rect(&anchor), placement, gap);
    let probe = float.surface();
    // Escape and an outside click fade the popover out, then close it, as a menu does (the
    // macOS polish pass; `Anim::MenuOut`, `--t-quick`).
    let fade = use_motion_timer(Anim::MenuOut);
    let mut leaving = use_signal(|| Leaving::Staying);
    let close = EventHandler::new(move |()| {
        if *leaving.peek() == Leaving::Staying {
            leaving.set(Leaving::Fading);
            fade.start(onclose);
        }
    });
    let presence = match leaving() {
        Leaving::Fading => Some("leaving"),
        Leaving::Staying => None,
    };
    float.show(
        rsx! {
            div {
                class: "ds-popover",
                "data-elevation": elevation.slug(),
                "data-layer": layer_slug(layer),
                "data-presence": presence,
                style: position_style(at),
                onmounted: move |event| probe.on_mounted(event),
                onkeydown: move |event| escape_closes(float, &event, close),
                {children}
            }
        },
        close,
    );
    rsx! {}
}
