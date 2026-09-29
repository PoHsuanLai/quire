//! Edit Widgets' placed lists: what is on the desktop and in the notification
//! center, a row each with the widget's name and Remove. A row that arrives while the gallery is
//! open rises in (`row-in`, `--t-big --e-spring`, when the person's own Add placed it; `rise`,
//! `--t-move --e-out`, when something else did: design/05 principle 2) and is brought into view
//! in the placed column, moving the column the least that shows it. Rows already placed when
//! the gallery opened sit still.

use crate::components::content::text_runs::{TextLine, text};
use crate::components::controls::button::{Button, ButtonVariant};
use crate::core::task::spawn_in;
use crate::host::measure::MountedRef;
use crate::host::reveal::reveal;
use crate::motion::detail::touch::Touch;
use crate::motion::{
    anim::Anim,
    timer::{TimerPhase, use_motion_timer},
};
use crate::shell::catalog::placement::PlacementId;
use crate::shell::widget::gallery::GalleryWords;
use crate::shell::widget::kind::WidgetHost;
use crate::shell::widget::layout::{WidgetAt, WidgetEdit, WidgetLayout, WidgetPlacement};
use crate::shell::widget::registry::WidgetRegistry;
use dioxus::core::current_scope_id;
use dioxus::prelude::*;
use std::collections::BTreeMap;

/// The placements on `host`, in their order there.
pub(crate) fn on_host(layout: &WidgetLayout, host: WidgetHost) -> Vec<WidgetPlacement> {
    let mut items: Vec<WidgetPlacement> = layout
        .items()
        .iter()
        .filter(|item| item.at.host() == host)
        .cloned()
        .collect();
    items.sort_by_key(at_key);
    items
}

/// A placement's sort key on its surface: the column's order, or the desktop's cell by row.
fn at_key(item: &WidgetPlacement) -> (u16, u16) {
    match item.at {
        WidgetAt::Desktop(cell) => (cell.row, cell.column),
        WidgetAt::Center(order) => (order.0, 0),
    }
}

/// What the placed column needs from the gallery: the registry for names, the layout, the rows
/// that arrived with this layout and who caused each, and the edit handler.
pub(crate) struct Placed<'a> {
    pub(crate) registry: &'a WidgetRegistry,
    pub(crate) layout: &'a WidgetLayout,
    pub(crate) arrived: &'a BTreeMap<PlacementId, Touch>,
    pub(crate) onedit: EventHandler<WidgetEdit>,
    pub(crate) words: &'a GalleryWords,
}

/// What is placed on each surface, in a column that scrolls on its own.
pub(crate) fn placed(placed: Placed<'_>, mut list: CopyValue<Option<MountedRef>>) -> Element {
    let Placed {
        registry,
        layout,
        arrived,
        onedit,
        words,
    } = placed;
    let surfaces = [
        (WidgetHost::Desktop, words.desktop.clone()),
        (WidgetHost::Tile, words.center.clone()),
    ];
    rsx! {
        div { class: "ds-widget-gallery-placed",
            onmounted: move |event: MountedEvent| list.set(Some(MountedRef(event.data()))),
            for (host, heading) in surfaces {
                div { key: "{host.slug()}", class: "ds-widget-gallery-surface", "data-host": host.slug(),
                    span { class: "ds-widget-gallery-heading", {text(&heading)} }
                    if on_host(layout, host).is_empty() {
                        span { class: "ds-widget-gallery-none", {text(&words.none)} }
                    }
                    for item in on_host(layout, host) {
                        PlacedRow {
                            key: "{item.id.0}",
                            id: item.id,
                            name: name_of(registry, &item),
                            remove: words.remove.clone(),
                            arrival: arrived.get(&item.id).copied(),
                            list,
                            onedit,
                        }
                    }
                }
            }
        }
    }
}

/// The widget's name in the registry, or its kind when the registry does not hold it.
fn name_of(registry: &WidgetRegistry, item: &WidgetPlacement) -> String {
    registry.get(&item.kind).map_or_else(
        || item.kind.as_str().to_owned(),
        |info| info.name.plain_text(),
    )
}

/// How a row that arrived rises: the spring only for the person's own press.
fn entrance(touch: Touch) -> Anim {
    match touch {
        Touch::Contact(_) => Anim::RowIn,
        Touch::Remote => Anim::Rise,
    }
}

/// One placed widget's row: its name and Remove. `arrival` is read once, as the row mounts: a
/// row that arrived rises and is brought into view in `list`; later renders never replay it.
#[component]
fn PlacedRow(
    id: PlacementId,
    name: String,
    remove: TextLine,
    arrival: Option<Touch>,
    list: CopyValue<Option<MountedRef>>,
    onedit: EventHandler<WidgetEdit>,
) -> Element {
    let arrived = use_hook(|| arrival);
    let anim = arrived.map(entrance);
    let timer = use_motion_timer(anim.unwrap_or(Anim::Rise));
    use_hook(|| {
        if arrived.is_some() {
            timer.start(EventHandler::new(|()| {}));
        }
    });
    let rising = anim.filter(|_| timer.phase() != TimerPhase::Settled);
    let class = match rising {
        Some(anim) => format!("ds-widget-gallery-row {}", anim.class()),
        None => "ds-widget-gallery-row".to_owned(),
    };
    let scope = use_hook(current_scope_id);
    rsx! {
        div {
            class,
            "data-pulse": rising.map(|_| "a"),
            onmounted: move |event: MountedEvent| {
                let (Some(_), Some(scroller)) = (arrived, list.peek().clone()) else {
                    return;
                };
                let row = MountedRef(event.data());
                spawn_in(scope, async move {
                    let _ = reveal(&scroller.0, &row.0).await;
                });
            },
            span { class: "ds-widget-gallery-row-name", "{name}" }
            Button { variant: ButtonVariant::Danger, label: remove,
                onclick: move |_| onedit.call(WidgetEdit::Remove(id)) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{at_key, entrance, on_host};
    use crate::motion::anim::Anim;
    use crate::motion::detail::touch::{Contact, Touch};
    use crate::shell::widget::kind::{WidgetHost, WidgetSize};
    use crate::shell::widget::layout::{DesktopGrid, WidgetEdit, WidgetLayout, apply};
    use crate::shell::widget::{
        battery::BatteryWidget, calendar::MonthWidget, clock::WorldClockWidget, contract::Widget,
    };

    #[test]
    fn each_surface_lists_only_its_own_in_their_order() {
        const GRID: DesktopGrid = DesktopGrid {
            columns: 4,
            rows: 3,
        };
        let adds = [
            (BatteryWidget::kind(), WidgetHost::Desktop),
            (MonthWidget::kind(), WidgetHost::Tile),
            (WorldClockWidget::kind(), WidgetHost::Desktop),
        ];
        let layout = adds
            .into_iter()
            .fold(WidgetLayout::default(), |layout, (kind, host)| {
                let edit = WidgetEdit::Add {
                    kind,
                    size: WidgetSize::Small,
                    host,
                };
                apply(layout.clone(), edit, GRID).unwrap_or(layout)
            });
        let desktop = on_host(&layout, WidgetHost::Desktop);
        assert_eq!(desktop.len(), 2);
        assert!(
            desktop
                .windows(2)
                .all(|pair| at_key(&pair[0]) <= at_key(&pair[1]))
        );
        assert_eq!(on_host(&layout, WidgetHost::Tile).len(), 1);
    }

    #[test]
    fn only_the_persons_add_springs_its_row_in() {
        assert_eq!(entrance(Touch::Contact(Contact::for_tests())), Anim::RowIn);
        assert_eq!(entrance(Touch::Remote), Anim::Rise);
    }
}
