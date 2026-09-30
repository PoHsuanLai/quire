//! PreviewPane: a Quick-Look-style pane for one thing (design/04-COMPONENTS.md
//! section 47): its picture, page, text, icon or facts over a caption, and its actions as
//! buttons with their keys as a plain chord. The launcher sets it beside its results as `CommandPalette
//! { aside }`; the Quick Look app (design/20 section 2.4) is to reuse it.
//!
//! The pane never takes the keyboard: the launcher's field keeps it, and `focused` draws which
//! action the caller's keys have reached, with the focus ring (design/27 section 6.4), so Tab
//! and Enter stay the caller's. A click on an action calls `onaction` with its number.
//!
//! Its moments come from the caller's cue (`preview_cue`): an in-place change cross-fades the
//! media, and a running load shows the pending look in the media box.

use crate::components::controls::chord::Chord;
use crate::components::controls::spinner::SPIN;
use crate::components::lists::preview::content::{PaneContent, caption, media};
use crate::components::lists::preview::cue::{PaneCue, pending_look};
use dioxus::prelude::*;
use ds_core::vocab::Shortcut;
use ds_core::vocab::Shown;
use ds_core::word::Word;
use ds_motion::anim::Anim;
use ds_motion::detail::{
    once::use_cross_fade, operation::Operation, pending::PendingFrame, use_pending::use_pending,
};
use ds_motion::presence::Exit;
use ds_motion::presence::spec::PresenceSpec;
use ds_motion::presence::use_presence::{Presented, use_presence};

/// One action under the preview: its words in the ink, then its keys as a plain [`Chord`] in
/// the secondary ink at the words' size (Spotlight's "Reveal in Files ⌘R"; an empty shortcut
/// draws none).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneAction {
    /// What it does ("Open", "Reveal in Files").
    pub label: String,
    /// Its keys.
    pub shortcut: Shortcut,
}

/// A preview of `content` with `actions` under it. `focused` is the action the caller's keys
/// rest on (drawn with the focus ring); `onaction` hears a click on action `i`.
///
/// `shown` plays the pane in and out: it slides in from the right as it mounts shown or turns
/// `Visible` (`slide-r` over `--t-move`: `Anim::PaneInR`), and out to the right as it turns `Hidden` (`Anim::PaneOutR`), calling `on_hidden` when that
/// settles, which is when the caller drops it (and the palette's `aside`). Hidden and settled,
/// nothing is laid out.
///
/// `cue` says what caused the latest change ([`PaneCue`]: the caller's `use_detail` cue for the
/// pane's state, or a bare `Touch`); a Preview, Change or Failure cue swaps the media with a
/// `fade` at `--t-quick`, never replaying the entrance.
///
/// `operation` is the load the media waits on (a text file's head, a PDF's page): drawn with
/// `use_pending`, the pending look replaces the media at once (a dashed ring stepping a twelfth
/// of a turn per `--t-spin-step` over the words "Loading…"); the media comes back when it is
/// `Idle`.
#[component]
pub fn PreviewPane(
    content: PaneContent,
    #[props(default)] actions: Vec<PaneAction>,
    #[props(default)] focused: Option<usize>,
    #[props(default)] onaction: EventHandler<usize>,
    #[props(default = Shown::Visible)] shown: Shown,
    #[props(default)] on_hidden: EventHandler<()>,
    #[props(into, default)] cue: PaneCue,
    #[props(default)] operation: Operation,
) -> Element {
    let spec = PresenceSpec {
        enter: Anim::PaneInR,
        exit: Exit::PaneOut,
    };
    let Presented {
        presence: phase,
        alias,
    } = use_presence(shown, spec, Some(on_hidden));
    let frame = use_pending(operation, SPIN);
    let (media_class, fading) = use_cross_fade(cue.cue()).wear("ds-preview-media");
    let busy = match operation {
        Operation::Running(_) => Some("true"),
        Operation::Idle => None,
    };
    let shows = match frame {
        PendingFrame::Idle => media(&content),
        PendingFrame::Step(_) => pending_look(frame),
    };
    let label = content.label();
    rsx! {
        div {
            class: "ds-preview",
            role: "region",
            "aria-label": "{label}",
            "data-content": content.slug(),
            "data-shown": phase.shown().slug(),
            "data-presence": phase.drawn_slug(),
            "data-pulse": alias.slug(),
            "aria-busy": busy,
            div { class: media_class, "data-pulse": fading, "data-pending": frame.slug(), {shows} }
            {caption(&content)}
            if !actions.is_empty() {
                div { class: "ds-preview-actions",
                    for (index , action) in actions.into_iter().enumerate() {
                        button {
                            key: "{index}",
                            r#type: "button",
                            class: "ds-preview-action",
                            tabindex: "-1",
                            "data-focused": (focused == Some(index)).then_some("true"),
                            onmousedown: move |event| event.prevent_default(),
                            onclick: move |_| onaction.call(index),
                            span { class: "ds-preview-action-label", "{action.label}" }
                            Chord { shortcut: action.shortcut.clone() }
                        }
                    }
                }
            }
        }
    }
}
