//! A harness holds two thread-scoped guards, an entered Tokio runtime and an installed clock.
//! They must not care in which order harnesses on one thread are dropped (Tokio's own guards
//! panic out of order; a clock restored out of order would be stale), and a wall-clock harness
//! built while a virtual one is alive must read the wall clock, not the virtual one's.

use dioxus::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Viewport};
use std::time::{Duration, Instant};

const VIEW: Viewport = Viewport {
    width: 200,
    height: 100,
    scale_percent: 100,
};

/// Far enough from the wall clock that no scheduling jitter can confuse the two.
const HOUR: Duration = Duration::from_secs(3600);

#[allow(non_snake_case)]
fn Empty() -> Element {
    rsx! { div { "idle" } }
}

/// A harness on `clock`; a virtual one is moved an hour on, so its time cannot pass for the wall's.
fn harness(clock: Clock) -> Harness {
    let mut harness = Harness::new(Empty, HarnessConfig::new(VIEW).with_clock(clock));
    if clock == Clock::Virtual {
        harness.advance(HOUR);
    }
    harness
}

fn reads_wall() -> bool {
    let wall = Instant::now();
    let seen = ds_core::time::clock::now();
    seen.max(wall) - seen.min(wall) < Duration::from_secs(60)
}

/// Whether the clock `ds_core::time::clock::now` reads on this thread is `harness`'s.
fn reads(harness: &Harness) -> bool {
    match harness.clock() {
        Clock::Virtual => ds_core::time::clock::now() == harness.now(),
        Clock::Wall => reads_wall(),
    }
}

fn runtime_entered() -> bool {
    tokio::runtime::Handle::try_current().is_ok()
}

#[derive(Debug, Clone, Copy)]
enum Order {
    /// The first harness built goes first.
    OldestFirst,
    NewestFirst,
}

#[test]
fn two_harnesses_drop_in_either_order_without_a_panic_or_a_stale_clock() {
    use Clock::{Virtual, Wall};
    let pairs = [
        (Wall, Wall),
        (Virtual, Virtual),
        (Virtual, Wall),
        (Wall, Virtual),
    ];
    for (older, newer) in pairs {
        for order in [Order::OldestFirst, Order::NewestFirst] {
            let case = format!("{older:?} then {newer:?}, dropped {order:?}");
            let (first, second) = (harness(older), harness(newer));
            assert!(
                reads(&second),
                "{case}: the newest harness's clock is in force"
            );
            assert!(runtime_entered(), "{case}: the runtime is entered");
            let survivor = match order {
                Order::OldestFirst => {
                    drop(first);
                    second
                }
                Order::NewestFirst => {
                    drop(second);
                    first
                }
            };
            assert!(reads(&survivor), "{case}: the survivor's clock is in force");
            assert!(runtime_entered(), "{case}: the survivor keeps the runtime");
            drop(survivor);
            assert!(reads_wall(), "{case}: no stale clock after both drop");
            assert!(!runtime_entered(), "{case}: the runtime is left");
        }
    }
}

#[test]
fn a_wall_harness_beside_a_live_virtual_one_reads_the_wall_clock() {
    let virtual_harness = harness(Clock::Virtual);
    let wall = harness(Clock::Wall);
    assert!(reads_wall());
    // The wall harness's own `now` is the wall's, an hour behind the virtual one's.
    assert!(virtual_harness.now() > wall.now() + HOUR / 2);
    assert!(ds_core::time::clock::now() < virtual_harness.now());
}
