//! PaneSwitcher on a real Blitz document (sill FINDINGS Q80): a switch plays both panes at once,
//! the arriving one entering and the outgoing one leaving out of the flow, the height following
//! the arriving pane; it settles on the new pane after `settle(PaneInR)` and reports it; and a
//! switch asked for mid-slide reverses: the panes trade animations, the reversed round's settle
//! never lands, and the switcher rests on the last pane asked for.

use dioxus::prelude::*;
use ds::{
    Appearance, Button, ButtonVariant, Ds, Icon, Material, Pane, PaneSwitcher, RowTrailing,
    SettingsRow, Switch,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

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

    harness.advance(ms(350));
    assert_eq!(harness.count(".ds-pane"), 1, "the root was dropped");
    assert_eq!(presence(&harness, "detail").as_deref(), Some("present"));
    assert_eq!(harness.attr(".ds-panes", "data-moving"), None);
    assert_eq!(log(&harness), "detail");
}

#[test]
fn a_switch_during_a_slide_reverses_cleanly() {
    let mut harness = Harness::new(PanesApp, VIEW);
    harness.advance(ms(50));
    ask(&mut harness, "to-detail");
    harness.advance(ms(150));
    assert_eq!(presence(&harness, "detail").as_deref(), Some("entering"));

    ask(&mut harness, "to-root");
    harness.advance(ms(30));
    assert_eq!(harness.count(".ds-pane"), 2);
    assert_eq!(
        presence(&harness, "root").as_deref(),
        Some("entering"),
        "the root turns back"
    );
    assert_eq!(presence(&harness, "detail").as_deref(), Some("leaving"));

    // Past the first round's settle (284 ms after it began), before the reversal's.
    harness.advance(ms(170));
    assert_eq!(
        harness.count(".ds-pane"),
        2,
        "the reversed round's settle did not land"
    );
    assert_eq!(log(&harness), "");

    harness.advance(ms(250));
    assert_eq!(harness.count(".ds-pane"), 1);
    assert_eq!(presence(&harness, "root").as_deref(), Some("present"));
    assert_eq!(log(&harness), "root", "only the last round reported");
}
