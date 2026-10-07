//! A hold that lasts `hold_ms` from the last press, as a `Machine`, and the harness plumbing the
//! machine hook tests share: the outputs it hands out, the lock a test sets from outside the
//! machine, and a running document with a "state" label and a "press" target.

use std::cell::{Cell, RefCell};
use std::time::Duration;

use ds::base::machine::{Elapsed, Machine};
use ds::base::time::stamp::Stamp;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hold {
    Idle,
    Held { until: Stamp },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldIn {
    Press,
    Tick,
    Touch,
}

impl From<Elapsed> for HoldIn {
    fn from(_: Elapsed) -> Self {
        HoldIn::Tick
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldOut {
    Pressed,
    Released,
    /// Woken before the hold was due: a wake that should have been dropped.
    Early,
    /// Due while the lock was on: held for another round.
    Extended,
    Touched,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoldParams {
    pub hold_ms: u64,
}

/// An outside fact the machine reads at each step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lock {
    Free,
    Locked,
}

impl Machine for Hold {
    type In = HoldIn;
    type Out = HoldOut;
    type Params = HoldParams;
    type Ctx = Lock;

    fn step(
        self,
        input: HoldIn,
        at: Stamp,
        params: &HoldParams,
        lock: &Lock,
    ) -> (Hold, Vec<HoldOut>) {
        let held = Hold::Held {
            until: at.after(params.hold_ms),
        };
        match (self, input, lock) {
            (_, HoldIn::Press, _) => (held, vec![HoldOut::Pressed]),
            (Hold::Held { until }, HoldIn::Tick, Lock::Free) if at >= until => {
                (Hold::Idle, vec![HoldOut::Released])
            }
            (Hold::Held { until }, HoldIn::Tick, Lock::Locked) if at >= until => {
                (held, vec![HoldOut::Extended])
            }
            (state, HoldIn::Tick, _) => (state, vec![HoldOut::Early]),
            (state, HoldIn::Touch, _) => (state, vec![HoldOut::Touched]),
        }
    }

    fn wake(&self) -> Option<Stamp> {
        match self {
            Hold::Idle => None,
            Hold::Held { until } => Some(*until),
        }
    }
}

thread_local! {
    static OUTS: RefCell<Vec<HoldOut>> = const { RefCell::new(Vec::new()) };
    static HOLD_MS: Cell<u64> = const { Cell::new(500) };
    static LOCK: Cell<Lock> = const { Cell::new(Lock::Free) };
}

pub fn record(out: HoldOut) {
    OUTS.with(|outs| outs.borrow_mut().push(out));
}

pub fn outs() -> Vec<HoldOut> {
    OUTS.with(|outs| outs.borrow().clone())
}

/// The hold time the apps read at render.
pub fn hold_ms() -> u64 {
    HOLD_MS.with(Cell::get)
}

/// The lock the apps' context closure reads at each step.
pub fn lock() -> Lock {
    LOCK.with(Cell::get)
}

pub fn set_lock(to: Lock) {
    LOCK.with(|lock| lock.set(to));
}

/// `app` on the virtual clock with a 500 ms hold unless `hold_ms` says otherwise.
pub fn running(app: fn() -> dioxus::prelude::Element, hold_ms: u64) -> Harness {
    OUTS.with(|outs| outs.borrow_mut().clear());
    HOLD_MS.with(|ms| ms.set(hold_ms));
    LOCK.with(|lock| lock.set(Lock::Free));
    let viewport = Viewport {
        width: 100,
        height: 40,
        scale_percent: 100,
    };
    let mut harness = Harness::new(app, HarnessConfig::new(viewport).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(20));
    harness
}

pub fn press(harness: &mut Harness) {
    let at = harness.centre("#press").expect("the press target");
    harness.send(Input::click(at));
}

pub fn word(harness: &Harness) -> Option<String> {
    harness.text_of("#state")
}

/// What the machine's state label says: "idle" or "held".
pub fn word_of(state: Hold) -> &'static str {
    match state {
        Hold::Idle => "idle",
        Hold::Held { .. } => "held",
    }
}
