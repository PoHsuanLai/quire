//! Placing a submenu beside its parent row (design/13 section 13.3.4, `y = item.top - 5`): the
//! row and the panel are measured once they hold still, and the submenu is shown at the result.
//!
//! The measuring starts when the pointer comes to rest on the parent row ([`Hands::prepare`]), not
//! when the delay ends, so a submenu opens the moment the delay does: the settle takes a few
//! frames, which would otherwise be added to the delay (a 200 ms rest would show the submenu at
//! about 540 ms). A submenu asked for with no rest (a key, a click) measures when it is asked.

use super::tracker::{Hands, OpenSub, path};
use crate::host::measure::{MountedRef, follow_rects};
use crate::stack::menu_track::types::{ItemPath, MenuTrack};
use dioxus::prelude::*;
use ds_core::geometry::units::{Point, Rect, Size};
use ds_core::time::{FRAME_SLACK, clock::sleep};
use ds_motion::machine::MachineRef;
use ds_style::task::spawn_in;

/// How many frames a submenu waits for its parent row to mount.
const ROW_WAITS: usize = 4;

/// Where a submenu goes, measured: the rect it is placed against and its parent panel's rect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Placing {
    anchor: Rect,
    panel: Rect,
}

/// The measuring of a submenu's place.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Warm {
    /// Nothing measured or being measured.
    Idle,
    /// A choice's row and the panel are being measured.
    Measuring(usize),
    /// A choice's place is known; its submenu has not opened yet.
    Ready(usize, Placing),
}

impl Hands {
    /// The pointer rests on choice `index`: measure its place while the delay runs.
    pub(super) fn prepare(self, index: usize, machine: MachineRef<MenuTrack<()>>) {
        let warm = *self.warm.peek();
        if !matches!(warm, Warm::Measuring(held) if held == index) {
            self.measure(index, machine);
        }
    }

    /// The machine opened choice `index`'s submenu: show it at the place measured while the
    /// pointer rested, or measure it now.
    pub(super) fn open(self, index: usize, machine: MachineRef<MenuTrack<()>>) {
        let warm = *self.warm.peek();
        match warm {
            Warm::Ready(held, placing) if held == index => self.show(index, placing),
            Warm::Measuring(held) if held == index => {}
            Warm::Idle | Warm::Measuring(_) | Warm::Ready(..) => self.measure(index, machine),
        }
    }

    /// A highlight moved off the measured choice: its place is no longer wanted.
    pub(super) fn forget_warm_but(self, highlighted: Option<&ItemPath>) {
        let mut warm = self.warm;
        let kept = *warm.peek();
        let held = match kept {
            Warm::Ready(held, _) => held,
            Warm::Idle | Warm::Measuring(_) => return,
        };
        if highlighted != Some(&path(held)) {
            warm.set(Warm::Idle);
        }
    }

    /// Measure choice `index`'s row and the panel, then place its submenu if the machine has it
    /// open by then, or keep the place for when it does.
    fn measure(self, index: usize, machine: MachineRef<MenuTrack<()>>) {
        let mut warm = self.warm;
        warm.set(Warm::Measuring(index));
        spawn_in(self.scope, async move {
            let placing = self.measured(index).await;
            let mut warm = self.warm;
            let Ok(state) = machine.state().try_peek().map(|state| state.clone()) else {
                return;
            };
            let next = match placing {
                Some(placing) if state.open_on(&path(index)) => {
                    self.show(index, placing);
                    Warm::Idle
                }
                Some(placing) if state.pending_on(&path(index)) => Warm::Ready(index, placing),
                Some(_) | None => Warm::Idle,
            };
            let _ = warm.try_write().map(|mut warm| *warm = next);
        });
    }

    /// Show choice `index`'s submenu at `placing`.
    fn show(self, index: usize, placing: Placing) {
        let mut placed = self.placed;
        let via = *self.via.peek();
        let _ = placed.try_write().map(|mut placed| {
            *placed = Some(OpenSub {
                choice: index,
                anchor: placing.anchor,
                panel: placing.panel,
                via,
            });
        });
        let mut warm = self.warm;
        let _ = warm.try_write().map(|mut warm| *warm = Warm::Idle);
    }

    /// Choice `index`'s row and the panel, once each holds still: a menu drawn in a card is
    /// measured while its surroundings are still settling (`follow_rects`, the one
    /// settle-until-stable for placement).
    async fn measured(self, index: usize) -> Option<Placing> {
        let row = self.mounted_row(index).await?;
        let panel = self.panel.peek().clone();
        let (row, panel) = follow_rects(&row.0, panel.as_ref().map(|panel| &*panel.0)).await?;
        let panel = panel.unwrap_or(row);
        let anchor = Rect {
            origin: Point {
                x: panel.left(),
                y: row.top() - self.pad,
            },
            size: Size {
                width: panel.size.width,
                height: row.size.height,
            },
        };
        Some(Placing { anchor, panel })
    }

    /// Choice `index`'s row element, waiting a few frames for it to mount (a submenu asked
    /// for as the menu mounts, before its rows exist).
    async fn mounted_row(self, index: usize) -> Option<MountedRef> {
        for _ in 0..ROW_WAITS {
            if let Some(row) = self.rows.peek().get(index).cloned().flatten() {
                return Some(row);
            }
            sleep(FRAME_SLACK).await;
        }
        None
    }
}
