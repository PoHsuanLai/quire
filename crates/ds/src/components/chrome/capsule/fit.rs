//! What a capsule shows in the width it has. A capsule is as wide as its slots, and its stage can
//! be narrower than that, so [`fit`] drops the least important slots, a rank at a time, until what
//! is left fits: the way QuickTime and Preview thin their controls as the window narrows.
//!
//! It is a pure function of the slots and the stage width, so the same width always shows the
//! same slots and a slot that returns takes its old place: the result is one entry per input
//! slot, in the input's order.
//!
//! The widths are `capsule.css` read through its tokens: a button is a Large control square, a
//! gap is `--s-4`, the capsule pads `--s-8` each side and floats `--s-16` from the stage's
//! edges, a readout is at least `--s-36` wide, a divider is a hairline between `--s-4` margins,
//! the progress bar is at least [`SCRUB_LEAST`] and the level is 96, and a capsule that holds a
//! progress bar is at most 640 wide.

use super::model::CapsuleSlot;
use super::priority::{RankedSlot, SlotPriority};
use ds_style::tokens::control_size::ControlSize;

const S4: u32 = 4;
const S8: u32 = 8;
const S16: u32 = 16;
/// `--s-36`, the least width of a readout.
const READOUT_LEAST: u32 = 36;
/// A hairline, `--hair`.
const HAIR: u32 = 1;
/// The progress bar's least width, `.ds-capsule-scrub` `min-width`.
pub const SCRUB_LEAST: u32 = 120;
/// The level's width, `.ds-capsule-level` `width`.
const LEVEL: u32 = 96;
/// The widest a capsule that holds a progress bar grows, `.ds-capsule[data-span=wide]`.
const WIDE_MOST: u32 = 640;

/// Whether one slot is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotShown {
    /// Drawn.
    Shown,
    /// Not drawn: no room.
    Dropped,
}

/// How wide one slot draws, at its narrowest.
fn width_of<T>(slot: &CapsuleSlot<T>) -> u32 {
    match slot {
        CapsuleSlot::Item(_) => u32::from(ControlSize::Large.scale().height.0),
        CapsuleSlot::Readout(text) => {
            // Tabular figures, a little over half the font size each.
            let glyph = u32::from(ControlSize::Large.scale().font.0) * 6 / 10;
            let chars = u32::try_from(text.chars().count()).unwrap_or(u32::MAX);
            READOUT_LEAST.max(chars.saturating_mul(glyph).saturating_add(S4 * 2))
        }
        CapsuleSlot::Divider => HAIR + S4 * 2,
        CapsuleSlot::Scrub(_) => SCRUB_LEAST,
        CapsuleSlot::Level(_) => LEVEL,
    }
}

/// The last slot drawn so far, as far as dividers care.
#[derive(Clone, Copy, PartialEq)]
enum Last {
    Nothing,
    Divider(usize),
    Control,
}

/// `shown` with every divider that is at an end or beside another dropped too.
fn without_stray_dividers<T>(slots: &[RankedSlot<T>], shown: &[SlotShown]) -> Vec<SlotShown> {
    let mut tidy = shown.to_vec();
    let mut last = Last::Nothing;
    for (at, (one, state)) in slots.iter().zip(shown).enumerate() {
        if *state == SlotShown::Dropped {
            continue;
        }
        match (&one.slot, last) {
            (CapsuleSlot::Divider, Last::Control) => last = Last::Divider(at),
            (CapsuleSlot::Divider, _) => tidy[at] = SlotShown::Dropped,
            _ => last = Last::Control,
        }
    }
    if let Last::Divider(at) = last {
        tidy[at] = SlotShown::Dropped;
    }
    tidy
}

/// Whether the slots `shown` fit a stage `stage` pixels wide at their narrowest: the capsule's
/// padding, a gap between slots and each slot at its least, in the room the stage leaves, which
/// a capsule with a progress bar caps.
fn fits<T>(slots: &[RankedSlot<T>], shown: &[SlotShown], stage: u32) -> bool {
    let drawn: Vec<&CapsuleSlot<T>> = slots
        .iter()
        .zip(shown)
        .filter(|(_, state)| **state == SlotShown::Shown)
        .map(|(one, _)| &one.slot)
        .collect();
    let gaps = u32::try_from(drawn.len().saturating_sub(1)).unwrap_or(u32::MAX) * S4;
    let least = drawn
        .iter()
        .map(|slot| width_of(slot))
        .fold(S8 * 2 + gaps, u32::saturating_add);
    let room = stage.saturating_sub(S16 * 2);
    let capped = drawn
        .iter()
        .any(|slot| matches!(slot, CapsuleSlot::Scrub(_)))
        .then_some(WIDE_MOST);
    least <= capped.map_or(room, |most| room.min(most))
}

/// The droppable ranks in the order they go: the highest number first.
fn ranks<T>(slots: &[RankedSlot<T>]) -> Vec<u8> {
    let mut ranks: Vec<u8> = slots
        .iter()
        .filter_map(|one| match one.priority {
            SlotPriority::Droppable(rank) => Some(rank),
            SlotPriority::Essential => None,
        })
        .collect();
    ranks.sort_unstable_by(|a, b| b.cmp(a));
    ranks.dedup();
    ranks
}

/// Which of `slots` show on a stage `stage` pixels wide, one entry per slot in the same order.
/// A whole rank goes at once, the highest number first, until the rest fit (a group ranked
/// alike never thins piecemeal); dividers left with nothing to divide go with them. Essential
/// slots stay however narrow the stage, and with no width to go by (the stage is not measured
/// yet) every slot shows.
pub fn fit<T>(slots: &[RankedSlot<T>], stage: Option<u32>) -> Vec<SlotShown> {
    let mut shown = vec![SlotShown::Shown; slots.len()];
    let Some(stage) = stage else {
        return shown;
    };
    let mut tidy = without_stray_dividers(slots, &shown);
    for rank in ranks(slots) {
        if fits(slots, &tidy, stage) {
            break;
        }
        for (state, one) in shown.iter_mut().zip(slots) {
            if one.priority == SlotPriority::Droppable(rank) {
                *state = SlotShown::Dropped;
            }
        }
        tidy = without_stray_dividers(slots, &shown);
    }
    tidy
}

#[cfg(test)]
mod tests;
