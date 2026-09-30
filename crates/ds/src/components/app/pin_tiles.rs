//! PinTiles: the pinned grid (design/30 section 2.11): a group of `PinTile`s four to a row, the
//! Add tile last, and a tile dragged onto another's place takes it (the drop line shows where it
//! lands). The grid never reorders itself: it hands the new order to the caller, who owns it.

use crate::components::app::pin_order::moved;
use crate::components::app::pin_tile::{PinFace, PinTile};
use crate::components::content::provider_mark::MarkStyle;
use crate::host::measure::client_rect;
use crate::root::common::Common;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use ds_core::geometry::units::{Point, Px};
use ds_core::vocab::{DropState, Selection};
use ds_motion::drag::{DRAG_THRESHOLD, DragPhase, use_drag};
use std::rc::Rc;

/// One pinned tile: its identity (an account's id), what it shows and its unread count.
#[derive(Debug, Clone, PartialEq)]
pub struct PinItem<K> {
    /// The tile's identity, stable across renders; what `onpick` and `onreorder` speak in.
    pub key: K,
    /// What it shows.
    pub face: PinFace,
    /// The `Badge` on its corner; zero draws none.
    pub unread: u32,
}

/// The tile after the items that adds one.
#[derive(Debug, Clone, PartialEq)]
pub struct PinAdd {
    /// What names it to assistive technology ("Add account").
    pub label: String,
    /// The hover hint ("Add account…").
    pub hint: Option<String>,
    /// It was pressed.
    pub onadd: EventHandler<()>,
}

/// The Add tile's key, apart from every index.
const ADD_KEY: &str = "add";

/// A press's point in the surface's own coordinates.
fn point(event: &PointerEvent) -> Point {
    let at = event.client_coordinates();
    Point {
        x: Px(at.x as f32),
        y: Px(at.y as f32),
    }
}

/// How a tile stands in the drag now: the one held is the source, the one a drop would land on
/// is the target, every other is idle.
fn drop_state<K: PartialEq>(phase: &DragPhase<K>, key: &K, index: usize) -> DropState {
    match phase {
        DragPhase::Live { key: held, .. } if held == key => DropState::Source,
        DragPhase::Live {
            target: Some(target),
            ..
        } if *target == index => DropState::Target,
        DragPhase::Idle | DragPhase::Pending { .. } | DragPhase::Live { .. } => DropState::Idle,
    }
}

/// The grid of `items`, `selected` being the one that is the list's filter. `onpick` hears a
/// press on a tile; with `onreorder` a tile can be dragged (past 3 px) onto another's place and
/// the new order comes back as the keys in order. A drag never also picks.
#[component]
pub fn PinTiles<K: Clone + PartialEq + 'static>(
    label: String,
    items: Vec<PinItem<K>>,
    #[props(default)] selected: Option<K>,
    #[props(default)] add: Option<PinAdd>,
    onpick: EventHandler<K>,
    #[props(default)] onreorder: Option<EventHandler<Vec<K>>>,
    #[props(default = MarkStyle::Letter)] mark: MarkStyle,
    #[props(default)] common: Common,
) -> Element {
    let drag = use_drag::<K>(DRAG_THRESHOLD);
    let elements = use_hook(|| CopyValue::new(Vec::<Option<Rc<MountedData>>>::new()));
    let mut kept = use_hook(|| CopyValue::new(false));
    let order: Vec<K> = items.iter().map(|item| item.key.clone()).collect();
    let phase = drag.phase();
    let mut finish = {
        let order = order.clone();
        move || {
            let live = matches!(drag.phase(), DragPhase::Live { .. });
            let dropped = drag.up();
            if live {
                kept.set(true);
            }
            if let (Some((key, to)), Some(onreorder)) = (dropped, onreorder) {
                onreorder.call(moved(&order, &key, to));
            }
        }
    };
    let mut leave = finish.clone();
    let class = common.class("ds-pin-tiles");
    let data = common.data_attributes();
    let name = common.aria_label.clone().unwrap_or(label);
    let tiles = items.into_iter().enumerate().map(|(index, item)| {
        let PinItem { key, face, unread } = item;
        let selection = Selection::of(&Some(key.clone()), &selected);
        let state = drop_state(&phase, &key, index);
        let picked = key.clone();
        let held = key.clone();
        let hook = EventHandler::new(move |event: MountedEvent| {
            let mut elements = elements;
            let mut list = elements.write();
            if list.len() <= index {
                list.resize(index + 1, None);
            }
            list[index] = Some(event.data());
        });
        rsx! {
            PinTile {
                key: "{index}",
                face,
                selection,
                unread,
                drop: state,
                mark: mark.clone(),
                onpointerdown: onreorder.map(|_| {
                    EventHandler::new(move |event: PointerEvent| {
                        if !matches!(event.trigger_button(), None | Some(MouseButton::Primary)) {
                            return;
                        }
                        drag.down(held.clone(), point(&event));
                        let rects = elements.peek().clone();
                        spawn(async move {
                            let mut measured = Vec::new();
                            for element in rects.into_iter().flatten() {
                                measured.extend(client_rect(&element).await);
                            }
                            drag.set_targets(measured);
                        });
                    })
                }),
                onclick: move |_| {
                    if kept.replace(false) {
                        return;
                    }
                    onpick.call(picked.clone());
                },
                common: Common { mounted: Some(hook), ..Common::default() },
            }
        }
    });
    rsx! {
        div {
            class,
            id: common.id.clone(),
            role: "group",
            "aria-label": name,
            onmounted: move |event| common.mounted(event),
            onpointermove: move |event| drag.moved(point(&event)),
            onpointerup: move |_| finish(),
            onpointerleave: move |_| leave(),
            ..data,
            {tiles}
            if let Some(PinAdd { label, hint, onadd }) = add {
                PinTile {
                    key: "{ADD_KEY}",
                    face: PinFace::Add { label, hint },
                    onclick: move |_| onadd.call(()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::drop_state;
    use ds_core::geometry::units::{Point, Px};
    use ds_core::vocab::DropState;
    use ds_motion::drag::DragPhase;

    #[test]
    fn a_live_drag_marks_its_source_and_its_target_only() {
        let at = Point {
            x: Px(0.0),
            y: Px(0.0),
        };
        let live = DragPhase::Live {
            key: 'a',
            at,
            target: Some(2),
        };
        // (tile key, tile index, state)
        const CASES: &[(char, usize, DropState)] = &[
            ('a', 0, DropState::Source),
            ('b', 1, DropState::Idle),
            ('c', 2, DropState::Target),
        ];
        for &(key, index, want) in CASES {
            assert_eq!(drop_state(&live, &key, index), want, "{key}");
        }
        for idle in [DragPhase::Idle, DragPhase::Pending { key: 'a', from: at }] {
            assert_eq!(drop_state(&idle, &'a', 0), DropState::Idle);
        }
    }
}
