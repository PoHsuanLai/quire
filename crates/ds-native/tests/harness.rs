//! Components driven through a real Blitz document: pointer, keys and time.
//!
//! The four overlay and list cases are the plan's (a menu opens on click and closes on Escape; a
//! hover card appears after 450 ms and not before; a toast hides after 5200 ms; a row leaves and
//! the rows below heal), beside the control cases, the same timings driven through the hubs the
//! root provides, and the wave 2 integration props (a field focused on mount takes typing; the
//! Space editor reports the dot picked inside it).

use dioxus::prelude::*;
use ds::{
    Anchor, Anim, AnimatedList, Appearance, Availability, Button, ButtonVariant, Count, Ds,
    Emphasis, Exit, HoverCard, HoverEvent, HoverKey, HoverKind, HoverTarget, Key, ListPresence,
    ListRow, Material, Menu, MenuEntry, MenuKind, Point, PulseKey, Px, RowPitch, Selection, Switch,
    Toggle, Trail, use_hover_hub, use_roster, use_toast_hub, use_toasts,
};
use ds::{
    DotIndex, Focus, Grain, InputVariant, PRESETS, Scheme, SpaceEditor, SpaceLook, TextInput, Theme,
};
use ds_native::harness::settle_until;
use ds_native::{Harness, Viewport};
use ds_settings::{AppName, use_environment};
use std::time::{Duration, Instant};

const VIEW: Viewport = Viewport {
    width: 480,
    height: 360,
    scale_percent: 100,
};

/// Wraps a test's content in the root every quire surface draws inside.
#[component]
fn Root(children: Element) -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, {children} }
    }
}

fn flip(switch: Switch) -> Switch {
    match switch {
        Switch::On => Switch::Off,
        Switch::Off => Switch::On,
    }
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
    let mut pinned = use_signal(|| Switch::Off);
    rsx! {
        Root {
            Button {
                variant: ButtonVariant::Secondary,
                label: "Pin",
                pressed: Some(pinned()),
                onclick: move |_| pinned.set(flip(pinned())),
            }
        }
    }
}

#[test]
fn a_button_click_flips_aria_pressed() {
    let mut harness = Harness::new(PressApp, VIEW);
    let button = centre(&harness, ".ds-button");
    let pressed = |harness: &Harness| harness.attr(".ds-button", "aria-pressed");
    assert_eq!(pressed(&harness).as_deref(), Some("false"));
    harness.click(button);
    assert_eq!(pressed(&harness).as_deref(), Some("true"));
    harness.click(button);
    assert_eq!(pressed(&harness).as_deref(), Some("false"));
}

#[test]
fn the_root_stamps_the_last_input_modality() {
    let mut harness = Harness::new(PressApp, VIEW);
    let modality = |harness: &Harness| harness.attr(".ds", "data-modality");
    assert_eq!(modality(&harness).as_deref(), Some("pointer"));
    harness.key(Key::Tab);
    assert_eq!(modality(&harness).as_deref(), Some("keyboard"));
    harness.click(centre(&harness, ".ds-button"));
    assert_eq!(modality(&harness).as_deref(), Some("pointer"));
}

#[allow(non_snake_case)]
fn ToggleApp() -> Element {
    let mut wifi = use_signal(|| Switch::Off);
    rsx! {
        Root {
            Toggle { label: "Wi-Fi", value: wifi(), onchange: move |next| wifi.set(next) }
        }
    }
}

#[test]
fn a_toggle_switches() {
    let mut harness = Harness::new(ToggleApp, VIEW);
    let checked = |harness: &Harness| harness.attr(".ds-toggle", "aria-checked");
    assert_eq!(checked(&harness).as_deref(), Some("false"));
    harness.click(centre(&harness, ".ds-toggle"));
    assert_eq!(checked(&harness).as_deref(), Some("true"));
    harness.click(centre(&harness, ".ds-toggle"));
    assert_eq!(checked(&harness).as_deref(), Some("false"));
}

#[allow(non_snake_case)]
fn CountApp() -> Element {
    let mut unread = use_signal(|| 0u32);
    rsx! {
        Root {
            Button {
                variant: ButtonVariant::Mini,
                label: "More",
                onclick: move |_| unread += 1,
            }
            Count { value: unread() }
        }
    }
}

#[test]
fn a_count_bumps_after_its_value_changes() {
    let mut harness = Harness::new(CountApp, VIEW);
    let bump = Anim::Bump.class();
    assert_eq!(harness.text_of(".ds-count").as_deref(), Some(""));
    assert!(!harness.has_class(".ds-count", bump), "bumped on mount");
    assert_eq!(harness.attr(".ds-count", "data-pulse"), None);

    harness.click(centre(&harness, ".ds-button"));
    assert_eq!(harness.text_of(".ds-count").as_deref(), Some("1"));
    assert!(harness.has_class(".ds-count", bump));
    let first = harness.attr(".ds-count", "data-pulse");
    assert!(first.is_some());

    // The next change swaps the alias, which is what restarts the animation (spike S5).
    harness.click(centre(&harness, ".ds-button"));
    assert_eq!(harness.text_of(".ds-count").as_deref(), Some("2"));
    let second = harness.attr(".ds-count", "data-pulse");
    assert!(second.is_some());
    assert_ne!(first, second);
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
        ds::ToastState::Hidden => "hidden",
        ds::ToastState::Shown { .. } => "shown",
    };
    rsx! {
        Button {
            variant: ButtonVariant::Secondary,
            label: "Archive",
            onclick: move |_| toasts.push("Archived".into(), None),
        }
        p { class: "probe-toast", "{state}" }
    }
}

#[test]
fn the_toast_hub_hides_after_its_hold_and_not_before() {
    let mut harness = Harness::new(ToastHubApp, VIEW);
    let state = |harness: &Harness| harness.text_of(".probe-toast");
    assert_eq!(state(&harness).as_deref(), Some("hidden"));
    let pushed = Instant::now();
    harness.click(centre(&harness, ".ds-button"));
    assert_eq!(state(&harness).as_deref(), Some("shown"));
    // Half the 5200 ms hold, not 5000 ms (fixed 2026-09-25, FINDINGS "Timing tests"): the old
    // check left only 200 ms of margin (4 % of the window) against a loaded machine's overshoot
    // on `advance`.
    harness.advance(ms(2600));
    assert_eq!(
        state(&harness).as_deref(),
        Some("shown"),
        "hid well before the hold ends"
    );
    let hidden = settle_until(&mut harness, |h| state(h).as_deref() == Some("hidden"));
    assert!(
        hidden.duration_since(pushed) >= ms(5200),
        "hid only once the full hold had run: {:?}",
        hidden.duration_since(pushed)
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
    let card = (HoverKey("thread:1".into()), HoverKind::Thread);
    let state = match hub.open() {
        Some(_) => "open",
        None => "closed",
    };
    rsx! {
        div {
            class: "probe-target",
            style: "width: 200px; height: 40px",
            onmouseenter: move |_| hub.feed(HoverEvent::Over(card.clone())),
            onmouseleave: move |_| hub.feed(HoverEvent::Out),
            "Dana Okafor"
        }
        p { class: "probe-hover", "{state}" }
    }
}

#[test]
fn the_hover_hub_opens_after_450_ms_and_not_before() {
    let mut harness = Harness::new(HoverHubApp, VIEW);
    let state = |harness: &Harness| harness.text_of(".probe-hover");
    let rested = Instant::now();
    harness.pointer_move(centre(&harness, ".probe-target"));
    assert_eq!(state(&harness).as_deref(), Some("closed"));
    // Under half the 450 ms open delay, not 400 ms (fixed 2026-09-25, FINDINGS "Timing tests"):
    // the old check left only 50 ms of margin (11 % of the window).
    harness.advance(ms(200));
    assert_eq!(
        state(&harness).as_deref(),
        Some("closed"),
        "open well before 450 ms"
    );
    let opened = settle_until(&mut harness, |h| state(h).as_deref() == Some("open"));
    assert!(
        opened.duration_since(rested) >= ms(450),
        "opened only once the full delay had run: {:?}",
        opened.duration_since(rested)
    );
}

// ---- The plan's component cases (wave 2 overlays and lists) ------------------------------

#[allow(non_snake_case)]
fn MenuApp() -> Element {
    rsx! {
        Root { MenuDemo {} }
    }
}

#[allow(non_snake_case)]
fn MenuDemo() -> Element {
    let mut open = use_signal(|| Switch::Off);
    let entries = ["Later today", "Tomorrow", "Next week"]
        .into_iter()
        .zip(0u8..)
        .map(|(title, value)| MenuEntry::Item {
            availability: Availability::Enabled,
            value,
            title: title.into(),
            detail: None,
            tile: None,
            trail: Trail::None,
            check: None,
        })
        .collect::<Vec<_>>();
    rsx! {
        Button {
            variant: ButtonVariant::Secondary,
            label: "Snooze",
            onclick: move |_| open.set(Switch::On),
        }
        if open() == Switch::On {
            Menu {
                kind: MenuKind::Slim,
                anchor: Anchor::Point(Point { x: Px(24.0), y: Px(64.0) }),
                entries,
                onpick: move |_: u8| open.set(Switch::Off),
                onclose: move |_| open.set(Switch::Off),
            }
        }
    }
}

#[test]
fn a_menu_opens_on_click_and_closes_on_escape() {
    let mut harness = Harness::new(MenuApp, VIEW);
    assert_eq!(harness.count(".ds-menu"), 0);
    harness.click(centre(&harness, ".ds-button"));
    assert_eq!(harness.count(".ds-menu"), 1, "{}", harness.html());
    assert_eq!(harness.count(".ds-menu-item"), 3);
    harness.key(Key::Escape);
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
        if let Some((_, kind)) = hub.open() {
            HoverCard { kind, p { "dana@example.com" } }
        }
    }
}

#[test]
fn a_hover_card_appears_after_450_ms_and_not_before() {
    let mut harness = Harness::new(HoverCardApp, VIEW);
    let rested = Instant::now();
    harness.pointer_move(centre(&harness, ".ds-hover-target"));
    // Under half the 450 ms open delay, not 400 ms (fixed 2026-09-25, FINDINGS "Timing tests"):
    // the old check left only 50 ms of margin (11 % of the window).
    harness.advance(ms(200));
    assert_eq!(harness.count(".ds-hovercard"), 0, "open well before 450 ms");
    let opened = settle_until(&mut harness, |h| h.count(".ds-hovercard") == 1);
    assert!(
        opened.duration_since(rested) >= ms(450),
        "appeared only once the full delay had run: {:?}",
        opened.duration_since(rested)
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
            variant: ButtonVariant::Secondary,
            label: "Archive",
            onclick: move |_| toasts.push("Archived".into(), None),
        }
    }
}

#[test]
fn a_toast_hides_after_5200_ms() {
    let mut harness = Harness::new(ToastApp, VIEW);
    let shown = |harness: &Harness| harness.attr(".ds-toast", "data-shown");
    let pushed = Instant::now();
    harness.click(centre(&harness, ".ds-button"));
    // It mounts below the edge for a frame, so the spring rises from there (gallery fix A).
    assert_eq!(shown(&harness).as_deref(), Some("hidden"));
    harness.advance(ms(100));
    assert_eq!(shown(&harness).as_deref(), Some("shown"));
    assert_eq!(
        harness.text_of(".ds-toast-text").as_deref(),
        Some("Archived")
    );
    // Half the 5200 ms hold, not 5000 ms (fixed 2026-09-25, FINDINGS "Timing tests"): the old
    // check left only 200 ms of margin (4 % of the window) against a loaded machine's overshoot
    // on `advance`.
    harness.advance(ms(2500));
    assert_eq!(
        shown(&harness).as_deref(),
        Some("shown"),
        "hid well before the hold ends"
    );
    let hidden = settle_until(&mut harness, |h| shown(h).as_deref() == Some("hidden"));
    assert!(
        hidden.duration_since(pushed) >= ms(5200),
        "hid only once the full hold had run: {:?}",
        hidden.duration_since(pushed)
    );
    // Sunk: after `--t-big` and a frame nothing is laid out.
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
    let roster = use_roster(keys(), RowPitch(Px(79.0)));
    rsx! {
        AnimatedList { label: "Threads", presence: ListPresence::Present,
            for entry in roster.entries() {
                ListRow {
                    key: "{entry.key}",
                    selection: Selection::Unselected,
                    emphasis: Emphasis::Plain,
                    index: entry.index,
                    presence: entry.presence,
                    name: format!("Sender {}", entry.key),
                    via: None,
                    subject: format!("Subject {}", entry.key),
                    snippet: None,
                    time: "09:41",
                    tags: rsx! {},
                    star: None,
                    star_pulse: PulseKey::rest(Anim::Bump),
                    strip: None,
                    onclick: move |_| {
                        roster.leave(entry.key, Exit::Fold, Emphasis::Plain);
                        keys.retain(|key| *key != entry.key);
                    },
                }
            }
        }
    }
}

#[test]
fn a_row_leaves_and_the_rows_below_heal() {
    let mut harness = Harness::new(ListApp, VIEW);
    // Let the first-show entrance settle.
    harness.advance(ms(1500));
    assert_eq!(harness.count(".ds-row"), 3);
    let first_top = harness
        .rect(".ds-row:nth-child(1)")
        .map(|rect| rect.origin.y);
    let presence = |harness: &Harness, n: usize| {
        harness.attr(&format!(".ds-row:nth-child({n})"), "data-presence")
    };

    harness.click(centre(&harness, ".ds-row:nth-child(1)"));
    assert_eq!(presence(&harness, 1).as_deref(), Some("leaving"));
    assert_eq!(
        harness.count(".ds-row"),
        3,
        "dropped before its exit played"
    );

    // The fold settles at 454 ms (Standard, `settle(Anim::Fold)`): the row is gone and the rows
    // below slide up into its place, healing for another 284 ms and more.
    harness.advance(ms(500));
    assert_eq!(harness.count(".ds-row"), 2, "{}", harness.html());
    assert_eq!(presence(&harness, 1).as_deref(), Some("healing"));
    assert_eq!(presence(&harness, 2).as_deref(), Some("healing"));

    harness.advance(ms(1500));
    assert_eq!(presence(&harness, 1).as_deref(), Some("present"));
    assert_eq!(presence(&harness, 2).as_deref(), Some("present"));
    assert_eq!(
        harness
            .rect(".ds-row:nth-child(1)")
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
            Field { focus: Focus::OnMount }
        }
    }
}

#[allow(non_snake_case)]
fn ManualApp() -> Element {
    rsx! {
        Root {
            Field { focus: Focus::Manual }
        }
    }
}

#[component]
fn Field(focus: Focus) -> Element {
    let mut text = use_signal(String::new);
    rsx! {
        TextInput {
            variant: InputVariant::Boxed,
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
        let mut harness = Harness::new(app, VIEW);
        harness.advance(ms(100));
        assert_eq!(harness.text_of(".probe-text").as_deref(), Some("[]"));
        harness.key(Key::Char('a'));
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
        card_accent: ds::CardAccent::SpaceHue,
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
        Viewport {
            height: 900,
            ..VIEW
        },
    );
    assert_eq!(harness.text_of(".probe-dot").as_deref(), Some("0"));
    assert!(harness.count(".ds-stop") > 1, "{}", harness.html());
    harness.click(centre(&harness, ".ds-stop:nth-child(2)"));
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
    // `ds_settings::use_environment` spawns the portal watch (zbus's `tokio` feature) and the
    // file-watch debounce with `tokio::spawn`, which panics ("there is no reactor running")
    // unless a runtime is entered on this thread. `Harness` enters one before this component's
    // first render (`ds_native::runtime`), so this must not panic.
    let env = use_environment(AppName("consumer-test"));
    let now = env();
    rsx! {
        Root {
            p { class: "probe-theme", "{now.settings.appearance.theme:?}" }
        }
    }
}

#[test]
fn use_environment_does_not_panic_under_the_harness() {
    let mut harness = Harness::new(EnvironmentApp, VIEW);
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
}
