//! The hover-card intent machine: wait for intent, stay warm, never mark read, never fetch
//! (design/06-INTERACTIONS.md section 3). Pure: an event and the time in, the next state and
//! one effect out.

use crate::style::tokens::delay::DelayToken;
use std::time::{Duration, Instant};

/// Which hover interface the pointer is resting on, and so how long it waits (design/30
/// section 1.2): the three profiles of one machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HoverProfile {
    /// A tooltip: opens after 1 s, closes at once.
    Tip,
    /// A hover card: opens after 500 ms, closes after 150 ms.
    #[default]
    Card,
    /// A dock label: opens after 100 ms, closes at once.
    Label,
}

impl HoverProfile {
    /// How long the pointer rests before it opens, cold.
    pub fn open(self) -> Duration {
        match self {
            HoverProfile::Tip => DelayToken::TipOpen.delay(),
            HoverProfile::Card => DelayToken::CardOpen.delay(),
            HoverProfile::Label => DelayToken::LabelOpen.delay(),
        }
    }

    /// How long after the pointer leaves it closes.
    pub fn close(self) -> Duration {
        match self {
            HoverProfile::Card => DelayToken::CardClose.delay(),
            HoverProfile::Tip | HoverProfile::Label => Duration::ZERO,
        }
    }
}

/// Whether cards and fly labels open at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HoverWarmth {
    /// A card is open or closed less than HoverWarm ago: no wait.
    Warm,
    /// Wait for intent.
    #[default]
    Cold,
}

/// Where the machine is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentPhase<K> {
    /// No card, nothing pending.
    Idle,
    /// The pointer rests on `key`; its card opens at `due`.
    Pending {
        /// The target under the pointer.
        key: K,
        /// When the open timer fires.
        due: Instant,
    },
    /// `key`'s card is open.
    Open {
        /// The open card's key.
        key: K,
    },
    /// The pointer left `key`'s target and card; it closes at `due`.
    Closing {
        /// The closing card's key.
        key: K,
        /// When the close timer fires.
        due: Instant,
    },
}

/// Something that happened to the hover machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoverEvent<K> {
    /// The pointer came over the innermost target `key`, whose interface is `profile`.
    Over(K, HoverProfile),
    /// The pointer came over a target while cards are suppressed (a leaving row, an open peek
    /// or palette).
    OverSuppressed,
    /// The pointer left the target for something that is neither the card nor the same target.
    Out,
    /// The pointer entered the open card.
    EnterCard,
    /// The pointer left the open card.
    LeaveCard,
    /// The open timer fired.
    OpenDue,
    /// The close timer fired.
    CloseDue,
    /// A click landed in the list (capture phase).
    ClickInList,
    /// Space was pressed with focus outside a text field.
    SpaceKey,
}

/// What the caller must do after a step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentEffect<K> {
    /// Nothing.
    None,
    /// Cancel any open timer and start one that fires after `after` (0 when warm).
    StartOpen {
        /// HoverOpen, or zero when warm.
        after: Duration,
    },
    /// Cancel the open timer.
    CancelOpen,
    /// Start the close timer (HoverClose).
    StartClose {
        /// HoverClose.
        after: Duration,
    },
    /// Cancel the close timer.
    CancelClose,
    /// Remove any open card instantly and open `key`'s card (`hc-in`).
    Open(K),
    /// Play `hc-out` on `key`'s card and unmount it at `settle(HcOut)`; the hub is warm.
    Close(K),
    /// Remove `key`'s card instantly, no exit, not warm.
    Remove(K),
    /// Close `key`'s card (warm) and open the peek for it.
    Peek(K),
}

/// The machine: a phase plus an independent warm deadline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverIntent<K> {
    phase: IntentPhase<K>,
    warm_until: Option<Instant>,
    profile: HoverProfile,
}

impl<K> Default for HoverIntent<K> {
    fn default() -> Self {
        HoverIntent {
            phase: IntentPhase::Idle,
            warm_until: None,
            profile: HoverProfile::default(),
        }
    }
}

impl<K> HoverIntent<K> {
    /// A machine in `phase`, on the default profile: `step` sets the profile after.
    fn of(phase: IntentPhase<K>, warm_until: Option<Instant>) -> Self {
        HoverIntent {
            phase,
            warm_until,
            profile: HoverProfile::default(),
        }
    }
}

impl<K: Clone + PartialEq> HoverIntent<K> {
    /// Apply `event` at `now` (the profile's open and close, 400 ms warm).
    ///
    /// Timer events carry no key, so a stale timer is recognised by its time: `OpenDue` and
    /// `CloseDue` act only once `now` has reached the phase's own `due`.
    pub fn step(self, event: HoverEvent<K>, now: Instant) -> (Self, IntentEffect<K>) {
        let warm = self.warmth(now);
        let held = self.profile;
        let profile = match &event {
            HoverEvent::Over(_, profile) => *profile,
            _ => held,
        };
        let HoverIntent {
            phase, warm_until, ..
        } = self;
        let keep = |phase| (HoverIntent::of(phase, warm_until), IntentEffect::None);
        let (next, effect) = match (phase, event) {
            (phase, HoverEvent::OverSuppressed) => keep(phase),
            (phase, HoverEvent::Over(key, profile)) => {
                over(phase, key, profile, warm_until, warm, now)
            }
            (IntentPhase::Pending { .. }, HoverEvent::Out) => {
                to(IntentPhase::Idle, warm_until, IntentEffect::CancelOpen)
            }
            (IntentPhase::Open { key }, HoverEvent::Out | HoverEvent::LeaveCard) => {
                let after = held.close();
                let due = now + after;
                to(
                    IntentPhase::Closing { key, due },
                    warm_until,
                    IntentEffect::StartClose { after },
                )
            }
            (IntentPhase::Closing { key, .. }, HoverEvent::EnterCard) => to(
                IntentPhase::Open { key },
                warm_until,
                IntentEffect::CancelClose,
            ),
            (IntentPhase::Pending { key, due }, HoverEvent::OpenDue) if now >= due => to(
                IntentPhase::Open { key: key.clone() },
                warm_until,
                IntentEffect::Open(key),
            ),
            (IntentPhase::Closing { key, due }, HoverEvent::CloseDue) if now >= due => to(
                IntentPhase::Idle,
                Some(now + DelayToken::HoverWarm.delay()),
                IntentEffect::Close(key),
            ),
            (
                IntentPhase::Open { key } | IntentPhase::Closing { key, .. },
                HoverEvent::ClickInList,
            ) => to(IntentPhase::Idle, None, IntentEffect::Remove(key)),
            (IntentPhase::Pending { .. }, HoverEvent::ClickInList) => {
                to(IntentPhase::Idle, warm_until, IntentEffect::CancelOpen)
            }
            (IntentPhase::Open { key }, HoverEvent::SpaceKey) => to(
                IntentPhase::Idle,
                Some(now + DelayToken::HoverWarm.delay()),
                IntentEffect::Peek(key),
            ),
            (phase, _) => keep(phase),
        };
        (HoverIntent { profile, ..next }, effect)
    }

    /// Where the machine is.
    pub fn phase(&self) -> &IntentPhase<K> {
        &self.phase
    }

    /// Whether cards open at once: a card is open, or one closed less than HoverWarm ago.
    pub fn warmth(&self, now: Instant) -> HoverWarmth {
        let open = matches!(
            self.phase,
            IntentPhase::Open { .. } | IntentPhase::Closing { .. }
        );
        let lingering = self.warm_until.is_some_and(|until| now < until);
        if open || lingering {
            HoverWarmth::Warm
        } else {
            HoverWarmth::Cold
        }
    }
}

/// The pointer came over `key`: keep the card when it is the same key, else wait for intent
/// (none when warm).
fn over<K: Clone + PartialEq>(
    phase: IntentPhase<K>,
    key: K,
    profile: HoverProfile,
    warm_until: Option<Instant>,
    warm: HoverWarmth,
    now: Instant,
) -> (HoverIntent<K>, IntentEffect<K>) {
    match phase {
        IntentPhase::Open { key: open } if open == key => to(
            IntentPhase::Open { key: open },
            warm_until,
            IntentEffect::None,
        ),
        IntentPhase::Closing { key: open, .. } if open == key => to(
            IntentPhase::Open { key: open },
            warm_until,
            IntentEffect::CancelClose,
        ),
        IntentPhase::Pending { key: pending, due } if pending == key => to(
            IntentPhase::Pending { key: pending, due },
            warm_until,
            IntentEffect::None,
        ),
        _ => {
            let after = match warm {
                HoverWarmth::Warm => Duration::ZERO,
                HoverWarmth::Cold => profile.open(),
            };
            to(
                IntentPhase::Pending {
                    key,
                    due: now + after,
                },
                warm_until,
                IntentEffect::StartOpen { after },
            )
        }
    }
}

fn to<K>(
    phase: IntentPhase<K>,
    warm_until: Option<Instant>,
    effect: IntentEffect<K>,
) -> (HoverIntent<K>, IntentEffect<K>) {
    (HoverIntent::of(phase, warm_until), effect)
}
