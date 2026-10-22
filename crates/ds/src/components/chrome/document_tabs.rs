//! DocumentTabs: the tab strip of a document window, as Finder and Safari draw it (design/27
//! section 5, pre-Liquid-Glass): a full-width strip under the toolbar whose tabs share the width
//! equally, each with a close button that shows under the pointer and on the selected tab, and a
//! plus at the end. It is not `TabView` (`NSTabView`: a framed body of at most six tabs behind a
//! segmented control); this strip has no body, no limit and a close button, and the document
//! under it is the app's.
//!
//! The strip does not hide itself: the app draws it when [`strip_shown`] says two or more tabs
//! are open and leaves it out otherwise. The strip owns nothing: it hears `onselect(index)`,
//! `onclose(index)`, `onnew` and, when it is given, `onreorder((from, to))` (a tab dragged past
//! 3 px onto another's place), and the app changes its tabs. A tab's full title is its tooltip.
//!
//! Keys: with the keyboard in the strip the arrows, Home and End select a tab, and the standard
//! New and Close actions (chordkit has none for the next or previous tab) call `onnew` and
//! `onclose` of the selected tab; an app that binds them elsewhere maps them with
//! [`command_of`] instead. The strip reads no modifiers.
//!
//! Markup: `div.ds-document-tabs[role=tablist]` of `div.ds-document-tab[role=tab]` (a
//! `span.ds-document-tab-title` and `button.ds-document-tab-close`) and
//! `button.ds-document-tabs-new`.

use crate::components::chrome::document_tabs_model::{TabCommand, command_of, reorder, rove_to};
use crate::components::content::tip_text::TipText;
use crate::components::content::title_tip::use_tip;
use crate::focus::click::kept_click;
use crate::host::measure::client_rect;
use crate::keys::use_keys;
use crate::root::common::Common;
use crate::stack::roving::Rove;
use chordkit::Context;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use ds_core::geometry::units::{Point, Px};
use ds_core::vocab::{DropState, Selection};
use ds_motion::drag::{DRAG_THRESHOLD, DragPhase, use_drag};
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};
use std::rc::Rc;

/// One tab: the document's title, shown in full in its tooltip.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentTab {
    /// The document's title.
    pub title: String,
    /// The tab's own `id`, `data-*`, classes and mount hook.
    pub common: Common,
}

impl DocumentTab {
    /// A tab titled `title`.
    pub fn new(title: impl Into<String>) -> Self {
        DocumentTab {
            title: title.into(),
            common: Common::default(),
        }
    }
}

/// A press's point in the page's own coordinates.
fn point(event: &PointerEvent) -> Point {
    let at = event.client_coordinates();
    Point {
        x: Px(at.x as f32),
        y: Px(at.y as f32),
    }
}

/// How tab `index` stands in the drag now: the one held is the source, the one a drop would land
/// on is the target.
fn drop_state(phase: &DragPhase<usize>, index: usize) -> DropState {
    match phase {
        DragPhase::Live { key, .. } if *key == index => DropState::Source,
        DragPhase::Live {
            target: Some(target),
            ..
        } if *target == index => DropState::Target,
        DragPhase::Idle | DragPhase::Pending { .. } | DragPhase::Live { .. } => DropState::Idle,
    }
}

/// The strip of `tabs`, `selected` the open one (an index; one past the end selects none).
/// `onselect`, `onclose` and `onnew` speak in indexes; with `onreorder` a tab can be dragged onto
/// another's place and `(from, to)` comes back. A drag never also selects.
#[component]
pub fn DocumentTabs(
    #[props(into)] label: String,
    tabs: Vec<DocumentTab>,
    selected: usize,
    onselect: EventHandler<usize>,
    onclose: EventHandler<usize>,
    onnew: EventHandler<()>,
    #[props(default)] onreorder: Option<EventHandler<(usize, usize)>>,
    #[props(default)] common: Common,
) -> Element {
    let drag = use_drag::<usize>(DRAG_THRESHOLD);
    let elements = use_hook(|| CopyValue::new(Vec::<Option<Rc<MountedData>>>::new()));
    let mut kept = use_hook(|| CopyValue::new(false));
    let keys = use_keys();
    let count = tabs.len();
    let phase = drag.phase();
    let mut finish = move || {
        let live = matches!(drag.phase(), DragPhase::Live { .. });
        let dropped = drag.up();
        if live {
            kept.set(true);
        }
        if let (Some((from, to)), Some(onreorder)) = (dropped, onreorder)
            && let Some(pair) = reorder(from, to)
        {
            onreorder.call(pair);
        }
    };
    let mut leave = finish;
    let class = common.class("ds-document-tabs");
    let data = common.data_attributes();
    let name = common.aria_label.clone().unwrap_or(label);
    let items = tabs.into_iter().enumerate().map(|(index, tab)| {
        let hook = EventHandler::new(move |event: MountedEvent| {
            let mut elements = elements;
            let mut list = elements.write();
            if list.len() <= index {
                list.resize(index + 1, None);
            }
            list[index] = Some(event.data());
        });
        let hold = onreorder.map(|_| {
            EventHandler::new(move |event: PointerEvent| {
                if !matches!(event.trigger_button(), None | Some(MouseButton::Primary)) {
                    return;
                }
                drag.down(index, point(&event));
                let rects: Vec<_> = elements.peek().iter().take(count).cloned().collect();
                spawn(async move {
                    let mut measured = Vec::new();
                    for element in rects.into_iter().flatten() {
                        measured.extend(client_rect(&element).await);
                    }
                    drag.set_targets(measured);
                });
            })
        });
        let common = Common {
            mounted: Some(hook),
            ..tab.common
        };
        rsx! {
            DocumentTabItem {
                key: "{index}",
                index,
                title: tab.title,
                selection: Selection::of(&index, &selected),
                state: drop_state(&phase, index),
                onselect: move |at| {
                    if !kept.replace(false) {
                        onselect.call(at);
                    }
                },
                onclose,
                onhold: hold,
                common,
            }
        }
    });
    let on_key = move |event: KeyboardEvent| {
        let key = event.key();
        let roved = matches!(key, Key::ArrowLeft | Key::ArrowRight | Key::Home | Key::End)
            .then(|| Rove::of(&key))
            .flatten();
        if let Some(rove) = roved {
            event.prevent_default();
            let to = rove_to(rove, selected, count);
            if to != selected && to < count {
                onselect.call(to);
                if let Some(Some(element)) = elements.peek().get(to).cloned() {
                    spawn(async move {
                        let _ = element.set_focus(true).await;
                    });
                }
            }
            return;
        }
        let command = keys
            .action_of(&event, Context::Normal)
            .as_ref()
            .and_then(command_of);
        match command {
            Some(TabCommand::New) => onnew.call(()),
            Some(TabCommand::CloseSelected) if selected < count => onclose.call(selected),
            Some(TabCommand::CloseSelected) | None => return,
        }
        event.prevent_default();
        event.stop_propagation();
    };
    rsx! {
        div {
            class,
            id: common.id.clone(),
            role: "tablist",
            "aria-label": name,
            "aria-orientation": "horizontal",
            onmounted: move |event| common.mounted(event),
            onkeydown: on_key,
            onpointermove: move |event| drag.moved(point(&event)),
            onpointerup: move |_| finish(),
            onpointerleave: move |_| leave(),
            ..data,
            {items}
            NewTab { onnew }
        }
    }
}

/// One tab. Its own component for the tooltip's hooks.
#[component]
fn DocumentTabItem(
    index: usize,
    title: String,
    selection: Selection,
    state: DropState,
    onselect: EventHandler<usize>,
    onclose: EventHandler<usize>,
    #[props(default)] onhold: Option<EventHandler<PointerEvent>>,
    common: Common,
) -> Element {
    let tip = use_tip(Some(TipText::new(title.clone())));
    let class = common.class("ds-document-tab");
    let data = common.data_attributes();
    rsx! {
        div {
            id: common.id.clone(),
            class,
            role: "tab",
            "aria-selected": selection.aria(),
            "aria-label": title.clone(),
            tabindex: if selection == Selection::Selected { "0" } else { "-1" },
            "data-selected": (selection == Selection::Selected).then_some("true"),
            "data-drop": state.drop_attr(),
            "data-drag": state.drag_attr(),
            title: tip.native(),
            onmouseover: {
                let tip = tip.clone();
                move |event| tip.over(&event)
            },
            onmouseleave: {
                let tip = tip.clone();
                move |_| tip.out()
            },
            onpointerdown: {
                let tip = tip.clone();
                move |event| {
                    tip.press();
                    if let Some(hold) = onhold {
                        hold.call(event);
                    }
                }
            },
            onclick: move |_| onselect.call(index),
            onmounted: {
                let tip = tip.clone();
                move |event| {
                    tip.mounted(&event);
                    common.mounted(event);
                }
            },
            ..data,
            span { class: "ds-document-tab-title ds-truncate", "{title}" }
            button {
                r#type: "button",
                class: "ds-document-tab-close",
                "aria-label": "Close {title}",
                // A press here is the close button's own: it neither selects the tab nor starts
                // its drag.
                onpointerdown: move |event| event.stop_propagation(),
                onclick: move |event| {
                    event.stop_propagation();
                    onclose.call(index);
                    kept_click(&event);
                },
                Glyph { icon: Icon::X, size: IconSize::Micro }
            }
        }
        {tip.surface()}
    }
}

/// The plus at the end of the strip.
#[component]
fn NewTab(onnew: EventHandler<()>) -> Element {
    let tip = use_tip(Some(TipText::new("New Tab")));
    rsx! {
        button {
            r#type: "button",
            class: "ds-document-tabs-new",
            "aria-label": "New Tab",
            title: tip.native(),
            onmouseover: {
                let tip = tip.clone();
                move |event| tip.over(&event)
            },
            onmouseleave: {
                let tip = tip.clone();
                move |_| tip.out()
            },
            onpointerdown: {
                let tip = tip.clone();
                move |_| tip.press()
            },
            onclick: move |_| onnew.call(()),
            onmounted: {
                let tip = tip.clone();
                move |event| tip.mounted(&event)
            },
            Glyph { icon: Icon::Plus, size: IconSize::Compact }
        }
        {tip.surface()}
    }
}
