//! PreviewPane: a Quick-Look-style pane for one thing (sill Q292; design/04-COMPONENTS.md
//! section 47): its picture, page, text, icon or facts over a caption, and its actions as
//! buttons with their keys. The launcher sets it beside its results as `CommandPalette
//! { aside }`; the Quick Look app (design/20 section 2.4) is to reuse it.
//!
//! The pane never takes the keyboard: the launcher's field keeps it, and `focused` draws which
//! action the caller's keys have reached, with the focus ring (design/27 section 6.4), so Tab
//! and Enter stay the caller's. A click on an action calls `onaction` with its number.

use crate::components::kbd::{Kbd, KbdSize};
use crate::components::preview_content::{PaneContent, caption, media};
use crate::components::shown_phase::use_shown_phase;
use crate::components::tooltip::Shown;
use crate::components::vocab::Shortcut;
use crate::motion::anim::Anim;
use dioxus::prelude::*;

/// One action under the preview: its words and its keys (drawn as `Kbd`; an empty shortcut
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
/// `Visible` (`Anim::PaneInR`, `slide-r` at `--t-move --e-spring`), and out to the right as it
/// turns `Hidden` (`Anim::PaneOutR`), calling `on_hidden` when that settles, which is when the
/// caller drops it (and the palette's `aside`). Hidden and settled, nothing is laid out.
#[component]
pub fn PreviewPane(
    content: PaneContent,
    #[props(default)] actions: Vec<PaneAction>,
    #[props(default)] focused: Option<usize>,
    #[props(default)] onaction: EventHandler<usize>,
    #[props(default = Shown::Visible)] shown: Shown,
    #[props(default)] on_hidden: EventHandler<()>,
) -> Element {
    let (phase, alias) = use_shown_phase(shown, on_hidden, Anim::PaneInR, Anim::PaneOutR);
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
            div { class: "ds-preview-media", {media(&content)} }
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
                            Kbd { shortcut: action.shortcut.clone(), size: KbdSize::Small }
                        }
                    }
                }
            }
        }
    }
}
