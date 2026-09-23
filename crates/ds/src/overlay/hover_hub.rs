//! The one hover manager every card goes through: it owns the [`crate::HoverIntent`] machine
//! and its timers, and stamps `data-hover="warm|cold"` on `.ds` (design/04-COMPONENTS.md
//! O-11).

use crate::components::vocab::StaggerIndex;
use crate::motion::anim::Anim;
use crate::motion::hover_intent::{HoverEvent, HoverIntent, IntentEffect, IntentPhase};
use crate::motion::settle::settle;
use crate::root::env::Env;
use crate::time::{sleep, spawn_in};
use crate::tokens::DelayToken;
use dioxus::core::{Task, current_scope_id};
use dioxus::prelude::*;
use std::time::{Duration, Instant};

/// A card the hub is tracking: the consumer's key and the kind of card.
type Card = (HoverKey, HoverKind);

/// What a hover target is, by the consumer's own key: `sender:3`, `thread:88`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HoverKey(pub String);

/// Which card a target opens (design/04-COMPONENTS.md section 22).
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

/// Whether cards and fly labels open at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HoverWarmth {
    /// A card is open or closed less than HoverWarm ago: no wait.
    Warm,
    /// Wait for intent.
    #[default]
    Cold,
}

/// The hover manager, provided as context by `Ds`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoverHub {
    intent: Signal<HoverIntent<(HoverKey, HoverKind)>>,
    leaving: Signal<Option<Card>>,
    peeked: Signal<Option<Card>>,
    open_timer: Signal<Option<Task>>,
    close_timer: Signal<Option<Task>>,
    warm_tick: Signal<u32>,
    env: Signal<Env>,
    scope: ScopeId,
}

impl HoverHub {
    /// Feed an event; the hub starts and cancels its own timers.
    pub fn feed(&self, event: HoverEvent<(HoverKey, HoverKind)>) {
        let (next, effect) = self.intent.peek().clone().step(event, Instant::now());
        let mut intent = self.intent;
        if *intent.peek() != next {
            intent.set(next);
        }
        self.apply(effect);
    }

    /// The card that is open (or closing), for the consumer to render.
    pub fn open(&self) -> Option<(HoverKey, HoverKind)> {
        match self.intent.read().phase() {
            IntentPhase::Open { key } | IntentPhase::Closing { key, .. } => Some(key.clone()),
            IntentPhase::Idle | IntentPhase::Pending { .. } => None,
        }
    }

    /// The card playing `hc-out`, until `settle(HcOut)` unmounts it; render it with
    /// `data-presence="leaving"`.
    pub fn leaving(&self) -> Option<(HoverKey, HoverKind)> {
        self.leaving.read().clone()
    }

    /// The card Space last turned into a peek (design/06-INTERACTIONS.md section 3); the
    /// consumer opens its peek when this changes.
    pub fn peeked(&self) -> Option<(HoverKey, HoverKind)> {
        self.peeked.read().clone()
    }

    /// Whether cards open at once right now.
    pub fn warmth(&self) -> HoverWarmth {
        let _expiry = self.warm_tick.read();
        self.intent.read().warmth(Instant::now())
    }

    fn apply(&self, effect: IntentEffect<Card>) {
        match effect {
            IntentEffect::None => {}
            IntentEffect::StartOpen { after } if after.is_zero() => {
                stop(self.open_timer);
                self.feed(HoverEvent::OpenDue);
            }
            IntentEffect::StartOpen { after } => {
                self.restart(self.open_timer, after, HoverEvent::OpenDue);
            }
            IntentEffect::CancelOpen => stop(self.open_timer),
            IntentEffect::StartClose { after } => {
                self.restart(self.close_timer, after, HoverEvent::CloseDue);
            }
            IntentEffect::CancelClose => stop(self.close_timer),
            IntentEffect::Open(_) | IntentEffect::Remove(_) => {
                stop(self.close_timer);
                set_if_changed(self.leaving, None);
            }
            IntentEffect::Close(card) => self.close(card),
            IntentEffect::Peek(card) => {
                self.close(card.clone());
                set_if_changed(self.peeked, Some(card));
            }
        }
    }

    /// Play `hc-out` on `card`, unmount it once that settles, and end the warm window.
    fn close(&self, card: Card) {
        stop(self.close_timer);
        set_if_changed(self.leaving, Some(card.clone()));
        let level = self.env.peek().resolved.motion;
        let out = settle(Anim::HcOut, level, StaggerIndex::default());
        let leaving = self.leaving;
        spawn_in(self.scope, async move {
            sleep(out).await;
            if *leaving.peek() == Some(card) {
                set_if_changed(leaving, None);
            }
        });
        let warm = DelayToken::HoverWarm.delay(level);
        let mut tick = self.warm_tick;
        spawn_in(self.scope, async move {
            sleep(warm).await;
            tick += 1;
        });
    }

    /// Cancel `timer` and start it again, feeding `event` after `after`.
    fn restart(&self, timer: Signal<Option<Task>>, after: Duration, event: HoverEvent<Card>) {
        stop(timer);
        let hub = *self;
        let started = spawn_in(self.scope, async move {
            sleep(after).await;
            let mut done = timer;
            done.set(None);
            hub.feed(event);
        });
        let mut slot = timer;
        slot.set(Some(started));
    }
}

/// Cancel the task in `timer`, if any.
fn stop(timer: Signal<Option<Task>>) {
    if let Some(running) = *timer.peek() {
        running.cancel();
        let mut slot = timer;
        slot.set(None);
    }
}

fn set_if_changed<T: PartialEq + 'static>(signal: Signal<T>, value: T) {
    if *signal.peek() != value {
        let mut signal = signal;
        signal.set(value);
    }
}

/// A new hub for `Ds` to provide, timing `hc-out` at the root's motion level.
pub(crate) fn use_hover_hub_provider(env: Signal<Env>) -> HoverHub {
    let scope = use_hook(current_scope_id);
    use_context_provider(|| HoverHub {
        intent: Signal::new(HoverIntent::default()),
        leaving: Signal::new(None),
        peeked: Signal::new(None),
        open_timer: Signal::new(None),
        close_timer: Signal::new(None),
        warm_tick: Signal::new(0),
        env,
        scope,
    })
}

/// The enclosing `Ds`'s hover manager.
pub fn use_hover_hub() -> HoverHub {
    use_context::<HoverHub>()
}
