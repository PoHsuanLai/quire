//! A surface's presence driven by the caller's `shown` (the OSD card, the screenshot thumbnail,
//! the preview pane): entering, present, leaving, then hidden with `on_hidden` at the exit's
//! settle, and a show while it leaves taking the hide back. The rules are the pure machine in
//! `step`. Present keeps the entrance declared in each surface's CSS, so the entrance's settle
//! timer (wall clock) never cancels an entrance the frame clock is still playing, and a
//! taken-back hide plays `hold` (Blitz at the pin keeps a cancelled animation's last value).

use super::Presence;
use super::spec::PresenceSpec;
use super::step::{PresenceEffect, PresenceInput, input, step};
use crate::motion::timer::{MotionTimer, use_motion_timer};
use dioxus::core::queue_effect;
use dioxus::prelude::*;
use ds_core::vocab::Shown;
use ds_core::word::Word;

/// Which of its entrance's two names a surface plays: flipped on each showing, so the entrance
/// restarts even where the engine kept the element's styles (design/05 section 9 rule 2). A
/// surface present again after its hide was taken back plays `hold` instead (`Held`): the exit
/// it drops is replaced by an animation that moves nothing, not by a second entrance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum EntranceAlias {
    A,
    B,
    Held,
}

impl EntranceAlias {
    fn flipped(self) -> Self {
        match self {
            EntranceAlias::A => EntranceAlias::B,
            EntranceAlias::B | EntranceAlias::Held => EntranceAlias::A,
        }
    }
}

/// What a surface draws this render: where it is in its life and which entrance name it plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Presented {
    /// Its life: `data-presence` and `data-shown`.
    pub(crate) presence: Presence,
    /// Its entrance's name: `data-pulse`.
    pub(crate) alias: EntranceAlias,
}

/// A surface's presence and entrance alias for this render, driven by `shown`: a change of
/// `shown` steps the machine at once (so the render draws the new phase), and its timers, the
/// entrance's and the exit's settles, start in an effect after it. `on_hidden` runs when the
/// exit has settled.
pub fn use_presence(
    shown: Shown,
    spec: PresenceSpec,
    on_hidden: Option<EventHandler<()>>,
) -> Presented {
    let fade_in = use_motion_timer(spec.enter);
    let fade_out = use_motion_timer(spec.exit.anim());
    let mut presence = use_hook(|| CopyValue::new(Presence::Hidden));
    let mut alias = use_hook(|| CopyValue::new(EntranceAlias::A));
    let mut last = use_hook(|| CopyValue::new(None::<Shown>));
    let mut hidden = use_hook(|| CopyValue::new(on_hidden));
    hidden.set(on_hidden);
    // Read in render so a settled timer renders the surface again.
    let _ = (fade_in.phase(), fade_out.phase());
    let in_settled = use_hook(|| {
        EventHandler::new(move |()| {
            let (next, _) = step(*presence.peek(), PresenceInput::InSettled, spec.exit);
            presence.set(next);
        })
    });
    let out_settled = use_hook(|| {
        EventHandler::new(move |()| {
            let (next, effect) = step(*presence.peek(), PresenceInput::OutSettled, spec.exit);
            presence.set(next);
            if let (PresenceEffect::Gone, Some(on_hidden)) = (effect, *hidden.peek()) {
                on_hidden.call(());
            }
        })
    });
    let change = input(*last.peek(), shown);
    last.set(Some(shown));
    if let Some(change) = change {
        let (next, effect) = step(*presence.peek(), change, spec.exit);
        presence.set(next);
        match effect {
            PresenceEffect::PlayIn => {
                let flipped = alias.peek().flipped();
                alias.set(flipped);
            }
            PresenceEffect::CancelOut => alias.set(EntranceAlias::Held),
            PresenceEffect::None | PresenceEffect::PlayOut | PresenceEffect::Gone => {}
        }
        run(
            effect,
            Timers {
                fade_in,
                fade_out,
                in_settled,
                out_settled,
            },
        );
    }
    Presented {
        presence: *presence.peek(),
        alias: *alias.peek(),
    }
}

/// The surface's two timers and what each calls when it settles.
#[derive(Clone, Copy)]
struct Timers {
    fade_in: MotionTimer,
    fade_out: MotionTimer,
    in_settled: EventHandler<()>,
    out_settled: EventHandler<()>,
}

/// Start or stop the timers `effect` asks for, after this render.
fn run(effect: PresenceEffect, timers: Timers) {
    match effect {
        PresenceEffect::PlayIn => queue_effect(move || timers.fade_in.start(timers.in_settled)),
        PresenceEffect::PlayOut => queue_effect(move || timers.fade_out.start(timers.out_settled)),
        PresenceEffect::CancelOut => queue_effect(move || timers.fade_out.cancel()),
        PresenceEffect::None | PresenceEffect::Gone => {}
    }
}
