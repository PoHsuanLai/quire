//! The rest timer starts after the render, never from it, and two reconciles before the effect
//! runs start one timer.

use crate::motion::{
    presence::{Exit, Presence},
    roster::RowPitch,
    use_roster::{LeaveBy, RosterSpec, use_roster},
};
use crate::style::appearance::{
    accent::Accent, motion::MotionLevel, resolve::Resolved, theme::Scheme,
};
use crate::style::appearance::{blur::BlurState, material::Material};
use crate::style::scope::Scope;
use dioxus::core::{NoOpMutations, VirtualDom};
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::vocab::{Activity, InputModality};
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
        Signal::new(Scope {
            resolved,
            scheme: resolved.scheme,
            material: Material::Window,
            blur: BlurState::Unavailable,
            modality: InputModality::Pointer,
            activity: Activity::Active,
        })
    });
    let keys = use_signal(|| vec!["a", "b"]);
    KEYS.set(Some(keys));
    let spec = RosterSpec {
        leave: LeaveBy::Action,
        exit: Exit::Row,
        pitch: RowPitch(Px(79.0)),
        on_settled: None,
    };
    let roster = use_roster(keys(), spec);
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
fn the_first_show_starts_no_timer() {
    let mut dom = VirtualDom::new(List);
    dom.rebuild_in_place();
    dom.process_events();
    assert_eq!(spawned(), 0, "rows shown first are simply there");
}

#[test]
fn two_reconciles_before_the_effect_runs_start_one_timer() {
    let mut dom = VirtualDom::new(List);
    dom.rebuild_in_place();
    dom.process_events();
    assert_eq!(spawned(), 0);

    set_keys(&mut dom, vec!["a", "b", "x"]);
    dom.render_immediate(&mut NoOpMutations);
    set_keys(&mut dom, vec!["a", "b", "x", "y"]);
    dom.render_immediate(&mut NoOpMutations);
    assert_eq!(spawned(), 0, "a reconcile in a render spawns nothing");
    let presences = PRESENCES.get().expect("rendered");
    assert_eq!(
        dom.in_runtime(|| presences.peek().clone()),
        vec![
            Presence::Present,
            Presence::Present,
            Presence::Entering,
            Presence::Entering
        ],
        "both reconciles happened in their renders"
    );

    dom.process_events();
    assert_eq!(spawned(), 1, "one effect, one timer, for both reconciles");
    dom.process_events();
    assert_eq!(spawned(), 1, "and nothing more once it ran");
}
