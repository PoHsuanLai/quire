//! `Osd`: the on-screen display's card and its fade (design/20 section 1.7; sill FINDINGS Q74 to
//! Q76). One card, a title line over a read-only [`LevelControl`], in the Osd material with the
//! Space gradient at its tint, styled as a control-center module (`--r-tile`, the grid's padding,
//! design/13 section 13.3.7). It enters with `Anim::OsdIn` and leaves with `Anim::OsdOut`, whose
//! direction follows [`OsdPosition`] through `--osd-dy`, and calls `on_hidden` when the exit has
//! settled, so the host can unmap the surface. The hold is the caller's: it knows `osd.hold_ms`.
//!
//! **Where it goes.** Put it directly inside a transparent Osd root:
//! `Ds { material: Material::Osd, chrome: Some(RootChrome::Transparent), stack, tint_alpha, look,
//! .. }`. The root paints nothing and gives the card every token (the motion tokens its fade
//! reads, the frame's `--f-*`, the material's `--m-*`, the stack and tint alpha); the card paints
//! what a tinted root would on its own box. One root, not a painted root nested in a transparent
//! one (Q76's complaint). The card's margin from its edge is `--osd-margin` (`osd.margin_px`,
//! written by [`crate::OsdMetrics::style_attr`] on any element around it); a host sizes its
//! surface to the card, its margins and the material's shadow, anchored to that edge.

use crate::components::level::{LevelControl, LevelGlyph, LevelLook, LevelMode};
use crate::components::osd_phase::{OsdEffect, OsdInput, OsdPhase, input, step};
use crate::components::tooltip::Shown;
use crate::components::vocab::Fraction;
use crate::motion::anim::Anim;
use crate::motion::timer::{MotionTimer, use_motion_timer};
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// The level an OSD shows: the value and the glyph that follows it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Level {
    /// The level, 0 to 1000.
    pub value: Fraction,
    /// The speaker (heard or muted) or the sun.
    pub glyph: LevelGlyph,
}

/// Where the card sits (`osd.position`, design/22 section 3.16).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OsdPosition {
    /// Top right, under the bar's reserve, as current macOS shows its volume and brightness
    /// panel: the card drops in from above and lifts away (user, 2026-09-25).
    #[default]
    TopRight,
    /// Bottom centre, above the dock's reserve: it rises in and drops away.
    BottomCentre,
}

impl OsdPosition {
    /// The `data-position` value.
    pub fn slug(self) -> &'static str {
        match self {
            OsdPosition::TopRight => "top-right",
            OsdPosition::BottomCentre => "bottom-centre",
        }
    }
}

/// Which of `osd-in`'s two names the card plays: flipped on each showing, so the entrance
/// restarts even where the engine kept the element's styles (design/05 section 9 rule 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Alias {
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

    fn slug(self) -> &'static str {
        match self {
            Alias::A => "a",
            Alias::B => "b",
        }
    }
}

/// The on-screen display. `shown` is the caller's, and `on_hidden` runs once the card has faded
/// out after a hide; a show while it fades takes the hide back (it is present again at once and
/// `on_hidden` does not run for that hide).
#[component]
pub fn Osd(
    shown: Shown,
    #[props(default)] level: Option<Level>,
    #[props(default)] label: Option<String>,
    #[props(default)] on_hidden: EventHandler<()>,
    #[props(default)] position: OsdPosition,
    #[props(default)] look: LevelLook,
    #[props(default)] id: Option<String>,
    children: Element,
) -> Element {
    let phase = use_osd_phase(shown, on_hidden);
    let (now, alias) = phase;
    let level_label = label.clone().unwrap_or_else(|| "Level".to_owned());
    let named = label.clone();
    rsx! {
        div {
            class: "ds-osd",
            id,
            role: "status",
            "aria-label": named,
            "data-position": position.slug(),
            "data-shown": now.shown().slug(),
            "data-presence": now.presence(),
            "data-pulse": alias.slug(),
            div { class: "ds-frame",
                div { class: "ds-grain" }
            }
            if let Some(title) = label {
                div { class: "ds-osd-title", "{title}" }
            }
            if let Some(Level { value, glyph }) = level {
                LevelControl { label: level_label, value, glyph, mode: LevelMode::ReadOnly, look }
            }
            {children}
        }
    }
}

/// The card's phase and entrance alias for this render, driven by `shown`: a change of `shown`
/// steps the machine at once (so the render draws the new phase), and its timers start in an
/// effect after it.
fn use_osd_phase(shown: Shown, on_hidden: EventHandler<()>) -> (OsdPhase, Alias) {
    let fade_in = use_motion_timer(Anim::OsdIn);
    let fade_out = use_motion_timer(Anim::OsdOut);
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
