//! The two forms of `ds::machine`'s hook on the virtual clock: the owned form seeded by the
//! caller, and the controlled form over a state the caller holds; a context read at the step a
//! wake causes; a follow-up sent from the output handler; and an input sent from a render.

use crate::support::hold;

use std::time::Duration;

use dioxus::prelude::*;
use ds::machine::{use_machine, use_machine_in, use_machine_state};
use ds_harness::Driver;
use hold::{
    Hold, HoldIn, HoldOut, HoldParams, Lock, lock, outs, press, record, running, set_lock, word,
    word_of,
};

fn params() -> HoldParams {
    HoldParams {
        hold_ms: hold::hold_ms(),
    }
}

fn label(state: Hold) -> Element {
    let word = word_of(state);
    rsx! { div { id: "state", style: "width:60px;height:20px", "{word}" } }
}

/// Owned, seeded: held for 300 ms from the moment it mounts, with nothing ever sent.
fn seeded() -> Element {
    let machine = use_machine::<Hold>(
        |now| Hold::Held {
            until: now.after(300),
        },
        params(),
        lock,
        |out, _| record(out),
    );
    label(machine.state()())
}

#[test]
fn a_seeded_machine_wakes_with_no_input_sent() {
    let mut harness = running(seeded, 500);
    assert_eq!(word(&harness).as_deref(), Some("held"), "seeded held");
    assert_eq!(outs(), vec![]);
    harness.advance(Duration::from_millis(200));
    assert_eq!(word(&harness).as_deref(), Some("held"), "220 ms in");
    harness.advance(Duration::from_millis(100));
    assert_eq!(word(&harness).as_deref(), Some("idle"), "320 ms in");
    assert_eq!(
        outs(),
        vec![HoldOut::Released],
        "released once, by the clock alone"
    );
}

/// Controlled: the state is the caller's, and a handler writes it without `send`.
fn controlled() -> Element {
    let held = use_machine_state::<Hold>(|_| Hold::Idle);
    let _machine = use_machine_in(held, params(), lock, |out, _| record(out));
    rsx! {
        {label(*held.state.read())}
        div {
            id: "press",
            style: "width:60px;height:20px",
            onpointerdown: move |_| {
                let mut state = held.state;
                state.set(Hold::Held { until: held.clock.now().after(200) });
            },
        }
    }
}

#[test]
fn a_controlled_state_written_outside_send_still_wakes_the_machine() {
    let mut harness = running(controlled, 500);
    assert_eq!(word(&harness).as_deref(), Some("idle"));
    press(&mut harness);
    harness.advance(Duration::from_millis(100));
    assert_eq!(word(&harness).as_deref(), Some("held"));
    assert_eq!(outs(), vec![], "written, not sent: no Pressed");
    harness.advance(Duration::from_millis(150));
    assert_eq!(
        word(&harness).as_deref(),
        Some("idle"),
        "250 ms after the write"
    );
    assert_eq!(outs(), vec![HoldOut::Released]);
}

/// The context closure reads the lock at each step, and a follow-up is sent from the handler.
fn contextual() -> Element {
    let machine = use_machine::<Hold>(
        |_| Hold::Idle,
        params(),
        lock,
        |out, machine| {
            record(out);
            if out == HoldOut::Pressed {
                machine.send(HoldIn::Touch);
            }
        },
    );
    rsx! {
        {label(machine.state()())}
        div {
            id: "press",
            style: "width:60px;height:20px",
            onpointerdown: move |_| machine.send(HoldIn::Press),
        }
    }
}

#[test]
fn the_context_is_read_at_the_step_a_wake_causes() {
    let mut harness = running(contextual, 500);
    press(&mut harness);
    harness.advance(Duration::from_millis(300));
    set_lock(Lock::Locked);
    harness.advance(Duration::from_millis(300));
    assert_eq!(
        word(&harness).as_deref(),
        Some("held"),
        "due at 500, locked: held again"
    );
    assert_eq!(
        outs(),
        vec![HoldOut::Pressed, HoldOut::Touched, HoldOut::Extended],
        "the lock was set after the last render, and the wake still read it"
    );
    set_lock(Lock::Free);
    harness.advance(Duration::from_millis(500));
    assert_eq!(word(&harness).as_deref(), Some("idle"));
    assert_eq!(outs().last(), Some(&HoldOut::Released));
}

#[test]
fn an_output_handler_can_send_a_follow_up_that_runs_after_it() {
    let mut harness = running(contextual, 500);
    press(&mut harness);
    harness.advance(Duration::from_millis(20));
    assert_eq!(outs(), vec![HoldOut::Pressed, HoldOut::Touched]);
}

/// An input sent from the body: the first render already draws it.
fn from_render() -> Element {
    let machine = use_machine::<Hold>(|_| Hold::Idle, params(), lock, |out, _| record(out));
    use_hook(|| machine.send_from_render(HoldIn::Press));
    label(machine.state()())
}

#[test]
fn an_input_sent_from_a_render_is_drawn_at_once_and_its_outputs_follow() {
    let mut harness = running(from_render, 500);
    assert_eq!(word(&harness).as_deref(), Some("held"));
    assert_eq!(outs(), vec![HoldOut::Pressed], "once, after the render");
    harness.advance(Duration::from_millis(500));
    assert_eq!(word(&harness).as_deref(), Some("idle"));
    assert_eq!(outs(), vec![HoldOut::Pressed, HoldOut::Released]);
    harness.advance(Duration::from_millis(3000));
    assert_eq!(outs().len(), 2, "at rest: no timer");
}
