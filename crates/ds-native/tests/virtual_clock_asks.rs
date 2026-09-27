//! sill G310-G312 on the virtual clock. An ask made between two advances runs at the instant it
//! was made, not at the end of the next advance (G311); a focus asked while the renderer holds
//! the document lands in that same frame every run, never a `FRAME_SLACK` later in some (G312);
//! and a palette reports its newly selected row in the frame of the selection, so an actions key
//! at once anchors to that row (G310).

use dioxus::prelude::*;
use ds::{
    Appearance, Availability, CommandPalette, CommandPaletteHost, Ds, Key, Material, MenuEntry,
    Rect, Trail, use_focus_request,
};
use ds_native::{Clock, Harness, HarnessConfig, Viewport};
use std::cell::RefCell;
use std::time::{Duration, Instant};
use tokio::sync::watch;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 480,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn virtual_harness(app: fn() -> Element) -> Harness {
    Harness::with_config(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

// ---- G311: a feed's wake runs at the instant of the ask --------------------------------------

thread_local! {
    /// The feed the page listens to; each test thread makes its own.
    static FEED: RefCell<Option<watch::Receiver<u32>>> = const { RefCell::new(None) };
}

/// How long the page shows an ask for before it answers: a timer started at the ask.
const ANSWER_AFTER: Duration = Duration::from_millis(150);

#[allow(non_snake_case)]
fn Listener() -> Element {
    let mut heard = use_signal(|| None::<Instant>);
    let mut answered = use_signal(|| None::<Instant>);
    use_hook(|| {
        let Some(mut feed) = FEED.with(|slot| slot.borrow_mut().take()) else {
            return;
        };
        spawn(async move {
            while feed.changed().await.is_ok() {
                heard.set(Some(ds::time::now()));
                ds::sleep(ANSWER_AFTER).await;
                answered.set(Some(ds::time::now()));
            }
        });
    });
    let shown = match answered() {
        Some(_) => "answered",
        None => "waiting",
    };
    rsx! {
        div { id: "feed", "data-state": shown }
        {heard().map(|at| rsx! { div { id: "heard", "data-at": "{at:?}" } })}
    }
}

fn answered(harness: &Harness) -> bool {
    harness.attr("#feed", "data-state").as_deref() == Some("answered")
}

#[test]
fn an_ask_between_advances_runs_at_the_instant_it_was_asked() {
    let (tell, feed) = watch::channel(0_u32);
    FEED.with(|slot| *slot.borrow_mut() = Some(feed));
    let mut harness = virtual_harness(Listener);
    harness.advance(ms(100));
    let asked = harness.now();
    tell.send(1).expect("the page listens");
    harness.advance(ANSWER_AFTER - ms(1));
    assert!(
        harness.count("#heard") == 1,
        "the ask was heard during the advance after it"
    );
    assert!(!answered(&harness), "not answered a millisecond early");
    harness.advance(ms(1));
    assert!(
        answered(&harness),
        "answered {ANSWER_AFTER:?} after the ask, not after the advance that followed it"
    );
    assert_eq!(harness.now() - asked, ANSWER_AFTER);
}

// ---- G312: a focus asked while the document is borrowed ---------------------------------------

/// A page whose field asks for the keyboard as it mounts while the same mount re-renders the
/// page (the switcher's key target): the ask's task is woken in the turn the page is dirty, so
/// dioxus polls it inside the renderer's pass, where the host's write finds the document busy.
#[allow(non_snake_case)]
fn Mounting() -> Element {
    let mut mounts = use_signal(|| 0_u32);
    rsx! {
        div { id: "count", "data-n": "{mounts}" }
        div {
            id: "target",
            tabindex: "0",
            onmounted: move |event: MountedEvent| {
                ds::focus_soon(event.data());
                mounts += 1;
            },
            "Key target"
        }
    }
}

/// A page that shows the field on a press, re-rendering a sibling in the same handler.
#[allow(non_snake_case)]
fn Shown() -> Element {
    let mut open = use_signal(|| false);
    let mut presses = use_signal(|| 0_u32);
    rsx! {
        div { id: "presses", "data-n": "{presses}" }
        div {
            id: "open",
            style: "width: 120px; height: 40px",
            onclick: move |_| {
                open.set(true);
                presses += 1;
            },
            "Open"
        }
        if open() {
            Mounting {}
        }
    }
}

/// When the focus reached `#target`, in virtual time after `start`, stepping a millisecond at a
/// time (`None` if it never did within 100 ms).
fn focus_landed(harness: &mut Harness, start: Instant) -> Option<Duration> {
    for _ in 0..=100 {
        if harness.is_focused("#target") {
            return Some(harness.now() - start);
        }
        harness.advance(ms(1));
    }
    None
}

#[test]
fn a_focus_asked_while_the_document_is_busy_lands_in_the_same_frame_every_run() {
    let landed: Vec<Option<Duration>> = (0..50)
        .map(|_| {
            let mut harness = virtual_harness(Mounting);
            let start = harness.now();
            focus_landed(&mut harness, start)
        })
        .collect();
    assert!(
        landed.iter().all(|at| *at == Some(Duration::ZERO)),
        "every run lands at once: {landed:?}"
    );
}

#[test]
fn a_focus_asked_from_a_press_lands_in_the_same_frame_every_run() {
    let landed: Vec<Option<Duration>> = (0..50)
        .map(|_| {
            let mut harness = virtual_harness(Shown);
            harness.advance(ms(50));
            let open = harness.centre("#open").expect("the button is laid out");
            let start = harness.now();
            harness.click(open);
            assert_eq!(harness.attr("#presses", "data-n").as_deref(), Some("1"));
            focus_landed(&mut harness, start)
        })
        .collect();
    assert!(
        landed.iter().all(|at| *at == Some(Duration::ZERO)),
        "every run lands at once: {landed:?}"
    );
}

// ---- G310: the palette's selected row, reported in the frame of the selection ---------------

fn item(value: u8, title: &str) -> MenuEntry<u8> {
    MenuEntry::Item {
        value,
        title: title.to_string(),
        detail: None,
        tile: None,
        trail: Trail::None,
        check: None,
        availability: Availability::Enabled,
    }
}

static ROW: GlobalSignal<Option<Rect>> = Signal::global(|| None);

#[allow(non_snake_case)]
fn Palette() -> Element {
    let field = use_focus_request();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "width:600px; height:400px",
                CommandPalette::<u8> {
                    label: "Launch".to_string(),
                    placeholder: "Search".to_string(),
                    query: String::new(),
                    tokens: Vec::new(),
                    groups: vec![("Applications".to_string(), vec![item(1, "Files"), item(2, "Firefox"), item(3, "Fonts")])],
                    empty: "Nothing".to_string(),
                    oninput: move |_| {},
                    onpick: move |_| {},
                    onclose: move |()| {},
                    host: CommandPaletteHost::Surface,
                    id: "card".to_string(),
                    focus: field,
                    on_select_rect: move |rect: Rect| *ROW.write() = Some(rect),
                }
            }
        }
    }
}

fn nth_row(n: usize) -> String {
    format!("#card .ds-menu-item:nth-child({})", n + 1)
}

fn reported(harness: &mut Harness) -> Option<Rect> {
    harness.within(|| *ROW.peek())
}

#[test]
fn the_palette_reports_the_newly_selected_row_in_the_frame_it_was_selected() {
    let mut harness = virtual_harness(Palette);
    harness.advance(ms(300));
    let first = harness.rect(&nth_row(1)).expect("the first row");
    assert_eq!(reported(&mut harness), Some(first));
    harness.key(Key::Down);
    let second = harness.rect(&nth_row(2)).expect("the second row");
    assert!(second.origin.y.0 > first.origin.y.0);
    assert_eq!(
        reported(&mut harness),
        Some(second),
        "an actions key now would anchor to the row just selected"
    );
    harness.advance(ms(100));
    assert_eq!(reported(&mut harness), Some(second), "and it stays there");
}
