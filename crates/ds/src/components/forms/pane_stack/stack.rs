//! PaneStack: the pages of a settings pane, one shown at a time, the rest under it. The caller
//! owns the path ([`PanePath`]); choosing a row pushes a key onto it, `on_back` pops it. The
//! stack keeps only the transition's phase (which page is leaving, and in which slot).
//!
//! Motion (design/34 section 7): a push brings the new page in from the right and the old one
//! leaves to the left, 26 px and a fade; a pop is the mirror. One spring in Rust carries it
//! (`use_pane_slide`, bounce 0), `--pane-q` is how far the arrival has come and `--pane-way` its
//! sign, so a push turned back mid-slide turns around from where the pages are. Both pages are
//! drawn until it rests; the leaving one is out of the flow so the stack is as tall as the page
//! arriving. Under Reduced the pages only cross-fade.
//!
//! Focus: after a push the focus moves to the new page's back button; after a pop it returns to
//! the element that opened the page, named by the caller's `opener` (the id of the row, from the
//! page's key). The stack never reads the document to remember an opener, so the caller's path
//! stays the one source of truth.
//!
//! Markup: `div.ds-pane-stack` of one or two `div.ds-pane-stack-page` (`data-pane`, `data-presence`),
//! each a `PageHeader` over the caller's page.

use super::header::PageHeader;
use super::keys::goes_back;
use super::landing::{back_id, landing};
use super::path::PanePath;
use super::track::PaneTrack;
use crate::focus::select::Select;
use crate::focus::selector::focus_by_selector;
use crate::keys::{register_quire_actions, use_keys};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_motion::pane_slide::{Pane, PaneRole, PaneSlide};
use ds_motion::use_pane_slide::use_pane_slide;

/// A stack of pages at `path`. `title` names a page for its header and for the back button of the
/// page above it; `page` draws a page's content (a `Form`, usually). `opener` gives the id of the
/// element that opens the page for a key, so a pop can return the focus to it. `on_back` hears
/// the back button, Escape, the command chord `[` and Alt+Left, never at the root. `common.id`
/// names the stack (`ds-pane-stack` when absent): its back buttons are `<id>-back-root` and
/// `<id>-back-detail`.
#[component]
pub fn PaneStack<K: Clone + PartialEq + 'static>(
    path: PanePath<K>,
    title: Callback<K, String>,
    page: Callback<K, Element>,
    #[props(default)] opener: Option<Callback<K, Option<String>>>,
    #[props(default)] on_back: EventHandler<()>,
    #[props(default)] common: Common,
) -> Element {
    let id = common.id.clone().unwrap_or_else(|| "ds-pane-stack".into());
    let keys = use_keys();
    use_hook(|| register_quire_actions(&keys));
    let track = use_track(&path, &id, opener);
    let (slide, frame) = use_pane_slide(track.slot, None);
    let arrival = arrival(track.slot, frame.position());
    let at_root = path.is_root();
    let data = common.data_attributes();
    let drawn: Vec<(Pane, PaneRole, PanePath<K>)> = [Pane::Root, Pane::Detail]
        .into_iter()
        .filter_map(|slot| Some((slot, slide.role(slot), track.in_slot(slot)?.clone())))
        .filter(|(_, role, _)| *role != PaneRole::Absent)
        .collect();
    rsx! {
        div {
            id: "{id}",
            class: common.class("ds-pane-stack"),
            "aria-label": common.aria_label.clone(),
            "data-way": track.way.slug(),
            "data-moving": moving(slide),
            style: "--pane-q:{arrival:.4};--pane-way:{sign(&track)}",
            onmounted: move |event| common.mounted(event),
            onkeydown: move |event| {
                if !at_root
                    && keys.with_keymap(|keymap| goes_back(keymap, &event.key(), event.modifiers()))
                {
                    event.prevent_default();
                    event.stop_propagation();
                    on_back.call(());
                }
            },
            ..data,
            for (slot , role , at) in drawn {
                div {
                    key: "{slot.slug()}",
                    class: "ds-pane-stack-page",
                    "data-pane": slot.slug(),
                    "data-presence": presence(role),
                    "aria-hidden": hidden(role),
                    PageHeader {
                        title: title.call(at.current().clone()),
                        back: at.parent().map(|parent| title.call(parent.clone())),
                        back_id: Some(back_id(&id, slot)),
                        on_back: move |()| on_back.call(()),
                    }
                    {page.call(at.current().clone())}
                }
            }
        }
    }
}

/// The transition's phase for this render: the caller's `path` followed, and on a change the
/// focus asked to move: the task waits for the new page's element and for the document to be free.
fn use_track<K: Clone + PartialEq + 'static>(
    path: &PanePath<K>,
    id: &str,
    opener: Option<Callback<K, Option<String>>>,
) -> PaneTrack<K> {
    let mut held = use_hook(|| CopyValue::new(PaneTrack::opening(path)));
    let next = held.peek().clone().following(path);
    if next == *held.peek() {
        return next;
    }
    held.set(next.clone());
    let wanted = landing(&next, id, |key| {
        opener.and_then(|opener| opener.call(key.clone()))
    });
    if let Some(selector) = wanted {
        spawn(async move {
            let _ = focus_by_selector(selector, Select::None).await;
        });
    }
    next
}

/// How far the arriving page has come, 0 to 1: the spring's position is toward the detail slot.
fn arrival(slot: Pane, position: f32) -> f32 {
    let toward = position.clamp(0.0, 1.0);
    match slot {
        Pane::Detail => toward,
        Pane::Root => 1.0 - toward,
    }
}

/// `--pane-way`: 1 when the arrival comes from the right (a push), -1 from the left (a pop).
fn sign<K>(track: &PaneTrack<K>) -> i8 {
    match track.way {
        super::path::PaneWay::Push => 1,
        super::path::PaneWay::Pop => -1,
    }
}

fn moving(slide: PaneSlide) -> Option<&'static str> {
    match slide {
        PaneSlide::Moving { .. } => Some("true"),
        PaneSlide::Rest(_) => None,
    }
}

/// `data-presence` for a page's role.
fn presence(role: PaneRole) -> &'static str {
    match role {
        PaneRole::Shown | PaneRole::Absent => "present",
        PaneRole::Arriving => "entering",
        PaneRole::Leaving => "leaving",
    }
}

/// A leaving page is on its way out of the reading order.
fn hidden(role: PaneRole) -> Option<&'static str> {
    match role {
        PaneRole::Leaving => Some("true"),
        PaneRole::Shown | PaneRole::Arriving | PaneRole::Absent => None,
    }
}
