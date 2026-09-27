//! Edit Widgets as the person meets it (sill Q520-Q523, G423): a desktop over the calm
//! wallpaper with the widgets placed on it, each at its grid cell from the right, and quire's
//! `Panel` at the bottom edge holding `WidgetGallery`, no taller than half the desktop, so the
//! top rows where a new widget lands stay in view. Live: Add places the widget on the desktop
//! above the sheet at the size it takes there, and the gallery's own list shows it; Remove takes
//! it away.

use crate::axes::Axes;
use crate::wallpaper;
use dioxus::prelude::*;
use ds::widget::{DesktopGrid, WidgetAt, WidgetEdit, WidgetLayout, WidgetPlacement, apply};
use ds::{
    Appearance, BatteryWidget, Ds, Inject, Lift, Material, MonthWidget, Panel, PanelEdge, Px,
    RootChrome, Shown, SpaceLook, Widget, WidgetGallery, WidgetHost, WidgetMetrics, WidgetSize,
    WorldClockWidget, use_env, use_widget_registry,
};

/// The desktop's grid: six columns of the 164 cell and 16 gap in the stage's 1120.
const GRID: DesktopGrid = DesktopGrid {
    columns: 6,
    rows: 3,
};

/// One cell and its gap, in pixels (`WidgetMetrics::default()`).
const PITCH: u16 = 164 + 16;

/// The layout the page opens on: the batteries and the world clock on the desktop, the month in
/// the notification center, each at the size it takes there.
fn opening() -> WidgetLayout {
    [
        (BatteryWidget::kind(), WidgetHost::Desktop),
        (WorldClockWidget::kind(), WidgetHost::Desktop),
        (MonthWidget::kind(), WidgetHost::Tile),
    ]
    .into_iter()
    .fold(WidgetLayout::default(), |layout, (kind, host)| {
        let size = size_of(&kind, host);
        apply(layout.clone(), WidgetEdit::Add { kind, size, host }, GRID).unwrap_or(layout)
    })
}

/// The size `kind` takes in `host`, from quire's registry.
fn size_of(kind: &ds::WidgetKind, host: WidgetHost) -> WidgetSize {
    ds::WidgetRegistry::quire()
        .get(kind)
        .map(|info| info.size_in(host))
        .unwrap_or_default()
}

/// The desktop and the sheet.
#[component]
pub(super) fn EditWidgetsStage() -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (theme, accent, motion, blur, look) = {
        let axes = axes.read();
        (
            axes.theme,
            axes.accent,
            axes.motion,
            axes.blur,
            axes.look.clone(),
        )
    };
    let scheme = use_env().scheme;
    let mut layout = use_signal(opening);
    let metrics = WidgetMetrics::default().style_attr();
    rsx! {
        div { class: "g-we-stage", style: "background-image:url(\"{wallpaper::calm_uri(scheme)}\")",
            Ds {
                appearance: Appearance { theme, accent, motion },
                look: SpaceLook { theme, ..look },
                material: Material::Widget,
                blur,
                chrome: Some(RootChrome::Transparent),
                stylesheet: Inject::Host,
                div { class: "g-we-desk", style: "{metrics}",
                    for item in on_desktop(&layout()) {
                        DeskCard { key: "{item.id.0}", item }
                    }
                }
                Panel { label: "Edit Widgets", shown: Shown::Visible, edge: PanelEdge::Bottom, width: Px(1040.0), height: Px(330.0), material: Material::Sheet,
                    div { class: "g-we-sheet", style: "{metrics}",
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
}

/// The desktop's placements.
fn on_desktop(layout: &WidgetLayout) -> Vec<WidgetPlacement> {
    layout
        .items()
        .iter()
        .filter(|item| item.at.host() == WidgetHost::Desktop)
        .cloned()
        .collect()
}

/// One placed widget at its cell, drawn with its preview entry.
#[component]
fn DeskCard(item: WidgetPlacement) -> Element {
    let registry = use_widget_registry();
    let WidgetAt::Desktop(cell) = item.at else {
        return rsx! {};
    };
    let (left, top) = (8 + cell.column * PITCH, 8 + cell.row * PITCH);
    let card = registry
        .get(&item.kind)
        .map(|info| info.preview(item.size, WidgetHost::Desktop, Lift::Rest));
    rsx! {
        div { class: "g-we-cell", style: "left:{left}px;top:{top}px",
            {card}
        }
    }
}
