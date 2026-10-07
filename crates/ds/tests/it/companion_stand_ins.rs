//! The stand-in ports of an app with no companion client answer, they never panic: the signals
//! they hand out hold nothing and never change, and the calls that would reach a client do
//! nothing.

use dioxus::prelude::*;
use ds::components::companion::port::{CompanionPort, DictationPort, NoDictation, NoPort};
use ds_intents::{DictateSerial, SummonSerial};
use std::cell::RefCell;
use std::rc::Rc;

/// What the stand-ins answered inside a render: whether each signal held nothing.
#[derive(Clone, Default)]
struct Seen(Rc<RefCell<Vec<(&'static str, bool)>>>);

fn root() -> Element {
    let seen = consume_context::<Seen>();
    let note = |what: &'static str, empty: bool| seen.0.borrow_mut().push((what, empty));
    note("answers", NoPort.answers().read().is_none());
    note("heard", NoPort.heard().read().is_none());
    note("text", NoDictation.text().read().is_none());
    // The calls with nothing to answer are no-ops, not panics.
    NoPort.cancel(SummonSerial(1));
    NoDictation.stop(DictateSerial(1));
    rsx! {}
}

#[test]
fn the_no_companion_stand_ins_answer_empty_signals_instead_of_panicking() {
    let seen = Seen::default();
    let mut dom = VirtualDom::new(root);
    dom.provide_root_context(seen.clone());
    dom.rebuild_in_place();
    assert_eq!(
        *seen.0.borrow(),
        [("answers", true), ("heard", true), ("text", true)]
    );
}
