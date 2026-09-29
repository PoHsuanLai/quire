//! Blitz at the pinned rev keeps a CSS animation's last value on an
//! element whose `animation-name` goes away before the animation ends, in paint and in the hit
//! test, while `Harness::rect` (layout, no transform) reads the resting box. The raw case is
//! sill's repro, ported; it passes only with Blitz patched to restyle such an element. The
//! quire cases are the defence that holds on the unpatched pin: a `Panel` whose entrance timer
//! (wall clock) settles while the frame clock has barely started the slide, and a hide taken back
//! halfway through `panel-out`; neither may leave the panel off its resting box.

use dioxus::prelude::*;
use ds::{
    Anim, Appearance, Ds, Material, MotionLevel, Panel, Point, Px, RootExtent, Shown, settle,
};
use ds_native::harness::settle_until;
use ds_native::{Harness, Viewport};
use std::time::Duration;

const CSS: &str = "@keyframes slide{ from{ transform:translateX(400px); } to{ transform:none; } }
.box{ position:absolute; left:0; top:0; width:100px; height:100px; background:red; }
.box.on{ animation:slide 1000ms linear; }
.btn{ position:absolute; left:600px; top:0; width:50px; height:50px; }";

/// A box sliding in, and a button that takes its animation away.
fn sliding_box() -> Element {
    let mut on = use_signal(|| true);
    rsx! {
        style { {CSS} }
        div { class: if on() { "box on" } else { "box" } }
        div { class: "btn", onpointerdown: move |_| on.set(false) }
    }
}

const BOX_VIEW: Viewport = Viewport {
    width: 800,
    height: 200,
    scale_percent: 100,
};

fn at(x: f32, y: f32) -> Point {
    Point { x: Px(x), y: Px(y) }
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// Take the box's animation away `after` into it, then let two seconds pass.
fn cancelled(after: u64) -> Harness {
    let mut h = Harness::new(sliding_box, BOX_VIEW);
    h.advance(ms(after));
    h.click(at(620.0, 20.0));
    h.advance(ms(2000));
    assert_eq!(h.attr(".box", "class").as_deref(), Some("box"));
    assert!(!h.is_animating());
    h
}

#[test]
fn an_animation_removed_after_its_end_leaves_nothing() {
    let h = cancelled(1100);
    assert!(h.hits(at(50.0, 50.0), ".box"));
}

#[test]
fn an_animation_removed_midway_leaves_nothing() {
    let mut h = cancelled(20);
    let rect = h.rect(".box").expect("the box");
    assert!(rect.origin.x.0 < 10.0, "layout says at rest: {rect:?}");
    let pixel = h.render().expect("paint").get_pixel(50, 50).0;
    assert_eq!(pixel, [255, 0, 0, 255], "painted where it rests");
    assert!(h.hits(at(50.0, 50.0), ".box"), "hit where it rests");
}

static SHOWN: GlobalSignal<Shown> = Signal::global(|| Shown::Visible);

#[allow(non_snake_case)]
fn Center() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover, extent: RootExtent::Viewport,
            Panel { label: "Notification Center", shown: SHOWN(), width: Px(384.0),
                p { "Nothing new." }
            }
        }
    }
}

const PANEL_VIEW: Viewport = Viewport {
    width: 800,
    height: 500,
    scale_percent: 100,
};

fn presence(h: &Harness) -> Option<String> {
    h.attr(".ds-panel", "data-presence")
}

/// A point 20 px inside the panel's resting box (layout ignores transforms), at mid-height.
fn inside_resting_panel(h: &Harness) -> Point {
    let rest = h.rect(".ds-panel").expect("the panel is laid out");
    at(
        rest.origin.x.0 + 20.0,
        rest.origin.y.0 + rest.size.height.0 / 2.0,
    )
}

fn standard(anim: Anim) -> Duration {
    settle(anim, MotionLevel::Standard)
}

/// The entrance's timer settles on the wall clock while the frame clock has not moved (a first
/// frame that stalls): the panel turns present about a millisecond into its slide.
#[test]
fn a_panel_present_before_its_slide_has_played_still_comes_to_rest() {
    // Kept on Wall: this reproduces the split clock on purpose (a real futures-timer
    // settling while the harness's frame clock has barely advanced past the first frame),
    // which Virtual's unified clock cannot produce (its `advance` only fires a timer once the
    // frame clock reaches it, so this same scenario is proven inert there in
    // virtual_clock.rs::a_stalled_first_frame_cannot_end_the_entrance_early).
    let mut h = Harness::new(Center, PANEL_VIEW);
    assert_eq!(presence(&h).as_deref(), Some("entering"));
    std::thread::sleep(standard(Anim::PanelIn) + ms(50));
    h.advance(ms(1));
    assert_eq!(
        presence(&h).as_deref(),
        Some("present"),
        "the timer settled"
    );
    h.advance(ms(600));
    assert!(!h.is_animating());
    let point = inside_resting_panel(&h);
    assert!(
        h.hits(point, ".ds-panel"),
        "the panel takes a press where it rests"
    );
}

/// Shown again halfway through its slide out: since H1 (design/05 section 14) the slide out is
/// a spring, so the panel turns back from where it is (entering), comes to rest in place, and
/// nothing is left of the exit.
#[test]
fn a_hide_taken_back_midway_leaves_the_panel_at_rest() {
    let mut h = Harness::new(Center, PANEL_VIEW);
    settle_until(&mut h, |h| presence(h).as_deref() == Some("present"));
    h.within(|| *SHOWN.write() = Shown::Hidden);
    h.advance(standard(Anim::PanelOut) / 2);
    assert_eq!(presence(&h).as_deref(), Some("leaving"));
    h.within(|| *SHOWN.write() = Shown::Visible);
    h.advance(ms(1));
    assert_eq!(
        presence(&h).as_deref(),
        Some("entering"),
        "taken back at once: it turns round from where it is"
    );
    assert_eq!(h.attr(".ds-panel", "data-drive").as_deref(), Some("spring"));
    settle_until(&mut h, |h| presence(h).as_deref() == Some("present"));
    assert!(!h.is_animating());
    let point = inside_resting_panel(&h);
    assert!(
        h.hits(point, ".ds-panel"),
        "the panel takes a press where it rests"
    );
}
