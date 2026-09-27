//! WidgetGallery: "Edit Widgets" (design/23-WIDGETS.md section 9.7; the reference's widget
//! gallery). The person browses every registered widget (its name and description), sees the one
//! they are looking at drawn once, at the one size it takes (the user's decision, 2026-09-28: one
//! size per widget, no size picker), and adds it to the desktop or the notification center, each
//! at the size the widget takes there ([`WidgetInfo::size_in`]); beside it, what is placed on
//! each surface, by name, with a way to take it away. The gallery changes nothing itself: every
//! choice is a [`WidgetEdit`] handed to the host, which applies it to the layout it keeps in its
//! settings ([`crate::widget::apply`]) and passes the new layout back.

use crate::components::button::{Button, ButtonVariant};
use crate::components::text_runs::{Text, text};
use crate::components::widget_kind::{Lift, WidgetHost};
use crate::widget::layout::{WidgetAt, WidgetEdit, WidgetLayout, WidgetPlacement};
use crate::widget::registry::{WidgetInfo, WidgetRegistry, use_widget_registry};
use dioxus::prelude::*;

/// The gallery's words, the host's to translate; English by default.
#[derive(Debug, Clone, PartialEq)]
pub struct GalleryWords {
    /// The button that adds the widget to the desktop.
    pub add_desktop: Text,
    /// The button that adds it to the notification center.
    pub add_center: Text,
    /// The button that takes a placed widget away.
    pub remove: Text,
    /// The desktop's heading over its placed widgets.
    pub desktop: Text,
    /// The notification center's heading.
    pub center: Text,
    /// A surface with nothing placed.
    pub none: Text,
    /// The size picker's label. Unused since the gallery offers one size per widget
    /// (2026-09-28); kept so a host's words still build.
    pub size: Text,
    /// Each size's name: Small, Medium, Large. Unused since 2026-09-28, as `size` is.
    pub sizes: [Text; 3],
}

impl Default for GalleryWords {
    fn default() -> Self {
        GalleryWords {
            add_desktop: "Add to Desktop".into(),
            add_center: "Add to Notification Center".into(),
            remove: "Remove".into(),
            desktop: "Desktop".into(),
            center: "Notification Center".into(),
            none: "No widgets".into(),
            size: "Size".into(),
            sizes: ["Small".into(), "Medium".into(), "Large".into()],
        }
    }
}

/// The registry's widgets (an ancestor's `provide_widget_registry`, else quire's), `layout` as
/// placed, each choice sent as a `WidgetEdit` to `onedit`.
#[component]
pub fn WidgetGallery(
    layout: WidgetLayout,
    onedit: EventHandler<WidgetEdit>,
    #[props(default)] words: GalleryWords,
) -> Element {
    let registry = use_widget_registry();
    let first = registry.all().first().map(|info| info.kind.clone());
    let mut looking = use_signal(|| first);
    let shown = looking().and_then(|kind| registry.get(&kind).cloned());
    rsx! {
        div { class: "ds-widget-gallery",
            div { class: "ds-widget-gallery-kinds", role: "listbox",
                for info in registry.all().iter().cloned() {
                    button {
                        key: "{info.kind.as_str()}",
                        r#type: "button",
                        class: "ds-widget-gallery-kind",
                        role: "option",
                        "aria-selected": if shown.as_ref().is_some_and(|on| on.kind == info.kind) { "true" } else { "false" },
                        onclick: {
                            let kind = info.kind.clone();
                            move |_| looking.set(Some(kind.clone()))
                        },
                        span { class: "ds-widget-gallery-name", {text(&info.name)} }
                        span { class: "ds-widget-gallery-description", {text(&info.description)} }
                    }
                }
            }
            if let Some(info) = shown {
                {detail(info, onedit, &words)}
            }
            {placed(&registry, &layout, onedit, &words)}
        }
    }
}

/// The widget looked at, drawn once at its desktop size, and the two ways to add it.
fn detail(info: WidgetInfo, onedit: EventHandler<WidgetEdit>, words: &GalleryWords) -> Element {
    let add = |host: WidgetHost| {
        let (kind, size) = (info.kind.clone(), info.size_in(host));
        move |_| {
            onedit.call(WidgetEdit::Add {
                kind: kind.clone(),
                size,
                host,
            })
        }
    };
    let size = info.size_in(WidgetHost::Desktop);
    rsx! {
        div { class: "ds-widget-gallery-detail",
            div { class: "ds-widget-gallery-preview", "data-size": size.slug(),
                {info.preview(size, WidgetHost::Desktop, Lift::Rest)}
            }
            div { class: "ds-widget-gallery-actions",
                Button { variant: ButtonVariant::Primary, label: words.add_desktop.clone(), onclick: add(WidgetHost::Desktop) }
                Button { variant: ButtonVariant::Secondary, label: words.add_center.clone(), onclick: add(WidgetHost::Tile) }
            }
        }
    }
}

/// The placements on `host`, in their order there.
fn on_host(layout: &WidgetLayout, host: WidgetHost) -> Vec<WidgetPlacement> {
    let mut items: Vec<WidgetPlacement> = layout
        .items()
        .iter()
        .filter(|item| item.at.host() == host)
        .cloned()
        .collect();
    items.sort_by_key(at_key);
    items
}

/// What is placed on each surface: its name and the way to remove it.
fn placed(
    registry: &WidgetRegistry,
    layout: &WidgetLayout,
    onedit: EventHandler<WidgetEdit>,
    words: &GalleryWords,
) -> Element {
    rsx! {
        div { class: "ds-widget-gallery-placed",
            for (host, heading) in [(WidgetHost::Desktop, words.desktop.clone()), (WidgetHost::Tile, words.center.clone())] {
                div { key: "{host.slug()}", class: "ds-widget-gallery-surface", "data-host": host.slug(),
                    span { class: "ds-widget-gallery-heading", {text(&heading)} }
                    if on_host(layout, host).is_empty() {
                        span { class: "ds-widget-gallery-none", {text(&words.none)} }
                    }
                    for item in on_host(layout, host) {
                        {row(registry, item, onedit, words)}
                    }
                }
            }
        }
    }
}

/// One placed widget's row: its name and Remove.
fn row(
    registry: &WidgetRegistry,
    item: WidgetPlacement,
    onedit: EventHandler<WidgetEdit>,
    words: &GalleryWords,
) -> Element {
    let name = registry.get(&item.kind).map_or_else(
        || item.kind.as_str().to_owned(),
        |info| info.name.plain_text(),
    );
    let id = item.id;
    rsx! {
        div { key: "{id.0}", class: "ds-widget-gallery-row",
            span { class: "ds-widget-gallery-row-name", "{name}" }
            Button { variant: ButtonVariant::Danger, label: words.remove.clone(),
                onclick: move |_| onedit.call(WidgetEdit::Remove(id)) }
        }
    }
}

/// A placement's sort key on its surface: the column's order, or the desktop's cell by row.
fn at_key(item: &WidgetPlacement) -> (u16, u16) {
    match item.at {
        WidgetAt::Desktop(cell) => (cell.row, cell.column),
        WidgetAt::Center(order) => (order.0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::{at_key, on_host};
    use crate::components::widget_kind::{WidgetHost, WidgetSize};
    use crate::widget::layout::{DesktopGrid, WidgetEdit, WidgetLayout, apply};
    use crate::widget::{BatteryWidget, MonthWidget, Widget, WorldClockWidget};

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
}
