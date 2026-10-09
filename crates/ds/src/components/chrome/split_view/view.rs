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
//! The last pane may have a least width of its own (`least_rest`): when the view is too narrow
//! for every pane's width, the panes before it give up width, down to their own least, before
//! it does, so a reader beside a list is never squeezed to nothing.
//!
//! `axis` sets whether the panes lie side by side (`SplitAxis::Row`, the default) or one above
//! the next (`Column`; a divider then moves with Up and Down). A pane's size is `PaneSize`: pixels,
//! or a `Share` of the view's length along the axis, which keeps following the view as it
//! resizes. A `SplitView` inside a pane or as the rest is a split of its own, with its own axis
//! and its own length. `unfocused: UnfocusedPanes::Dim` draws the panes that do not hold the
//! focus at `--pane-dim`: the focus is where the caller says (`focused`) or, failing that, where a
//! press or a key last landed (renderers send no focus event up the tree to follow).
//!
//! Markup: `div.ds-split[role=group][data-axis=column][data-unfocused=dim][data-dragging]` of `div.ds-split-pane[data-shown][data-folded][data-away]` (its body inside
//! `.ds-split-pane-body`), `div.ds-split-divider[role=separator]` after each, and last
//! `div.ds-split-rest` holding `children`.

use crate::components::chrome::split_view::model::{
    DividerDrag, PaneAt, PaneFocus, PaneSize, SplitAxis, SplitPane, UnfocusedPanes,
};
use crate::components::chrome::split_view::pane::{Mover, PaneBox};
use crate::components::controls::edge_grab::EdgeGrab;
use crate::host::measure::use_rect;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::vocab::Shown;
use ds_core::word::Word;

/// How far an arrow key moves a divider, in pixels.
const KEY_STEP: f32 = 10.0;

/// The panes' sizes, which divider is held, and what the view is along its axis.
#[derive(Clone, Copy)]
struct Splits {
    sizes: Signal<Vec<Option<PaneSize>>>,
    grab: Signal<Option<(usize, EdgeGrab)>>,
    axis: SplitAxis,
    /// The view's length along its axis; none until it is laid out.
    extent: Px,
}

impl Splits {
    /// Pane `at`'s size now, in pixels: what the person left it, else its preferred one.
    fn size_of(self, panes: &[SplitPane], at: usize) -> Px {
        let kept = (self.sizes)().get(at).copied().flatten();
        kept.or_else(|| panes.get(at).map(|pane| pane.spec.preferred))
            .map_or(Px(0.0), |size| size.resolve(self.extent))
    }

    /// Pane `at`'s size as drawn: its own while open, none while folded.
    fn shown_size(self, panes: &[SplitPane], at: usize) -> Px {
        match panes.get(at).map(|pane| pane.shown) {
            Some(Shown::Visible) => self.size_of(panes, at),
            Some(Shown::Hidden) | None => Px(0.0),
        }
    }

    /// Where the pointer is along the axis.
    fn along(self, event: &PointerEvent) -> Px {
        let at = event.client_coordinates();
        Px(match self.axis {
            SplitAxis::Row => at.x,
            SplitAxis::Column => at.y,
        } as f32)
    }
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
    match pane.spec.dragged(raw, splits.extent) {
        DividerDrag::Resize(size) => {
            let mut sizes = splits.sizes;
            let mut next = sizes.peek().clone();
            next.resize(panes.len(), None);
            if let Some(slot) = next.get_mut(at) {
                *slot = Some(pane.spec.preferred.like(size, splits.extent));
            }
            sizes.set(next);
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

/// The two arrow keys that move a divider along `axis`: the one that shrinks the pane before it,
/// and the one that grows it.
fn arrows(axis: SplitAxis) -> (Key, Key) {
    match axis {
        SplitAxis::Row => (Key::ArrowLeft, Key::ArrowRight),
        SplitAxis::Column => (Key::ArrowUp, Key::ArrowDown),
    }
}

/// The divider after pane `at`.
fn divider(
    splits: Splits,
    panes: &[SplitPane],
    at: usize,
    on_shown: EventHandler<(usize, Shown)>,
) -> Element {
    let mut grab = splits.grab;
    let shown = panes.get(at).map_or(Shown::Hidden, |pane| pane.shown);
    let shown_size = splits.shown_size(panes, at);
    let (double, keys) = (panes.to_vec(), panes.to_vec());
    let (shrink, grow) = arrows(splits.axis);
    let orientation = match splits.axis {
        SplitAxis::Row => "vertical",
        SplitAxis::Column => "horizontal",
    };
    rsx! {
        div {
            class: "ds-split-divider",
            role: "separator",
            tabindex: "0",
            "aria-orientation": orientation,
            "aria-valuenow": "{shown_size.0}",
            "data-shown": shown.slug(),
            onpointerdown: move |event: PointerEvent| {
                event.stop_propagation();
                grab.set(Some((at, EdgeGrab::new(splits.along(&event), shown_size))));
            },
            ondoubleclick: move |_| {
                let preferred = double.get(at).map_or(Px(0.0), |pane| pane.spec.preferred_px(splits.extent));
                apply(splits, &double, at, preferred, on_shown);
            },
            onkeydown: move |event: KeyboardEvent| {
                let now = shown_size.0;
                let key = event.key();
                let raw = if key == shrink {
                    now - KEY_STEP
                } else if key == grow {
                    now + KEY_STEP
                } else if key == Key::Enter {
                    keys.get(at).map_or(now, |pane| pane.spec.preferred_px(splits.extent).0)
                } else {
                    return;
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
    /// Whether the panes lie side by side or one above the next.
    #[props(default)]
    axis: SplitAxis,
    /// Whether the panes without the focus are dimmed.
    #[props(default)]
    unfocused: UnfocusedPanes,
    /// The pane that holds the focus, when the caller knows (a terminal focuses its panes
    /// itself). While it is `Some` the view draws what it is given; while `None` the view
    /// follows a press or a key in a pane.
    #[props(default)]
    focused: Option<PaneAt>,
    /// A press or a key in a pane that did not hold the focus.
    #[props(default)]
    on_focus_pane: EventHandler<PaneAt>,
    #[props(default)] on_shown: EventHandler<(usize, Shown)>,
    /// The least the last pane is drawn at; the panes before it shrink first, to their least.
    #[props(default)]
    least_rest: Px,
    children: Element,
    #[props(default)] common: Common,
) -> Element {
    let probe = use_rect();
    let extent = probe.rect().map_or(Px(0.0), |rect| match axis {
        SplitAxis::Row => rect.size.width,
        SplitAxis::Column => rect.size.height,
    });
    let splits = Splits {
        sizes: use_signal(Vec::new),
        grab: use_signal(|| None),
        axis,
        extent,
    };
    let mut grab = splits.grab;
    let dragging = grab().is_some();
    let mut holder = use_signal(|| None::<PaneAt>);
    let holding = focused.or(holder());
    let hold = use_callback(move |at: PaneAt| {
        if holder.peek().as_ref() != Some(&at) {
            holder.set(Some(at));
            on_focus_pane.call(at);
        }
    });
    let mut seen_extent = use_hook(|| CopyValue::new(extent));
    let resized = *seen_extent.peek() != extent;
    if resized {
        seen_extent.set(extent);
    }
    // A pane follows the pointer, and a resized view, 1:1; anything else springs.
    let mover = if dragging || resized {
        Mover::Hand
    } else {
        Mover::Spring
    };
    let data = common.data_attributes();
    let held = panes.clone();
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-split"),
            role: "group",
            "aria-label": common.aria_label.clone().unwrap_or(label),
            "data-axis": (axis != SplitAxis::Row).then(|| axis.slug()),
            "data-unfocused": (unfocused != UnfocusedPanes::Keep).then(|| unfocused.slug()),
            "data-dragging": if dragging { Some("true") } else { None },
            onmounted: move |event: MountedEvent| {
                probe.on_mounted(event.clone());
                common.mounted(event);
            },
            onpointermove: move |event: PointerEvent| {
                if let Some((at, held_grab)) = grab() {
                    let raw = held_grab.size_at(splits.along(&event));
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
                        width: splits.size_of(&panes, at),
                        least: pane.spec.min.resolve(extent),
                        shown: pane.shown,
                        folded: pane.folded,
                        mover,
                        focus: PaneFocus::of(PaneAt::Pane(at), holding),
                        on_hold: move |()| hold.call(PaneAt::Pane(at)),
                        body: pane.body.clone(),
                    }
                    {divider(splits, &panes, at, on_shown)}
                }
            }
            div {
                class: "ds-split-rest",
                style: "--rest-least:{least_rest.0}px",
                "data-focus": PaneFocus::of(PaneAt::Rest, holding).attribute(),
                onpointerdown: move |_| hold.call(PaneAt::Rest),
                onkeydown: move |_| hold.call(PaneAt::Rest),
                {children}
            }
        }
    }
}
