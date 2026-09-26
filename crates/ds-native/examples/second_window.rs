//! A second window from a running app (mailo item 8: a message in its own window).
//!
//! `cargo run -p ds-native --example second_window`
//!
//! The first window has an "Open message" button: each press opens the message in a window of
//! its own (`ds_native::open_window_with`), handed its subject by props and a shared count of
//! opens through an `Arc` (a `Signal` cannot cross into another VirtualDom). The message window
//! closes with its own button, its frame, or the first window; each close prints that its
//! VirtualDom was dropped.
//!
//! `QUIRE_AUTOPILOT=1` drives it without a person: it prints the host contexts each window's
//! document sees, opens a message window, focuses it, closes it by its handle, opens one that
//! closes itself through its frame's close (`ds::WindowHost`), opens a third and then closes the
//! first window with that one still open, printing each step. The process exits 0 once `launch`
//! returns.

use dioxus::prelude::*;
use ds::{
    Appearance, Button, ButtonVariant, Ds, HostCaret, HostClickFocus, HostFileDrop, HostFind,
    HostModality, HostScale, Material,
};
use ds_native::{AppConfig, AppId, WindowHandle, WindowSpec, launch, open_window_with};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// The app's own shared state, provided at every window's root.
#[derive(Clone, Default)]
struct Opens(Arc<AtomicUsize>);

fn main() {
    launch(
        App,
        AppConfig::new("quire: first window", 480, 320)
            .with_app_id(AppId("dev.quire.SecondWindow".to_owned()))
            .with_context(Opens::default()),
    );
    println!("autopilot: launch returned");
}

fn autopilot() -> bool {
    std::env::var("QUIRE_AUTOPILOT").is_ok_and(|value| value == "1")
}

/// Which host seams this window's document was given.
fn seams(window: &str) -> String {
    let seen = [
        (
            "HostModality",
            try_consume_context::<HostModality>().is_some(),
        ),
        ("HostScale", try_consume_context::<HostScale>().is_some()),
        ("HostCaret", try_consume_context::<HostCaret>().is_some()),
        (
            "HostClickFocus",
            try_consume_context::<HostClickFocus>().is_some(),
        ),
        ("HostFind", try_consume_context::<HostFind>().is_some()),
        (
            "HostFileDrop",
            try_consume_context::<HostFileDrop>().is_some(),
        ),
        (
            "WindowHost",
            try_consume_context::<ds::WindowHost>().is_some(),
        ),
        (
            "Opens (app context)",
            try_consume_context::<Opens>().is_some(),
        ),
    ];
    let present: Vec<&str> = seen
        .iter()
        .filter(|(_, is)| *is)
        .map(|(name, _)| *name)
        .collect();
    format!(
        "{window}: {} of {} seams: {}",
        present.len(),
        seen.len(),
        present.join(", ")
    )
}

#[allow(non_snake_case)]
fn App() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, First {} }
    }
}

fn open_message(subject: &str) -> Option<WindowHandle> {
    let opens = consume_context::<Opens>();
    opens.0.fetch_add(1, Ordering::SeqCst);
    open_window_with(
        WindowSpec::new(format!("Message: {subject}"), 420, 260),
        Message,
        MessageProps {
            subject: subject.to_owned(),
        },
    )
    .inspect_err(|error| println!("open failed: {error}"))
    .ok()
}

#[allow(non_snake_case)]
fn First() -> Element {
    let mut opened = use_signal(Vec::<WindowHandle>::new);
    use_hook(|| {
        println!("{}", seams("first"));
        if autopilot() {
            spawn(drive());
        }
    });
    rsx! {
        div { style: "padding:20px; display:flex; flex-direction:column; gap:12px",
            p { "{opened.read().len()} message windows opened" }
            Button {
                variant: ButtonVariant::Primary,
                label: "Open message",
                onclick: move |_| {
                    if let Some(handle) = open_message("Quarterly report") {
                        opened.write().push(handle);
                    }
                },
            }
        }
    }
}

/// The autopilot's steps, in the first window's scope.
async fn drive() {
    let pause = |ms| ds::sleep(Duration::from_millis(ms));
    pause(600).await;
    let Some(first) = open_message("autopilot one") else {
        return;
    };
    println!("autopilot: asked, life {:?}", first.life());
    pause(1200).await;
    println!("autopilot: after a wait, life {:?}", first.life());
    first.focus();
    pause(400).await;
    first.close();
    pause(600).await;
    println!("autopilot: after close, life {:?}", first.life());
    let Some(second) = open_message("autopilot two, closes itself") else {
        return;
    };
    pause(2000).await;
    println!(
        "autopilot: after it closed itself by its frame's close, life {:?}",
        second.life()
    );
    let Some(third) = open_message("autopilot three") else {
        return;
    };
    pause(1200).await;
    println!(
        "autopilot: third message window life {:?}; closing the first window",
        third.life()
    );
    if let Some(host) = window_host_here() {
        host.host().close();
    }
}

/// The first window's host, read from the root context (a task has no hooks).
fn window_host_here() -> Option<ds::WindowHost> {
    try_consume_context::<ds::WindowHost>()
}

#[allow(non_snake_case)]
#[component]
fn Message(subject: String) -> Element {
    let opens = use_context::<Opens>();
    let title = subject.clone();
    use_hook(move || println!("{}", seams(&format!("message '{title}'"))));
    let gone = subject.clone();
    use_drop(move || println!("message '{gone}': VirtualDom dropped"));
    let closes_itself = autopilot() && subject.ends_with("closes itself");
    use_hook(move || {
        if closes_itself {
            spawn(async {
                ds::sleep(Duration::from_millis(800)).await;
                if let Some(host) = window_host_here() {
                    host.host().close();
                }
            });
        }
    });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "padding:20px; display:flex; flex-direction:column; gap:12px",
                h2 { "{subject}" }
                p { "Opens so far, shared by Arc: {opens.0.load(Ordering::SeqCst)}" }
                Button {
                    variant: ButtonVariant::Secondary,
                    label: "Close",
                    onclick: move |_| {
                        if let Some(host) = window_host_here() {
                            host.host().close();
                        }
                    },
                }
            }
        }
    }
}
