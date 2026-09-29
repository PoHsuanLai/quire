//! "Added" in Edit Widgets on a real Blitz document and the virtual clock: when the
//! layout the host hands back holds the widget an Add asked for, the button settles to a check
//! (the check grows in on the spring, the person pressed it) and says "Added" for the check's
//! draw and hold, then is itself again; the new row rises in (`row-in`) and is scrolled into view
//! in the placed column even when the column had to scroll to show it; then nothing asks for a
//! frame.

use dioxus::prelude::*;
use ds::widget::{DesktopGrid, WidgetEdit, WidgetLayout, apply};
use ds::{
    Anim, Appearance, BatteryWidget, DelayToken, Ds, DurationToken, Material, MotionLevel, Panel,
    PanelEdge, Px, RootExtent, Shown, StaggerIndex, Widget, WidgetGallery, WidgetHost,
    WidgetMetrics, WidgetSize, settle,
};
use ds_native::harness::assert_settles_to_zero_frames;
use ds_native::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 1280,
    height: 800,
    scale_percent: 100,
};

const GRID: DesktopGrid = DesktopGrid {
    columns: 6,
    rows: 3,
};

/// Eight batteries on the desktop: more rows than the sheet's placed column shows at once.
fn crowded() -> WidgetLayout {
    (0..8).fold(WidgetLayout::default(), |layout, _| {
        let edit = WidgetEdit::Add {
            kind: BatteryWidget::kind(),
            size: WidgetSize::Small,
            host: WidgetHost::Desktop,
        };
        apply(layout.clone(), edit, GRID).unwrap_or(layout)
    })
}

#[allow(non_snake_case)]
fn Sheet() -> Element {
    let mut layout = use_signal(crowded);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet, extent: RootExtent::Viewport,
            Panel { label: "Edit Widgets", shown: Shown::Visible, edge: PanelEdge::Bottom, width: Px(1040.0), height: Px(440.0), material: Material::Sheet,
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

const CENTER_ADD: &str = ".ds-widget-gallery-actions .ds-button:nth-child(2)";
const NEW_ROW: &str = ".ds-widget-gallery-surface[*|data-host=tile] .ds-widget-gallery-row";

fn inside(row: ds::Rect, list: ds::Rect) -> bool {
    row.origin.y.0 >= list.origin.y.0 - 0.5
        && row.origin.y.0 + row.size.height.0 <= list.origin.y.0 + list.size.height.0 + 0.5
}

#[test]
fn an_add_that_lands_settles_to_a_check_and_its_row_rises_into_view() {
    let mut harness =
        Harness::with_config(Sheet, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(600));
    assert_eq!(
        harness.count(".ds-widget-gallery-row"),
        8,
        "opened on eight"
    );
    assert_eq!(
        harness.count(".ds-widget-gallery-row.a-row-in, .ds-widget-gallery-row.a-rise"),
        0,
        "rows placed before the gallery opened sit still"
    );
    assert_eq!(harness.count(".ds-widget-gallery-added"), 0);
    let list = harness
        .rect(".ds-widget-gallery-placed")
        .expect("the placed column");
    let none = harness
        .rect(".ds-widget-gallery-surface[*|data-host=tile] .ds-widget-gallery-none")
        .expect("the center's empty line");
    assert!(
        !inside(none, list),
        "the center's list starts below the column's fold: {none:?} {list:?}"
    );

    let add = harness
        .centre(CENTER_ADD)
        .expect("Add to Notification Center");
    harness.click(add);
    harness.advance(Duration::ZERO);
    assert_eq!(harness.count(NEW_ROW), 1, "the host placed it");
    assert!(
        harness.has_class(NEW_ROW, "a-row-in"),
        "the person's Add springs its row in"
    );
    assert_eq!(
        harness.text_of(CENTER_ADD).as_deref(),
        Some("Added"),
        "the button says so"
    );
    assert!(
        harness.has_class(".ds-widget-gallery-added", "a-morph-in-spring"),
        "the check grows in on the spring"
    );
    assert_eq!(harness.count(".ds-widget-gallery-added .ds-check-mark"), 1);
    assert_eq!(
        harness
            .text_of(".ds-widget-gallery-actions .ds-button:nth-child(1)")
            .as_deref(),
        Some("Add to Desktop"),
        "only the button that added"
    );

    let unscrolled = harness.rect(NEW_ROW).expect("the new row");
    assert!(
        !inside(unscrolled, list),
        "placed below the fold: {unscrolled:?} in {list:?}"
    );
    harness.advance(Duration::from_millis(50));
    // `list` is the column's box measured before it scrolled: Blitz reports a scroller's own
    // rect shifted by its scroll offset, so the box it shows is the one measured at rest.
    let row = harness.rect(NEW_ROW).expect("the new row");
    assert!(inside(row, list), "scrolled into view: {row:?} in {list:?}");
    assert!(
        (row.origin.y.0 + row.size.height.0 - (list.origin.y.0 + list.size.height.0)).abs() < 0.6,
        "moved the least: its bottom on the column's: {row:?} in {list:?}"
    );

    let level = MotionLevel::Standard;
    let row_in = settle(Anim::RowIn, level, StaggerIndex::default());
    harness.advance(row_in);
    assert!(
        !harness.has_class(NEW_ROW, "a-row-in"),
        "the rise is taken off at settle(RowIn)"
    );

    let hold = DelayToken::SettleHold.delay();
    let draw = DurationToken::Move.duration(level);
    assert_eq!(
        harness.text_of(CENTER_ADD).as_deref(),
        Some("Added"),
        "still settling well inside draw + hold"
    );
    harness.advance((draw + hold).saturating_sub(row_in) + Duration::from_millis(100));
    assert_eq!(
        harness.text_of(CENTER_ADD).as_deref(),
        Some("Add to Notification Center"),
        "itself again once the check's hold ends"
    );
    assert_eq!(harness.count(".ds-widget-gallery-added"), 0);
    assert_settles_to_zero_frames(&mut harness);
}
