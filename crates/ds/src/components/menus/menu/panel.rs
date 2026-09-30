//! What the menu and each of its submenus share: one panel of choices, its tracker, its
//! pointer and key handling, and the submenu it opens (design/13-BEHAVIOUR-menus-windows.md
//! section 13.3.4). `Menu` is the root panel; `SubMenu` is every panel below it.

use crate::components::menus::item::item::MenuItem;
use crate::components::menus::item::lines::{Drawn, render_lines};
use crate::components::menus::item::view::client_point;
use crate::components::menus::menu::blink::Blink;
use crate::components::menus::menu::blink::highlighted as lit;
use crate::components::menus::menu::choices::{Act, Choice, liveness};
use crate::components::menus::menu::cursor::{MenuCursor, highlighted, seed};
use crate::components::menus::menu::decide::{Child, Decision, Level, decide};
use crate::components::menus::menu::keys::{KeyAct, key_act};
use crate::components::menus::menu::pick::Picked;
use crate::components::menus::menu::placement::Keys;
use crate::components::menus::menu::submenu::SubMenu;
use crate::components::menus::menu::tracker::{Tracker, Via, target};
use crate::host::measure::MountedRef;
use crate::stack::menu_track::types::{MenuTarget, MenuTiming};
use dioxus::prelude::*;
use ds_core::geometry::units::Point;
use ds_core::press::Press;
use ds_core::vocab::Availability;

/// One panel's working parts, rebuilt each render.
#[derive(Clone)]
pub(crate) struct Panel<T: 'static> {
    /// The panel's tracker.
    pub tracker: Tracker,
    /// Its choices, in order.
    pub choices: Vec<Choice<T>>,
    /// Root or submenu.
    pub level: Level,
    /// Where in the tree of panels this one is: the root is 0.
    pub depth: u8,
    /// The pick in flight: while a picked item blinks, its highlight goes out and comes back.
    pub blink: Blink,
    /// Whether key equivalents show.
    pub keys: Keys,
    /// A pick of a choice of the panel at this depth: the menu blinks, yields the value, then
    /// closes; an item that keeps the menu open yields at once and nothing else happens.
    pub onpick: EventHandler<Picked<T>>,
    /// A submenu tells its parent where the pointer is, so the parent's safe triangle and
    /// highlight hold while the pointer is inside the submenu.
    pub onhover: Option<EventHandler<Point>>,
    /// The root panel reports which choice the pointer is over, `None` once it is over none.
    pub onitem: Option<EventHandler<Option<usize>>>,
    /// The root panel hears a button released over a choice (press-drag-release).
    pub onrelease: Option<EventHandler<(usize, Press)>>,
    /// Whose highlight the panel shows: a submenu's is always its own.
    pub cursor: MenuCursor,
    /// Under a caller's cursor, where a key asked the highlight to go.
    pub on_active: Option<EventHandler<Option<usize>>>,
}

impl<T: Clone + PartialEq + 'static> Panel<T> {
    /// The highlighted choice: the panel's own selection settled onto an enabled choice, or
    /// the caller's; `None` when the caller highlights nothing or there is no choice.
    pub(crate) fn current(&self) -> Option<usize> {
        highlighted(
            self.cursor,
            self.tracker.selected(),
            &liveness(&self.choices),
        )
    }

    /// Whether this panel's own submenu is open.
    pub(crate) fn child(&self) -> Child {
        if self.tracker.has_open() {
            Child::Open
        } else {
            Child::Closed
        }
    }

    /// The rows of `items`, wired to this panel.
    pub(crate) fn body(&self, items: &[MenuItem<T>]) -> Element {
        let tracker = self.tracker;
        let choices = self.choices.clone();
        let pointed = self.choices.clone();
        let (onpick, onhover, onitem, depth) = (self.onpick, self.onhover, self.onitem, self.depth);
        render_lines(
            items,
            Drawn {
                selected: self.current().filter(|_| lit(self.blink, self.depth)),
                open: tracker.open().map(|open| open.choice),
                keys: self.keys,
                onpick: EventHandler::new(move |index: usize| match choices.get(index) {
                    Some(Choice {
                        act: Act::Pick(value, after),
                        ..
                    }) => onpick.call(Picked {
                        value: value.clone(),
                        depth,
                        after: *after,
                    }),
                    Some(Choice {
                        act: Act::Open(_), ..
                    }) => tracker.expand(index, Via::Pointer),
                    None => {}
                }),
                onpoint: EventHandler::new(move |(index, at): (usize, Point)| {
                    if let Some(choice) = pointed.get(index) {
                        tracker.point(at, target(index, choice));
                    }
                    if let Some(onhover) = onhover {
                        onhover.call(at);
                    }
                    if let Some(onitem) = onitem {
                        onitem.call(Some(index));
                    }
                }),
                onmounted: EventHandler::new(move |(index, event): (usize, MountedEvent)| {
                    tracker.row_mounted(index, MountedRef(event.data()));
                }),
                onrelease: self.onrelease,
            },
        )
    }

    /// The pointer moved over the panel but not a row (its padding, a header, a rule).
    pub(crate) fn hovered(&self, event: &MouseEvent) {
        let at = client_point(event);
        self.tracker.point(at, MenuTarget::Menu);
        if let Some(onhover) = self.onhover {
            onhover.call(at);
        }
        self.left_items();
    }

    /// The pointer is over no choice of this panel.
    pub(crate) fn left_items(&self) {
        if let Some(onitem) = self.onitem {
            onitem.call(None);
        }
    }

    /// What `event` means here; the caller acts on `Query`, `CloseMenu` and `Back`, which
    /// belong to the root or the parent.
    pub(crate) fn key(&self, event: &KeyboardEvent) -> Decision<T> {
        let Some(act) = key_act(&event.key(), event.modifiers()) else {
            return Decision::Nothing;
        };
        let decision = match &act {
            KeyAct::Jump(text) => self.jump(text),
            _ => self.decision(&act),
        };
        let own = matches!(
            decision,
            Decision::Select(_) | Decision::Pick(..) | Decision::Expand(_) | Decision::CloseSub
        );
        if own || matches!(decision, Decision::Back) {
            event.prevent_default();
            event.stop_propagation();
        }
        match &decision {
            Decision::Select(index) => self.select(*index),
            Decision::Pick(value, after) => self.onpick.call(Picked {
                value: value.clone(),
                depth: self.depth,
                after: *after,
            }),
            Decision::Expand(index) => self.tracker.expand(*index, Via::Keyboard),
            Decision::CloseSub => self.tracker.close_sub(),
            Decision::Back | Decision::CloseMenu | Decision::Nothing => {}
        }
        decision
    }

    /// Letters typed: the highlight goes to the next enabled choice whose title starts with
    /// them.
    fn jump(&self, text: &str) -> Decision<T> {
        let labels: Vec<&str> = self
            .choices
            .iter()
            .map(|choice| match choice.availability {
                Availability::Enabled => choice.title.as_str(),
                Availability::Disabled | Availability::Busy => "",
            })
            .collect();
        let from = self.current().unwrap_or(labels.len().saturating_sub(1));
        self.tracker
            .typed(text, &labels, from)
            .map_or(Decision::Nothing, Decision::Select)
    }

    /// What `act` means from the highlighted choice. With nothing highlighted a move starts
    /// from an end and a pick or an open does nothing.
    fn decision(&self, act: &KeyAct) -> Decision<T> {
        let (child, level) = (self.child(), self.level);
        match (self.current(), seed(act, self.choices.len())) {
            (Some(at), _) | (None, Some(at)) => decide(act, at, &self.choices, child, level),
            (None, None) => match act {
                KeyAct::Pick | KeyAct::Open => Decision::Nothing,
                KeyAct::Move(_)
                | KeyAct::Edge(_)
                | KeyAct::Back
                | KeyAct::Close
                | KeyAct::Jump(_) => decide(act, 0, &self.choices, child, level),
            },
        }
    }

    /// Move the highlight to `index`: the panel's own moves, the caller's is asked for.
    fn select(&self, index: usize) {
        match (self.cursor, self.on_active) {
            (MenuCursor::Auto, _) => self.tracker.select(index),
            (MenuCursor::Controlled(_), Some(on_active)) => on_active.call(Some(index)),
            (MenuCursor::Controlled(_), None) => {}
        }
    }

    /// The open submenu, if any: its own panel, placed beside this one.
    pub(crate) fn submenu(&self, timing: MenuTiming) -> Element {
        let Some(open) = self.tracker.open() else {
            return rsx! {};
        };
        let Some(Choice {
            act: Act::Open(children),
            ..
        }) = self.choices.get(open.choice)
        else {
            return rsx! {};
        };
        let tracker = self.tracker;
        rsx! {
            SubMenu::<T> {
                key: "{open.choice}",
                anchor: open.anchor,
                via: open.via,
                items: children.clone(),
                timing,
                depth: self.depth + 1,
                blink: self.blink,
                keys: self.keys,
                onpick: self.onpick,
                onback: move |()| tracker.close_sub(),
                onhover: move |at| tracker.point(at, MenuTarget::Menu),
                onplaced: move |rect| tracker.placed(rect),
            }
        }
    }
}
