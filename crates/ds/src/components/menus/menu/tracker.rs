//! A menu panel's submenu, driven by the menu-tracking machine (`MenuTrack`, design/13 sections
//! 13.3.4, 13.4 and 13.5): the 200 ms rest, the safe triangle and its timeout, Right, Enter and
//! Left. Every panel (the menu and each open submenu) owns one tracker for its own children, so
//! submenus nest to any depth with one machine per level and no second state machine.
//!
//! The machine is the one truth of which choice is highlighted and which submenu is open; the
//! tracker reads both from it. What it keeps besides is what a machine cannot know: the panel's
//! elements, and where the open submenu was measured to go ([`OpenSub`], shown only while the
//! machine says that submenu is open). It performs the machine's effects: a rest on a parent row
//! measures it while the delay runs, an open places the submenu beside that row, and a close
//! forgets the placement. `Close`, `Pick` and `Adjacent` are the panel's own business (a ds menu
//! closes and picks itself), so they are ignored here.

use crate::components::menus::menu::choices::{Act, Choice};
use crate::components::menus::menu::placing::Warm;
use crate::host::measure::MountedRef;
use crate::stack::menu_track::types::{
    Branch, ItemPath, MenuKey, MenuTarget, MenuTiming, MenuTrack, MenuTrackEffect, MenuTrackEvent,
    Pickable, Submenu,
};
use crate::stack::typeahead::Typeahead;
use dioxus::core::{ScopeId, current_scope_id};
use dioxus::prelude::*;
use ds_core::geometry::units::{Point, Px, Rect};
use ds_core::time::{
    FRAME_SLACK,
    clock::{now, sleep},
};
use ds_core::vocab::Availability;
use ds_motion::machine::{MachineRef, use_machine};

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

/// One panel's tracker. Copy: every field is a handle.
#[derive(Clone, Copy)]
pub(crate) struct Tracker {
    machine: MachineRef<MenuTrack<()>>,
    hands: Hands,
    typeahead: CopyValue<Typeahead>,
}

/// What the machine's effects work on: handles only, so the effect handler owns no tracker.
#[derive(Clone, Copy)]
pub(super) struct Hands {
    pub(super) placed: Signal<Option<OpenSub>>,
    pub(super) warm: CopyValue<Warm>,
    pub(super) via: CopyValue<Via>,
    pub(super) rows: CopyValue<Vec<Option<MountedRef>>>,
    pub(super) panel: CopyValue<Option<MountedRef>>,
    pub(super) scope: ScopeId,
    pub(super) pad: Px,
}

/// A tracker for a panel whose rows sit `pad` inside its edge, open in click mode.
pub(crate) fn use_tracker(timing: MenuTiming, pad: Px) -> Tracker {
    let hands = Hands {
        placed: use_signal(|| None),
        warm: use_hook(|| CopyValue::new(Warm::Idle)),
        via: use_hook(|| CopyValue::new(Via::Pointer)),
        rows: use_hook(|| CopyValue::new(Vec::new())),
        panel: use_hook(|| CopyValue::new(None)),
        scope: use_hook(current_scope_id),
        pad,
    };
    Tracker {
        machine: use_machine(
            |_| MenuTrack::open(()),
            timing,
            || (),
            move |effect, machine| hands.apply(effect, machine),
        ),
        hands,
        typeahead: use_hook(|| CopyValue::new(Typeahead::default())),
    }
}

/// The machine's view of choice `index`: its path, whether it can be picked, and whether it
/// opens a submenu.
pub(crate) fn target<T>(index: usize, choice: &Choice<T>) -> MenuTarget<()> {
    MenuTarget::Item {
        path: path(index),
        pick: match choice.availability {
            Availability::Enabled => Pickable::Enabled,
            Availability::Disabled | Availability::Busy => Pickable::Inert,
        },
        branch: match choice.act {
            Act::Pick(..) => Branch::Leaf,
            Act::Open(_) => Branch::Submenu,
        },
    }
}

pub(super) fn path(index: usize) -> ItemPath {
    ItemPath(vec![u16::try_from(index).unwrap_or(u16::MAX)])
}

/// The choice an item path names at this panel's level.
fn choice_of(path: &ItemPath) -> Option<usize> {
    path.0.first().copied().map(usize::from)
}

impl Tracker {
    /// Type `text`: the choice, among those labelled `labels`, the selection jumps to.
    pub(crate) fn typed(&self, text: &str, labels: &[&str], from: usize) -> Option<usize> {
        let (next, found) = self
            .typeahead
            .peek()
            .clone()
            .typed(text, now(), labels, from);
        let mut held = self.typeahead;
        held.set(next);
        found
    }

    /// The selected choice (not yet settled onto an enabled one): where the machine's cursor
    /// stands, the first choice before anything was highlighted.
    pub(crate) fn selected(&self) -> usize {
        self.machine
            .state()
            .read()
            .cursor()
            .and_then(choice_of)
            .unwrap_or(0)
    }

    /// The submenu shown: its placement, once its parent row is measured and for as long as the
    /// machine has that submenu open.
    pub(crate) fn open(&self) -> Option<OpenSub> {
        let placed = (self.hands.placed)()?;
        let open = self.opens(placed.choice);
        open.then_some(placed)
    }

    /// Whether the machine has choice `index`'s submenu open (shown or still being measured).
    fn opens(&self, index: usize) -> bool {
        self.machine.state().read().open_on(&path(index))
    }

    /// Whether the machine has a submenu open (shown or still being measured).
    pub(crate) fn has_open(&self) -> bool {
        matches!(
            self.machine.state().read().submenu(),
            Some(Submenu::Open { .. })
        )
    }

    /// The panel element mounted: kept to measure it and to take the focus back.
    pub(crate) fn panel_mounted(&self, element: MountedRef) {
        let mut panel = self.hands.panel;
        panel.set(Some(element));
    }

    /// Choice `index`'s row mounted: kept to measure it when its submenu opens.
    pub(crate) fn row_mounted(&self, index: usize, element: MountedRef) {
        let mut rows = self.hands.rows;
        rows.with_mut(|rows| {
            if rows.len() <= index {
                rows.resize(index + 1, None);
            }
            rows[index] = Some(element);
        });
    }

    /// The keyboard moved the selection to `index`.
    pub(crate) fn select(&self, index: usize) {
        self.machine.send(MenuTrackEvent::Select(path(index)));
    }

    /// The pointer is at `at` over `target`.
    pub(crate) fn point(&self, at: Point, target: MenuTarget<()>) {
        let mut via = self.hands.via;
        via.set(Via::Pointer);
        self.machine.send(MenuTrackEvent::Move(at, target));
    }

    /// Open choice `index`'s submenu now.
    pub(crate) fn expand(&self, index: usize, how: Via) {
        let mut via = self.hands.via;
        via.set(how);
        self.machine.send(MenuTrackEvent::Expand(path(index)));
    }

    /// Left or Escape with a submenu open: close it, and take the focus back from it.
    pub(crate) fn close_sub(&self) {
        self.machine.send(MenuTrackEvent::Key(MenuKey::Left));
        self.refocus();
    }

    /// Give the panel the focus back, a frame from now: the key or click that asked for it is
    /// still being handled, and the renderer holds the document until it is. Used after a submenu
    /// closes, and after a pick that keeps the menu open
    /// (Blitz ends a click on a plain element by clearing the focus).
    pub(crate) fn refocus(&self) {
        if let Some(panel) = self.hands.panel.peek().clone() {
            spawn(async move {
                sleep(FRAME_SLACK).await;
                let _ = crate::focus::soon::focus_element(&panel.0).await;
            });
        }
    }

    /// The open submenu landed at `sub`: arm the safe triangle toward its near edge.
    pub(crate) fn placed(&self, sub: Rect) {
        let Some(open) = *self.hands.placed.peek() else {
            return;
        };
        let centre = open.panel.left().0 + open.panel.size.width.0 / 2.0;
        let x = if sub.left().0 >= centre {
            sub.left()
        } else {
            sub.right()
        };
        self.machine.send(MenuTrackEvent::SubPlaced {
            top: Point { x, y: sub.top() },
            bottom: Point { x, y: sub.bottom() },
        });
    }
}

impl Hands {
    /// Carry out one effect of the machine.
    fn apply(self, effect: MenuTrackEffect<()>, machine: MachineRef<MenuTrack<()>>) {
        match effect {
            MenuTrackEffect::Highlight(highlighted) => self.forget_warm_but(highlighted.as_ref()),
            MenuTrackEffect::Prepare(item) => {
                if let Some(index) = choice_of(&item) {
                    self.prepare(index, machine);
                }
            }
            MenuTrackEffect::OpenSub(item) => {
                if let Some(index) = choice_of(&item) {
                    self.open(index, machine);
                }
            }
            MenuTrackEffect::CloseSub => {
                let mut placed = self.placed;
                if placed.peek().is_some() {
                    placed.set(None);
                }
            }
            MenuTrackEffect::Open(..)
            | MenuTrackEffect::Check(_)
            | MenuTrackEffect::Close(_)
            | MenuTrackEffect::Pick(_)
            | MenuTrackEffect::Adjacent(_) => {}
        }
    }
}
