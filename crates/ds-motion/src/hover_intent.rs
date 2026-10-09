//! The hover-card intent machine: wait for intent, stay warm, never mark read, never fetch
//! (design/06-INTERACTIONS.md section 3). A `Machine`: an event and the time in, the next state
//! and the effects out; the open and close deadlines are in the state, so its wake is the one
//! timer and a stale wake changes nothing.

use ds_core::machine::{Elapsed, Machine};
use ds_core::time::stamp::Stamp;
use ds_style::tokens::delay::{DelayToken, TipDelay};
use std::time::Duration;

/// Which hover interface the pointer is resting on, and so how long it waits (design/30
/// section 1.2): the three profiles of one machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HoverProfile {
    /// A tooltip: opens at once by default ([`TipDelay::Immediate`]) or after 1 s under
    /// [`TipDelay::Standard`]; closes at once.
    Tip,
    /// A hover card: opens after 500 ms, closes after 150 ms.
    #[default]
    Card,
    /// A dock label: opens after 100 ms, closes at once.
    Label,
}

impl HoverProfile {
    /// How long the pointer rests before it opens, cold; a tip waits as `tip` says.
    pub fn open(self, tip: TipDelay) -> Duration {
        match self {
            HoverProfile::Tip => tip.open(),
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
        due: Stamp,
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
        due: Stamp,
    },
}

/// Something that happened to the hover machine.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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
    /// The deadline asked for by `wake` came due: the open or the close, whichever the phase
    /// is waiting on.
    Elapsed,
    /// A click landed in the list (capture phase).
    ClickInList,
    /// Space was pressed with focus outside a text field.
    SpaceKey,
}

impl<K> From<Elapsed> for HoverEvent<K> {
    fn from(_: Elapsed) -> Self {
        HoverEvent::Elapsed
    }
}

/// What the caller must do after a step. The timers are not among them: the state's deadlines
/// are the machine's `wake`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum IntentEffect<K> {
    /// Remove any open card instantly and open `key`'s card.
    Open(K),
    /// Fade `key`'s card out (`Exit::Fade`) and unmount it once that settles; the hub is warm.
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
    warm_until: Option<Stamp>,
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
    fn of(phase: IntentPhase<K>, warm_until: Option<Stamp>) -> Self {
        HoverIntent {
            phase,
            warm_until,
            profile: HoverProfile::default(),
        }
    }
}

impl<K: Clone + PartialEq + 'static> Machine for HoverIntent<K> {
    type In = HoverEvent<K>;
    type Out = IntentEffect<K>;
    /// The delays are the profile's and the tokens'; the app chooses how long a tip waits.
    type Params = TipDelay;
    type Ctx = ();

    /// Apply `event` at `now` (the profile's open and close, 400 ms warm).
    ///
    /// A wake carries no key, so a stale one is recognised by its time: `Elapsed` acts only once
    /// `now` has reached the phase's own `due`.
    fn step(
        self,
        event: HoverEvent<K>,
        now: Stamp,
        tip: &TipDelay,
        _: &(),
    ) -> (Self, Vec<IntentEffect<K>>) {
        let (next, effect) = self.advance(event, now, *tip);
        (next, effect.into_iter().collect())
    }

    fn wake(&self) -> Option<Stamp> {
        match self.phase {
            IntentPhase::Pending { due, .. } | IntentPhase::Closing { due, .. } => Some(due),
            IntentPhase::Idle | IntentPhase::Open { .. } => None,
        }
    }
}

/// A step's effect, when it has one.
type Effect<K> = Option<IntentEffect<K>>;

impl<K: Clone + PartialEq> HoverIntent<K> {
    fn advance(self, event: HoverEvent<K>, now: Stamp, tip: TipDelay) -> (Self, Effect<K>) {
        let warm = self.warmth(now);
        let held = self.profile;
        let profile = match &event {
            HoverEvent::Over(_, profile) => *profile,
            _ => held,
        };
        let HoverIntent {
            phase, warm_until, ..
        } = self;
        let keep = |phase| to(phase, warm_until, None);
        let lingers = Some(now.after_span(DelayToken::HoverWarm.delay()));
        let (next, effect) = match (phase, event) {
            (phase, HoverEvent::OverSuppressed) => keep(phase),
            (phase, HoverEvent::Over(key, profile)) => {
                over(phase, key, profile.open(tip), warm_until, warm, now)
            }
            (IntentPhase::Pending { .. }, HoverEvent::Out) => keep(IntentPhase::Idle),
            (IntentPhase::Open { key }, HoverEvent::Out | HoverEvent::LeaveCard) => to(
                IntentPhase::Closing {
                    key,
                    due: now.after_span(held.close()),
                },
                warm_until,
                None,
            ),
            (IntentPhase::Closing { key, .. }, HoverEvent::EnterCard) => {
                keep(IntentPhase::Open { key })
            }
            (IntentPhase::Pending { key, due }, HoverEvent::Elapsed) if now >= due => to(
                IntentPhase::Open { key: key.clone() },
                warm_until,
                Some(IntentEffect::Open(key)),
            ),
            (IntentPhase::Closing { key, due }, HoverEvent::Elapsed) if now >= due => {
                to(IntentPhase::Idle, lingers, Some(IntentEffect::Close(key)))
            }
            (
                IntentPhase::Open { key } | IntentPhase::Closing { key, .. },
                HoverEvent::ClickInList,
            ) => to(IntentPhase::Idle, None, Some(IntentEffect::Remove(key))),
            (IntentPhase::Pending { .. }, HoverEvent::ClickInList) => keep(IntentPhase::Idle),
            (IntentPhase::Open { key }, HoverEvent::SpaceKey) => {
                to(IntentPhase::Idle, lingers, Some(IntentEffect::Peek(key)))
            }
            (phase, _) => keep(phase),
        };
        (HoverIntent { profile, ..next }, effect)
    }

    /// Where the machine is.
    pub fn phase(&self) -> &IntentPhase<K> {
        &self.phase
    }

    /// Whether cards open at once: a card is open, or one closed less than HoverWarm ago.
    pub fn warmth(&self, now: Stamp) -> HoverWarmth {
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
/// (none when warm: the card opens at once).
fn over<K: Clone + PartialEq>(
    phase: IntentPhase<K>,
    key: K,
    wait: Duration,
    warm_until: Option<Stamp>,
    warm: HoverWarmth,
    now: Stamp,
) -> (HoverIntent<K>, Effect<K>) {
    match phase {
        IntentPhase::Open { key: open } if open == key => {
            to(IntentPhase::Open { key: open }, warm_until, None)
        }
        IntentPhase::Closing { key: open, .. } if open == key => {
            to(IntentPhase::Open { key: open }, warm_until, None)
        }
        IntentPhase::Pending { key: pending, due } if pending == key => {
            to(IntentPhase::Pending { key: pending, due }, warm_until, None)
        }
        _ => match (warm, wait.is_zero()) {
            (HoverWarmth::Warm, _) | (HoverWarmth::Cold, true) => to(
                IntentPhase::Open { key: key.clone() },
                warm_until,
                Some(IntentEffect::Open(key)),
            ),
            (HoverWarmth::Cold, false) => to(
                IntentPhase::Pending {
                    key,
                    due: now.after_span(wait),
                },
                warm_until,
                None,
            ),
        },
    }
}

fn to<K>(
    phase: IntentPhase<K>,
    warm_until: Option<Stamp>,
    effect: Effect<K>,
) -> (HoverIntent<K>, Effect<K>) {
    (HoverIntent::of(phase, warm_until), effect)
}
