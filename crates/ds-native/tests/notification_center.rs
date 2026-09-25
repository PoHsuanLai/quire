//! The notification center's edge panel on a real Blitz document (sill Q123): shown, it slides in
//! (`data-presence="entering"`) and comes to rest at the right edge, `width` wide; hidden, it
//! slides out and `on_hidden` runs at `settle(PanelOut)`, not before, after which nothing is laid
//! out; shown again while it leaves, the hide is taken back.

use dioxus::prelude::*;
use ds::{
    Anim, Appearance, Ds, Material, MotionLevel, Panel, Px, RootExtent, Shown, StaggerIndex, settle,
};
use ds_native::harness::settle_until;
use ds_native::{Harness, Viewport};
use std::time::{Duration, Instant};

static SHOWN: GlobalSignal<Shown> = Signal::global(|| Shown::Visible);
static HIDDEN: GlobalSignal<u32> = Signal::global(|| 0);

const VIEW: Viewport = Viewport {
    width: 800,
    height: 500,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Center() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover, extent: RootExtent::Viewport,
            Panel { label: "Notification Center", shown: SHOWN(), width: Px(384.0), on_hidden: move |()| *HIDDEN.write() += 1,
                p { "Nothing new." }
            }
        }
    }
}

fn presence(harness: &Harness) -> Option<String> {
    harness.attr(".ds-panel", "data-presence")
}

fn show(harness: &mut Harness, shown: Shown) {
    harness.within(|| *SHOWN.write() = shown);
    harness.advance(Duration::from_millis(1));
}

fn hidden(harness: &mut Harness) -> u32 {
    harness.within(|| *HIDDEN.peek())
}

#[test]
fn shown_it_comes_to_rest_at_the_right_edge() {
    let mut harness = Harness::new(Center, VIEW);
    assert_eq!(presence(&harness).as_deref(), Some("entering"));
    settle_until(&mut harness, |h| presence(h).as_deref() == Some("present"));
    let panel = harness.rect(".ds-panel").expect("the panel is laid out");
    assert!((panel.size.width.0 - 384.0).abs() < 0.5, "{panel:?}");
    assert!(
        (VIEW.width as f32 - panel.right().0 - 8.0).abs() < 0.5,
        "8 in from the right edge: {panel:?}"
    );
    assert!(
        (panel.size.height.0 - (VIEW.height as f32 - 16.0)).abs() < 0.5,
        "full height, 8 in top and bottom: {panel:?}"
    );
}

#[test]
fn hidden_it_slides_out_and_on_hidden_runs_at_settle_and_not_before() {
    let mut harness = Harness::new(Center, VIEW);
    settle_until(&mut harness, |h| presence(h).as_deref() == Some("present"));
    let out = settle(
        Anim::PanelOut,
        MotionLevel::Standard,
        StaggerIndex::default(),
    );
    let hiding = Instant::now();
    show(&mut harness, Shown::Hidden);
    assert_eq!(presence(&harness).as_deref(), Some("leaving"));
    harness.advance(out / 2);
    assert_eq!(hidden(&mut harness), 0, "not well before settle(PanelOut)");
    let gone = settle_until(&mut harness, |h| presence(h).is_none());
    assert!(
        gone.duration_since(hiding) >= out,
        "{:?}",
        gone.duration_since(hiding)
    );
    assert_eq!(hidden(&mut harness), 1);
    assert_eq!(
        harness.attr(".ds-panel-stage", "data-shown").as_deref(),
        Some("hidden")
    );
    // Shown again: it enters from its first frame.
    show(&mut harness, Shown::Visible);
    assert_eq!(presence(&harness).as_deref(), Some("entering"));
}

#[test]
fn a_show_while_it_leaves_takes_the_hide_back() {
    let mut harness = Harness::new(Center, VIEW);
    settle_until(&mut harness, |h| presence(h).as_deref() == Some("present"));
    show(&mut harness, Shown::Hidden);
    harness.advance(Duration::from_millis(60));
    show(&mut harness, Shown::Visible);
    assert_eq!(presence(&harness).as_deref(), Some("present"));
    harness.advance(Duration::from_millis(500));
    assert_eq!(hidden(&mut harness), 0);
}
