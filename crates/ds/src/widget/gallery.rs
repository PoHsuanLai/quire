//! WidgetGallery: "Edit Widgets" (design/23-WIDGETS.md section 9.7; the reference's widget
//! gallery). The person browses every registered widget (its name and description), sees it
//! drawn at each size it offers with its preview entry, picks a size (the chosen card lifts, the
//! same `Lift` a dragged desktop widget takes), and adds it to the desktop or the notification
//! center; below, what is placed on each, with its size and a way to take it away. The gallery
//! changes nothing itself: every choice is a [`WidgetEdit`] handed to the host, which applies it
//! to the layout it keeps in its settings ([`crate::widget::apply`]) and passes the new layout
//! back.

use crate::components::button::{Button, ButtonVariant};
use crate::components::segmented::SegmentedControl;
use crate::components::text_runs::{Text, text};
use crate::components::widget_kind::{Lift, WidgetHost, WidgetSize};
use crate::widget::contract::WidgetKind;
use crate::widget::layout::{WidgetAt, WidgetEdit, WidgetLayout, WidgetPlacement};
use crate::widget::registry::{WidgetInfo, WidgetRegistry, use_widget_registry};
use dioxus::prelude::*;

/// The gallery's words, the host's to translate; English by default.
#[derive(Debug, Clone, PartialEq)]
pub struct GalleryWords {
    /// The button that adds the chosen size to the desktop.
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
    /// The size picker's label.
    pub size: Text,
    /// Each size's name: Small, Medium, Large.
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

impl GalleryWords {
    fn size_name(&self, size: WidgetSize) -> String {
        let [small, medium, large] = &self.sizes;
        match size {
            WidgetSize::Small => small,
            WidgetSize::Medium => medium,
            WidgetSize::Large => large,
        }
        .plain_text()
    }
}

/// The widget the person is looking at and the size they picked, if any.
#[derive(Debug, Clone, PartialEq)]
struct Choice {
    kind: WidgetKind,
    size: Option<WidgetSize>,
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
    let mut choice = use_signal(|| first.map(|kind| Choice { kind, size: None }));
    let shown = choice().and_then(|choice| {
        registry
            .get(&choice.kind)
            .cloned()
            .map(|info| (info, choice.size))
    });
    rsx! {
        div { class: "ds-widget-gallery",
            div { class: "ds-widget-gallery-kinds", role: "listbox",
                for info in registry.all().iter().cloned() {
                    button {
                        key: "{info.kind.as_str()}",
                        r#type: "button",
                        class: "ds-widget-gallery-kind",
                        role: "option",
                        "aria-selected": if shown.as_ref().is_some_and(|(on, _)| on.kind == info.kind) { "true" } else { "false" },
                        onclick: {
                            let kind = info.kind.clone();
                            move |_| choice.set(Some(Choice { kind: kind.clone(), size: None }))
                        },
                        span { class: "ds-widget-gallery-name", {text(&info.name)} }
                        span { class: "ds-widget-gallery-description", {text(&info.description)} }
                    }
                }
            }
            if let Some((info, size)) = shown {
                {detail(info, size, choice, onedit, &words)}
            }
            {placed(&registry, &layout, onedit, &words)}
        }
    }
}

/// The chosen widget at each of its sizes, the picked one lifted, and the two ways to add it.
fn detail(
    info: WidgetInfo,
    picked: Option<WidgetSize>,
    mut choice: Signal<Option<Choice>>,
    onedit: EventHandler<WidgetEdit>,
    words: &GalleryWords,
) -> Element {
    let adding = picked
        .or_else(|| info.sizes.first().copied())
        .unwrap_or_default();
    let add = |host: WidgetHost| {
        let kind = info.kind.clone();
        move |_| {
            onedit.call(WidgetEdit::Add {
                kind: kind.clone(),
                size: adding,
                host,
            })
        }
    };
    rsx! {
        div { class: "ds-widget-gallery-detail",
            div { class: "ds-widget-gallery-sizes",
                for size in ascending(info.sizes) {
                    div {
                        key: "{size.slug()}",
                        class: "ds-widget-gallery-size",
                        role: "button",
                        tabindex: "0",
                        "data-size": size.slug(),
                        "aria-pressed": if picked == Some(size) { "true" } else { "false" },
                        "aria-label": "{info.name.plain_text()}, {words.size_name(size)}",
                        onclick: {
                            let kind = info.kind.clone();
                            move |_| choice.set(Some(Choice { kind: kind.clone(), size: Some(size) }))
                        },
                        {info.preview(size, WidgetHost::Desktop, lift_of(picked, size))}
                        span { class: "ds-widget-gallery-size-name", "{words.size_name(size)}" }
                    }
                }
            }
            div { class: "ds-widget-gallery-actions",
                Button { variant: ButtonVariant::Primary, label: words.add_desktop.clone(), onclick: add(WidgetHost::Desktop) }
                Button { variant: ButtonVariant::Secondary, label: words.add_center.clone(), onclick: add(WidgetHost::Tile) }
            }
        }
    }
}

/// `sizes` smallest first, whatever order the widget lists them in (its first is the size it
/// is added at, not the picker's order).
fn ascending(sizes: &[WidgetSize]) -> impl Iterator<Item = WidgetSize> + '_ {
    [WidgetSize::Small, WidgetSize::Medium, WidgetSize::Large]
        .into_iter()
        .filter(move |size| sizes.contains(size))
}

/// The picked size's card is lifted; the others rest.
fn lift_of(picked: Option<WidgetSize>, size: WidgetSize) -> Lift {
    match picked {
        Some(on) if on == size => Lift::Lifted,
        Some(_) | None => Lift::Rest,
    }
}

/// What is placed on each surface: its name, its size (changeable) and the way to remove it.
fn placed(
    registry: &WidgetRegistry,
    layout: &WidgetLayout,
    onedit: EventHandler<WidgetEdit>,
    words: &GalleryWords,
) -> Element {
    let on = |host: WidgetHost| -> Vec<WidgetPlacement> {
        let mut items: Vec<WidgetPlacement> = layout
            .items()
            .iter()
            .filter(|item| item.at.host() == host)
            .cloned()
            .collect();
        items.sort_by_key(at_key);
        items
    };
    rsx! {
        div { class: "ds-widget-gallery-placed",
            for (host, heading) in [(WidgetHost::Desktop, words.desktop.clone()), (WidgetHost::Tile, words.center.clone())] {
                div { key: "{host.slug()}", class: "ds-widget-gallery-surface", "data-host": host.slug(),
                    span { class: "ds-widget-gallery-heading", {text(&heading)} }
                    if on(host).is_empty() {
                        span { class: "ds-widget-gallery-none", {text(&words.none)} }
                    }
                    for item in on(host) {
                        {row(registry, item, onedit, words)}
                    }
                }
            }
        }
    }
}

/// One placed widget's row.
fn row(
    registry: &WidgetRegistry,
    item: WidgetPlacement,
    onedit: EventHandler<WidgetEdit>,
    words: &GalleryWords,
) -> Element {
    let info = registry.get(&item.kind);
    let name = info.map_or_else(
        || item.kind.as_str().to_owned(),
        |info| info.name.plain_text(),
    );
    let sizes: Vec<(WidgetSize, String)> =
        ascending(info.map(|info| info.sizes).unwrap_or_default())
            .map(|size| (size, words.size_name(size)))
            .collect();
    let id = item.id;
    rsx! {
        div { key: "{id.0}", class: "ds-widget-gallery-row",
            span { class: "ds-widget-gallery-row-name", "{name}" }
            SegmentedControl {
                label: words.size.plain_text(),
                options: sizes,
                value: item.size,
                onchange: move |size| onedit.call(WidgetEdit::Resize(id, size)),
            }
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
