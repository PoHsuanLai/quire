//! `ds::machine::use_machine` on the virtual clock: the machine steps on the inputs sent to it, is woken by
//! the clock alone at the time it asked for, hands every output to the surface's handler once,
//! and runs no timer while it is at rest.

use crate::support::hold;

use std::time::Duration;

use dioxus::prelude::*;
use ds::machine::use_machine;
use ds_harness::{Driver, Input};
use hold::{Hold, HoldIn, HoldOut, HoldParams, lock, outs, press, record, running, word, word_of};

fn app() -> Element {
    let machine = use_machine::<Hold>(
        |_| Hold::Idle,
        HoldParams {
            hold_ms: hold::hold_ms(),
        },
        lock,
        |out, _| record(out),
    );
    let word = word_of(machine.state()());
    rsx! {
        div { id: "state", style: "width:60px;height:20px", "{word}" }
        div {
            id: "press",
            style: "width:60px;height:20px",
            onpointerdown: move |_| machine.send(HoldIn::Press),
        }
        div {
            id: "long-press",
            style: "width:60px;height:20px",
            onpointerdown: move |_| {
                machine.set_params(HoldParams { hold_ms: 2000 });
                machine.send(HoldIn::Press);
            },
        }
    }
}

#[test]
fn the_clock_wakes_the_machine_at_the_time_it_asked_for() {
    let mut harness = running(app, 500);
    assert_eq!(word(&harness).as_deref(), Some("idle"));
    press(&mut harness);
    harness.advance(Duration::from_millis(20));
    assert_eq!(word(&harness).as_deref(), Some("held"));
    assert_eq!(outs(), vec![HoldOut::Pressed]);
    harness.advance(Duration::from_millis(400));
    assert_eq!(
        word(&harness).as_deref(),
        Some("held"),
        "420 ms in: still held"
    );
    harness.advance(Duration::from_millis(140));
    assert_eq!(
        word(&harness).as_deref(),
        Some("idle"),
        "560 ms in: released"
    );
    assert_eq!(outs(), vec![HoldOut::Pressed, HoldOut::Released]);
}

#[test]
fn a_press_while_held_restarts_the_hold_and_the_earlier_wake_never_fires() {
    let mut harness = running(app, 500);
    press(&mut harness);
    harness.advance(Duration::from_millis(300));
    press(&mut harness);
    harness.advance(Duration::from_millis(300));
    assert_eq!(
        word(&harness).as_deref(),
        Some("held"),
        "600 ms after the first press, 300 after the second"
    );
    assert_eq!(outs(), vec![HoldOut::Pressed, HoldOut::Pressed]);
    harness.advance(Duration::from_millis(300));
    assert_eq!(word(&harness).as_deref(), Some("idle"));
    assert_eq!(
        outs(),
        vec![HoldOut::Pressed, HoldOut::Pressed, HoldOut::Released],
        "one release, from the second press"
    );
}

#[test]
fn a_machine_at_rest_runs_no_timer() {
    let mut harness = running(app, 500);
    press(&mut harness);
    harness.advance(Duration::from_millis(600));
    assert_eq!(word(&harness).as_deref(), Some("idle"));
    harness.advance(Duration::from_millis(5000));
    assert_eq!(outs().len(), 2, "nothing more after the release");
}

#[test]
fn parameters_set_before_a_send_are_the_ones_that_step_reads() {
    let mut harness = running(app, 500);
    let at = harness
        .centre("#long-press")
        .expect("the long press target");
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(600));
    assert_eq!(
        word(&harness).as_deref(),
        Some("held"),
        "600 ms in, with the 500 ms of the render's parameters it would be released"
    );
    harness.advance(Duration::from_millis(1500));
    assert_eq!(
        word(&harness).as_deref(),
        Some("idle"),
        "2100 ms in: released"
    );
}
