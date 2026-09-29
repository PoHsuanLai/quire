//! WidgetGallery: "Edit Widgets" (design/23-WIDGETS.md section 9.7; the reference's widget
//! gallery). The person browses every registered widget (its name and description), sees the one
//! they are looking at drawn once, at the one size it takes (the user's decision, 2026-09-28: one
//! size per widget, no size picker), and adds it to the desktop or the notification center, each
//! at the size the widget takes there ([`WidgetInfo::size_in`]); beside it, what is placed on
//! each surface, by name, with a way to take it away. An Add that lands says so: its button
//! settles to a check and the new row rises in and is brought into view. The gallery changes
//! nothing itself: every
//! choice is a [`WidgetEdit`] handed to the host, which applies it to the layout it keeps in its
//! settings ([`crate::shell::widget::layout::apply`]) and passes the new layout back.

use crate::components::content::text_runs::{Text, text};
use crate::components::controls::button::ButtonVariant;
use crate::core::press::Press;
use crate::host::measure::MountedRef;
use crate::motion::detail::touch::{Contact, Touch};
use crate::shell::widget::contract::WidgetKind;
use crate::shell::widget::gallery_add::AddButton;
use crate::shell::widget::gallery_book::{Asked, Book};
use crate::shell::widget::gallery_rows::{Placed, placed};
use crate::shell::widget::kind::{Lift, WidgetHost, WidgetSize};
use crate::shell::widget::layout::{WidgetEdit, WidgetLayout};
use crate::shell::widget::registry::{WidgetInfo, use_widget_registry};
use dioxus::prelude::*;

/// The gallery's words, the host's to translate; English by default.
#[derive(Debug, Clone, PartialEq)]
pub struct GalleryWords {
    /// The button that adds the widget to the desktop.
    pub add_desktop: Text,
    /// The button that adds it to the notification center.
    pub add_center: Text,
    /// What an Add button says, beside its check, while the widget it added settles in
    ///.
    pub added: Text,
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
            added: "Added".into(),
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
/// placed, each choice sent as a `WidgetEdit` to `onedit`. When the layout the host hands back
/// holds the widget an Add asked for, that button settles to a check and the new row rises in
/// and is brought into view.
#[component]
pub fn WidgetGallery(
    layout: WidgetLayout,
    onedit: EventHandler<WidgetEdit>,
    #[props(default)] words: GalleryWords,
) -> Element {
    let registry = use_widget_registry();
    let first = registry.all().first().map(|info| info.kind.clone());
    let mut looking = use_signal(|| first);
    let mut book = use_hook(|| CopyValue::new(Book::default()));
    let arrived = book.write().read(&layout);
    let list = use_hook(|| CopyValue::new(None::<MountedRef>));
    let shown = looking().and_then(|kind| registry.get(&kind).cloned());
    let rows = Placed {
        registry: &registry,
        layout: &layout,
        arrived: &arrived,
        onedit,
        words: &words,
    };
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
                {detail(info, book, onedit, &words)}
            }
            {placed(rows, list)}
        }
    }
}

/// The widget looked at, drawn once at its desktop size, and the two ways to add it, each
/// keyed to the widget so a check never follows the person to another widget.
fn detail(
    info: WidgetInfo,
    book: CopyValue<Book>,
    onedit: EventHandler<WidgetEdit>,
    words: &GalleryWords,
) -> Element {
    let size = info.size_in(WidgetHost::Desktop);
    let buttons = [
        (
            WidgetHost::Desktop,
            ButtonVariant::Primary,
            words.add_desktop.clone(),
        ),
        (
            WidgetHost::Tile,
            ButtonVariant::Secondary,
            words.add_center.clone(),
        ),
    ];
    let add = move |kind: WidgetKind, host: WidgetHost, size: WidgetSize| {
        let mut book = book;
        move |press: Press| {
            book.write().ask(Asked {
                kind: kind.clone(),
                host,
                touch: Touch::Contact(Contact::pressed(&press)),
            });
            onedit.call(WidgetEdit::Add {
                kind: kind.clone(),
                size,
                host,
            });
        }
    };
    rsx! {
        div { class: "ds-widget-gallery-detail",
            div { class: "ds-widget-gallery-preview", "data-size": size.slug(),
                {info.preview(size, WidgetHost::Desktop, Lift::Rest)}
            }
            div { class: "ds-widget-gallery-actions",
                for (host, variant, label) in buttons {
                    AddButton {
                        key: "{info.kind.as_str()}-{host.slug()}",
                        variant,
                        label,
                        added: words.added.clone(),
                        landing: book.peek().landing(&info.kind, host),
                        onclick: add(info.kind.clone(), host, info.size_in(host)),
                    }
                }
            }
        }
    }
}
