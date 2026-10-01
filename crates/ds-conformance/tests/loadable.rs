//! `Loadable` on a real Blitz document, on the virtual clock (design/30 sections 1.3 and 2.9): it
//! draws the placeholder while `Loading` (the default spinner turns only while the `Operation`
//! runs: R4), the children when `Ready`, and a Failure `EmptyState` whose Retry calls the
//! caller's handler when `Failed`; a change of phase kind fades the arriving layer in
//! (`data-enter="fade"`), and nothing else does: not the first render, not a new operation.

use dioxus::prelude::*;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 360,
    height: 260,
    scale_percent: 100,
};

static PHASE: GlobalSignal<Phase> = Signal::global(|| Phase::Loading(Operation::default()));
static CUSTOM: GlobalSignal<bool> = Signal::global(|| false);
thread_local! {
    static RETRIES: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let placeholder = CUSTOM().then(|| rsx! { SkeletonRow {} });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "width:340px;height:240px;display:flex",
                Loadable {
                    phase: PHASE(),
                    placeholder,
                    onretry: move |()| RETRIES.set(RETRIES.get() + 1),
                    p { id: "content", "Hello" }
                }
            }
        }
    }
}

fn harness() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(50));
    harness
}

fn go(harness: &mut Harness, phase: Phase) {
    harness.within(|| *PHASE.write() = phase);
    harness.advance(ms(20));
}

fn running() -> Phase {
    Phase::Loading(Operation::Running(PendingToken::start()))
}

fn failed() -> Phase {
    Phase::Failed {
        title: "Could not load".to_owned(),
        description: Some("The server did not answer.".into()),
    }
}

#[test]
fn it_draws_the_subtree_of_its_phase() {
    let mut harness = harness();
    go(&mut harness, running());
    assert_eq!(
        harness.attr(".ds-loadable", "data-phase").as_deref(),
        Some("loading")
    );
    assert_eq!(
        harness.attr(".ds-loadable", "aria-busy").as_deref(),
        Some("true")
    );
    assert_eq!(
        harness.count(".ds-loadable-layer .ds-progress"),
        1,
        "the default placeholder"
    );
    assert_eq!(harness.count("#content"), 0);
    assert_eq!(harness.count(".ds-empty-state"), 0);

    go(&mut harness, Phase::Ready);
    assert_eq!(
        harness.attr(".ds-loadable", "data-phase").as_deref(),
        Some("ready")
    );
    assert_eq!(harness.attr(".ds-loadable", "aria-busy"), None);
    assert_eq!(harness.text_of("#content").as_deref(), Some("Hello"));
    assert_eq!(harness.count(".ds-progress"), 0);
    assert_eq!(harness.count(".ds-empty-state"), 0);

    go(&mut harness, failed());
    assert_eq!(
        harness.attr(".ds-loadable", "data-phase").as_deref(),
        Some("failed")
    );
    assert_eq!(harness.attr(".ds-loadable", "aria-busy"), None);
    assert_eq!(
        harness.attr(".ds-empty-state", "data-form").as_deref(),
        Some("failure")
    );
    assert_eq!(
        harness.text_of(".ds-empty-state-title").as_deref(),
        Some("Could not load")
    );
    assert_eq!(harness.count("#content"), 0);
}

#[test]
fn the_spinner_turns_only_while_the_operation_runs() {
    let mut harness = harness();
    go(&mut harness, Phase::Loading(Operation::Idle));
    assert_eq!(
        harness
            .attr(".ds-loadable .ds-progress", "data-pending")
            .as_deref(),
        Some("idle"),
        "loading without a running operation draws no turning spinner (R4)"
    );
    go(&mut harness, running());
    assert_eq!(
        harness
            .attr(".ds-loadable .ds-progress", "data-pending")
            .as_deref(),
        Some("step")
    );
}

#[test]
fn a_change_of_kind_fades_the_arriving_layer_in_and_nothing_else_does() {
    let mut harness = harness();
    assert_eq!(
        harness.attr(".ds-loadable-layer", "data-enter"),
        None,
        "the first render plays nothing"
    );
    go(&mut harness, running());
    go(&mut harness, running());
    assert_eq!(
        harness.attr(".ds-loadable-layer", "data-enter"),
        None,
        "a new operation of the same kind plays nothing"
    );
    go(&mut harness, Phase::Ready);
    assert_eq!(
        harness.attr(".ds-loadable-layer", "data-enter").as_deref(),
        Some("fade")
    );
    go(&mut harness, failed());
    assert_eq!(
        harness.attr(".ds-loadable-layer", "data-enter").as_deref(),
        Some("fade")
    );
    harness.advance(settle(Anim::MorphFadeIn, MotionLevel::Standard) + ms(50));
    assert_eq!(
        harness.attr(".ds-loadable-layer", "data-kind").as_deref(),
        Some("failed")
    );
}

#[test]
fn retry_calls_the_handler() {
    RETRIES.set(0);
    let mut harness = harness();
    go(&mut harness, failed());
    let retry = harness
        .centre(".ds-empty-state-action .ds-button")
        .expect("the Retry button");
    harness.send(Input::click(retry));
    harness.advance(ms(20));
    assert_eq!(RETRIES.get(), 1);
}

#[test]
fn a_placeholder_of_the_callers_replaces_the_spinner() {
    let mut harness = harness();
    harness.within(|| *CUSTOM.write() = true);
    go(&mut harness, running());
    assert_eq!(harness.count(".ds-loadable .ds-skeleton-row"), 1);
    assert_eq!(harness.count(".ds-loadable .ds-progress"), 0);
}
