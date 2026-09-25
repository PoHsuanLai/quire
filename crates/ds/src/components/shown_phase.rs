//! A surface's presence driven by the caller's `shown` (sill FINDINGS Q76; notification parts,
//! sill Q123): entering, present, leaving, then hidden with `on_hidden` at the exit's settle, and
//! a show while it leaves taking the hide back. The OSD card and the notification center's panel
//! both run it, each with its own pair of animations; the rules are the pure machine in
//! `osd_phase`.

use crate::components::osd_phase::{OsdEffect, OsdInput, OsdPhase, input, step};
use crate::components::tooltip::Shown;
use crate::motion::anim::Anim;
use crate::motion::timer::{MotionTimer, use_motion_timer};
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// Which of its entrance's two names a surface plays: flipped on each showing, so the entrance
/// restarts even where the engine kept the element's styles (design/05 section 9 rule 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Alias {
    A,
    B,
}

impl Alias {
    fn flipped(self) -> Self {
        match self {
            Alias::A => Alias::B,
            Alias::B => Alias::A,
        }
    }

    pub(crate) fn slug(self) -> &'static str {
        match self {
            Alias::A => "a",
            Alias::B => "b",
        }
    }
}

/// A surface's phase and entrance alias for this render, driven by `shown`: a change of `shown`
/// steps the machine at once (so the render draws the new phase), and its timers, `enter`'s and
/// `exit`'s settles, start in an effect after it. `on_hidden` runs when `exit` has settled.
pub(crate) fn use_shown_phase(
    shown: Shown,
    on_hidden: EventHandler<()>,
    enter: Anim,
    exit: Anim,
) -> (OsdPhase, Alias) {
    let fade_in = use_motion_timer(enter);
    let fade_out = use_motion_timer(exit);
    let mut phase = use_hook(|| CopyValue::new(OsdPhase::Hidden));
    let mut alias = use_hook(|| CopyValue::new(Alias::A));
    let mut last = use_hook(|| CopyValue::new(None::<Shown>));
    let mut hidden = use_hook(|| CopyValue::new(on_hidden));
    hidden.set(on_hidden);
    // Read in render so a settled timer renders the card again.
    let _ = (fade_in.phase(), fade_out.phase());
    let in_settled = use_hook(|| {
        EventHandler::new(move |()| {
            let (next, _) = step(*phase.peek(), OsdInput::InSettled);
            phase.set(next);
        })
    });
    let out_settled = use_hook(|| {
        EventHandler::new(move |()| {
            let (next, effect) = step(*phase.peek(), OsdInput::OutSettled);
            phase.set(next);
            if effect == OsdEffect::Gone {
                hidden.peek().call(());
            }
        })
    });
    let change = input(*last.peek(), shown);
    last.set(Some(shown));
    if let Some(change) = change {
        let (next, effect) = step(*phase.peek(), change);
        phase.set(next);
        if effect == OsdEffect::PlayIn {
            let flipped = alias.peek().flipped();
            alias.set(flipped);
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
    (*phase.peek(), *alias.peek())
}

/// The card's two timers and what each calls when it settles.
#[derive(Clone, Copy)]
struct Timers {
    fade_in: MotionTimer,
    fade_out: MotionTimer,
    in_settled: EventHandler<()>,
    out_settled: EventHandler<()>,
}

/// Start or stop the timers `effect` asks for, after this render.
fn run(effect: OsdEffect, timers: Timers) {
    match effect {
        OsdEffect::PlayIn => queue_effect(move || timers.fade_in.start(timers.in_settled)),
        OsdEffect::PlayOut => queue_effect(move || timers.fade_out.start(timers.out_settled)),
        OsdEffect::CancelOut => queue_effect(move || timers.fade_out.cancel()),
        OsdEffect::None | OsdEffect::Gone => {}
    }
}
