//! What the menu and each of its submenus share: one panel of choices, its tracker, its
//! pointer and key handling, and the submenu it opens (design/13-BEHAVIOUR-menus-windows.md
//! section 13.3.4). `Menu` is the root panel; [`SubMenu`] is every panel below it.

use crate::components::menu::MenuKind;
use crate::components::menu_cursor::{Cursor, highlighted, seed};
use crate::components::menu_entry::MenuEntry;
use crate::components::menu_item::client_point;
use crate::components::menu_keys::{Child, Decision, Level, decide};
use crate::components::menu_lines::{
    Act, Choice, Filter, KeyAct, Line, choices, key_act, lines, liveness,
};
use crate::components::menu_rows::{Drawn, render_lines};
use crate::components::menu_tracker::{Tracker, Via, target, use_tracker};
use crate::components::popover::{Stacking, layer_slug, position_style, use_float};
use crate::components::press::Press;
use crate::geometry::{Align, MountedRef, Placement, Point, Px, Rect, Side};
use crate::overlay::menu_track::{MenuTarget, MenuTiming};
use crate::tokens::ZLayer;
use dioxus::prelude::*;

/// How far a submenu sits from its parent panel (design/13 section 13.3.4: "gap 2").
const SUB_GAP: Px = Px(2.0);

/// One panel's working parts, rebuilt each render.
#[derive(Clone)]
pub(crate) struct Panel<T: 'static> {
    /// The panel's tracker.
    pub tracker: Tracker,
    /// Its choices, in order.
    pub choices: Vec<Choice<T>>,
    /// Root or submenu.
    pub level: Level,
    /// Close the whole menu, then yield a value.
    pub onpick: EventHandler<T>,
    /// A submenu tells its parent where the pointer is, so the parent's safe triangle and
    /// highlight hold while the pointer is inside the submenu.
    pub onhover: Option<EventHandler<Point>>,
    /// The root panel reports which choice the pointer is over, `None` once it is over none.
    pub onitem: Option<EventHandler<Option<usize>>>,
    /// The root panel hears a button released over a choice (press-drag-release).
    pub onrelease: Option<EventHandler<(usize, Press)>>,
    /// Whose highlight the panel shows: a submenu's is always its own.
    pub cursor: Cursor,
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

    /// The rows of `lines`, wired to this panel.
    pub(crate) fn body(&self, shown: &[Line<'_, T>], kind: MenuKind) -> Element {
        let tracker = self.tracker;
        let choices = self.choices.clone();
        let pointed = self.choices.clone();
        let (onpick, onhover, onitem) = (self.onpick, self.onhover, self.onitem);
        render_lines(
            shown,
            kind.row(),
            Drawn {
                selected: self.current(),
                open: tracker.open().map(|open| open.choice),
                onpick: EventHandler::new(move |index: usize| match choices.get(index) {
                    Some(Choice {
                        act: Act::Pick(value),
                        ..
                    }) => onpick.call(value.clone()),
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
    pub(crate) fn key(&self, event: &KeyboardEvent, filter: Filter) -> Decision<T> {
        let Some(act) = key_act(&event.key(), event.modifiers(), filter) else {
            return Decision::Nothing;
        };
        let decision = self.decision(&act);
        let own = matches!(
            decision,
            Decision::Select(_) | Decision::Pick(_) | Decision::Expand(_) | Decision::CloseSub
        );
        if own || matches!(decision, Decision::Back | Decision::Query) {
            event.prevent_default();
            event.stop_propagation();
        }
        match &decision {
            Decision::Select(index) => self.select(*index),
            Decision::Pick(value) => self.onpick.call(value.clone()),
            Decision::Expand(index) => self.tracker.expand(*index, Via::Keyboard),
            Decision::CloseSub => self.tracker.close_sub(),
            Decision::Back | Decision::CloseMenu | Decision::Query | Decision::Nothing => {}
        }
        decision
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
                | KeyAct::Back
                | KeyAct::Close
                | KeyAct::Type(_)
                | KeyAct::Erase => decide(act, 0, &self.choices, child, level),
            },
        }
    }

    /// Move the highlight to `index`: the panel's own moves, the caller's is asked for.
    fn select(&self, index: usize) {
        match (self.cursor, self.on_active) {
            (Cursor::Auto, _) => self.tracker.select(index),
            (Cursor::Controlled(_), Some(on_active)) => on_active.call(Some(index)),
            (Cursor::Controlled(_), None) => {}
        }
    }

    /// The open submenu, if any: its own panel, placed beside this one.
    pub(crate) fn submenu(&self, kind: MenuKind, timing: MenuTiming, depth: u8) -> Element {
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
                kind,
                anchor: open.anchor,
                via: open.via,
                entries: children.clone(),
                timing,
                depth: depth + 1,
                onpick: self.onpick,
                onback: move |()| tracker.close_sub(),
                onhover: move |at| tracker.point(at, MenuTarget::Menu),
                onplaced: move |rect| tracker.placed(rect),
            }
        }
    }
}

/// The choices of `entries` as a panel lists them with no query.
pub(crate) fn panel_choices<T: Clone>(entries: &[MenuEntry<T>]) -> Vec<Choice<T>> {
    choices(&lines(entries, ""))
}

/// A submenu: a panel of `entries` placed right of its parent panel (left when it would cross
/// the edge), its first row level with the parent row, opening with no animation (design/13
/// section 13.3.4). It floats on the menu layer without joining the layer stack: its menu's
/// layer takes Escape and the outside click, and it closes with its parent.
#[component]
pub(crate) fn SubMenu<T: Clone + PartialEq + 'static>(
    kind: MenuKind,
    anchor: Rect,
    via: Via,
    entries: Vec<MenuEntry<T>>,
    timing: MenuTiming,
    depth: u8,
    onpick: EventHandler<T>,
    onback: EventHandler<()>,
    onhover: EventHandler<Point>,
    onplaced: EventHandler<Rect>,
) -> Element {
    let float = use_float(ZLayer::Menu, Stacking::Passive);
    let tracker = use_tracker(timing, kind.pad());
    let panel = Panel {
        tracker,
        choices: panel_choices(&entries),
        level: Level::Sub,
        onpick,
        onhover: Some(onhover),
        onitem: None,
        onrelease: None,
        cursor: Cursor::Auto,
        on_active: None,
    };
    let want = Placement::new(Side::Right, Align::Start);
    let at = float.origin(Some(anchor), want, SUB_GAP);
    let placed = float.placed(Some(anchor), want, SUB_GAP);
    let mut reported = use_signal(|| None::<Rect>);
    use_effect(use_reactive!(|placed| {
        if placed.is_some() && *reported.peek() != placed {
            reported.set(placed);
            if let Some(rect) = placed {
                onplaced.call(rect);
            }
        }
    }));
    let shown = lines(&entries, "");
    let body = panel.body(&shown, kind);
    let child = panel.submenu(kind, timing, depth);
    let probe = float.surface();
    let keys = panel.clone();
    let hover = panel.clone();
    float.show(
        rsx! {
            div {
                class: "ds-popover ds-menu",
                "data-elevation": "pop",
                "data-layer": layer_slug(ZLayer::Menu),
                "data-kind": kind.slug(),
                "data-depth": "{depth}",
                role: "listbox",
                "data-presence": "present",
                tabindex: "-1",
                style: position_style(at),
                onmounted: move |event| {
                    let element = event.data();
                    tracker.panel_mounted(MountedRef(element.clone()));
                    probe.on_mounted(event);
                    if via == Via::Keyboard {
                        crate::focus::host::focus_soon(element);
                    }
                },
                onmousemove: move |event| hover.hovered(&event),
                onkeydown: move |event| {
                    if keys.key(&event, Filter::None) == Decision::Back {
                        onback.call(());
                    }
                },
                {body}
            }
        },
        EventHandler::new(move |()| onback.call(())),
    );
    rsx! {
        {child}
    }
}
