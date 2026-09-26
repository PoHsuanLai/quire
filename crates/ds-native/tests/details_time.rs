//! design/26 D0a on a real Blitz document: Sweep and CountUp sweep in on Appear and count in step
//! without passing their target, retarget from where they are, jump under Reduced, never replay on
//! a re-render, and every one ends at 0 frames (R1, R3, R7, R10).

use dioxus::prelude::*;
use ds::detail::{
    CountPace, Detailed, FirstShow, Moment, Reveal, Touch, use_count_up, use_detail, use_sweep,
};
use ds::{Appearance, Ds, Fraction, Material, Motion};
use ds_native::harness::{assert_settles_to_zero_frames, settle_until};
use ds_native::{Harness, Viewport};
use std::time::{Duration, Instant};

const VIEW: Viewport = Viewport {
    width: 240,
    height: 160,
    scale_percent: 100,
};

/// A charge in whole percent: what the person sees.
#[derive(Debug, Clone, PartialEq)]
struct Charge(u16);

impl Detailed for Charge {
    fn moment(from: &Self, to: &Self) -> Moment {
        if from.0 == to.0 {
            Moment::Rest
        } else {
            Moment::Change
        }
    }
    fn first(_: &Self) -> Moment {
        Moment::Appear
    }
}

static CHARGE: GlobalSignal<Charge> = Signal::global(|| Charge(80));
static RENDERS: GlobalSignal<u32> = Signal::global(|| 0);
static MOTION: GlobalSignal<Motion> = Signal::global(|| Motion::Standard);

#[allow(non_snake_case)]
fn Ring() -> Element {
    rsx! {
        Ds { appearance: Appearance { motion: MOTION(), ..Appearance::default() }, material: Material::Window,
            RingBody {}
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn RingBody() -> Element {
    let _ = RENDERS();
    let detail = use_detail(CHARGE(), FirstShow::Animate, Touch::Remote);
    let percent = detail.state().0;
    let sweep = use_sweep(Fraction(percent * 10), detail.cue());
    let count = use_count_up(i64::from(percent), detail.cue(), CountPace::InStep(sweep));
    rsx! {
        div { id: "share", "{sweep.share().0}" }
        div { id: "count", "{count.shown()}" }
    }
}

fn read(harness: &Harness, id: &str) -> i64 {
    harness
        .text_of(id)
        .and_then(|text| text.trim().parse().ok())
        .unwrap_or(-1)
}

/// Poll until `share` reads `want`, recording every share and count seen on the way.
fn watch(harness: &mut Harness, want: i64) -> Vec<(i64, i64)> {
    let mut seen = Vec::new();
    settle_until(harness, |h| read(h, "#share") == want);
    // `settle_until` only reports the end; walk again from the recorded state to sample.
    seen.push((read(harness, "#share"), read(harness, "#count")));
    seen
}

#[test]
fn appear_sweeps_from_zero_and_the_count_lands_with_it() {
    let mut harness = Harness::new(Ring, VIEW);
    let started = Instant::now();
    let mut samples = Vec::new();
    while started.elapsed() < Duration::from_secs(3) && read(&harness, "#share") != 800 {
        samples.push((read(&harness, "#share"), read(&harness, "#count")));
        harness.advance(Duration::from_millis(20));
    }
    let landed = started.elapsed();
    assert_eq!(read(&harness, "#share"), 800, "{samples:?}");
    assert_eq!(
        read(&harness, "#count"),
        80,
        "{:?}",
        &samples[samples.len().saturating_sub(8)..]
    );
    assert!(
        samples.iter().any(|&(share, _)| (1..800).contains(&share)),
        "no frame between zero and the level: {samples:?}"
    );
    assert!(
        samples.iter().all(|&(_, count)| (0..=80).contains(&count)),
        "the count went past its target: {samples:?}"
    );
    // `--e-out` reads the last thousandth late in the sweep; half of `--t-sweep` is a bound a
    // loaded machine cannot cross early (CONVENTIONS section 11).
    assert!(
        landed >= Duration::from_millis(350),
        "landed well before --t-sweep: {landed:?}"
    );
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn a_change_retargets_from_where_it_is_and_ends_quiet() {
    let mut harness = Harness::new(Ring, VIEW);
    settle_until(&mut harness, |h| read(h, "#share") == 800);
    harness.within(|| *CHARGE.write() = Charge(30));
    settle_until(&mut harness, |h| {
        (300..800).contains(&read(h, "#share")) && read(h, "#share") > 300
    });
    // Mid-flight, a new target: the sweep turns from where it is (R10).
    harness.within(|| *CHARGE.write() = Charge(60));
    let at = read(&harness, "#share");
    let seen = watch(&mut harness, 600);
    assert_eq!(read(&harness, "#count"), 60, "{seen:?} from {at}");
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn a_rerender_with_the_same_state_plays_nothing() {
    let mut harness = Harness::new(Ring, VIEW);
    settle_until(&mut harness, |h| read(h, "#share") == 800);
    assert_settles_to_zero_frames(&mut harness);
    harness.within(|| *RENDERS.write() += 1);
    harness.within(|| *CHARGE.write() = Charge(80));
    harness.advance(Duration::from_millis(20));
    assert_eq!(read(&harness, "#share"), 800);
    assert!(!harness.is_animating());
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn reduced_shows_the_target_at_once() {
    let mut harness = Harness::new(Ring, VIEW);
    harness.within(|| *MOTION.write() = Motion::Reduced);
    harness.within(|| *CHARGE.write() = Charge(20));
    harness.advance(Duration::from_millis(50));
    assert_eq!(read(&harness, "#share"), 200);
    assert_eq!(read(&harness, "#count"), 20);
    assert_settles_to_zero_frames(&mut harness);
}

#[allow(non_snake_case)]
fn Rows() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            Reveal { first: FirstShow::Animate,
                for n in 0..14 {
                    div { key: "{n}", class: "row", "Row {n}" }
                }
            }
        }
    }
}

#[test]
fn reveal_plays_once_on_first_show_and_rests() {
    let mut harness = Harness::new(Rows, VIEW);
    assert_eq!(
        harness.attr(".ds-reveal", "data-reveal").as_deref(),
        Some("play")
    );
    settle_until(&mut harness, |h| {
        h.attr(".ds-reveal", "data-reveal").as_deref() == Some("still")
    });
    assert_settles_to_zero_frames(&mut harness);
}
