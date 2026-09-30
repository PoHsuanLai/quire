//! PaneSwitcher on a real Blitz document: a switch plays both panes at once,
//! the arriving one entering and the outgoing one leaving out of the flow, the height following
//! the arriving pane; it settles on the new pane after `settle(PaneInR)` and reports it; and a
//! switch asked for mid-slide reverses: the panes trade animations, the reversed round's settle
//! never lands, and the switcher rests on the last pane asked for.

use dioxus::prelude::*;
use ds::components::lists::preview::switcher::PaneSwitcher;
use ds::components::lists::row::size::RowSize;
use ds::motion::pane_slide::Pane;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;
use ds_harness::harness::settle_until;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

// `Harness::advance` on `Clock::Wall` lets real time pass: quire's settle timers are
// `futures-timer` sleeps, which such a harness cannot fake. Under a loaded parallel
// `cargo test --workspace` an `advance(ms(170))` can stretch past a settle it meant to stop short
// of, so a test racing an instant against a bound runs on `Clock::Virtual` instead,
// whose `advance` fires every ds timer at its exact due instant; a test that only polls with
// `ds_harness::harness::settle_until` up to a bound and asserts order stays on the default `Wall`.

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
                Button { common: Common { id: Some("to-root".to_string()), ..Common::default() }, size: ControlSize::Mini, label: "Root", onclick: move |_| shown.set(Pane::Root) }
                Button { common: Common { id: Some("to-detail".to_string()), ..Common::default() }, size: ControlSize::Mini, label: "Detail", onclick: move |_| shown.set(Pane::Detail) }
            }
            div { style: "width:320px",
                PaneSwitcher {
                    shown: shown(),
                    on_settled: move |pane: Pane| log.with_mut(|log| log.push(pane.slug())),
                    root: rsx! {
                        for name in ["Wi-Fi", "Bluetooth", "Focus"] {
                            Row { key: "{name}", leading: RowLeading::Icon(Icon::Wifi), title: name, size: RowSize::Settings, onclick: |_| {} }
                        }
                    },
                    detail: rsx! {
                        Row { leading: RowLeading::Icon(Icon::Wifi), title: "Home", accessory: Accessory::Check(Check::On), size: RowSize::Settings, onclick: |_| {} }
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
    harness.send(Input::click(at));
}

/// How long a pane switch takes to settle at the Standard level (both panes play `--t-move`).
fn slide() -> Duration {
    settle(Anim::PaneInR, MotionLevel::Standard)
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

    settle_until(&mut harness, at_rest);
    assert_eq!(harness.count(".ds-pane"), 1, "the root was dropped");
    assert_eq!(presence(&harness, "detail").as_deref(), Some("present"));
    assert_eq!(harness.attr(".ds-panes", "data-moving"), None);
    assert_eq!(log(&harness), "detail");
}

#[test]
fn a_switch_during_a_slide_reverses_cleanly() {
    let mut harness = Harness::new(
        PanesApp,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    harness.advance(ms(50));
    let first = harness.now();
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

    let reversal = harness.now();
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
    });
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
