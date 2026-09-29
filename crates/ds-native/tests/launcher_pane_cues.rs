//! On a real Blitz document: the preview pane takes the launcher's cue. A
//! showing the person caused springs in and one a service caused eases in (design/26 R5);
//! another kind while shown, a load landing and a load failing cross-fade the media in place and
//! never replay the entrance; a load past its grace shows the pending look (a ring over
//! "Loading…") and holds it still from its deadline; Reduced snaps and never loops. Every moment
//! ends at 0 frames (R3). The pane's state and its table are sill's (`preview/moment.rs`).

use dioxus::prelude::*;
use ds::detail::{Detailed, FirstShow, Moment, Touch, use_detail, use_operation};
use ds::{Appearance, Ds, Material, Motion, PaneContent, PreviewPane, Shown};
use ds_native::harness::{assert_settles_to_zero_frames, settle_until};
use ds_native::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 420,
    scale_percent: 100,
};

/// What the pane previews.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Web,
    Emoji,
}

/// The pane as the person sees it: sill's `PaneState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pane {
    Hidden,
    Showing(Kind),
    Pending,
    Failed,
}

impl Detailed for Pane {
    fn moment(from: &Self, to: &Self) -> Moment {
        use Pane::{Failed, Hidden, Pending, Showing};
        match (from, to) {
            (Hidden, Hidden) | (Pending, Pending) | (Failed, Failed) => Moment::Rest,
            (Hidden, Showing(_) | Failed) => Moment::Appear,
            (Showing(was), Showing(now)) if was == now => Moment::Rest,
            (Showing(_) | Failed, Showing(_)) | (Showing(_), Failed) => Moment::Preview,
            (Hidden | Showing(_) | Failed, Pending) => Moment::Pending,
            (Pending, Showing(_)) => Moment::Change,
            (Pending, Failed) => Moment::Failure,
            (Showing(_) | Pending | Failed, Hidden) => Moment::Dismiss,
        }
    }

    fn first(state: &Self) -> Moment {
        match state {
            Pane::Hidden => Moment::Rest,
            Pane::Showing(_) | Pane::Failed => Moment::Appear,
            Pane::Pending => Moment::Pending,
        }
    }
}

static PANE: GlobalSignal<Pane> = Signal::global(|| Pane::Hidden);
static TOUCH: GlobalSignal<Touch> = Signal::global(|| Touch::Remote);
static MOTION: GlobalSignal<Motion> = Signal::global(|| Motion::Standard);

fn content(pane: Pane) -> PaneContent {
    match pane {
        Pane::Showing(Kind::Emoji) => PaneContent::Emoji {
            glyph: "🙂".to_string(),
            name: "slightly smiling face".to_string(),
        },
        Pane::Hidden | Pane::Showing(Kind::Web) | Pane::Pending | Pane::Failed => {
            PaneContent::Web {
                host: "duckduckgo.com".to_string(),
                url: "https://duckduckgo.com".to_string(),
            }
        }
    }
}

#[allow(non_snake_case)]
fn Launcher() -> Element {
    let pane = PANE();
    let detail = use_detail(pane, FirstShow::Animate, TOUCH());
    let operation = use_operation(detail.cue());
    let shown = match pane {
        Pane::Hidden => Shown::Hidden,
        Pane::Showing(_) | Pane::Pending | Pane::Failed => Shown::Visible,
    };
    rsx! {
        Ds { appearance: Appearance { motion: MOTION(), ..Appearance::default() }, material: Material::Sheet,
            button {
                class: "look",
                onclick: move |event| {
                    *TOUCH.write() = Touch::from_event(&event);
                    *PANE.write() = Pane::Showing(Kind::Web);
                },
                "Quick Look"
            }
            div { style: "width:360px;height:360px;display:flex",
                PreviewPane { content: content(pane), shown, cue: detail.cue(), operation }
            }
        }
    }
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A change nobody touched.
fn remote(harness: &mut Harness, pane: Pane) {
    harness.within(|| {
        *TOUCH.write() = Touch::Remote;
        *PANE.write() = pane;
    });
    harness.advance(ms(20));
}

fn attr(harness: &Harness, selector: &str, name: &str) -> String {
    harness.attr(selector, name).unwrap_or_default()
}

fn fading(harness: &Harness) -> bool {
    attr(harness, ".ds-preview-media", "class").contains("a-morph-fade-in")
}

fn shown_remotely(harness: &mut Harness) {
    remote(harness, Pane::Showing(Kind::Web));
    settle_until(harness, |h| {
        attr(h, ".ds-preview", "data-presence") == "present"
    });
    assert_settles_to_zero_frames(harness);
}

#[test]
fn a_remote_showing_eases_in_and_a_contact_springs() {
    let mut harness = Harness::new(Launcher, VIEW);
    remote(&mut harness, Pane::Showing(Kind::Web));
    assert_eq!(attr(&harness, ".ds-preview", "data-presence"), "entering");
    assert_eq!(attr(&harness, ".ds-preview", "data-touch"), "remote");
    assert!(
        !fading(&harness),
        "an Appear plays the entrance, not the fade"
    );
    assert_settles_to_zero_frames(&mut harness);
    remote(&mut harness, Pane::Hidden);
    settle_until(&mut harness, |h| {
        attr(h, ".ds-preview", "data-shown") == "hidden"
    });
    assert_settles_to_zero_frames(&mut harness);
    let at = harness.centre(".look").expect("the button");
    harness.click(at);
    harness.advance(ms(20));
    assert_eq!(attr(&harness, ".ds-preview", "data-presence"), "entering");
    assert_eq!(attr(&harness, ".ds-preview", "data-touch"), "contact");
    assert_settles_to_zero_frames(&mut harness);
    // Present keeps the entrance declared as it played: a later remote change does
    // not swap its easing, which would restart or drop it.
    remote(&mut harness, Pane::Showing(Kind::Emoji));
    assert_eq!(attr(&harness, ".ds-preview", "data-presence"), "present");
    assert_eq!(attr(&harness, ".ds-preview", "data-touch"), "contact");
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn another_kind_cross_fades_the_media_and_never_replays_the_entrance() {
    let mut harness = Harness::new(Launcher, VIEW);
    shown_remotely(&mut harness);
    let alias = attr(&harness, ".ds-preview", "data-pulse");
    remote(&mut harness, Pane::Showing(Kind::Emoji));
    assert_eq!(harness.count(".ds-preview-emoji"), 1);
    assert!(fading(&harness), "a Preview fades the media in");
    assert_eq!(attr(&harness, ".ds-preview", "data-presence"), "present");
    assert_eq!(attr(&harness, ".ds-preview", "data-pulse"), alias);
    settle_until(&mut harness, |h| !fading(h));
    assert_settles_to_zero_frames(&mut harness);
    // The same kind again is Rest: nothing plays.
    remote(&mut harness, Pane::Showing(Kind::Emoji));
    assert!(!fading(&harness));
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn a_slow_load_shows_the_pending_look_and_its_landing_cross_fades() {
    let mut harness = Harness::with_config(
        Launcher,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    shown_remotely(&mut harness);
    let asked = harness.now();
    remote(&mut harness, Pane::Pending);
    assert_eq!(attr(&harness, ".ds-preview", "aria-busy"), "true");
    if harness.now().duration_since(asked) < ms(200) {
        assert_eq!(
            harness.count(".ds-preview-pending"),
            0,
            "nothing inside the grace"
        );
    }
    settle_until(&mut harness, |h| h.count(".ds-preview-pending") == 1);
    assert!(
        harness.now().duration_since(asked) >= ms(400),
        "not before PendingGrace"
    );
    assert_eq!(
        harness.text_of(".ds-preview-pending-words").as_deref(),
        Some("Loading\u{2026}")
    );
    assert_eq!(
        attr(&harness, ".ds-preview-pending .ds-spinner", "data-pending"),
        "step"
    );
    assert_eq!(attr(&harness, ".ds-preview", "data-presence"), "present");
    remote(&mut harness, Pane::Showing(Kind::Web));
    assert_eq!(harness.count(".ds-preview-pending"), 0);
    assert_eq!(harness.attr(".ds-preview", "aria-busy"), None);
    assert!(
        fading(&harness),
        "a load landing is a Change: the media fades in"
    );
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn a_failed_load_cross_fades_to_its_still_words() {
    let mut harness = Harness::new(Launcher, VIEW);
    shown_remotely(&mut harness);
    remote(&mut harness, Pane::Pending);
    settle_until(&mut harness, |h| h.count(".ds-preview-pending") == 1);
    remote(&mut harness, Pane::Failed);
    assert_eq!(harness.count(".ds-preview-pending"), 0);
    assert!(fading(&harness), "a Failure fades, it does not shake");
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn a_stuck_load_holds_its_still_frame_from_the_cap() {
    // On the virtual clock: the cap is 10 s, and a wall-clock bound of 10.5 s raced it.
    let mut harness = Harness::with_config(
        Launcher,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    shown_remotely(&mut harness);
    remote(&mut harness, Pane::Pending);
    let asked = harness.now();
    let ring = ".ds-preview-pending .ds-spinner";
    settle_until(&mut harness, |h| attr(h, ring, "data-pending") == "step");
    while harness.now().duration_since(asked) < Duration::from_millis(10_500)
        && attr(&harness, ring, "data-pending") != "still"
    {
        harness.advance(ms(250));
    }
    assert_eq!(attr(&harness, ring, "data-pending"), "still");
    assert!(
        harness.now().duration_since(asked) >= Duration::from_secs(10),
        "held before the cap"
    );
    assert_eq!(
        harness.text_of(".ds-preview-pending-words").as_deref(),
        Some("Loading\u{2026}")
    );
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn reduced_snaps_and_never_loops() {
    let mut harness = Harness::new(Launcher, VIEW);
    harness.within(|| *MOTION.write() = Motion::Reduced);
    shown_remotely(&mut harness);
    remote(&mut harness, Pane::Showing(Kind::Emoji));
    assert!(!fading(&harness), "Reduced swaps the media at once");
    assert_eq!(harness.count(".ds-preview-emoji"), 1);
    assert_settles_to_zero_frames(&mut harness);
    remote(&mut harness, Pane::Pending);
    settle_until(&mut harness, |h| h.count(".ds-preview-pending") == 1);
    assert_eq!(
        attr(&harness, ".ds-preview-pending .ds-spinner", "data-pending"),
        "still"
    );
    assert_settles_to_zero_frames(&mut harness);
}
