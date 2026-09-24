//! A menu panel's submenu, driven by the menu-tracking machine (`MenuTrack`, design/13 sections
//! 13.3.4, 13.4 and 13.5): the 200 ms rest, the safe triangle and its timeout, Right, Enter and
//! Left. Every panel (the menu and each open submenu) owns one tracker for its own children, so
//! submenus nest to any depth with one machine per level and no second state machine.
//!
//! The tracker performs the machine's effects: a highlight moves the panel's selection, a
//! submenu opens once its parent row is measured (placed beside it with `place()`), a close
//! drops it, and a tick request becomes a timer. `Close`, `Pick` and `Adjacent` are the
//! panel's own business (a ds menu closes and picks itself), so they are ignored here.

use crate::components::menu_lines::{Act, Choice};
use crate::components::vocab::Availability;
use crate::geometry::measure::client_rect;
use crate::geometry::{MountedRef, Point, Px, Rect, Size};
use crate::overlay::menu_track::{
    Branch, ItemPath, MenuKey, MenuPhase, MenuTarget, MenuTiming, MenuTrack, MenuTrackEffect,
    MenuTrackEvent, Pickable, Submenu,
};
use crate::time::{FRAME_SLACK, sleep};
use dioxus::prelude::*;
use std::time::Instant;

/// How a submenu was asked for: by the keyboard it takes the focus, by the pointer it leaves
/// the focus where it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Via {
    /// A rest, or a click.
    Pointer,
    /// Right or Enter.
    Keyboard,
}

/// The submenu a panel shows: which choice it belongs to, where it is anchored, and how it
/// was asked for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct OpenSub {
    /// The parent choice.
    pub choice: usize,
    /// The rect the submenu is placed against: the parent panel's width, from the parent row's
    /// top less the panel padding (design/13 section 13.3.4, `y = item.top - 5`).
    pub anchor: Rect,
    /// The parent panel's rect, to tell which side the submenu ended on.
    pub panel: Rect,
    /// Pointer or keyboard.
    pub via: Via,
}

/// How many frames a submenu waits for its parent row to mount.
const ROW_WAITS: usize = 4;

/// One panel's tracker. Copy: every field is a handle.
#[derive(Clone, Copy)]
pub(crate) struct Tracker {
    track: CopyValue<MenuTrack<()>>,
    selected: Signal<usize>,
    open: Signal<Option<OpenSub>>,
    via: CopyValue<Via>,
    rows: CopyValue<Vec<Option<MountedRef>>>,
    panel: CopyValue<Option<MountedRef>>,
    pad: Px,
}

/// A tracker for a panel whose rows sit `pad` inside its edge, open in click mode.
pub(crate) fn use_tracker(timing: MenuTiming, pad: Px) -> Tracker {
    Tracker {
        track: use_hook(|| CopyValue::new(MenuTrack::open(timing, ()))),
        selected: use_signal(|| 0usize),
        open: use_signal(|| None),
        via: use_hook(|| CopyValue::new(Via::Pointer)),
        rows: use_hook(|| CopyValue::new(Vec::new())),
        panel: use_hook(|| CopyValue::new(None)),
        pad,
    }
}

/// The machine's view of choice `index`: its path, whether it can be picked, and whether it
/// opens a submenu.
pub(crate) fn target<T>(index: usize, choice: &Choice<T>) -> MenuTarget<()> {
    MenuTarget::Item {
        path: path(index),
        pick: match choice.availability {
            Availability::Enabled => Pickable::Enabled,
            Availability::Disabled => Pickable::Inert,
        },
        branch: match choice.act {
            Act::Pick(_) => Branch::Leaf,
            Act::Open(_) => Branch::Submenu,
        },
    }
}

fn path(index: usize) -> ItemPath {
    ItemPath(vec![u16::try_from(index).unwrap_or(u16::MAX)])
}

impl Tracker {
    /// The selected choice (not yet settled onto an enabled one).
    pub(crate) fn selected(&self) -> usize {
        (self.selected)()
    }

    /// Put the selection on `index` without telling the machine (a reset after typing).
    pub(crate) fn reset(&self, index: usize) {
        let mut selected = self.selected;
        selected.set(index);
    }

    /// The submenu shown, once its parent row is measured.
    pub(crate) fn open(&self) -> Option<OpenSub> {
        (self.open)()
    }

    /// Whether the machine has a submenu open (shown or still being measured).
    pub(crate) fn has_open(&self) -> bool {
        matches!(
            &self.track.peek().phase,
            MenuPhase::Tracking(session) if matches!(session.sub, Submenu::Open { .. })
        )
    }

    /// The panel element mounted: kept to measure it and to take the focus back.
    pub(crate) fn panel_mounted(&self, element: MountedRef) {
        let mut panel = self.panel;
        panel.set(Some(element));
    }

    /// Choice `index`'s row mounted: kept to measure it when its submenu opens.
    pub(crate) fn row_mounted(&self, index: usize, element: MountedRef) {
        let mut rows = self.rows;
        rows.with_mut(|rows| {
            if rows.len() <= index {
                rows.resize(index + 1, None);
            }
            rows[index] = Some(element);
        });
    }

    /// The keyboard moved the selection to `index`.
    pub(crate) fn select(&self, index: usize) {
        self.reset(index);
        self.feed(MenuTrackEvent::Select(path(index)));
    }

    /// The pointer is at `at` over `target`.
    pub(crate) fn point(&self, at: Point, target: MenuTarget<()>) {
        let mut via = self.via;
        via.set(Via::Pointer);
        self.feed(MenuTrackEvent::Move(at, target));
    }

    /// Open choice `index`'s submenu now.
    pub(crate) fn expand(&self, index: usize, how: Via) {
        let mut via = self.via;
        via.set(how);
        self.feed(MenuTrackEvent::Expand(path(index)));
    }

    /// Left or Escape with a submenu open: close it, and take the focus back from it.
    pub(crate) fn close_sub(&self) {
        self.feed(MenuTrackEvent::Key(MenuKey::Left));
        if let Some(panel) = self.panel.peek().clone() {
            // A frame later: the key that closed the submenu is still being handled, and the
            // renderer holds the document until it is (FINDINGS "W2 integration").
            spawn(async move {
                sleep(FRAME_SLACK).await;
                let _ = crate::focus::host::focus_element(&panel.0).await;
            });
        }
    }

    /// The open submenu landed at `sub`: arm the safe triangle toward its near edge.
    pub(crate) fn placed(&self, sub: Rect) {
        let Some(open) = *self.open.peek() else {
            return;
        };
        let centre = open.panel.left().0 + open.panel.size.width.0 / 2.0;
        let x = if sub.left().0 >= centre {
            sub.left()
        } else {
            sub.right()
        };
        self.feed(MenuTrackEvent::SubPlaced {
            top: Point { x, y: sub.top() },
            bottom: Point { x, y: sub.bottom() },
        });
    }

    fn feed(&self, event: MenuTrackEvent<()>) {
        let mut track = self.track;
        let current = track.peek().clone();
        let (next, effects) = current.step(event, Instant::now());
        track.set(next);
        for effect in effects {
            self.apply(effect);
        }
    }

    fn apply(&self, effect: MenuTrackEffect<()>) {
        match effect {
            MenuTrackEffect::Highlight(Some(ItemPath(steps))) => {
                if let Some(&index) = steps.first()
                    && *self.selected.peek() != usize::from(index)
                {
                    self.reset(usize::from(index));
                }
            }
            MenuTrackEffect::OpenSub(ItemPath(steps)) => {
                if let Some(&index) = steps.first() {
                    self.show(usize::from(index));
                }
            }
            MenuTrackEffect::CloseSub => {
                let mut open = self.open;
                if open.peek().is_some() {
                    open.set(None);
                }
            }
            MenuTrackEffect::RequestTick(at) => {
                let tracker = *self;
                spawn(async move {
                    sleep(at.saturating_duration_since(Instant::now())).await;
                    tracker.feed(MenuTrackEvent::Tick);
                });
            }
            MenuTrackEffect::Highlight(None)
            | MenuTrackEffect::Open(..)
            | MenuTrackEffect::Switch(_)
            | MenuTrackEffect::Close(_)
            | MenuTrackEffect::Pick(_)
            | MenuTrackEffect::Adjacent(_) => {}
        }
    }

    /// Measure choice `index`'s row and the panel, then show its submenu beside them, unless
    /// the machine closed it meanwhile.
    fn show(&self, index: usize) {
        let via = *self.via.peek();
        let tracker = *self;
        spawn(async move {
            let Some(row) = tracker.mounted_row(index).await else {
                return;
            };
            let Some(row) = client_rect(&row.0).await else {
                return;
            };
            let panel = tracker.panel.peek().clone();
            let panel = match panel {
                Some(panel) => client_rect(&panel.0).await.unwrap_or(row),
                None => row,
            };
            if !tracker.is_open_on(index) {
                return;
            }
            let anchor = Rect {
                origin: Point {
                    x: panel.left(),
                    y: row.top() - tracker.pad,
                },
                size: Size {
                    width: panel.size.width,
                    height: row.size.height,
                },
            };
            let mut open = tracker.open;
            open.set(Some(OpenSub {
                choice: index,
                anchor,
                panel,
                via,
            }));
        });
    }

    /// Choice `index`'s row element, waiting a few frames for it to mount (a submenu asked
    /// for as the menu mounts, before its rows exist).
    async fn mounted_row(&self, index: usize) -> Option<MountedRef> {
        for _ in 0..ROW_WAITS {
            if let Some(row) = self.rows.peek().get(index).cloned().flatten() {
                return Some(row);
            }
            sleep(FRAME_SLACK).await;
        }
        None
    }

    fn is_open_on(&self, index: usize) -> bool {
        matches!(
            &self.track.peek().phase,
            MenuPhase::Tracking(session)
                if matches!(&session.sub, Submenu::Open { item, .. } if *item == path(index))
        )
    }
}
