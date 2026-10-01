//! `InlineBanner` presence on a real Blitz document, on the virtual clock (design/30 sections
//! 1.3 and 2.9): hidden, it fades and its height closes over `--t-move` so the content under it
//! slides up, and `on_hidden` runs once, at `settle(OsdOut)` and not before; shown again, it
//! fades in while its height opens and the content slides down; a show while it closes takes the
//! hide back. Under Reduced the height snaps (the content moves once, not in between) and the
//! banner cross-fades, still leaving only at its settle.

use dioxus::prelude::*;
use ds::components::overlays::inline_banner::InlineBanner;
use ds::prelude::*;
use ds_harness::harness::settle_until;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 320,
    scale_percent: 100,
};

static SHOWN: GlobalSignal<Shown> = Signal::global(|| Shown::Visible);
static HIDDEN: GlobalSignal<u32> = Signal::global(|| 0);

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn stage(motion: Motion) -> Element {
    rsx! {
        Ds { appearance: Appearance { motion, ..Appearance::default() }, material: Material::Window,
            div { style: "width:480px",
                InlineBanner {
                    text: "Remote images are blocked.",
                    detail: "Loading them tells the sender you opened this.",
                    shown: SHOWN(),
                    on_hidden: move |()| *HIDDEN.write() += 1,
                }
                p { class: "body", "The message" }
            }
        }
    }
}

#[allow(non_snake_case)]
fn Standard() -> Element {
    stage(Motion::Standard)
}

#[allow(non_snake_case)]
fn Reduced() -> Element {
    stage(Motion::Reduced)
}

fn virtual_harness(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(300));
    harness
}

fn presence(harness: &Harness) -> Option<String> {
    harness.attr(".ds-inline-banner", "data-presence")
}

fn body_top(harness: &Harness) -> f32 {
    harness
        .rect(".body")
        .map_or(f32::NAN, |rect| rect.origin.y.0)
}

fn set(harness: &mut Harness, shown: Shown) {
    harness.within(|| *SHOWN.write() = shown);
    harness.advance(ms(1));
}

fn hidden(harness: &mut Harness) -> u32 {
    harness.within(|| *HIDDEN.peek())
}

#[test]
fn hidden_it_closes_over_t_move_and_on_hidden_runs_once_at_settle() {
    let mut harness = virtual_harness(Standard);
    assert_eq!(presence(&harness).as_deref(), Some("present"));
    let open_top = body_top(&harness);
    assert!(
        open_top > 20.0,
        "the message stands under the banner: {open_top}"
    );
    let out = settle(Anim::OsdOut, MotionLevel::Standard);
    let hiding = harness.now();
    set(&mut harness, Shown::Hidden);
    assert_eq!(presence(&harness).as_deref(), Some("leaving"));
    assert_eq!(
        harness.attr(".ds-inline-banner", "aria-hidden").as_deref(),
        Some("true"),
        "a banner on its way out is not announced"
    );

    harness.advance(out / 2);
    let middle = body_top(&harness);
    assert!(
        middle > 0.0 && middle < open_top - 2.0,
        "the content is sliding up: {middle} of {open_top}"
    );
    assert_eq!(
        hidden(&mut harness),
        0,
        "not before settle(OsdOut) = {out:?}"
    );
    assert_eq!(presence(&harness).as_deref(), Some("leaving"));

    let gone = settle_until(&mut harness, |h| presence(h).is_none());
    assert!(
        gone.duration_since(hiding) >= out,
        "{:?}",
        gone.duration_since(hiding)
    );
    assert_eq!(hidden(&mut harness), 1, "on_hidden ran once");
    assert!(
        body_top(&harness) < 2.0,
        "the message took its place: {}",
        body_top(&harness)
    );
    harness.advance(ms(500));
    assert_eq!(hidden(&mut harness), 1, "and only once");
}

#[test]
fn shown_again_it_fades_in_while_the_content_slides_down() {
    let mut harness = virtual_harness(Standard);
    let open_top = body_top(&harness);
    set(&mut harness, Shown::Hidden);
    harness.advance(settle(Anim::OsdOut, MotionLevel::Standard) + ms(60));
    assert_eq!(harness.count(".ds-inline-banner"), 0);
    set(&mut harness, Shown::Visible);
    assert_eq!(presence(&harness).as_deref(), Some("entering"));
    harness.advance(ms(80));
    let middle = body_top(&harness);
    assert!(
        middle > 0.0 && middle < open_top - 2.0,
        "the content is sliding down: {middle} of {open_top}"
    );
    harness.advance(settle(Anim::OsdOut, MotionLevel::Standard) + ms(100));
    assert_eq!(presence(&harness).as_deref(), Some("present"));
    assert!(
        (body_top(&harness) - open_top).abs() < 1.0,
        "back where it began"
    );
    assert_eq!(
        harness.attr(".ds-inline-banner-frame", "style"),
        None,
        "at rest the banner takes its natural height"
    );
}

#[test]
fn a_show_while_it_closes_takes_the_hide_back() {
    let mut harness = virtual_harness(Standard);
    set(&mut harness, Shown::Hidden);
    harness.advance(ms(100));
    set(&mut harness, Shown::Visible);
    assert_eq!(presence(&harness).as_deref(), Some("present"));
    harness.advance(ms(600));
    assert_eq!(hidden(&mut harness), 0, "the hide was taken back");
    assert_eq!(presence(&harness).as_deref(), Some("present"));
}

#[test]
fn under_reduced_the_height_snaps_and_the_banner_cross_fades() {
    let mut harness = virtual_harness(Reduced);
    let open_top = body_top(&harness);
    let out = settle(Anim::OsdOut, MotionLevel::Reduced);
    set(&mut harness, Shown::Hidden);
    harness.advance(out / 2);
    assert_eq!(presence(&harness).as_deref(), Some("leaving"));
    assert!(
        (body_top(&harness) - open_top).abs() < 1.0,
        "the content does not slide: it holds its place while the banner fades"
    );
    assert_eq!(hidden(&mut harness), 0);
    harness.advance(out);
    assert_eq!(presence(&harness), None);
    assert_eq!(hidden(&mut harness), 1);
    assert!(
        body_top(&harness) < 2.0,
        "the content moved once, at the end"
    );
    set(&mut harness, Shown::Visible);
    harness.advance(ms(20));
    assert!(
        (body_top(&harness) - open_top).abs() < 1.0,
        "it opens at once"
    );
}
