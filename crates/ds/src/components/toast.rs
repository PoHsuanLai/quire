//! Toast: the undo toast, one dark pill with a pull tab (design/04-COMPONENTS.md section 23,
//! design/06-INTERACTIONS.md section 9).
//!
//! `Ds` renders `ToastHost` after the overlay host. It draws the hub's toast, which springs up
//! from below the card edge on `data-shown`, and nothing at all while the hub is empty: a hidden
//! toast mounts below the edge for a frame, rises, and after it sinks again is dropped (gallery
//! fix A: a laid-out hidden toast showed as a pill at the bottom of every root). It drives the
//! pull tab with
//! [`crate::overlay::pull_tab::PullTab`]: pulled right past 46 px it arms, and a release while
//! armed, or a tap that moved under 3 px, undoes.

use crate::geometry::Px;
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use crate::overlay::pull_tab::{Pull, PullPhase, PullTab};
use crate::overlay::toast_hub::{ToastHub, ToastState, use_toast_hub};
use crate::root::env::use_env_signal;
use crate::time::{FRAME_SLACK, sleep};
use crate::tokens::DurationToken;
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

/// What the host draws: nothing, or the toast below the edge or up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    /// No toast is laid out.
    Gone,
    /// Mounted below the edge for one frame, so the rise starts there.
    Rising,
    /// Up.
    Up,
    /// Sliding back below the edge; dropped once the spring has settled.
    Sinking,
}

/// Whether the hub has a toast up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Showing {
    Yes,
    No,
}

/// The timer a stage change starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Follow {
    /// Nothing to wait for.
    Nothing,
    /// Rise after a frame: `Rising` becomes `Up`.
    Rise,
    /// Drop once the sink has played: `Sinking` becomes `Gone`.
    Drop,
}

impl Stage {
    /// The stage the hub's news moves this one to, and the timer that follows.
    fn next(self, showing: Showing) -> (Stage, Follow) {
        match (self, showing) {
            (Stage::Gone, Showing::Yes) => (Stage::Rising, Follow::Rise),
            (Stage::Sinking, Showing::Yes) => (Stage::Up, Follow::Nothing),
            (Stage::Up, Showing::No) => (Stage::Sinking, Follow::Drop),
            (Stage::Rising, Showing::No) => (Stage::Gone, Follow::Nothing),
            (stage, _) => (stage, Follow::Nothing),
        }
    }

    /// `data-shown`, or `None` when nothing is drawn.
    fn shown(self) -> Option<&'static str> {
        match self {
            Stage::Gone => None,
            Stage::Rising | Stage::Sinking => Some("hidden"),
            Stage::Up => Some("shown"),
        }
    }
}

/// The stage the host is at, moved by the hub and by the timers each move starts.
fn use_stage(hub: ToastHub) -> Stage {
    let mut stage = use_signal(|| Stage::Gone);
    let env = use_env_signal();
    use_effect(move || {
        let showing = match hub.state() {
            ToastState::Shown { .. } => Showing::Yes,
            ToastState::Hidden => Showing::No,
        };
        let (next, follow) = stage.peek().next(showing);
        if next != *stage.peek() {
            stage.set(next);
        }
        let (wait, from, to) = match follow {
            Follow::Nothing => return,
            Follow::Rise => (FRAME_SLACK, Stage::Rising, Stage::Up),
            Follow::Drop => {
                let level = env.peek().resolved.motion;
                (
                    DurationToken::Big.duration(level) + FRAME_SLACK,
                    Stage::Sinking,
                    Stage::Gone,
                )
            }
        };
        spawn(async move {
            sleep(wait).await;
            if *stage.peek() == from {
                stage.set(to);
            }
        });
    });
    stage()
}

/// Renders the toast. `Ds` places it; consumers never do.
#[component]
pub fn ToastHost() -> Element {
    let hub = use_toast_hub();
    let mut tab = use_signal(PullTab::default);
    // The toast keeps its last words while it slides away: a plain cell, written as the state
    // is read, so keeping them schedules no second render.
    let mut last = use_hook(|| CopyValue::new((String::new(), Undoable::No)));
    if let ToastState::Shown { text, undo } = hub.state() {
        let undoable = if undo.is_some() {
            Undoable::Yes
        } else {
            Undoable::No
        };
        last.set((text, undoable));
    }
    let Some(shown) = use_stage(hub).shown() else {
        return rsx! {};
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

#[cfg(test)]
mod tests {
    use super::{Follow, Showing, Stage};

    #[test]
    fn nothing_is_drawn_until_a_toast_rises_and_after_it_sinks() {
        #[rustfmt::skip]
        const CASES: &[(Stage, Showing, Stage, Follow)] = &[
            (Stage::Gone, Showing::No, Stage::Gone, Follow::Nothing),
            (Stage::Gone, Showing::Yes, Stage::Rising, Follow::Rise),
            (Stage::Rising, Showing::Yes, Stage::Rising, Follow::Nothing),
            (Stage::Rising, Showing::No, Stage::Gone, Follow::Nothing),
            (Stage::Up, Showing::Yes, Stage::Up, Follow::Nothing),
            (Stage::Up, Showing::No, Stage::Sinking, Follow::Drop),
            (Stage::Sinking, Showing::No, Stage::Sinking, Follow::Nothing),
            (Stage::Sinking, Showing::Yes, Stage::Up, Follow::Nothing),
        ];
        for &(from, showing, to, follow) in CASES {
            assert_eq!(from.next(showing), (to, follow), "{from:?} {showing:?}");
        }
        assert_eq!(Stage::Gone.shown(), None);
        assert_eq!(Stage::Rising.shown(), Some("hidden"));
        assert_eq!(Stage::Up.shown(), Some("shown"));
        assert_eq!(Stage::Sinking.shown(), Some("hidden"));
    }
}
