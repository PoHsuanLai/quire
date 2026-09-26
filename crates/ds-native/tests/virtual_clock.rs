//! sill Q380: a harness on `Clock::Virtual` runs every ds timer (presence, hover intent, toast
//! hold) on the same clock its CSS resolves at, and `advance` steps that one clock. The same
//! scenario then produces the same frames, rects and state whether the machine is idle or
//! saturated, and an entrance timer can never end before the frame clock has played the
//! entrance (the G295 shape: a stalled first frame).

use dioxus::prelude::*;
use ds::{
    Anim, Appearance, Button, ButtonVariant, Ds, HoverCard, HoverKey, HoverKind, HoverTarget,
    Material, MotionLevel, Panel, Point, Px, Rect, RootExtent, Shown, StaggerIndex, settle,
    use_hover_hub, use_toasts,
};
use ds_native::harness::settle_until;
use ds_native::{Clock, Harness, HarnessConfig, Viewport};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::mpsc::{self, TryRecvError};
use std::thread::JoinHandle;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 900,
    height: 500,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn standard(anim: Anim) -> Duration {
    settle(anim, MotionLevel::Standard, StaggerIndex::default())
}

fn virtual_harness(app: fn() -> Element) -> Harness {
    Harness::with_config(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

// ---- The scenario: a panel entering, a hover card, a toast ------------------------------

#[allow(non_snake_case)]
fn Scene() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            SceneContent {}
        }
    }
}

#[allow(non_snake_case)]
fn SceneContent() -> Element {
    let hub = use_hover_hub();
    let toasts = use_toasts();
    rsx! {
        div { style: "position: absolute; left: 20px; top: 20px; width: 300px",
            HoverTarget { hover_key: HoverKey("sender:3".into()), kind: HoverKind::Sender,
                span { "Dana Okafor" }
            }
            Button {
                variant: ButtonVariant::Secondary,
                label: "Archive",
                onclick: move |_| toasts.push("Archived".into(), None),
            }
        }
        if let Some((_, kind)) = hub.open() {
            HoverCard { kind, p { "dana@example.com" } }
        }
        Panel { label: "Notification Center", shown: Shown::Visible, width: Px(384.0),
            p { "Nothing new." }
        }
    }
}

/// What one step of the scenario shows.
#[derive(Debug, Clone, PartialEq)]
struct Sample {
    at: Duration,
    panel: Option<String>,
    panel_rect: Option<Rect>,
    hover_cards: usize,
    toast: Option<String>,
    toast_rect: Option<Rect>,
    animating: Animating,
    /// A hash of the painted frame, taken at the steps [`paints_at`] names.
    frame: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Animating {
    Yes,
    No,
}

fn sample(harness: &mut Harness, at: Duration, paint: Paint) -> Sample {
    let frame = match paint {
        Paint::Frame => {
            let image = harness.render().expect("paint");
            let mut hasher = DefaultHasher::new();
            image.as_raw().hash(&mut hasher);
            Some(hasher.finish())
        }
        Paint::Skip => None,
    };
    Sample {
        at,
        panel: harness.attr(".ds-panel", "data-presence"),
        panel_rect: harness.rect(".ds-panel"),
        hover_cards: harness.count(".ds-hovercard"),
        toast: harness.attr(".ds-toast", "data-shown"),
        toast_rect: harness.rect(".ds-toast"),
        animating: if harness.is_animating() {
            Animating::Yes
        } else {
            Animating::No
        },
        frame,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Paint {
    Frame,
    Skip,
}

/// Paint mid-entrance, mid-hover-card, as the toast rises, and at rest.
fn paints_at(at: Duration) -> Paint {
    match u64::try_from(at.as_millis()).unwrap_or(u64::MAX) {
        60 | 120 | 700 | 900 | 1200 | 6600 => Paint::Frame,
        _ => Paint::Skip,
    }
}

/// Run the scenario in 20 ms steps (with the pointer resting on the hover target from 200 ms
/// and the toast pushed at 800 ms) and record every step.
fn run_scenario() -> Vec<Sample> {
    let mut harness = virtual_harness(Scene);
    assert_eq!(harness.clock(), Clock::Virtual);
    let step = ms(20);
    let mut at = Duration::ZERO;
    let mut samples = vec![sample(&mut harness, at, paints_at(at))];
    while at < ms(6600) {
        if at == ms(200) {
            let target = harness.centre(".ds-hover-target").expect("hover target");
            harness.pointer_move(target);
        }
        if at == ms(800) {
            let button = harness.centre(".ds-button").expect("button");
            harness.click(button);
        }
        harness.advance(step);
        at += step;
        samples.push(sample(&mut harness, at, paints_at(at)));
    }
    samples
}

/// Every core kept busy until this drops: spinning threads that stop when their channel closes.
struct Load {
    stops: Vec<mpsc::Sender<()>>,
    spinners: Vec<JoinHandle<u64>>,
}

impl Load {
    fn start() -> Self {
        let cores = std::thread::available_parallelism().map_or(4, usize::from);
        let (stops, spinners) = (0..cores * 2).map(|_| spin()).unzip();
        Load { stops, spinners }
    }
}

impl Drop for Load {
    fn drop(&mut self) {
        self.stops.clear();
        for spinner in self.spinners.drain(..) {
            let _ = spinner.join();
        }
    }
}

fn spin() -> (mpsc::Sender<()>, JoinHandle<u64>) {
    let (stop, stopped) = mpsc::channel::<()>();
    let spinner = std::thread::spawn(move || {
        let mut n = 0_u64;
        while let Err(TryRecvError::Empty) = stopped.try_recv() {
            n = n.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        }
        n
    });
    (stop, spinner)
}

#[test]
fn the_same_scenario_gives_the_same_frames_idle_or_under_load() {
    let idle = run_scenario();
    let load = Load::start();
    let loaded = run_scenario();
    let again = run_scenario();
    drop(load);

    assert_eq!(idle.len(), loaded.len());
    for (quiet, busy) in idle.iter().zip(&loaded) {
        assert_eq!(quiet, busy, "step at {:?} differs under load", quiet.at);
    }
    assert_eq!(loaded, again, "two loaded runs differ");

    // And the timings are the tokens', exactly: the panel is present at the first step on or
    // after settle(PanelIn); the hover card at the first step 450 ms after the pointer rested;
    // the toast hidden at the first step 5200 ms after the push.
    let first = |pick: &dyn Fn(&Sample) -> bool| {
        idle.iter()
            .find(|s| pick(s))
            .map(|s| s.at)
            .expect("the state is reached")
    };
    let present = first(&|s| s.panel.as_deref() == Some("present"));
    let entrance = standard(Anim::PanelIn);
    assert!(
        present >= entrance && present < entrance + ms(20),
        "{present:?}"
    );
    let opened = first(&|s| s.hover_cards == 1);
    assert!(opened >= ms(650) && opened < ms(670), "{opened:?}");
    // (It mounts hidden for a frame so its spring rises from below the edge.)
    let hidden = first(&|s| s.at > ms(1000) && s.toast.as_deref() == Some("hidden"));
    assert!(hidden >= ms(6000) && hidden < ms(6020), "{hidden:?}");
}

// ---- G295's shape: an entrance timer against a stalled first frame -------------------------

#[allow(non_snake_case)]
fn Center() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover, extent: RootExtent::Viewport,
            Panel { label: "Notification Center", shown: Shown::Visible, width: Px(384.0),
                p { "Nothing new." }
            }
        }
    }
}

fn inside(rest: Rect) -> Point {
    Point {
        x: Px(rest.origin.x.0 + 20.0),
        y: Px(rest.origin.y.0 + rest.size.height.0 / 2.0),
    }
}

/// On the wall clock a stall longer than the entrance lets the settle timer turn the panel
/// present one millisecond into its slide (sill G130). On the virtual clock the stall is
/// invisible: the timer fires only when `advance` reaches settle(PanelIn), after the slide has
/// played, and the panel is where it rests, every run.
#[test]
fn a_stalled_first_frame_cannot_end_the_entrance_early() {
    let entrance = standard(Anim::PanelIn);
    let runs: Vec<Vec<Sample>> = (0..2)
        .map(|_| {
            let mut h = virtual_harness(Center);
            assert_eq!(
                h.attr(".ds-panel", "data-presence").as_deref(),
                Some("entering")
            );
            std::thread::sleep(entrance + ms(50));
            h.advance(ms(1));
            assert_eq!(
                h.attr(".ds-panel", "data-presence").as_deref(),
                Some("entering"),
                "the wall-clock stall did not move the timer"
            );
            let mut trace = vec![sample(&mut h, ms(1), Paint::Frame)];
            h.advance(entrance - ms(2));
            assert_eq!(
                h.attr(".ds-panel", "data-presence").as_deref(),
                Some("entering"),
                "not a millisecond before settle(PanelIn)"
            );
            trace.push(sample(&mut h, entrance - ms(1), Paint::Frame));
            h.advance(ms(1));
            assert_eq!(
                h.attr(".ds-panel", "data-presence").as_deref(),
                Some("present"),
                "present exactly at settle(PanelIn) = {entrance:?}"
            );
            trace.push(sample(&mut h, entrance, Paint::Frame));
            let rested = settle_until(&mut h, |h| !h.is_animating());
            assert!(rested >= h.now() - ms(1));
            let rest = h.rect(".ds-panel").expect("the panel");
            assert!(h.hits(inside(rest), ".ds-panel"), "hit where it rests");
            trace.push(sample(&mut h, entrance, Paint::Frame));
            trace
        })
        .collect();
    assert_eq!(runs[0], runs[1]);
}

/// `settle_until` on the virtual clock measures virtual time: its instants are the harness's,
/// so an exact token shows up exactly.
#[test]
fn settle_until_reports_virtual_instants() {
    let mut h = virtual_harness(Scene);
    let rested = h.now();
    let target = h.centre(".ds-hover-target").expect("hover target");
    h.pointer_move(target);
    let opened = settle_until(&mut h, |h| h.count(".ds-hovercard") == 1);
    assert_eq!(opened.duration_since(rested), ms(450));
}
