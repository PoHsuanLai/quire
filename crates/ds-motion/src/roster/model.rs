//! The roster's state, inputs, outputs and settings.

use super::super::presence::{Exit, Presence};
use ds_core::geometry::units::Px;
use ds_core::machine::Elapsed;
use ds_core::time::stamp::Stamp;
use ds_style::appearance::motion::MotionLevel;

/// A row's height plus the gap below it: how far the rows below move when it goes.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct RowPitch(pub Px);

/// A row sliding up into the gap a removed row left: `data-presence="healing"`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Heal {
    /// How far it starts below its resting place.
    pub dy: Px,
}

/// The `data-presence` word of a row: `healing` while it heals, else its presence's.
pub fn presence_slug(presence: Presence, heal: Option<Heal>) -> &'static str {
    match heal {
        Some(_) => "healing",
        None => presence.slug(),
    }
}

/// One row the roster is drawing.
#[derive(Debug, Clone, PartialEq)]
pub struct RosterEntry<K> {
    /// The consumer's key.
    pub key: K,
    /// Its motion state.
    pub presence: Presence,
    /// Its heal, while it slides into a gap; only a present row heals.
    pub heal: Option<Heal>,
    /// When the exit it plays has settled and it is dropped; set exactly while it leaves.
    pub until: Option<Stamp>,
}

/// Every row on screen, in order, with its motion state, the keys the consumer listed last, and
/// when the entering and healing rows have settled.
#[derive(Debug, Clone, PartialEq)]
pub struct RosterState<K> {
    pub(super) entries: Vec<RosterEntry<K>>,
    pub(super) listed: Vec<K>,
    pub(super) rest_due: Option<Stamp>,
}

impl<K: Clone + PartialEq> RosterState<K> {
    /// A roster showing `keys` for the first time: every row simply there (nothing sweeps in on
    /// first show).
    pub fn first_show(keys: &[K]) -> Self {
        let entries = keys
            .iter()
            .map(|key| RosterEntry {
                key: key.clone(),
                presence: Presence::Present,
                heal: None,
                until: None,
            })
            .collect();
        RosterState {
            entries,
            listed: keys.to_vec(),
            rest_due: None,
        }
    }

    /// The rows to draw, leaving ones included.
    pub fn entries(&self) -> &[RosterEntry<K>] {
        &self.entries
    }
}

/// What a [`RosterState::stay`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stayed {
    /// The row was leaving and is present again.
    Restored,
    /// The row was not leaving (entering, present or healing); nothing changed.
    Unchanged,
}

/// Why a stay could not happen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StayError {
    /// The roster holds no row with that key: never listed, or already dropped after its exit.
    UnknownKey,
    /// The roster's owner is gone (the list unmounted); only the hook reports this.
    Unmounted,
}

/// How a row leaves the roster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LeaveBy {
    /// The consumer asks: `Roster::leave` plays the exit, and a key the consumer stops listing
    /// without asking is dropped at once.
    Action,
    /// The consumer stops listing the row: it plays the exit and stays drawn until it settles.
    Delist,
}

/// What moves the roster.
#[derive(Debug, Clone, PartialEq)]
pub enum RosterIn<K> {
    /// The consumer's keys, passed on every render. New keys enter; a key listed again while it
    /// leaves stays where it is; a key that goes missing leaves as [`RosterParams::leave`] says
    /// (keys missing at once are one batch). The same keys again change nothing.
    List(Vec<K>),
    /// Start the exits of these keys together, as one batch (an archive, a Clear, a group
    /// collapsing).
    Leave(Vec<K>),
    /// Take this leaving row's exit back.
    Stay(K),
    /// A deadline came due.
    Elapsed,
}

impl<K> From<Elapsed> for RosterIn<K> {
    fn from(_: Elapsed) -> Self {
        RosterIn::Elapsed
    }
}

/// What the roster's owner does after a step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RosterOut<K> {
    /// This key's exit has settled and the row is dropped (its batch's keys come out together).
    Settled(K),
}

/// What a roster does with its rows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RosterParams {
    /// How a row leaves.
    pub leave: LeaveBy,
    /// The exit its rows play, read when a batch starts.
    pub exit: Exit,
    /// How far the rows below heal for a row that measured nothing.
    pub pitch: RowPitch,
    /// The motion level the settles are measured at.
    pub motion: MotionLevel,
}

/// The heights rows measured, by key: how far the rows below heal when one leaves. The caller
/// provides it as the roster's context at each step.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Measured<K>(pub Vec<(K, RowPitch)>);

impl<K: PartialEq> Measured<K> {
    /// `key`'s measured pitch, if a row measured one.
    pub fn of(&self, key: &K) -> Option<RowPitch> {
        self.0
            .iter()
            .find(|(held, _)| held == key)
            .map(|(_, pitch)| *pitch)
    }
}
