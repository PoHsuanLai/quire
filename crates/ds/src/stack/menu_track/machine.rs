//! The tracker's transitions (design/13 section 13.4's table).

use super::pointer::{close_sub, moved, placed, ticked};
use super::types::{
    Entered, ItemPath, MenuAnim, MenuDirection, MenuKey, MenuPhase, MenuTarget, MenuTiming,
    MenuTrack, MenuTrackEffect, MenuTrackEvent, Pickable, Session, ShownBy, Submenu,
};
use ds_core::geometry::units::Point;
use ds_core::machine::Machine;
use ds_core::time::stamp::Stamp;
use ds_core::vocab::PressPhase;

/// The effects of one step, in order.
pub(super) type Effects<K> = Vec<MenuTrackEffect<K>>;

impl<K: Clone + PartialEq> MenuTrack<K> {
    /// A closed tracker.
    pub fn closed() -> Self {
        MenuTrack {
            phase: MenuPhase::Closed,
        }
    }

    /// A tracker already tracking `menu` in click mode: a menu that is open by the time the
    /// tracker exists (a ds `Menu` mounts open), so the opening press is over.
    pub fn open(menu: K) -> Self {
        MenuTrack {
            phase: opened(menu, ShownBy::Press, PressPhase::Idle),
        }
    }

    /// The open menu's key, if one is open.
    pub fn open_menu(&self) -> Option<&K> {
        match &self.phase {
            MenuPhase::Closed => None,
            MenuPhase::Tracking(session) => Some(&session.menu),
        }
    }

    /// The item the keyboard cursor stands on: the last one highlighted.
    pub fn cursor(&self) -> Option<&ItemPath> {
        match &self.phase {
            MenuPhase::Closed => None,
            MenuPhase::Tracking(session) => session.cursor.as_ref(),
        }
    }

    /// Whether `item`'s submenu is open, shown or still being placed.
    pub fn open_on(&self, item: &ItemPath) -> bool {
        matches!(self.submenu(), Some(Submenu::Open { item: open, .. }) if open == item)
    }

    /// Whether the pointer rests on `item` and its submenu waits for the delay.
    pub fn pending_on(&self, item: &ItemPath) -> bool {
        matches!(self.submenu(), Some(Submenu::Pending { item: wait, .. }) if wait == item)
    }

    /// The open or pending submenu, if a menu is open and has one.
    pub fn submenu(&self) -> Option<&Submenu> {
        match &self.phase {
            MenuPhase::Closed => None,
            MenuPhase::Tracking(session) => Some(&session.sub),
        }
    }
}

impl<K: Clone + PartialEq + 'static> Machine for MenuTrack<K> {
    type In = MenuTrackEvent<K>;
    type Out = MenuTrackEffect<K>;
    type Params = MenuTiming;
    type Ctx = ();

    /// The tracker after `event` at `at`, and what the caller must do, in order. A
    /// [`MenuTrackEvent::Tick`] that comes before its deadline changes nothing.
    fn step(
        self,
        event: MenuTrackEvent<K>,
        at: Stamp,
        timing: &MenuTiming,
        _: &(),
    ) -> (Self, Effects<K>) {
        let (phase, effects) = match self.phase {
            MenuPhase::Closed => closed(event),
            MenuPhase::Tracking(session) => tracking(session, event, at, *timing),
        };
        (MenuTrack { phase }, effects)
    }

    /// The submenu delay's end, or the safe triangle's timeout; none otherwise.
    fn wake(&self) -> Option<Stamp> {
        let MenuPhase::Tracking(session) = &self.phase else {
            return None;
        };
        match &session.sub {
            Submenu::Pending { until, .. } => Some(*until),
            Submenu::Open {
                guard: Some(guard), ..
            } => guard.timeout,
            Submenu::Open { guard: None, .. } | Submenu::None => None,
        }
    }
}

/// Nothing open: only a press on a title does anything.
fn closed<K: Clone>(event: MenuTrackEvent<K>) -> (MenuPhase<K>, Effects<K>) {
    match event {
        MenuTrackEvent::PressTitle(menu) => {
            let effects = vec![MenuTrackEffect::Open(menu.clone(), MenuAnim::Pop)];
            (opened(menu, ShownBy::Press, PressPhase::Pressed), effects)
        }
        _ => (MenuPhase::Closed, Vec::new()),
    }
}

/// A fresh session on `menu`.
fn opened<K>(menu: K, shown: ShownBy, held: PressPhase) -> MenuPhase<K> {
    MenuPhase::Tracking(Session {
        menu,
        shown,
        held,
        entered: Entered::NotYet,
        hot: None,
        cursor: None,
        sub: Submenu::None,
        pointer: Point::default(),
        under: MenuTarget::Outside,
    })
}

/// Close everything: `Close(Fade)` first, then `then` (a pick runs after the close).
fn close<K>(then: Effects<K>) -> (MenuPhase<K>, Effects<K>) {
    let mut effects = vec![MenuTrackEffect::Close(MenuAnim::Fade)];
    effects.extend(then);
    (MenuPhase::Closed, effects)
}

pub(super) fn keep<K>(session: Session<K>, effects: Effects<K>) -> (MenuPhase<K>, Effects<K>) {
    (MenuPhase::Tracking(session), effects)
}

/// A menu is open.
fn tracking<K: Clone + PartialEq>(
    session: Session<K>,
    event: MenuTrackEvent<K>,
    now: Stamp,
    timing: MenuTiming,
) -> (MenuPhase<K>, Effects<K>) {
    match event {
        MenuTrackEvent::PressTitle(other) if other == session.menu => match session.shown {
            ShownBy::Press => close(Vec::new()),
            // The pointer switched to this menu on its way to the press: the press is the click
            // that opens it, held as a fresh press on the title is.
            ShownBy::Hover => keep(
                Session {
                    shown: ShownBy::Press,
                    held: PressPhase::Pressed,
                    ..session
                },
                Vec::new(),
            ),
        },
        MenuTrackEvent::PressTitle(other) => {
            let effects = vec![MenuTrackEffect::Check(other.clone())];
            (opened(other, ShownBy::Press, PressPhase::Pressed), effects)
        }
        MenuTrackEvent::OutsidePress => close(Vec::new()),
        MenuTrackEvent::Release(target) => released(session, target),
        MenuTrackEvent::Move(_, MenuTarget::Title(other)) if other != session.menu => {
            let effects = vec![MenuTrackEffect::Check(other.clone())];
            (opened(other, ShownBy::Hover, session.held), effects)
        }
        MenuTrackEvent::Move(at, target) => moved(session, at, target, now, timing),
        MenuTrackEvent::SubPlaced { top, bottom } => keep(placed(session, top, bottom), vec![]),
        MenuTrackEvent::Tick => ticked(session, now, timing),
        MenuTrackEvent::Key(key) => keyed(session, key),
        MenuTrackEvent::Select(path) => selected(session, path),
        MenuTrackEvent::Expand(path) => expanded(session, path),
    }
}

/// The keyboard moved the highlight to `path`: another item's submenu closes and a pending
/// one is dropped; the highlight follows.
fn selected<K>(session: Session<K>, path: ItemPath) -> (MenuPhase<K>, Effects<K>) {
    let mut effects = Vec::new();
    let sub = match session.sub {
        Submenu::Open { item, guard } if item == path => Submenu::Open { item, guard },
        other => close_sub(other, &mut effects),
    };
    if session.hot.as_ref() != Some(&path) {
        effects.insert(0, MenuTrackEffect::Highlight(Some(path.clone())));
    }
    keep(
        Session {
            hot: Some(path.clone()),
            cursor: Some(path),
            sub,
            ..session
        },
        effects,
    )
}

/// Open `path`'s submenu now, closing another item's first; the highlight moves to it.
fn expanded<K>(session: Session<K>, path: ItemPath) -> (MenuPhase<K>, Effects<K>) {
    let mut effects = Vec::new();
    if session.hot.as_ref() != Some(&path) {
        effects.push(MenuTrackEffect::Highlight(Some(path.clone())));
    }
    let sub = match session.sub {
        Submenu::Open { item, guard } if item == path => Submenu::Open { item, guard },
        other => {
            let _ = close_sub(other, &mut effects);
            effects.push(MenuTrackEffect::OpenSub(path.clone()));
            Submenu::Open {
                item: path.clone(),
                guard: None,
            }
        }
    };
    keep(
        Session {
            hot: Some(path.clone()),
            cursor: Some(path),
            sub,
            ..session
        },
        effects,
    )
}

/// The button came up over `target`.
fn released<K: PartialEq>(
    session: Session<K>,
    target: MenuTarget<K>,
) -> (MenuPhase<K>, Effects<K>) {
    match target {
        MenuTarget::Item {
            path,
            pick: Pickable::Enabled,
            ..
        } => close(vec![MenuTrackEffect::Pick(path)]),
        MenuTarget::Title(title) if title == session.menu && session.held != PressPhase::Idle => {
            // A release on the title, even after dragging onto it, makes the menu the person's
            // own.
            keep(
                Session {
                    shown: ShownBy::Press,
                    held: PressPhase::Idle,
                    ..session
                },
                Vec::new(),
            )
        }
        _ if session.held != PressPhase::Idle => close(Vec::new()),
        _ => keep(session, Vec::new()),
    }
}

/// Escape closes one level; Left and Right cross into a submenu or the adjacent menu; Enter
/// picks the highlighted item.
fn keyed<K>(session: Session<K>, key: MenuKey) -> (MenuPhase<K>, Effects<K>) {
    match (key, session.sub) {
        (MenuKey::Escape | MenuKey::Left, Submenu::Open { .. }) => keep(
            Session {
                sub: Submenu::None,
                ..session
            },
            vec![MenuTrackEffect::CloseSub],
        ),
        (MenuKey::Escape, _) => close(Vec::new()),
        (MenuKey::Right, Submenu::Pending { item, .. }) => {
            let effects = vec![MenuTrackEffect::OpenSub(item.clone())];
            let sub = Submenu::Open { item, guard: None };
            keep(Session { sub, ..session }, effects)
        }
        (MenuKey::Left, sub) => keep(
            Session { sub, ..session },
            vec![MenuTrackEffect::Adjacent(MenuDirection::Left)],
        ),
        (MenuKey::Right, sub) => keep(
            Session { sub, ..session },
            vec![MenuTrackEffect::Adjacent(MenuDirection::Right)],
        ),
        (MenuKey::Enter, sub) => match session.hot.clone() {
            Some(path) => close(vec![MenuTrackEffect::Pick(path)]),
            None => keep(Session { sub, ..session }, Vec::new()),
        },
    }
}
