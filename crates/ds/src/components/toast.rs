//! Toast: the undo toast, one dark pill with a pull tab (design/04-COMPONENTS.md section 23,
//! design/06-INTERACTIONS.md section 9).
//!
//! `Ds` renders `ToastHost` after the overlay host. It draws the hub's toast, which springs up
//! from below the card edge on `data-shown`, and drives the pull tab with
//! [`crate::overlay::pull_tab::PullTab`]: pulled right past 46 px it arms, and a release while
//! armed, or a tap that moved under 3 px, undoes.

use crate::geometry::Px;
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use crate::overlay::pull_tab::{Pull, PullPhase, PullTab};
use crate::overlay::toast_hub::{ToastHub, ToastState, use_toast_hub};
use dioxus::prelude::*;

/// The enclosing `Ds`'s toast manager: `push(text, undo)`, one visible at a time.
pub fn use_toasts() -> ToastHub {
    use_toast_hub()
}

/// Whether the toast offers the pull tab: only an operation with an undo does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Undoable {
    Yes,
    No,
}

/// `data-drag`: live while the tab follows the pointer (its transition off).
fn drag_slug(phase: PullPhase) -> &'static str {
    match phase {
        PullPhase::Idle => "idle",
        PullPhase::Dragging { .. } => "live",
    }
}

/// Renders the toast. `Ds` places it; consumers never do.
#[component]
pub fn ToastHost() -> Element {
    let hub = use_toast_hub();
    let mut tab = use_signal(PullTab::default);
    // The toast keeps its last words while it slides away: a plain cell, written as the state
    // is read, so keeping them schedules no second render.
    let mut last = use_hook(|| CopyValue::new((String::new(), Undoable::No)));
    let shown = match hub.state() {
        ToastState::Shown { text, undo } => {
            let undoable = if undo.is_some() {
                Undoable::Yes
            } else {
                Undoable::No
            };
            last.set((text, undoable));
            "shown"
        }
        ToastState::Hidden => "hidden",
    };
    let (text, undoable) = last.peek().clone();
    let pull = tab();
    let undo = move |pull: Pull| {
        if pull == Pull::Undo {
            hub.undo();
        }
    };
    rsx! {
        div { class: "ds-overlay", "data-layer": "toast",
            div { class: "ds-toast", role: "status", "data-shown": shown,
                span { class: "ds-toast-text", "{text}" }
                if undoable == Undoable::Yes {
                    span { class: "ds-toast-hint", "pull →" }
                    button {
                        r#type: "button",
                        class: "ds-toast-tab",
                        "data-drag": drag_slug(pull.phase()),
                        "data-armed": pull.arm().slug(),
                        style: "transform:translateX({pull.dx().0}px)",
                        onpointerdown: move |event| {
                            let x = event.client_coordinates().x as f32;
                            let next = tab.peek().down(Px(x));
                            tab.set(next);
                        },
                        onpointermove: move |event| {
                            let x = event.client_coordinates().x as f32;
                            let next = tab.peek().moved(Px(x));
                            if next != *tab.peek() {
                                tab.set(next);
                            }
                        },
                        onpointerup: move |_| {
                            let (next, pull) = tab.peek().up();
                            tab.set(next);
                            undo(pull);
                        },
                        onpointercancel: move |_| {
                            let (next, _) = tab.peek().up();
                            tab.set(next);
                        },
                        onclick: move |_| {
                            let (next, pull) = tab.peek().click();
                            tab.set(next);
                            undo(pull);
                        },
                        Glyph { icon: Icon::Undo, size: IconSize::Small }
                        "Undo"
                    }
                }
            }
        }
    }
}
