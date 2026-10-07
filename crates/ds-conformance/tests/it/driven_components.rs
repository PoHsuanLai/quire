//! Components driven through a real Blitz document: pointer, keys and time.
//!
//! The four overlay and list cases are the plan's (a menu opens on click and closes on Escape; a
//! hover card appears after 500 ms and not before; a toast hides after 5000 ms; a row leaves and
//! the rows below heal), beside the control cases, the same timings driven through the hubs the
//! root provides, and the overlay and list props (a field focused on mount takes typing; the
//! Space editor reports the dot picked inside it).

use dioxus::prelude::*;
use ds::base::vocab::RowState;
use ds::components::app::space_editor::DotIndex;
use ds::components::app::thread_row::ThreadRow;
use ds::components::overlays::hover_card::target::HoverTarget;
use ds::host::measure::Anchor;
use ds::motion::hover_intent::{HoverEvent, HoverProfile};
use ds::prelude::*;
use ds::stack::hover_hub::{HoverKey, HoverKind, use_hover_hub};
use ds::stack::toast_hub::ToastState;
use ds::stack::toast_hub::use_toast_hub;
use ds::style::space::look::CardAccent;
use ds::style::space::presets::PRESETS;
use ds_blitz::TokioSpawner;
use ds_harness::harness::settle_until;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use ds_settings::{AppName, ConfigRoot, Store, SystemPrefsSource, use_environment};
use std::sync::Arc;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 360,
    scale_percent: 100,
};

/// Wraps a test's content in the root every quire surface draws inside.
#[component]
fn Root(children: Element) -> Element {
    rsx! {
        Ds { sheet: Some(ds_shell::stylesheet()), appearance: Appearance::default(), material: Material::Window, {children} }
    }
}

fn flip(switch: Check) -> Check {
    switch.flipped()
}

/// The centre of `selector`, which the test expects to be on screen.
fn centre(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

// ---- Controls ----------------------------------------------------------------------------

#[allow(non_snake_case)]
fn PressApp() -> Element {
    let mut pinned = use_signal(|| Check::Off);
    rsx! {
        Root {
            Button {
                label: "Pin",
                value: Some(pinned()),
                onclick: move |_| pinned.set(flip(pinned())),
            }
        }
    }
}

#[test]
fn a_button_click_flips_aria_pressed() {
    let mut harness = Harness::new(
        PressApp,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    let button = centre(&harness, ".ds-button");
    let pressed = |harness: &Harness| harness.attr(".ds-button", "aria-pressed");
    assert_eq!(pressed(&harness).as_deref(), Some("false"));
    harness.send(Input::click(button));
    assert_eq!(pressed(&harness).as_deref(), Some("true"));
    harness.send(Input::click(button));
    assert_eq!(pressed(&harness).as_deref(), Some("false"));
}

#[test]
fn the_root_stamps_the_last_input_modality() {
    let mut harness = Harness::new(
        PressApp,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    let modality = |harness: &Harness| harness.attr(".ds", "data-modality");
    assert_eq!(modality(&harness).as_deref(), Some("pointer"));
    harness.send(Input::key(ShortcutKey::Tab));
    assert_eq!(modality(&harness).as_deref(), Some("keyboard"));
    harness.send(Input::click(centre(&harness, ".ds-button")));
    assert_eq!(modality(&harness).as_deref(), Some("pointer"));
}

#[allow(non_snake_case)]
fn ToggleApp() -> Element {
    let mut wifi = use_signal(|| Check::Off);
    rsx! {
        Root {
            Toggle { label: "Wi-Fi", value: wifi(), onchange: move |next| wifi.set(next) }
        }
    }
}

#[test]
fn a_toggle_switches() {
    let mut harness = Harness::new(
        ToggleApp,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    let checked = |harness: &Harness| harness.attr(".ds-toggle", "aria-checked");
    assert_eq!(checked(&harness).as_deref(), Some("false"));
    harness.send(Input::click(centre(&harness, ".ds-toggle")));
    assert_eq!(checked(&harness).as_deref(), Some("true"));
    harness.send(Input::click(centre(&harness, ".ds-toggle")));
    assert_eq!(checked(&harness).as_deref(), Some("false"));
}

// ---- The hubs' timings, through the root that owns them ----------------------------------

#[allow(non_snake_case)]
fn ToastHubApp() -> Element {
    rsx! {
        Root { ToastHubProbe {} }
    }
}

#[allow(non_snake_case)]
fn ToastHubProbe() -> Element {
    let toasts = use_toast_hub();
    let state = match toasts.state() {
        ToastState::Hidden => "hidden",
        ToastState::Shown { .. } => "shown",
    };
    rsx! {
        Button {
            label: "Archive",
            onclick: move |_| toasts.push("Archived".into(), None),
        }
        p { class: "probe-toast", "{state}" }
    }
}

#[test]
fn the_toast_hub_hides_after_its_hold_and_not_before() {
    let mut harness = Harness::new(
        ToastHubApp,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    let state = |harness: &Harness| harness.text_of(".probe-toast");
    assert_eq!(state(&harness).as_deref(), Some("hidden"));
    let pushed = harness.now();
    harness.send(Input::click(centre(&harness, ".ds-button")));
    assert_eq!(state(&harness).as_deref(), Some("shown"));
    // Half the 5000 ms hold, not all of it (FINDINGS "Timing tests"): a check at the deadline
    // leaves no margin against a loaded machine's overshoot on `advance`.
    harness.advance(ms(2500));
    assert_eq!(
        state(&harness).as_deref(),
        Some("shown"),
        "hid well before the hold ends"
    );
    let hidden = settle_until(&mut harness, |h| state(h).as_deref() == Some("hidden"));
    assert_eq!(
        hidden.duration_since(pushed),
        ms(5000),
        "hid only once the full hold had run"
    );
}

#[allow(non_snake_case)]
fn HoverHubApp() -> Element {
    rsx! {
        Root { HoverHubProbe {} }
    }
}

#[allow(non_snake_case)]
fn HoverHubProbe() -> Element {
    let hub = use_hover_hub();
    let card = (HoverKey("thread:1".into()), HoverProfile::Card);
    let state = match hub.open() {
        Some(_) => "open",
        None => "closed",
    };
    rsx! {
        div {
            class: "probe-target",
            style: "width: 200px; height: 40px",
            onmouseenter: move |_| hub.feed(HoverEvent::Over(card.clone(), HoverProfile::Card)),
            onmouseleave: move |_| hub.feed(HoverEvent::Out),
            "Dana Okafor"
        }
        p { class: "probe-hover", "{state}" }
    }
}

#[test]
fn the_hover_hub_opens_after_500_ms_and_not_before() {
    let mut harness = Harness::new(
        HoverHubApp,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    let state = |harness: &Harness| harness.text_of(".probe-hover");
    let rested = harness.now();
    harness.send(Input::pointer_move(centre(&harness, ".probe-target")));
    assert_eq!(state(&harness).as_deref(), Some("closed"));
    // Under half the 500 ms open delay, not near it (fixed 2026-09-25, FINDINGS "Timing tests"):
    // the old check left only 50 ms of margin (11 % of the window).
    harness.advance(ms(200));
    assert_eq!(
        state(&harness).as_deref(),
        Some("closed"),
        "open well before 500 ms"
    );
    let opened = settle_until(&mut harness, |h| state(h).as_deref() == Some("open"));
    assert_eq!(
        opened.duration_since(rested),
        ms(500),
        "opened only once the full delay had run"
    );
}

// ---- Component cases (overlays and lists) ---------------------------------------------------

#[allow(non_snake_case)]
fn MenuApp() -> Element {
    rsx! {
        Root { MenuDemo {} }
    }
}

#[allow(non_snake_case)]
fn MenuDemo() -> Element {
    let mut open = use_signal(|| Check::Off);
    let entries = ["Later today", "Tomorrow", "Next week"]
        .into_iter()
        .zip(0u8..)
        .map(|(title, value)| MenuItem::Item {
            value,
            title: title.into(),
            image: None,
            key: None,
            check: None,
            availability: Availability::Enabled,
            hint: None,
            after: AfterPick::Close,
            text: Default::default(),
        })
        .collect::<Vec<_>>();
    rsx! {
        Button {
            label: "Snooze",
            onclick: move |_| open.set(Check::On),
        }
        if open() == Check::On {
            Menu {
                placement: MenuPlacement::Popup,
                anchor: Anchor::Point(Point { x: Px(24.0), y: Px(64.0) }),
                items: entries,
                onpick: move |_: u8| open.set(Check::Off),
                onclose: move |_| open.set(Check::Off),
            }
        }
    }
}

#[test]
fn a_menu_opens_on_click_and_closes_on_escape() {
    let mut harness = Harness::new(MenuApp, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    assert_eq!(harness.count(".ds-menu"), 0);
    harness.send(Input::click(centre(&harness, ".ds-button")));
    assert_eq!(harness.count(".ds-menu"), 1, "{}", harness.html());
    assert_eq!(harness.count(".ds-menu-item"), 3);
    harness.send(Input::key(ShortcutKey::Escape));
    // Long enough for any exit the menu plays.
    harness.advance(ms(600));
    assert_eq!(harness.count(".ds-menu"), 0, "{}", harness.html());
}

#[allow(non_snake_case)]
fn HoverCardApp() -> Element {
    rsx! {
        Root { HoverCardDemo {} }
    }
}

#[allow(non_snake_case)]
fn HoverCardDemo() -> Element {
    let hub = use_hover_hub();
    rsx! {
        HoverTarget { hover_key: HoverKey("sender:3".into()), kind: HoverKind::Sender,
            span { "Dana Okafor" }
        }
        if hub.open().is_some() {
            HoverCard { kind: HoverKind::Sender, p { "dana@example.com" } }
        }
    }
}

#[test]
fn a_hover_card_appears_after_500_ms_and_not_before() {
    let mut harness = Harness::new(
        HoverCardApp,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    let rested = harness.now();
    harness.send(Input::pointer_move(centre(&harness, ".ds-hover-target")));
    // Under half the 500 ms open delay, not near it (fixed 2026-09-25, FINDINGS "Timing tests"):
    // the old check left only 50 ms of margin (11 % of the window).
    harness.advance(ms(200));
    assert_eq!(harness.count(".ds-hovercard"), 0, "open well before 500 ms");
    let opened = settle_until(&mut harness, |h| h.count(".ds-hovercard") == 1);
    assert_eq!(
        opened.duration_since(rested),
        ms(500),
        "appeared only once the full delay had run"
    );
}

#[allow(non_snake_case)]
fn ToastApp() -> Element {
    rsx! {
        Root { ToastDemo {} }
    }
}

#[allow(non_snake_case)]
fn ToastDemo() -> Element {
    let toasts = use_toasts();
    rsx! {
        Button {
            label: "Archive",
            onclick: move |_| toasts.push("Archived".into(), None),
        }
    }
}

#[test]
fn a_toast_hides_after_its_hold() {
    let mut harness = Harness::new(
        ToastApp,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    let shown = |harness: &Harness| harness.attr(".ds-toast", "data-presence");
    let pushed = harness.now();
    harness.send(Input::click(centre(&harness, ".ds-button")));
    // It arrives sliding in from the right, and is present once that has settled.
    assert_eq!(shown(&harness).as_deref(), Some("entering"));
    harness.advance(ms(400));
    assert_eq!(shown(&harness).as_deref(), Some("present"));
    assert_eq!(
        harness.text_of(".ds-toast-body").as_deref(),
        Some("Archived")
    );
    // Half the 5000 ms hold, not all of it (FINDINGS "Timing tests"): a check at the deadline
    // leaves no margin against a loaded machine's overshoot on `advance`.
    harness.advance(ms(2500));
    assert_eq!(
        shown(&harness).as_deref(),
        Some("present"),
        "hid well before the hold ends"
    );
    let hidden = settle_until(&mut harness, |h| shown(h).as_deref() == Some("leaving"));
    assert_eq!(
        hidden.duration_since(pushed),
        ms(5000),
        "hid only once the full hold had run"
    );
    // Slid out: after `--t-quick` and a frame nothing is laid out.
    harness.advance(ms(500));
    assert_eq!(harness.count(".ds-toast"), 0);
}

#[allow(non_snake_case)]
fn ListApp() -> Element {
    rsx! {
        Root { ListDemo {} }
    }
}

#[allow(non_snake_case)]
fn ListDemo() -> Element {
    let mut keys = use_signal(|| vec![1u32, 2, 3]);
    let items: Vec<ListItem<u32>> = keys()
        .into_iter()
        .map(|key| {
            ListItem::row(
                key,
                format!("Subject {key}"),
                rsx! {
                    ThreadRow {
                        state: RowState { selection: Selection::Unselected, emphasis: Emphasis::Plain, ..RowState::default() },
                        name: format!("Sender {key}"),
                        via: None,
                        subject: format!("Subject {key}"),
                        snippet: None,
                        time: "09:41",
                        tags: rsx! {},
                        star: None,
                        strip: None,
                        onclick: move |_| keys.retain(|shown| *shown != key),
                    }
                },
            )
        })
        .collect();
    rsx! {
        List::<u32> { label: "Threads", items }
    }
}

#[test]
fn a_row_leaves_and_the_rows_below_heal() {
    let mut harness = Harness::new(ListApp, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    // Let the first-show entrance settle.
    harness.advance(ms(1500));
    assert_eq!(harness.count(".ds-list-item"), 3);
    let first_top = harness
        .rect(".ds-list-item:nth-child(1)")
        .map(|rect| rect.origin.y);
    let presence = |harness: &Harness, n: usize| {
        harness.attr(&format!(".ds-list-item:nth-child({n})"), "data-presence")
    };

    harness.send(Input::click(centre(&harness, ".ds-list-item:nth-child(1)")));
    assert_eq!(presence(&harness, 1).as_deref(), Some("leaving"));
    assert_eq!(
        harness.count(".ds-list-item"),
        3,
        "dropped before its exit played"
    );

    // The exit settles at `settle(Anim::RowOut)`: the row is gone and the rows below slide up
    // into its place, healing for `settle(Anim::Heal)` more.
    harness.advance(settle(Anim::RowOut, MotionLevel::Standard) + ms(20));
    assert_eq!(harness.count(".ds-list-item"), 2, "{}", harness.html());
    assert_eq!(presence(&harness, 1).as_deref(), Some("healing"));
    assert_eq!(presence(&harness, 2).as_deref(), Some("healing"));

    harness.advance(ms(1500));
    assert_eq!(presence(&harness, 1).as_deref(), Some("present"));
    assert_eq!(presence(&harness, 2).as_deref(), Some("present"));
    assert_eq!(
        harness
            .rect(".ds-list-item:nth-child(1)")
            .map(|rect| rect.origin.y),
        first_top,
        "the row below did not take the removed row's place"
    );
}

// ---- Wave 2 integration props ------------------------------------------------------------

#[allow(non_snake_case)]
fn FocusApp() -> Element {
    rsx! {
        Root {
            Field { focus: FieldFocus::OnMount }
        }
    }
}

#[allow(non_snake_case)]
fn ManualApp() -> Element {
    rsx! {
        Root {
            Field { focus: FieldFocus::Manual }
        }
    }
}

#[component]
fn Field(focus: FieldFocus) -> Element {
    let mut text = use_signal(String::new);
    rsx! {
        TextField {
            label: "Link",
            value: text(),
            focus,
            oninput: move |next| text.set(next),
        }
        p { class: "probe-text", "[{text}]" }
    }
}

#[test]
fn a_field_focused_on_mount_takes_typing_without_a_click() {
    /// An app, and what its field shows after one key.
    type Case = (fn() -> Element, &'static str);
    const CASES: &[Case] = &[(FocusApp, "[a]"), (ManualApp, "[]")];
    for &(app, want) in CASES {
        let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
        harness.advance(ms(100));
        assert_eq!(harness.text_of(".probe-text").as_deref(), Some("[]"));
        harness.send(Input::key(ShortcutKey::Char('a')));
        assert_eq!(
            harness.text_of(".probe-text").as_deref(),
            Some(want),
            "{}",
            harness.html()
        );
    }
}

#[allow(non_snake_case)]
fn EditorApp() -> Element {
    let mut active = use_signal(|| DotIndex(0));
    let mut look = use_signal(|| SpaceLook {
        dots: PRESETS[0].dots.to_vec(),
        grain: Grain(35),
        theme: Theme::System,
        card_accent: CardAccent::SpaceHue,
    });
    rsx! {
        Root {
            SpaceEditor {
                look: look(),
                scheme: Scheme::Light,
                active_dot: active(),
                name: "Work".to_string(),
                onchange: move |next| look.set(next),
                on_active_dot: move |dot| active.set(dot),
            }
            p { class: "probe-dot", "{active().0}" }
        }
    }
}

#[test]
fn the_space_editor_reports_the_dot_picked_inside_it() {
    let mut harness = Harness::new(
        EditorApp,
        HarnessConfig::new(Viewport {
            height: 900,
            ..VIEW
        })
        .with_clock(Clock::Virtual),
    );
    assert_eq!(harness.text_of(".probe-dot").as_deref(), Some("0"));
    assert!(harness.count(".ds-stop") > 1, "{}", harness.html());
    harness.send(Input::click(centre(&harness, ".ds-stop:nth-child(2)")));
    assert_eq!(harness.text_of(".probe-dot").as_deref(), Some("1"));
    assert_eq!(
        harness
            .attr(".ds-stop:nth-child(2)", "aria-pressed")
            .as_deref(),
        Some("true")
    );
}

// ---- Runtime -----------------------------------------------------------------------------

#[allow(non_snake_case)]
fn EnvironmentApp() -> Element {
    // `ds_settings::use_environment` runs the file watch on the spawner it is given, and
    // `TokioSpawner::current` needs a runtime entered on this thread. `Harness` enters one before
    // this component's first render (`ds_blitz::runtime`), so this must not panic. The store is
    // a scratch directory and the preferences are fixed: no real config, no session bus.
    let store = Store::new(
        ConfigRoot::Scratch(std::env::temp_dir().join("ds-blitz-harness-environment")),
        AppName("consumer-test"),
    );
    let system = SystemPrefsSource::Fixed(SystemPrefs::default());
    let env = use_environment(store, system, Arc::new(TokioSpawner::current()));
    let now = env();
    rsx! {
        Root {
            p { class: "probe-theme", "{now.settings.appearance.theme:?}" }
        }
    }
}

#[test]
fn use_environment_does_not_panic_under_the_harness() {
    let mut harness = Harness::new(
        EnvironmentApp,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    // The portal round-trip and the initial file load both run async; give them a turn to
    // settle before reading the probe. Whether the portal answered or was absent, and whatever
    // theme the settings resolved to, both are fine — a value landing at all is the proof this
    // rendered past the first frame instead of panicking.
    harness.advance(ms(200));
    assert!(
        harness.text_of(".probe-theme").is_some(),
        "the environment-reading component rendered past its first frame: {}",
        harness.html()
    );
    let _ = std::fs::remove_dir_all(std::env::temp_dir().join("ds-blitz-harness-environment"));
}
