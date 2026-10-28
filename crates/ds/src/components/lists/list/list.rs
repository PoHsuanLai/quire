//! List: the container rows live in (`NSTableView` / `NSOutlineView`, design/30 section 2.6).
//! Items come and go by the roster (an inserted row fades and slides in, a removed one leaves
//! and the rows below close the gap), the arrow keys, Home and End rove among the rows and stop
//! at the ends, letters jump to the next label that starts with them, and the list draws its
//! selection accent while it holds the keyboard and grey when it does not.
//!
//! With `onreorder` each row leads with a grip (`Icon::Grip`) and can be dragged by it: past
//! 3 px the row lifts and follows the pointer up or down, the rows it passes make room, and on
//! release the new order goes to the caller, who owns it (the list never reorders itself, as
//! `PinTiles` does not). Only the grip starts a drag, so a press on the row is still the row's.

use crate::components::lists::list::entry::{ListEntry, SetPlace};
use crate::components::lists::list::keys::{ListKey, labelled, list_key, moved};
use crate::components::lists::list::model::{ListItem, ListStyle};
use crate::components::lists::list::reorder::{held_to, stand};
use crate::components::lists::order::moved as reordered;
use crate::focus::soon::focus_soon;
use crate::host::measure::client_rect;
use crate::root::common::Common;
use crate::stack::typeahead::Typeahead;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use ds_core::geometry::units::{Point, Px, Rect};
use ds_core::time::clock::now;
use ds_core::word::Word;
use ds_motion::drag::{DRAG_THRESHOLD, DragPhase, use_drag};
use ds_motion::presence::Exit;
use ds_motion::roster::RowPitch;
use ds_motion::use_roster::{LeaveBy, RosterSpec, use_roster};
use ds_style::tokens::control_size::SidebarSize;
use std::hash::Hash;
use std::rc::Rc;

/// A list of `items`, the whole list on every render: a key missing from it leaves. `cursor` is
/// the row the keys rest on (the caller draws it selected through the row's own state);
/// `onselect` hears where an arrow, Home, End or a typed letter moved it, and `onpick` a press
/// of Enter or Space on it. Rows hear their own clicks.
///
/// The keys that go missing in one render are one batch and leave together, then the rows below
/// heal by the heights the leaving rows measured. `exit` is read when a batch starts;
/// `on_settled` hears each dropped key once its batch has settled. Multi-selection is the
/// caller's: it puts each selected row's own state, and the list only moves the cursor. Keys are
/// hashed to name the items' nodes, so a key must hash the same on every render.
///
/// `autofocus: true` gives the list the keyboard as it mounts, so the arrow keys work at once (a
/// sheet's list, as Choose Application's); the default is off.
///
/// `onreorder` makes the rows draggable by their grips; it hears the keys in their new order
/// once a row is dropped somewhere else.
#[component]
pub fn List<K: Clone + PartialEq + Hash + 'static>(
    label: String,
    items: Vec<ListItem<K>>,
    #[props(default)] style: ListStyle,
    #[props(default)] sidebar: SidebarSize,
    #[props(default)] cursor: Option<K>,
    #[props(default)] onselect: Option<EventHandler<K>>,
    #[props(default)] onpick: Option<EventHandler<K>>,
    #[props(default = Exit::Row)] exit: Exit,
    #[props(default)] on_settled: Option<EventHandler<K>>,
    #[props(default)] autofocus: bool,
    #[props(default)] onreorder: Option<EventHandler<Vec<K>>>,
    #[props(default)] common: Common,
) -> Element {
    let book = use_book(&items);
    let drag = use_drag::<K>(DRAG_THRESHOLD);
    // Each row's element, and the rows' rects and the pointer as the drag began.
    let rows = use_hook(|| CopyValue::new(Vec::<(K, Rc<MountedData>)>::new()));
    let measured = use_hook(|| CopyValue::new(Vec::<Rect>::new()));
    let began = use_hook(|| CopyValue::new(Point::default()));
    let order: Vec<K> = items.iter().map(|item| item.key.clone()).collect();
    let phase = drag.phase();
    let finish = {
        let order = order.clone();
        move || {
            if let (Some((key, to)), Some(onreorder)) = (drag.up(), onreorder) {
                let next = reordered(&order, &key, to);
                if next != order {
                    onreorder.call(next);
                }
            }
        }
    };
    let leave = finish.clone();
    let keys: Vec<K> = items.iter().map(|item| item.key.clone()).collect();
    let roster = use_roster(
        keys,
        RosterSpec {
            leave: LeaveBy::Delist,
            exit,
            pitch: RowPitch::default(),
            on_settled: Some(EventHandler::new(move |key: K| {
                book.forget(&key);
                if let Some(on_settled) = on_settled {
                    on_settled.call(key);
                }
            })),
        },
    );
    let pitches = roster.pitches();
    let entries = roster.entries();
    let len = entries.len();
    let mut held = use_signal(|| false);
    let typeahead = use_hook(|| CopyValue::new(Typeahead::default()));
    let data = common.data_attributes();
    let mounted = common.clone();
    let listed = items.clone();
    let cursor_key = cursor.clone();
    let onkey = move |event: KeyboardEvent| {
        let Some(act) = list_key(&event.key(), event.modifiers()) else {
            return;
        };
        match act {
            ListKey::Move(rove) => {
                if let (Some(key), Some(onselect)) =
                    (moved(&listed, cursor_key.as_ref(), rove), onselect)
                {
                    event.prevent_default();
                    onselect.call(key);
                }
            }
            ListKey::Pick => {
                if let (Some(key), Some(onpick)) = (cursor_key.clone(), onpick) {
                    event.prevent_default();
                    onpick.call(key);
                }
            }
            ListKey::Jump(text) => {
                let mut buffer = typeahead;
                let found = labelled(&listed, cursor_key.as_ref(), |labels, from| {
                    let (next, found) = buffer.peek().clone().typed(&text, now(), labels, from);
                    buffer.set(next);
                    found
                });
                if let (Some(key), Some(onselect)) = (found, onselect) {
                    onselect.call(key);
                }
            }
        }
    };
    let away = (!held()).then_some("away");
    rsx! {
        div {
            class: common.class("ds-list"),
            id: common.id.clone(),
            role: "listbox",
            tabindex: "0",
            "aria-label": common.aria_label.clone().unwrap_or(label),
            "data-style": style.drawn().slug(),
            "data-sidebar-size": (style == ListStyle::SourceList).then(|| sidebar.slug()),
            "data-focus": away,
            "data-reorder": onreorder.map(|_| "grip"),
            onmounted: move |event: MountedEvent| {
                if autofocus {
                    focus_soon(event.data());
                }
                mounted.mounted(event);
            },
            onpointermove: move |event| {
                if !matches!(drag.phase(), DragPhase::Idle) {
                    drag.moved(held_to(point(&event), &measured.peek()));
                }
            },
            onpointerup: move |_| finish(),
            onpointerleave: move |_| leave(),
            onfocus: move |_| held.set(true),
            onblur: move |_| held.set(false),
            onkeydown: onkey,
            ..data,
            for (index, entry) in entries.into_iter().enumerate() {
                ListEntry::<K> {
                    key: "{node_key(&entry.key)}",
                    content: book.of(&entry.key),
                    grip: onreorder.map(|_| grip(drag, entry.key.clone(), order.clone(), rows, measured, began)),
                    mounted: onreorder.map(|_| {
                        let key = entry.key.clone();
                        EventHandler::new(move |element: Rc<MountedData>| {
                            let mut rows = rows;
                            let mut rows = rows.write();
                            rows.retain(|(held, _)| *held != key);
                            rows.push((key.clone(), element));
                        })
                    }),
                    dragged: order
                        .iter()
                        .position(|held| *held == entry.key)
                        .map(|at| stand(&phase, &order, &measured.peek(), *began.peek(), at))
                        .unwrap_or_default(),
                    entry,
                    pitches,
                    place: SetPlace { index, len },
                }
            }
        }
    }
}

/// A press's point in the surface's own coordinates.
fn point(event: &PointerEvent) -> Point {
    let at = event.client_coordinates();
    Point {
        x: Px(at.x as f32),
        y: Px(at.y as f32),
    }
}

/// What a primary press on `key`'s grip does: hold the row, and measure every row in `order`, which
/// is where a drop can land.
fn grip<K: Clone + PartialEq + 'static>(
    drag: ds_motion::drag::DragTracker<K>,
    key: K,
    order: Vec<K>,
    rows: CopyValue<Vec<(K, Rc<MountedData>)>>,
    measured: CopyValue<Vec<Rect>>,
    began: CopyValue<Point>,
) -> EventHandler<PointerEvent> {
    EventHandler::new(move |event: PointerEvent| {
        if !matches!(event.trigger_button(), None | Some(MouseButton::Primary)) {
            return;
        }
        let at = point(&event);
        let mut began = began;
        began.set(at);
        drag.down(key.clone(), at);
        let elements: Vec<Rc<MountedData>> = order
            .iter()
            .filter_map(|wanted| {
                rows.peek()
                    .iter()
                    .find(|(held, _)| held == wanted)
                    .map(|(_, element)| element.clone())
            })
            .collect();
        spawn(async move {
            let mut rects = Vec::new();
            for element in elements {
                rects.extend(client_rect(&element).await);
            }
            let mut measured = measured;
            measured.set(rects.clone());
            drag.set_targets(rects);
        });
    })
}

/// An item's node key: its key's hash, in hex.
pub(crate) fn node_key<K: Hash>(key: &K) -> String {
    use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher};
    format!(
        "{:016x}",
        BuildHasherDefault::<DefaultHasher>::default().hash_one(key)
    )
}

/// The last content each key was listed with, so an item the caller has dropped still draws while
/// it leaves.
struct Book<K: 'static>(CopyValue<Vec<(K, Element)>>);

impl<K: 'static> Clone for Book<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: 'static> Copy for Book<K> {}

impl<K: Clone + PartialEq + 'static> Book<K> {
    fn of(&self, key: &K) -> Element {
        self.0
            .peek()
            .iter()
            .find(|(held, _)| held == key)
            .map_or_else(|| rsx! {}, |(_, content)| content.clone())
    }

    fn forget(&self, key: &K) {
        let mut book = self.0;
        let _ = book
            .try_write()
            .map(|mut book| book.retain(|(held, _)| held != key));
    }
}

/// The book, refreshed from this render's items (a listed item is always its newest content; a
/// dropped one keeps its last).
fn use_book<K: Clone + PartialEq + 'static>(items: &[ListItem<K>]) -> Book<K> {
    let mut book = use_hook(|| CopyValue::new(Vec::<(K, Element)>::new()));
    let kept: Vec<(K, Element)> = book
        .peek()
        .iter()
        .filter(|(key, _)| items.iter().all(|item| item.key != *key))
        .cloned()
        .collect();
    let next = items
        .iter()
        .map(|item| (item.key.clone(), item.content.clone()))
        .chain(kept)
        .collect();
    book.set(next);
    Book(book)
}

#[cfg(test)]
mod tests {
    use super::node_key;

    #[test]
    fn a_key_names_the_same_node_every_time() {
        assert_eq!(node_key(&7u32), node_key(&7u32));
        assert_ne!(node_key(&7u32), node_key(&8u32));
        assert_eq!(node_key(&7u32).len(), 16);
    }
}
