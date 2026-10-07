//! How soon a capsule slot goes when the capsule is too narrow, and a slot paired with it.

use super::model::CapsuleSlot;

/// How soon a slot goes when the capsule does not fit its stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotPriority {
    /// Never dropped: the capsule is not worth showing without it.
    Essential,
    /// Dropped when the capsule does not fit; the higher the number, the sooner. Slots of one
    /// number go together, so a group never thins piecemeal.
    Droppable(u8),
}

/// A slot and how soon it goes.
#[derive(Debug, Clone, PartialEq)]
pub struct RankedSlot<T> {
    /// How soon it goes.
    pub priority: SlotPriority,
    /// What it shows.
    pub slot: CapsuleSlot<T>,
}

impl<T> RankedSlot<T> {
    /// `slot`, dropped at `priority`.
    pub fn new(priority: SlotPriority, slot: CapsuleSlot<T>) -> Self {
        Self { priority, slot }
    }
}

impl<T> From<CapsuleSlot<T>> for RankedSlot<T> {
    /// A slot that is never dropped.
    fn from(slot: CapsuleSlot<T>) -> Self {
        Self::new(SlotPriority::Essential, slot)
    }
}

impl<T> CapsuleSlot<T> {
    /// This slot, never dropped.
    pub fn essential(self) -> RankedSlot<T> {
        self.into()
    }

    /// This slot, dropped at `rank` (the higher, the sooner).
    pub fn droppable(self, rank: u8) -> RankedSlot<T> {
        RankedSlot::new(SlotPriority::Droppable(rank), self)
    }
}

/// `slots`, none of them ever dropped: a capsule that does not thin out.
pub fn essentials<T>(slots: Vec<CapsuleSlot<T>>) -> Vec<RankedSlot<T>> {
    slots.into_iter().map(RankedSlot::from).collect()
}
