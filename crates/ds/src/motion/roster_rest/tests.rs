//! The rest timer starts after the render, never from it, and two reconciles before the effect
//! runs start one timer.

use crate::appearance::{Accent, MotionLevel, Resolved, Scheme};
use crate::material::{BlurState, Material};
use crate::motion::{Presence, RowPitch, use_roster};
use crate::root::env::Env;
use crate::{InputModality, Px};
use dioxus::core::{NoOpMutations, VirtualDom};
use dioxus::prelude::*;
use std::cell::Cell;

thread_local! {
    /// Rest tasks spawned on this thread (each test runs on its own thread).
    pub(super) static SPAWNED: Cell<u32> = const { Cell::new(0) };
    static KEYS: Cell<Option<Signal<Vec<&'static str>>>> = const { Cell::new(None) };
    static PRESENCES: Cell<Option<Signal<Vec<Presence>>>> = const { Cell::new(None) };
}

#[allow(non_snake_case)]
fn List() -> Element {
    use_context_provider(|| {
        let resolved = Resolved {
            scheme: Scheme::Light,
            accent: Accent::Postmark,
            motion: MotionLevel::Standard,
        };
        Signal::new(Env {
            resolved,
            scheme: resolved.scheme,
            material: Material::Window,
            blur: BlurState::Unavailable,
            modality: InputModality::Pointer,
        })
    });
    let keys = use_signal(|| vec!["a", "b"]);
    KEYS.set(Some(keys));
    let roster = use_roster(keys(), RowPitch(Px(79.0)));
    let mut seen = use_signal(Vec::new);
    PRESENCES.set(Some(seen));
    seen.set(
        roster
            .entries()
            .iter()
            .map(|entry| entry.presence)
            .collect(),
    );
    rsx! {
        for entry in roster.entries() {
            p { key: "{entry.key}", "{entry.key}" }
        }
    }
}

fn spawned() -> u32 {
    SPAWNED.with(Cell::get)
}

fn set_keys(dom: &mut VirtualDom, next: Vec<&'static str>) {
    let mut keys = KEYS.get().expect("the list rendered");
    dom.in_runtime(|| keys.set(next));
}

#[test]
fn the_rest_timer_is_spawned_after_the_render_not_from_it() {
    let mut dom = VirtualDom::new(List);
    dom.rebuild_in_place();
    assert_eq!(spawned(), 0, "the first render spawned nothing");
    dom.process_events();
    assert_eq!(
        spawned(),
        1,
        "the effect after it started the first-show rest"
    );
}

#[test]
fn two_reconciles_before_the_effect_runs_start_one_timer() {
    let mut dom = VirtualDom::new(List);
    dom.rebuild_in_place();
    dom.process_events();
    assert_eq!(spawned(), 1);

    set_keys(&mut dom, vec!["a", "b", "x"]);
    dom.render_immediate(&mut NoOpMutations);
    set_keys(&mut dom, vec!["a", "b", "x", "y"]);
    dom.render_immediate(&mut NoOpMutations);
    assert_eq!(spawned(), 1, "a reconcile in a render spawns nothing");
    let presences = PRESENCES.get().expect("rendered");
    assert_eq!(
        dom.in_runtime(|| presences.peek().clone()),
        vec![Presence::Entering; 4],
        "both reconciles happened in their renders"
    );

    dom.process_events();
    assert_eq!(spawned(), 2, "one effect, one timer, for both reconciles");
    dom.process_events();
    assert_eq!(spawned(), 2, "and nothing more once it ran");
}
