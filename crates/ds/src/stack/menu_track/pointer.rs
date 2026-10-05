//! What the pointer does to a tracking session: the safe triangle first, then the highlight, the
//! submenu delay and the triangle's timeout (design/13 sections 13.3.4 and 13.5).

use super::machine::{Effects, keep};
use super::triangle::shielded;
use super::types::{
    Branch, Entered, ItemPath, MenuPhase, MenuTarget, MenuTiming, MenuTrackEffect, Pickable,
    SafeTriangle, Session, Submenu,
};
use ds_core::geometry::units::Point;
use ds_core::time::stamp::Stamp;

/// The open submenu's corners are known: arm its safe triangle from the last pointer sample.
pub(super) fn placed<K>(session: Session<K>, top: Point, bottom: Point) -> Session<K> {
    let sub = match session.sub {
        Submenu::Open { item, .. } => Submenu::Open {
            item,
            guard: Some(SafeTriangle {
                from: session.pointer,
                top,
                bottom,
                timeout: None,
            }),
        },
        other => other,
    };
    Session { sub, ..session }
}

/// A pointer sample: the safe triangle first, then highlight and submenu timing.
pub(super) fn moved<K: Clone + PartialEq>(
    session: Session<K>,
    at: Point,
    target: MenuTarget<K>,
    now: Stamp,
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
            timeout: Some(now.after_span(timing.triangle_timeout)),
            ..*guard
        };
        let sub = Submenu::Open {
            item: item.clone(),
            guard: Some(guard),
        };
        let session = Session {
            entered: Entered::Entered,
            sub,
            pointer: at,
            under: target,
            ..session
        };
        return keep(session, Vec::new());
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
    now: Stamp,
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
        cursor: hot.clone().or_else(|| session.cursor.clone()),
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
    now: Stamp,
    timing: MenuTiming,
    effects: &mut Effects<K>,
) -> Submenu {
    match (sub, branch) {
        (Submenu::Open { item, guard }, _) if &item == path => Submenu::Open { item, guard },
        (Submenu::Pending { item, until }, Branch::Submenu) if &item == path => {
            Submenu::Pending { item, until }
        }
        (sub, Branch::Submenu) => {
            let _ = close_sub(sub, effects);
            effects.push(MenuTrackEffect::Prepare(path.clone()));
            Submenu::Pending {
                item: path.clone(),
                until: now.after_span(timing.submenu_delay),
            }
        }
        (sub, Branch::Leaf) => close_sub(sub, effects),
    }
}

pub(super) fn close_sub<K>(sub: Submenu, effects: &mut Effects<K>) -> Submenu {
    if matches!(sub, Submenu::Open { .. }) {
        effects.push(MenuTrackEffect::CloseSub);
    }
    Submenu::None
}

/// A timer: the submenu delay, or the safe triangle's timeout.
pub(super) fn ticked<K: Clone>(
    session: Session<K>,
    now: Stamp,
    timing: MenuTiming,
) -> (MenuPhase<K>, Effects<K>) {
    match session.sub {
        Submenu::Pending { item, until } if now >= until => {
            let effects = vec![MenuTrackEffect::OpenSub(item.clone())];
            let sub = Submenu::Open { item, guard: None };
            keep(Session { sub, ..session }, effects)
        }
        Submenu::Open {
            item,
            guard: Some(guard),
        } if guard.timeout.is_some_and(|timeout| now >= timeout) => {
            // The pointer stopped inside the triangle: the item under it wins.
            let sub = Submenu::Open { item, guard: None };
            let (at, under) = (session.pointer, session.under.clone());
            let (session, effects) = retarget(Session { sub, ..session }, at, under, now, timing);
            keep(session, effects)
        }
        sub => keep(Session { sub, ..session }, Vec::new()),
    }
}
