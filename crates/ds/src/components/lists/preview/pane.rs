//! PreviewPane: a Quick-Look-style pane for one thing (design/04-COMPONENTS.md
//! section 47): its picture, page, text, icon or facts over a caption, and its actions as
//! buttons with their keys as a plain chord. The launcher sets it beside its results as `CommandPalette
//! { aside }`; the Quick Look app (design/20 section 2.4) is to reuse it.
//!
//! The pane never takes the keyboard: the launcher's field keeps it, and `focused` draws which
//! action the caller's keys have reached, with the focus ring (design/27 section 6.4), so Tab
//! and Enter stay the caller's. A click on an action calls `onaction` with its number.
//!
//! Its moments come from the caller's cue (`preview_cue`): the entrance
//! springs only on contact, an in-place change cross-fades the media, and a load past its grace
//! shows the pending look in the media box.

use crate::components::controls::bump_on::bump_attrs;
use crate::components::controls::chord::Chord;
use crate::components::lists::preview::content::{PaneContent, caption, media};
use crate::components::lists::preview::cue::{
    PaneCue, pane_pending_spec, pending_look, touch_slug, use_entrance_touch,
};
use crate::components::overlays::shown_phase::use_shown_phase;
use crate::components::overlays::tooltip::Shown;
use crate::core::vocab::Shortcut;
use crate::motion::anim::Anim;
use crate::motion::detail::{
    once::use_cross_fade, operation::Operation, pending::PendingFrame, touch::Touch,
    use_pending::use_pending,
};
use dioxus::prelude::*;

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
/// `Visible` (`slide-r` over `--t-move`: `Anim::PaneInR` on contact, `Anim::PaneInROut` else),
/// and out to the right as it turns `Hidden` (`Anim::PaneOutR`), calling `on_hidden` when that
/// settles, which is when the caller drops it (and the palette's `aside`). Hidden and settled,
/// nothing is laid out.
///
/// `cue` says what caused the latest change ([`PaneCue`]: the caller's `use_detail` cue for the
/// pane's state, or a bare `Touch`). The entrance springs (`--e-spring`) only when a contact
/// caused the showing and settles at `--e-out` otherwise (design/26 R5); a Preview, Change or
/// Failure cue swaps the media with a `fade` at `--t-quick`, never replaying the entrance.
///
/// `operation` is the load the media waits on (a text file's head, a PDF's page): drawn with
/// `use_pending`, nothing for `PendingGrace`, then the pending look in place of the media (a
/// dashed ring stepping a quarter per `--t-pending-step` over the words "Loading…"), held still
/// from the token's deadline or at once under Reduced; the media comes back when it is `Idle`.
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
    let enter = match cue.touch() {
        Touch::Contact(_) => Anim::PaneInR,
        Touch::Remote => Anim::PaneInROut,
    };
    let (phase, alias) = use_shown_phase(shown, on_hidden, enter, Anim::PaneOutR);
    let entrance = use_entrance_touch(alias, cue.touch());
    let frame = use_pending(operation, pane_pending_spec());
    let (media_class, fading) = bump_attrs("ds-preview-media", use_cross_fade(cue.cue()));
    let busy = match operation {
        Operation::Running(_) => Some("true"),
        Operation::Idle => None,
    };
    let shows = match frame {
        PendingFrame::Idle => media(&content),
        PendingFrame::Step(_) | PendingFrame::Stalled => pending_look(frame),
    };
    let label = content.label();
    rsx! {
        div {
            class: "ds-preview",
            role: "region",
            "aria-label": "{label}",
            "data-content": content.slug(),
            "data-shown": phase.shown().slug(),
            "data-presence": phase.presence(),
            "data-pulse": alias.slug(),
            "data-touch": touch_slug(entrance),
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
