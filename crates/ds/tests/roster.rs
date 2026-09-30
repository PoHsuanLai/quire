//! The roster (design/30 section 1.3): the pure machine as tables, and the hook's timers.
//! An inserted row enters, a removed one leaves by `row-out` and is dropped when the exit settles,
//! and the rows below heal by the heights the dropped rows measured.

use ds::prelude::*;
use ds_motion::presence::{Exit, Presence};
use ds_motion::roster::{Heal, RosterState, RowPitch, StayError, Stayed};

const PITCH: RowPitch = RowPitch(Px(79.0));

/// A row's whole visible life: its presence, or its heal while it slides into a gap.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Life {
    Entering,
    Present,
    Leaving,
    Healing(Px),
}

impl Life {
    fn of(presence: Presence, heal: Option<Heal>) -> Life {
        match (heal, presence) {
            (Some(Heal { dy }), _) => Life::Healing(dy),
            (None, Presence::Entering) => Life::Entering,
            (None, Presence::Leaving(_)) => Life::Leaving,
            (None, Presence::Hidden | Presence::Present) => Life::Present,
        }
    }
}

fn rows(state: &RosterState<&'static str>) -> Vec<(&'static str, Life)> {
    state
        .entries()
        .iter()
        .map(|e| (e.key, Life::of(e.presence, e.heal)))
        .collect()
}

fn keys(state: &RosterState<&'static str>) -> Vec<&'static str> {
    state.entries().iter().map(|e| e.key).collect()
}

fn pitch(_: &&'static str) -> RowPitch {
    PITCH
}

#[test]
fn a_first_show_is_simply_there_and_later_arrivals_enter() {
    use Life::{Entering, Present};
    let state = RosterState::first_show(&["a", "b"]);
    assert_eq!(rows(&state), vec![("a", Present), ("b", Present)]);
    let state = state.reconcile(&["a", "x", "b", "y"]);
    assert_eq!(
        rows(&state),
        vec![
            ("a", Present),
            ("x", Entering),
            ("b", Present),
            ("y", Entering)
        ]
    );
    assert_eq!(
        rows(&state.rest())[1],
        ("x", Present),
        "rest settles entrances"
    );
}

#[test]
fn a_key_removed_without_an_exit_drops_at_once() {
    let state = RosterState::first_show(&["a", "b", "c"]).reconcile(&["a", "c"]);
    assert_eq!(keys(&state), vec!["a", "c"]);
}

#[test]
fn a_leaving_row_stays_where_it_was_after_a_reconcile() {
    use Life::{Leaving, Present};
    let (state, started) = RosterState::first_show(&["a", "b", "c"]).leave_batch(&["b"], Exit::Row);
    assert_eq!(started, vec!["b"]);
    let state = state.reconcile(&["a", "c"]);
    assert_eq!(
        rows(&state),
        vec![("a", Present), ("b", Leaving), ("c", Present)],
        "b lingers between a and c until it settles"
    );
    let (again, started) = state.leave_batch(&["b", "zz"], Exit::Row);
    assert!(
        started.is_empty(),
        "already leaving, or not held: nothing starts"
    );
    assert_eq!(keys(&again), vec!["a", "b", "c"]);
}

#[test]
fn settling_a_batch_drops_it_and_heals_by_the_heights_above() {
    use Life::{Healing, Present};
    let heights = |key: &&'static str| match *key {
        "a" => RowPitch(Px(40.0)),
        "c" => RowPitch(Px(55.0)),
        _ => RowPitch(Px(999.0)),
    };
    let (state, started) =
        RosterState::first_show(&["a", "b", "c", "d", "e"]).leave_batch(&["a", "c"], Exit::Row);
    assert_eq!(started, vec!["a", "c"]);
    let state = state.settled_batch(&["a", "c"], heights);
    assert_eq!(
        rows(&state),
        vec![
            ("b", Healing(Px(40.0))),
            ("d", Healing(Px(95.0))),
            ("e", Healing(Px(95.0)))
        ],
        "each row starts where it stood: b under a, d and e under both"
    );
    assert_eq!(rows(&state.rest())[0], ("b", Present), "heals settle");
}

#[test]
fn a_row_taken_back_meanwhile_is_not_dropped_by_its_batch() {
    let (state, _) = RosterState::first_show(&["a", "b"]).leave_batch(&["a", "b"], Exit::Row);
    let (state, stayed) = state.stay(&"a");
    assert_eq!(stayed, Ok(Stayed::Restored));
    let state = state.settled_batch(&["a", "b"], pitch);
    assert_eq!(keys(&state), vec!["a"]);
    assert_eq!(rows(&state)[0].1, Life::Present, "nothing above it left");
    let (_, unknown) = state.stay(&"zz");
    assert_eq!(unknown, Err(StayError::UnknownKey));
}

mod hook {
    use super::{Life, PITCH};
    use dioxus::core::{NoOpMutations, VirtualDom};
    use dioxus::prelude::*;
    use ds::prelude::*;
    use ds_core::vocab::{Activity, InputModality};
    use ds_motion::presence::Exit;
    use ds_motion::use_roster::{LeaveBy, Roster, RosterSpec, use_roster};
    use ds_style::appearance::blur::BlurState;
    use ds_style::appearance::resolve::Resolved;
    use ds_style::scope::Scope;
    use std::cell::Cell;
    use std::future::Future;
    use std::pin::pin;
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};
    use std::thread::{self, Thread};
    use std::time::{Duration, Instant};

    thread_local! {
        static ROSTER: Cell<Option<Roster<&'static str>>> = const { Cell::new(None) };
        static KEYS: Cell<Option<Signal<Vec<&'static str>>>> = const { Cell::new(None) };
    }

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[allow(non_snake_case)]
    fn List() -> Element {
        use_context_provider(|| {
            let resolved = Resolved {
                scheme: Scheme::Light,
                accent: Accent::Blue,
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
        let keys = use_signal(|| vec!["a", "b", "c", "d"]);
        KEYS.set(Some(keys));
        let leave = LIST_LEAVES.get();
        let spec = RosterSpec {
            leave,
            exit: Exit::Row,
            pitch: PITCH,
            on_settled: None,
        };
        let roster = use_roster(keys(), spec);
        ROSTER.set(Some(roster));
        rsx! {
            for entry in roster.entries() {
                p { key: "{entry.key}", "{entry.key}" }
            }
        }
    }

    thread_local! {
        static LIST_LEAVES: Cell<LeaveBy> = const { Cell::new(LeaveBy::Action) };
    }

    struct Unpark(Thread);

    impl Wake for Unpark {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }

    /// Let the dom's timers run for `span`, rendering whatever they dirty.
    fn run_for(dom: &mut VirtualDom, span: Duration) {
        let end = Instant::now() + span;
        let waker = Waker::from(Arc::new(Unpark(thread::current())));
        let mut cx = Context::from_waker(&waker);
        while Instant::now() < end {
            let ready = {
                let mut work = pin!(dom.wait_for_work());
                loop {
                    if let Poll::Ready(()) = work.as_mut().poll(&mut cx) {
                        break true;
                    }
                    let now = Instant::now();
                    if now >= end {
                        break false;
                    }
                    thread::park_timeout(end - now);
                }
            };
            if ready {
                dom.render_immediate(&mut NoOpMutations);
            }
        }
    }

    fn lives(dom: &VirtualDom, roster: Roster<&'static str>) -> Vec<(&'static str, Life)> {
        dom.in_runtime(|| {
            roster
                .entries()
                .into_iter()
                .map(|e| (e.key, Life::of(e.presence, e.heal)))
                .collect()
        })
    }

    #[test]
    fn an_action_leave_drops_the_row_and_heals_on_settle() {
        use Life::{Healing, Leaving, Present};
        LIST_LEAVES.set(LeaveBy::Action);
        let mut dom = VirtualDom::new(List);
        dom.rebuild_in_place();
        let roster = ROSTER.get().expect("the list rendered");
        assert!(
            lives(&dom, roster).iter().all(|(_, life)| *life == Present),
            "a first show is simply there"
        );
        dom.in_scope(ScopeId::APP, || roster.leave("b"));
        assert_eq!(lives(&dom, roster)[1], ("b", Leaving), "leaving at once");
        run_for(&mut dom, ms(90));
        assert_eq!(lives(&dom, roster).len(), 4, "still there mid-exit");
        run_for(&mut dom, ms(200));
        let healed = lives(&dom, roster);
        assert_eq!(
            healed.iter().map(|(k, _)| *k).collect::<Vec<_>>(),
            vec!["a", "c", "d"],
            "dropped after settle(RowOut)"
        );
        assert!(
            matches!(healed[1].1, Healing(_)) && matches!(healed[2].1, Healing(_)),
            "the rows below heal: {healed:?}"
        );
        run_for(&mut dom, ms(400));
        assert!(
            lives(&dom, roster).iter().all(|(_, life)| *life == Present),
            "heals settle"
        );
    }

    #[test]
    fn a_delisted_row_plays_its_exit_itself() {
        use Life::Leaving;
        LIST_LEAVES.set(LeaveBy::Delist);
        let mut dom = VirtualDom::new(List);
        dom.rebuild_in_place();
        let roster = ROSTER.get().expect("the list rendered");
        let mut keys = KEYS.get().expect("the list rendered");
        dom.in_runtime(|| keys.set(vec!["a", "c", "d"]));
        dom.render_immediate(&mut NoOpMutations);
        assert_eq!(
            lives(&dom, roster)[1],
            ("b", Leaving),
            "no longer listed, still drawn while it leaves"
        );
        run_for(&mut dom, ms(300));
        assert_eq!(
            lives(&dom, roster).len(),
            3,
            "dropped once the exit settled"
        );
    }
}
