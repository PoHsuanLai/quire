//! SplitView: the panes before the last have widths; the last takes what is left (design/30
//! section 2.7, `NSSplitView`). A divider is a hairline in a 6 px drag zone: dragging it sets the
//! pane before it 1:1, held to the pane's widths (`PaneSpec::dragged`); a pane that may fold
//! folds away past half its least and opens again when dragged back. A double-click on a divider
//! returns its pane to the preferred width, opening it if it was folded. A divider takes the
//! keyboard as a separator: Left and Right move it by ten pixels, Return resets it. Whether a
//! pane is open is the caller's (`SplitPane::shown`, reported by `on_shown`), so a toolbar
//! button can fold a sidebar; how wide it is when open is the view's. A pane folds by clipping
//! its body, unless it is `SplitPane::peeking`: its body is then an `EdgePeek`, which keeps its
//! strip at the window's edge and floats the sidebar over the content once the pane is folded.
//!
//! Markup: `div.ds-split[role=group][data-dragging]` of `div.ds-split-pane[data-shown][data-folded][data-away]` (its body inside
//! `.ds-split-pane-body`), `div.ds-split-divider[role=separator]` after each, and last
//! `div.ds-split-rest` holding `children`.

use crate::components::chrome::split_view::model::{DividerDrag, SplitPane};
use crate::components::chrome::split_view::pane::{Mover, PaneBox};
use crate::components::controls::edge_grab::EdgeGrab;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::vocab::Shown;
use ds_core::word::Word;

/// How far an arrow key moves a divider, in pixels.
const KEY_STEP: f32 = 10.0;

/// The panes' widths, and which divider is held.
#[derive(Clone, Copy)]
struct Splits {
    widths: Signal<Vec<Option<Px>>>,
    grab: Signal<Option<(usize, EdgeGrab)>>,
}

/// Pane `at`'s width now: what the person left it, else its preferred one.
fn width_of(widths: &[Option<Px>], panes: &[SplitPane], at: usize) -> Px {
    widths
        .get(at)
        .copied()
        .flatten()
        .or_else(|| panes.get(at).map(|pane| pane.spec.preferred))
        .unwrap_or(Px(0.0))
}

/// Apply a drag's outcome to pane `at`.
fn apply(
    splits: Splits,
    panes: &[SplitPane],
    at: usize,
    raw: Px,
    on_shown: EventHandler<(usize, Shown)>,
) {
    let Some(pane) = panes.get(at) else { return };
    match pane.spec.dragged(raw) {
        DividerDrag::Resize(width) => {
            let mut widths = splits.widths;
            let mut next = widths.peek().clone();
            next.resize(panes.len(), None);
            if let Some(slot) = next.get_mut(at) {
                *slot = Some(width);
            }
            widths.set(next);
            if pane.shown == Shown::Hidden {
                on_shown.call((at, Shown::Visible));
            }
        }
        DividerDrag::Collapse => {
            if pane.shown == Shown::Visible {
                on_shown.call((at, Shown::Hidden));
            }
        }
    }
}

/// The divider after pane `at`, `shown_width` from its pane's start.
fn divider(
    splits: Splits,
    panes: &[SplitPane],
    at: usize,
    shown: Shown,
    shown_width: Px,
    on_shown: EventHandler<(usize, Shown)>,
) -> Element {
    let mut grab = splits.grab;
    let (double, keys) = (panes.to_vec(), panes.to_vec());
    rsx! {
        div {
            class: "ds-split-divider",
            role: "separator",
            tabindex: "0",
            "aria-orientation": "vertical",
            "aria-valuenow": "{shown_width.0}",
            "data-shown": shown.slug(),
            onpointerdown: move |event: PointerEvent| {
                event.stop_propagation();
                let from = Px(event.client_coordinates().x as f32);
                grab.set(Some((at, EdgeGrab::new(from, shown_width))));
            },
            ondoubleclick: move |_| {
                let preferred = double.get(at).map_or(Px(0.0), |pane| pane.spec.preferred);
                apply(splits, &double, at, preferred, on_shown);
            },
            onkeydown: move |event: KeyboardEvent| {
                let now = shown_width.0;
                let raw = match event.key() {
                    Key::ArrowLeft => now - KEY_STEP,
                    Key::ArrowRight => now + KEY_STEP,
                    Key::Enter => keys.get(at).map_or(now, |pane| pane.spec.preferred.0),
                    _ => return,
                };
                event.prevent_default();
                apply(splits, &keys, at, Px(raw), on_shown);
            },
        }
    }
}

/// A split view of `panes` and then `children`. `on_shown` hears a pane a drag folded or opened.
#[component]
pub fn SplitView(
    #[props(into)] label: String,
    panes: Vec<SplitPane>,
    #[props(default)] on_shown: EventHandler<(usize, Shown)>,
    children: Element,
    #[props(default)] common: Common,
) -> Element {
    let splits = Splits {
        widths: use_signal(Vec::new),
        grab: use_signal(|| None),
    };
    let mut grab = splits.grab;
    let dragging = grab().is_some();
    let mover = if dragging { Mover::Hand } else { Mover::Spring };
    let widths = (splits.widths)();
    let data = common.data_attributes();
    let held = panes.clone();
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-split"),
            role: "group",
            "aria-label": common.aria_label.clone().unwrap_or(label),
            "data-dragging": if dragging { Some("true") } else { None },
            onmounted: move |event| common.mounted(event),
            onpointermove: move |event: PointerEvent| {
                if let Some((at, held_grab)) = grab() {
                    let raw = held_grab.size_at(Px(event.client_coordinates().x as f32));
                    apply(splits, &held, at, raw, on_shown);
                }
            },
            onpointerup: move |_| grab.set(None),
            onpointerleave: move |_| grab.set(None),
            ..data,
            for (at , pane) in panes.iter().enumerate() {
                Fragment { key: "{at}",
                    PaneBox {
                        index: at,
                        width: width_of(&widths, &panes, at),
                        shown: pane.shown,
                        folded: pane.folded,
                        mover,
                        body: pane.body.clone(),
                    }
                    {divider(splits, &panes, at, pane.shown, shown_width(&widths, &panes, at), on_shown)}
                }
            }
            div { class: "ds-split-rest", {children} }
        }
    }
}

/// Pane `at`'s width as drawn: its own while open, none while folded.
fn shown_width(widths: &[Option<Px>], panes: &[SplitPane], at: usize) -> Px {
    match panes.get(at).map(|pane| pane.shown) {
        Some(Shown::Visible) => width_of(widths, panes, at),
        Some(Shown::Hidden) | None => Px(0.0),
    }
}
