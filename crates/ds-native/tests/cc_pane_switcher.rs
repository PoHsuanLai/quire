//! PaneSwitcher on a real Blitz document (sill FINDINGS Q80): a switch plays both panes at once,
//! the arriving one entering and the outgoing one leaving out of the flow, the height following
//! the arriving pane; it settles on the new pane after `settle(PaneInR)` and reports it; and a
//! switch asked for mid-slide reverses: the panes trade animations, the reversed round's settle
//! never lands, and the switcher rests on the last pane asked for.

use dioxus::prelude::*;
use ds::{
    Anim, Appearance, Button, ButtonVariant, Ds, Icon, Material, Pane, PaneSwitcher, RowTrailing,
    SettingsRow, Switch,
};
use ds::{MotionLevel, StaggerIndex, settle};
use ds_native::{Harness, Viewport};
use std::time::{Duration, Instant};

// `Harness::advance` lets real (wall-clock) time pass: quire's settle timers are `futures-timer`
// sleeps, which a harness cannot fake (its module documentation). Under a loaded parallel
// `cargo test --workspace` an `advance(ms(170))` can stretch past a settle it meant to stop short
// of, so these tests never assert a state at one fixed instant around a settle: they poll with
// `settle_until` up to a bound, time the settle on the wall clock, and assert the order.

const VIEW: Viewport = Viewport {
    width: 480,
    height: 480,
    scale_percent: 100,
};

/// A tall root (three rows) and a short detail (one row), two buttons that ask for each pane,
/// and a log of every pane the switcher settled on.
#[allow(non_snake_case)]
fn PanesApp() -> Element {
    let mut shown = use_signal(|| Pane::Root);
    let mut log = use_signal(Vec::<&'static str>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover,
            div { class: "asks",
                Button { variant: ButtonVariant::Mini, label: "Root", id: "to-root", onclick: move |_| shown.set(Pane::Root) }
                Button { variant: ButtonVariant::Mini, label: "Detail", id: "to-detail", onclick: move |_| shown.set(Pane::Detail) }
            }
            div { style: "width:320px",
                PaneSwitcher {
                    shown: shown(),
                    on_settled: move |pane: Pane| log.with_mut(|log| log.push(pane.slug())),
                    root: rsx! {
                        for name in ["Wi-Fi", "Bluetooth", "Focus"] {
                            SettingsRow { key: "{name}", glyph: Icon::Wifi, title: name, onclick: |_| {} }
                        }
                    },
                    detail: rsx! {
                        SettingsRow { glyph: Icon::Wifi, title: "Home", trailing: RowTrailing::Check(Switch::On), onclick: |_| {} }
                    },
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn ask(harness: &mut Harness, id: &str) {
    let at = harness
        .centre(&format!("#{id}"))
        .unwrap_or_else(|| panic!("#{id} is missing:\n{}", harness.html()));
    harness.click(at);
}

/// How long a pane switch takes to settle at the Standard level (both panes play `--t-move`).
fn slide() -> Duration {
    settle(
        Anim::PaneInR,
        MotionLevel::Standard,
        StaggerIndex::default(),
    )
}

/// The most a settle may be stretched by a loaded machine before the test gives up.
const BOUND: Duration = Duration::from_secs(3);

/// Advance in 10 ms steps until `done` holds, for at most `BOUND`; the wall-clock instant it
/// first held, or `None` when it never did.
fn settle_until(harness: &mut Harness, done: impl Fn(&Harness) -> bool) -> Option<Instant> {
    let started = Instant::now();
    while started.elapsed() < BOUND {
        if done(harness) {
            return Some(Instant::now());
        }
        harness.advance(ms(10));
    }
    done(harness).then(Instant::now)
}

/// The switcher has come to rest: one pane drawn, nothing moving.
fn at_rest(harness: &Harness) -> bool {
    harness.count(".ds-pane") == 1 && harness.attr(".ds-panes", "data-moving").is_none()
}

fn presence(harness: &Harness, pane: &str) -> Option<String> {
    harness.attr(&format!(".ds-pane[*|data-pane={pane}]"), "data-presence")
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

fn height(harness: &Harness, selector: &str) -> f32 {
    harness
        .rect(selector)
        .unwrap_or_else(|| panic!("{selector} is not laid out"))
        .size
        .height
        .0
}

#[test]
fn a_switch_plays_both_panes_and_settles_on_the_new_one() {
    let mut harness = Harness::new(PanesApp, VIEW);
    harness.advance(ms(50));
    assert_eq!(harness.count(".ds-pane"), 1);
    assert_eq!(presence(&harness, "root").as_deref(), Some("present"));
    let tall = height(&harness, ".ds-panes");

    ask(&mut harness, "to-detail");
    harness.advance(ms(30));
    assert_eq!(
        harness.count(".ds-pane"),
        2,
        "both panes are drawn while moving"
    );
    assert_eq!(presence(&harness, "detail").as_deref(), Some("entering"));
    assert_eq!(presence(&harness, "root").as_deref(), Some("leaving"));
    assert_eq!(
        harness.attr(".ds-panes", "data-moving").as_deref(),
        Some("true")
    );
    let short = height(&harness, ".ds-panes");
    assert!(
        (short - height(&harness, ".ds-pane[*|data-pane=detail]")).abs() < 1.0 && short < tall,
        "the height follows the arriving pane: {short} of {tall}"
    );
    assert_eq!(log(&harness), "", "nothing settled yet");

    assert!(
        settle_until(&mut harness, at_rest).is_some(),
        "the switch settled within {BOUND:?}"
    );
    assert_eq!(harness.count(".ds-pane"), 1, "the root was dropped");
    assert_eq!(presence(&harness, "detail").as_deref(), Some("present"));
    assert_eq!(harness.attr(".ds-panes", "data-moving"), None);
    assert_eq!(log(&harness), "detail");
}

#[test]
fn a_switch_during_a_slide_reverses_cleanly() {
    let mut harness = Harness::new(PanesApp, VIEW);
    harness.advance(ms(50));
    let first = Instant::now();
    ask(&mut harness, "to-detail");
    harness.advance(ms(60));
    // The precondition: the first round is still moving when the reversal is asked for. It has
    // `slide()` (284 ms) of wall clock to spare against 60 ms of waiting.
    assert_eq!(
        harness.attr(".ds-panes", "data-moving").as_deref(),
        Some("true"),
        "still sliding to the detail"
    );
    assert_eq!(presence(&harness, "detail").as_deref(), Some("entering"));

    let reversal = Instant::now();
    ask(&mut harness, "to-root");
    harness.advance(ms(1));
    assert!(
        reversal.duration_since(first) < slide(),
        "the reversal came mid-slide"
    );
    assert_eq!(harness.count(".ds-pane"), 2);
    assert_eq!(
        presence(&harness, "root").as_deref(),
        Some("entering"),
        "the root turns back"
    );
    assert_eq!(presence(&harness, "detail").as_deref(), Some("leaving"));

    // Poll to rest, and watch that nothing reports the reversed round on the way.
    let rested = settle_until(&mut harness, |harness| {
        assert!(
            !log(harness).contains("detail"),
            "the reversed round never reports"
        );
        at_rest(harness)
    })
    .unwrap_or_else(|| panic!("the reversal settled within {BOUND:?}"));
    // Order, not an instant: the switcher rests only once the reversal's own round has run its
    // full settle from when it was asked, so it cannot have rested at the first round's settle.
    assert!(
        rested.duration_since(reversal) >= slide(),
        "the reversal's settle lands a full slide after it was asked: {:?}",
        rested.duration_since(reversal)
    );
    assert!(
        rested.duration_since(first) > slide(),
        "and after the first round's settle would have"
    );
    assert_eq!(presence(&harness, "root").as_deref(), Some("present"));
    assert_eq!(log(&harness), "root", "only the last round reported");
}
