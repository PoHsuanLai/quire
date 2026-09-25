//! The window frame of a client-decorated window (design/04-COMPONENTS.md "Window frame";
//! design/13-BEHAVIOUR-menus-windows.md section 13.3.11): a titlebar across the top that moves
//! the window when dragged and zooms it on a double-click, the three traffic lights at its start
//! (`traffic_lights`), and grab zones along every edge that resize it (`resize_edges`). Every
//! request goes to the host's [`HostWindow`](crate::HostWindow); without one the frame draws and
//! does nothing.
//!
//! `Ds { window: WindowFrame::Titlebar { .. } }` draws it: the root becomes a column of the
//! titlebar and `div.ds-window-body` (the app's children), and the edges lie over both. The
//! default, [`WindowFrame::None`], leaves the root's markup exactly as it was.

use crate::components::resize_edges::ResizeEdges;
use crate::components::traffic_lights::{TilePose, TrafficLightGroup};
use crate::geometry::{Point, Px};
use crate::window::grab::{Grab, GrabEffect};
use crate::window::{FrameTiming, WindowHost, Zoom, use_window_host, use_window_state};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

/// Whether the window shows its traffic lights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrafficLights {
    /// Close, minimize and zoom at the titlebar's start.
    #[default]
    Shown,
    /// None: a window whose controls live elsewhere (a panel, a kiosk).
    Hidden,
}

/// Whether a root draws a window frame, and what it holds.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum WindowFrame {
    /// No frame: the root is what it always was (the server decorates the window, or nothing
    /// does).
    #[default]
    None,
    /// A titlebar with `title` centred, the lights at its start, and resize edges.
    Titlebar {
        /// The window's title, centred and truncated.
        title: String,
        /// Whether the lights are shown.
        lights: TrafficLights,
        /// The move threshold and the tiling menu's delays, from the settings.
        timing: FrameTiming,
    },
}

impl WindowFrame {
    /// A titlebar with the settings' default timing.
    pub fn titlebar(title: impl Into<String>, lights: TrafficLights) -> Self {
        WindowFrame::Titlebar {
            title: title.into(),
            lights,
            timing: FrameTiming::default(),
        }
    }

    /// The root's `data-window-frame` word, absent with no frame.
    pub(crate) fn attribute(&self) -> Option<&'static str> {
        match self {
            WindowFrame::None => None,
            WindowFrame::Titlebar { .. } => Some("titlebar"),
        }
    }
}

/// The root's content under `frame`: the children alone, or the titlebar, the children in the
/// window body, and the resize edges.
pub(crate) fn framed(frame: WindowFrame, children: Element) -> Element {
    match frame {
        WindowFrame::None => children,
        WindowFrame::Titlebar {
            title,
            lights,
            timing,
        } => rsx! {
            WindowTitlebar { title, lights, timing }
            div { class: "ds-window-body", {children} }
            ResizeEdges {}
        },
    }
}

/// The titlebar alone: what `Ds { window }` draws at the top of a window, and what a gallery
/// draws to pose it (`pose: TilePose::Open` shows the tiling menu as it mounts). A primary press
/// on its empty area that travels past `timing.move_threshold` starts the move, once; a
/// double-click zooms. Neither happens on a light, nor while the window is maximized or
/// fullscreen. `data-first-mouse` lets the first click of an inactive window through
/// (design/13 section 13.3.8).
#[component]
pub fn WindowTitlebar(
    title: String,
    #[props(default)] lights: TrafficLights,
    #[props(default)] timing: FrameTiming,
    #[props(default)] pose: TilePose,
) -> Element {
    let host = use_window_host();
    let state = use_window_state();
    let mut grab = use_hook(|| CopyValue::new(Grab::Idle));
    let movable = state.movable();
    let (on_move, on_leave) = (host.clone(), host.clone());
    rsx! {
        div {
            class: "ds-titlebar",
            "data-window": state.slug(),
            "data-activation": state.activation_slug(),
            "data-first-mouse": "",
            onpointerdown: move |event: PointerEvent| {
                let primary = event.trigger_button() == Some(MouseButton::Primary);
                grab.set(match (primary, movable) {
                    (true, true) => Grab::down(point_of(&event)),
                    _ => Grab::Idle,
                });
            },
            onpointermove: move |event: PointerEvent| {
                step(grab, on_move.as_ref(), point_of(&event), timing.move_threshold);
            },
            // A fast drag can leave the band before its next move event: the leave's point
            // still counts, then the press is over as far as the titlebar hears.
            onpointerleave: move |event: PointerEvent| {
                step(grab, on_leave.as_ref(), point_of(&event), timing.move_threshold);
                grab.set(Grab::Idle);
            },
            onpointerup: move |_| grab.set(Grab::Idle),
            ondoubleclick: move |_| {
                if let Some(host) = host.as_ref() {
                    host.host().zoom(Zoom::Toggle);
                }
            },
            if lights == TrafficLights::Shown {
                TrafficLightGroup { timing, pose }
            }
            span { class: "ds-titlebar-title ds-truncate", "{title}" }
        }
    }
}

/// Advance the press to `at`, asking the host for the move when it crosses the threshold.
fn step(mut grab: CopyValue<Grab>, host: Option<&WindowHost>, at: Point, threshold: Px) {
    let (next, effect) = grab.peek().moved(at, threshold);
    grab.set(next);
    if let (GrabEffect::BeginMove, Some(host)) = (effect, host) {
        host.host().begin_move();
    }
}

/// A pointer event's client point.
fn point_of(event: &PointerEvent) -> Point {
    let at = event.client_coordinates();
    Point {
        x: Px(at.x as f32),
        y: Px(at.y as f32),
    }
}
