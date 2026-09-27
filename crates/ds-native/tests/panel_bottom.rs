//! Edit Widgets' bottom sheet on a real Blitz document (sill Q521): `PanelEdge::Bottom` stands
//! centred `--s-8` above the root's bottom, as wide as asked, never taller than half the root, so
//! the desktop's top rows, where the widgets gather and a new one lands (top right, column by
//! column), stay uncovered. It arrives as a sheet does and comes to rest at zero frames; hidden,
//! it leaves on the sheet's spring and `on_hidden` runs once it rests.

use dioxus::prelude::*;
use ds::widget::{DesktopGrid, WidgetEdit, WidgetLayout, apply};
use ds::{
    Appearance, BatteryWidget, Ds, Material, Panel, PanelEdge, Px, RootExtent, Shown, Timeline,
    Widget, WidgetCard, WidgetGallery, WidgetMetrics, WidgetSize, WorldClockWidget,
};
use ds_native::harness::{assert_settles_to_zero_frames, settle_until};
use ds_native::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 1280,
    height: 800,
    scale_percent: 100,
};

static SHOWN: GlobalSignal<Shown> = Signal::global(|| Shown::Visible);
static HIDDEN: GlobalSignal<u32> = Signal::global(|| 0);

const GRID: DesktopGrid = DesktopGrid {
    columns: 6,
    rows: 4,
};

/// The desktop: two widgets at the top right, as the layer places them, and the sheet.
#[allow(non_snake_case)]
fn Desktop() -> Element {
    let mut layout = use_signal(WidgetLayout::default);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet, extent: RootExtent::Viewport,
            div { class: "desk", style: "position:absolute;top:8px;right:8px;display:flex;gap:16px;{WidgetMetrics::default().style_attr()}",
                WidgetCard { widget: WorldClockWidget, timeline: Timeline::now(WorldClockWidget::preview(WidgetSize::Medium)), size: WidgetSize::Medium }
                WidgetCard { widget: BatteryWidget, timeline: Timeline::now(BatteryWidget::preview(WidgetSize::Small)), size: WidgetSize::Small }
            }
            Panel { label: "Edit Widgets", shown: SHOWN(), edge: PanelEdge::Bottom, width: Px(1040.0), height: Px(440.0), material: Material::Sheet,
                on_hidden: move |()| *HIDDEN.write() += 1,
                div { style: "display:flex;flex-direction:column;flex:1;min-height:0;{WidgetMetrics::default().style_attr()}",
                    WidgetGallery {
                        layout: layout(),
                        onedit: move |edit: WidgetEdit| {
                            if let Ok(next) = apply(layout(), edit, GRID) {
                                layout.set(next);
                            }
                        },
                    }
                }
            }
        }
    }
}

fn presence(harness: &Harness) -> Option<String> {
    harness.attr(".ds-panel", "data-presence")
}

#[test]
fn the_sheet_rests_at_the_bottom_and_leaves_the_desktops_top_uncovered() {
    let mut harness =
        Harness::with_config(Desktop, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    assert_eq!(
        harness.attr(".ds-panel-stage", "data-edge").as_deref(),
        Some("bottom")
    );
    assert_eq!(presence(&harness).as_deref(), Some("entering"));
    settle_until(&mut harness, |h| presence(h).as_deref() == Some("present"));
    assert_settles_to_zero_frames(&mut harness);
    let panel = harness.rect(".ds-panel").expect("the sheet is laid out");
    let (width, height) = (VIEW.width as f32, VIEW.height as f32);
    assert!(
        (panel.size.width.0 - 1040.0).abs() < 0.5,
        "as wide as asked: {panel:?}"
    );
    assert!(
        (panel.origin.x.0 - (width - 1040.0) / 2.0).abs() < 0.5,
        "centred: {panel:?}"
    );
    let bottom = panel.origin.y.0 + panel.size.height.0;
    assert!(
        (height - bottom - 8.0).abs() < 0.5,
        "8 above the bottom: {panel:?}"
    );
    assert!(
        (panel.size.height.0 - (height / 2.0 - 8.0)).abs() < 0.5,
        "440 asked, held to half the root less 8: {panel:?}"
    );
    assert!(panel.origin.y.0 >= height / 2.0 - 0.5, "{panel:?}");
    let desk = harness.rect(".desk").expect("the desktop's widgets");
    assert!(
        desk.origin.y.0 + desk.size.height.0 < panel.origin.y.0,
        "the top row of widgets is uncovered: {desk:?} {panel:?}"
    );
    let two_rows = 8.0 + 2.0 * 164.0 + 16.0;
    assert!(
        two_rows <= panel.origin.y.0,
        "two rows of cells at the top stay in view: {panel:?}"
    );
    assert_eq!(harness.count(".ds-panel .ds-widget-gallery"), 1);
}

#[test]
fn hidden_the_sheet_leaves_and_reports_once_it_rests() {
    let mut harness =
        Harness::with_config(Desktop, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    settle_until(&mut harness, |h| presence(h).as_deref() == Some("present"));
    harness.within(|| *SHOWN.write() = Shown::Hidden);
    harness.advance(Duration::from_millis(1));
    assert_eq!(presence(&harness).as_deref(), Some("leaving"));
    assert_eq!(
        harness.attr(".ds-panel", "data-drive").as_deref(),
        Some("spring")
    );
    assert_eq!(harness.within(|| *HIDDEN.peek()), 0);
    settle_until(&mut harness, |h| presence(h).is_none());
    assert_eq!(harness.within(|| *HIDDEN.peek()), 1);
    assert_settles_to_zero_frames(&mut harness);
}
