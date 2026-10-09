//! The one hover manager every card goes through: it owns the [`ds_motion::hover_intent::HoverIntent`] machine
//! (its deadlines are its open and close timers), and stamps `data-hover="warm|cold"` on `.ds` (design/04-COMPONENTS.md
//! O-11).
//!
//! The fade-out and warm timers are tasks of the root that provides it and drop with it; every
//! write a timer makes is a `try_set`, so a timer that outlives the hub's signals stops instead of panicking
//! (`ds_style::task`).

use dioxus::core::current_scope_id;
use dioxus::prelude::*;
use ds_core::time::clock::sleep;
use ds_motion::anim::Anim;
use ds_motion::hover_intent::{
    HoverEvent, HoverIntent, HoverProfile, HoverWarmth, IntentEffect, IntentPhase,
};
use ds_motion::machine::{MachineRef, use_machine};
use ds_motion::settle::settle;
use ds_style::scope::Scope;
use ds_style::task::{Gone, spawn_in, try_get, try_set, try_set_if_changed};
use ds_style::tokens::delay::{DelayToken, TipDelay};

/// A card the hub is tracking: the consumer's key and the profile it waits by.
type Card = (HoverKey, HoverProfile);

/// What a hover target is, by the consumer's own key: `sender:3`, `thread:88`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HoverKey(pub String);

/// Which hover card a target opens (design/30 section 2.5: kinds are content). It says where the
/// card stands against its target and how wide it is; every kind waits by [`HoverProfile::Card`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HoverKind {
    /// A list row: the thread card, placed right of the list column.
    Thread,
    /// A name: the sender card, placed below.
    Sender,
    /// An account tile.
    Account,
    /// A sidebar entry (pin, Today): the narrow side card, placed right of the target.
    Side,
}

/// What a card's closing leaves behind, and the scope its settle timers run in: the signals the
/// machine's effects write. Copy.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Fades {
    leaving: Signal<Option<Card>>,
    peeked: Signal<Option<Card>>,
    warm_tick: Signal<u32>,
    env: Signal<Scope>,
    scope: ScopeId,
}

/// The hover manager, provided as context by `Ds`.
#[derive(Debug, Clone, Copy)]
pub struct HoverHub {
    machine: MachineRef<HoverIntent<Card>>,
    fades: Fades,
}

impl HoverHub {
    /// Feed an event; the machine's own deadlines are the open and close timers.
    pub fn feed(&self, event: HoverEvent<Card>) {
        self.machine.send(event);
    }

    /// The card that is open (or closing), for the consumer to render.
    pub fn open(&self) -> Option<(HoverKey, HoverProfile)> {
        match self.machine.state().read().phase() {
            IntentPhase::Open { key } | IntentPhase::Closing { key, .. } => Some(key.clone()),
            IntentPhase::Idle | IntentPhase::Pending { .. } => None,
        }
    }

    /// The card fading out, until its exit settles and unmounts it; render it with
    /// `data-presence="leaving"`.
    pub fn leaving(&self) -> Option<(HoverKey, HoverProfile)> {
        self.fades.leaving.read().clone()
    }

    /// The card Space last turned into a peek (design/06-INTERACTIONS.md section 3); the
    /// consumer opens its peek when this changes.
    pub fn peeked(&self) -> Option<(HoverKey, HoverProfile)> {
        self.fades.peeked.read().clone()
    }

    /// Whether cards open at once right now.
    pub fn warmth(&self) -> HoverWarmth {
        let _expiry = self.fades.warm_tick.read();
        self.machine.state().read().warmth(self.machine.now())
    }
}

impl Fades {
    fn apply(&self, effect: IntentEffect<Card>) -> Result<(), Gone> {
        match effect {
            IntentEffect::Open(_) | IntentEffect::Remove(_) => {
                try_set_if_changed(self.leaving, None)
            }
            IntentEffect::Close(card) => self.close(card),
            IntentEffect::Peek(card) => {
                self.close(card.clone())?;
                try_set_if_changed(self.peeked, Some(card))
            }
        }
    }

    /// Fade `card` out, unmount it once that settles, and end the warm window.
    fn close(&self, card: Card) -> Result<(), Gone> {
        try_set_if_changed(self.leaving, Some(card.clone()))?;
        let level = try_get(self.env)?.resolved.motion;
        let out = settle(Anim::MenuOut, level);
        let leaving = self.leaving;
        spawn_in(self.scope, async move {
            sleep(out).await;
            if try_get(leaving) == Ok(Some(card)) {
                let _ = try_set(leaving, None);
            }
        });
        let warm = DelayToken::HoverWarm.delay();
        let tick = self.warm_tick;
        spawn_in(self.scope, async move {
            sleep(warm).await;
            if let Ok(count) = try_get(tick) {
                let _ = try_set(tick, count + 1);
            }
        });
        Ok(())
    }
}

/// A new hub for `Ds` to provide, timing the fade out at the root's motion level.
pub fn use_hover_hub_provider(env: Signal<Scope>, tip: TipDelay) -> HoverHub {
    let fades = Fades {
        leaving: use_signal(|| None),
        peeked: use_signal(|| None),
        warm_tick: use_signal(|| 0),
        env,
        scope: use_hook(current_scope_id),
    };
    let machine = use_machine(
        |_| HoverIntent::default(),
        tip,
        || (),
        move |effect, _| {
            let _ = fades.apply(effect);
        },
    );
    use_context_provider(|| HoverHub { machine, fades })
}

/// The enclosing `Ds`'s hover manager.
pub fn use_hover_hub() -> HoverHub {
    use_context::<HoverHub>()
}
