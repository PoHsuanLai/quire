//! The tracker's transitions (design/13 section 13.4's table).

use super::triangle::shielded;
use super::types::{
    Branch, Entered, Held, ItemPath, MenuAnim, MenuDirection, MenuKey, MenuPhase, MenuTarget,
    MenuTiming, MenuTrack, MenuTrackEffect, MenuTrackEvent, Pickable, SafeTriangle, Session,
    Submenu,
};
use crate::geometry::Point;
use std::time::Instant;

/// The effects of one step, in order.
type Effects<K> = Vec<MenuTrackEffect<K>>;

impl<K: Clone + PartialEq> MenuTrack<K> {
    /// A closed tracker with these timings.
    pub fn new(timing: MenuTiming) -> Self {
        MenuTrack {
            timing,
            phase: MenuPhase::Closed,
        }
    }

    /// A tracker already tracking `menu` in click mode: a menu that is open by the time the
    /// tracker exists (a ds `Menu` mounts open), so the opening press is over.
    pub fn open(timing: MenuTiming, menu: K) -> Self {
        MenuTrack {
            timing,
            phase: opened(menu, Held::Released),
        }
    }

    /// The open menu's key, if one is open.
    pub fn open_menu(&self) -> Option<&K> {
        match &self.phase {
            MenuPhase::Closed => None,
            MenuPhase::Tracking(session) => Some(&session.menu),
        }
    }

    /// The tracker after `event` at `now`, and what the caller must do, in order.
    pub fn step(self, event: MenuTrackEvent<K>, now: Instant) -> (Self, Effects<K>) {
        let MenuTrack { timing, phase } = self;
        let (phase, effects) = match phase {
            MenuPhase::Closed => closed(event),
            MenuPhase::Tracking(session) => tracking(session, event, now, timing),
        };
        (MenuTrack { timing, phase }, effects)
    }
}

/// Nothing open: only a press on a title does anything.
fn closed<K: Clone>(event: MenuTrackEvent<K>) -> (MenuPhase<K>, Effects<K>) {
    match event {
        MenuTrackEvent::PressTitle(menu) => {
            let effects = vec![MenuTrackEffect::Open(menu.clone(), MenuAnim::Pop)];
            (opened(menu, Held::Held), effects)
        }
        _ => (MenuPhase::Closed, Vec::new()),
    }
}

/// A fresh session on `menu`.
fn opened<K>(menu: K, held: Held) -> MenuPhase<K> {
    MenuPhase::Tracking(Session {
        menu,
        held,
        entered: Entered::NotYet,
        hot: None,
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

fn keep<K>(session: Session<K>, effects: Effects<K>) -> (MenuPhase<K>, Effects<K>) {
    (MenuPhase::Tracking(session), effects)
}

/// A menu is open.
fn tracking<K: Clone + PartialEq>(
    session: Session<K>,
    event: MenuTrackEvent<K>,
    now: Instant,
    timing: MenuTiming,
) -> (MenuPhase<K>, Effects<K>) {
    match event {
        MenuTrackEvent::PressTitle(other) if other == session.menu => close(Vec::new()),
        MenuTrackEvent::PressTitle(other) => {
            let effects = vec![MenuTrackEffect::Switch(other.clone())];
            (opened(other, Held::Held), effects)
        }
        MenuTrackEvent::OutsidePress => close(Vec::new()),
        MenuTrackEvent::Release(target) => released(session, target),
        MenuTrackEvent::Move(_, MenuTarget::Title(other)) if other != session.menu => {
            let effects = vec![MenuTrackEffect::Switch(other.clone())];
            (opened(other, session.held), effects)
        }
        MenuTrackEvent::Move(at, target) => moved(session, at, target, now, timing),
        MenuTrackEvent::SubPlaced { top, bottom } => {
            keep(placed(session, top, bottom, now), vec![])
        }
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
            hot: Some(path),
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
            hot: Some(path),
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
        MenuTarget::Title(title) if title == session.menu && session.held == Held::Held => keep(
            Session {
                held: Held::Released,
                ..session
            },
            Vec::new(),
        ),
        _ if session.held == Held::Held => close(Vec::new()),
        _ => keep(session, Vec::new()),
    }
}

/// The open submenu's corners are known: arm its safe triangle from the last pointer sample.
fn placed<K>(session: Session<K>, top: Point, bottom: Point, now: Instant) -> Session<K> {
    let sub = match session.sub {
        Submenu::Open { item, .. } => Submenu::Open {
            item,
            guard: Some(SafeTriangle {
                from: session.pointer,
                top,
                bottom,
                still_since: now,
            }),
        },
        other => other,
    };
    Session { sub, ..session }
}

/// A pointer sample: the safe triangle first, then highlight and submenu timing.
fn moved<K: Clone + PartialEq>(
    session: Session<K>,
    at: Point,
    target: MenuTarget<K>,
    now: Instant,
    timing: MenuTiming,
) -> (MenuPhase<K>, Effects<K>) {
    if let Submenu::Open {
        item,
        guard: Some(guard),
    } = &session.sub
        && shielded(guard, at)
    {
        let guard = SafeTriangle {
            from: at,
            still_since: now,
            ..*guard
        };
        let sub = Submenu::Open {
            item: item.clone(),
            guard: Some(guard),
        };
        let effects = vec![MenuTrackEffect::RequestTick(now + timing.triangle_timeout)];
        let session = Session {
            entered: Entered::Entered,
            sub,
            pointer: at,
            under: target,
            ..session
        };
        return keep(session, effects);
    }
    let (session, effects) = retarget(session, at, target, now, timing);
    keep(session, effects)
}

/// The pointer is at `at` over `target`, outside any safe triangle: highlight and submenus
/// follow it.
fn retarget<K: Clone>(
    session: Session<K>,
    at: Point,
    target: MenuTarget<K>,
    now: Instant,
    timing: MenuTiming,
) -> (Session<K>, Effects<K>) {
    let mut effects = Vec::new();
    let entered = match target {
        MenuTarget::Item { .. } | MenuTarget::Menu => Entered::Entered,
        _ => session.entered,
    };
    let (hot, sub) = match &target {
        MenuTarget::Item {
            path,
            pick: Pickable::Enabled,
            branch,
        } => {
            let sub = select(session.sub, path, *branch, now, timing, &mut effects);
            (Some(path.clone()), sub)
        }
        MenuTarget::Item {
            pick: Pickable::Inert,
            ..
        } => (None, close_sub(session.sub, &mut effects)),
        _ => (session.hot.clone(), session.sub),
    };
    if hot != session.hot {
        effects.insert(0, MenuTrackEffect::Highlight(hot.clone()));
    }
    let session = Session {
        entered,
        hot,
        sub,
        pointer: at,
        under: target,
        ..session
    };
    (session, effects)
}

/// The submenu state after the pointer lands on the pickable item `path`.
fn select<K>(
    sub: Submenu,
    path: &ItemPath,
    branch: Branch,
    now: Instant,
    timing: MenuTiming,
    effects: &mut Effects<K>,
) -> Submenu {
    match (sub, branch) {
        (Submenu::Open { item, guard }, _) if &item == path => Submenu::Open { item, guard },
        (Submenu::Pending { item, since }, Branch::Submenu) if &item == path => {
            Submenu::Pending { item, since }
        }
        (sub, Branch::Submenu) => {
            let _ = close_sub(sub, effects);
            effects.push(MenuTrackEffect::RequestTick(now + timing.submenu_delay));
            Submenu::Pending {
                item: path.clone(),
                since: now,
            }
        }
        (sub, Branch::Leaf) => close_sub(sub, effects),
    }
}

fn close_sub<K>(sub: Submenu, effects: &mut Effects<K>) -> Submenu {
    if matches!(sub, Submenu::Open { .. }) {
        effects.push(MenuTrackEffect::CloseSub);
    }
    Submenu::None
}

/// A timer: the submenu delay, or the safe triangle's timeout.
fn ticked<K: Clone>(
    session: Session<K>,
    now: Instant,
    timing: MenuTiming,
) -> (MenuPhase<K>, Effects<K>) {
    match session.sub {
        Submenu::Pending { item, since } if now.duration_since(since) >= timing.submenu_delay => {
            let effects = vec![MenuTrackEffect::OpenSub(item.clone())];
            let sub = Submenu::Open { item, guard: None };
            keep(Session { sub, ..session }, effects)
        }
        Submenu::Open {
            item,
            guard: Some(guard),
        } if now.duration_since(guard.still_since) >= timing.triangle_timeout => {
            // The pointer stopped inside the triangle: the item under it wins.
            let sub = Submenu::Open { item, guard: None };
            let (at, under) = (session.pointer, session.under.clone());
            let (session, effects) = retarget(Session { sub, ..session }, at, under, now, timing);
            keep(session, effects)
        }
        sub => keep(Session { sub, ..session }, Vec::new()),
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
