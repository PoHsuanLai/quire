//! Edit Widgets as the person meets it: a desktop over the calm
//! wallpaper with the widgets placed on it, each at its grid cell from the right, and quire's
//! `Sheet` (`Attach::Bottom`) holding `WidgetGallery`, no taller than half the desktop, so the
//! top rows where a new widget lands stay in view. Live: Add places the widget on the desktop
//! above the sheet at the size it takes there, and the gallery's own list shows it; Remove takes
//! it away, the card on the desktop fading out (`Shown::Hidden`)
//! before the page drops it.

use crate::axes::Axes;
use crate::wallpaper;
use dioxus::prelude::*;
use ds::components::overlays::sheet_width::SheetWidth;
use ds::{
    Appearance, Attach, Ds, Inject, Material, RootChrome, Sheet, Shown, SpaceLook, use_scope,
};
use ds_shell::widget::{DesktopGrid, WidgetAt, WidgetEdit, WidgetLayout, WidgetPlacement, apply};
use ds_shell::{
    BatteryWidget, MonthWidget, Timeline, Widget, WidgetCard, WidgetGallery, WidgetHost,
    WidgetMetrics, WidgetSize, WorldClockWidget,
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
fn size_of(kind: &ds_shell::WidgetKind, host: WidgetHost) -> WidgetSize {
    ds_shell::WidgetRegistry::quire()
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
    let scheme = use_scope().scheme;
    let mut layout = use_signal(opening);
    let mut leaving = use_signal(Vec::<WidgetPlacement>::new);
    let metrics = WidgetMetrics::default().style_attr();
    let onedit = move |edit: WidgetEdit| {
        let before = layout();
        if let Ok(next) = apply(before.clone(), edit, GRID) {
            leaving.with_mut(|going| going.extend(removed(&before, &next)));
            layout.set(next);
        }
    };
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
                        DeskCard { key: "{item.id.0}", item, shown: Shown::Visible, on_hidden: |()| {} }
                    }
                    for item in leaving() {
                        DeskCard { key: "{item.id.0}", item: item.clone(), shown: Shown::Hidden,
                            on_hidden: move |()| leaving.with_mut(|going| going.retain(|gone| gone.id != item.id)) }
                    }
                }
                Sheet { label: "Edit Widgets", onclose: |_| {}, attach: Attach::Bottom, width: SheetWidth::Wide,
                    div { class: "g-we-sheet", style: "{metrics};height:330px",
                        WidgetGallery { layout: layout(), onedit }
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

/// The desktop placements in `before` that `after` no longer holds.
fn removed(before: &WidgetLayout, after: &WidgetLayout) -> Vec<WidgetPlacement> {
    on_desktop(before)
        .into_iter()
        .filter(|item| after.items().iter().all(|kept| kept.id != item.id))
        .collect()
}

/// One placed widget at its cell, drawn with its preview entry, leaving when `shown` says.
#[component]
fn DeskCard(item: WidgetPlacement, shown: Shown, on_hidden: EventHandler<()>) -> Element {
    let WidgetAt::Desktop(cell) = item.at else {
        return rsx! {};
    };
    let (left, top) = (8 + cell.column * PITCH, 8 + cell.row * PITCH);
    let size = item.size;
    let on_hidden = Some(on_hidden);
    let card = match item.kind.as_str() {
        "quire.battery" => rsx! {
            WidgetCard { widget: BatteryWidget, timeline: Timeline::now(BatteryWidget::preview(size)), size, shown, on_hidden }
        },
        "quire.world-clock" => rsx! {
            WidgetCard { widget: WorldClockWidget, timeline: Timeline::now(WorldClockWidget::preview(size)), size, shown, on_hidden }
        },
        "quire.month" => rsx! {
            WidgetCard { widget: MonthWidget, timeline: Timeline::now(MonthWidget::preview(size)), size, shown, on_hidden }
        },
        _ => rsx! {},
    };
    rsx! {
        div { class: "g-we-cell", style: "left:{left}px;top:{top}px",
            {card}
        }
    }
}
