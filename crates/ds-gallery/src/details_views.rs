//! The Details page's specimens as components of their state, so the page (with its replay
//! buttons) and the frame strips (driven by a harness) draw the same thing.

use crate::details_states::{Bell, Charge, Net, Seal};
use dioxus::prelude::*;
use ds::detail::{
    CheckMark, CountPace, FirstShow, LayerGlyph, Layering, Layers, MorphGlyph, MorphStyle,
    PendingSpec, PendingStyle, SettleStyle, Settling, Slashed, Touch, use_count_up, use_detail,
    use_nudge, use_operation, use_pending, use_settle, use_shake, use_sweep,
};
use ds::{Fraction, Icon, IconSize};

/// The Wi-Fi glyph's pending loop: the dot and three arcs, one at a time.
const SEARCHING: PendingSpec = PendingSpec {
    style: PendingStyle::Iterate,
    layers: Layers(4),
};

/// `class` with a pulse's class and alias when it plays.
fn pulsed(class: &str, key: Option<(String, &'static str)>) -> (String, Option<&'static str>) {
    match key {
        Some((anim, alias)) => (format!("{class} {anim}"), Some(alias)),
        None => (class.to_owned(), None),
    }
}

/// A bar that sweeps to the charge and the percentage counting in step (Sweep + CountUp).
#[component]
pub fn SweepCountView(charge: Charge, first: FirstShow) -> Element {
    let detail = use_detail(charge, first, Touch::Remote);
    let Charge::Level(percent) = detail.state().clone();
    let sweep = use_sweep(Fraction(percent * 10), detail.cue());
    let count = use_count_up(i64::from(percent), detail.cue(), CountPace::InStep(sweep));
    let width = f32::from(sweep.share().0.min(1000)) / 10.0;
    rsx! {
        div { class: "g-detail g-detail-sweep",
            div { class: "g-bar", div { class: "g-bar-fill", style: "width:{width}%" } }
            span { class: "g-figure", "{count.shown()}%" }
        }
    }
}

/// A Wi-Fi glyph: its layers searching while joining (bounded, R4), filling once on success.
#[component]
pub fn NetView(net: Net) -> Element {
    let detail = use_detail(net, FirstShow::Still, Touch::Remote);
    let frame = use_pending(use_operation(detail.cue()), SEARCHING);
    let settling = use_settle(detail.cue(), SettleStyle::Fill(Layers(4)));
    let layering = match settling {
        Settling::Filling(upto) => Layering::Filling(upto),
        Settling::Rest | Settling::Drawing(_) | Settling::Sealing(_) => {
            Layering::Pending(frame, SEARCHING)
        }
    };
    rsx! {
        div { class: "g-detail g-detail-glyph",
            LayerGlyph { icon: Icon::Wifi, size: IconSize::Bar, layering }
        }
    }
}

/// A check that draws on when the state succeeds and holds for `SettleHold` (Settle(Check)).
#[component]
pub fn CheckView(net: Net) -> Element {
    let detail = use_detail(net, FirstShow::Still, Touch::Remote);
    let settling = use_settle(detail.cue(), SettleStyle::Check);
    rsx! {
        div { class: "g-detail g-detail-glyph",
            span { class: "g-check-well", CheckMark { settling, size: IconSize::Bar } }
        }
    }
}

/// A field that shakes once per new failure (Shake, R6).
#[component]
pub fn ShakeView(net: Net) -> Element {
    let detail = use_detail(net, FirstShow::Still, Touch::Remote);
    let (class, alias) = pulsed("g-shake-field", use_shake(detail.cue()).attrs());
    rsx! {
        div { class: "g-detail",
            div { class, "data-pulse": alias, "Password" }
        }
    }
}

/// A bell that lifts once per new request (Nudge, R6).
#[component]
pub fn NudgeView(bell: Bell) -> Element {
    let detail = use_detail(bell, FirstShow::Still, Touch::Remote);
    let (class, alias) = pulsed("g-nudge-bell", use_nudge(detail.cue()).attrs());
    rsx! {
        div { class: "g-detail",
            span { class, "data-pulse": alias, MorphGlyph { icon: Icon::Bell, size: IconSize::Bar, style: MorphStyle::CrossFade } }
        }
    }
}

/// A seal that settles with the spring when the person pressed it, and without when it came
/// from elsewhere (Settle(LockIn), R5).
#[component]
pub fn SealView(seal: Seal, touch: Touch) -> Element {
    let detail = use_detail(seal, FirstShow::Still, touch);
    let settling = use_settle(detail.cue(), SettleStyle::LockIn);
    let key = match settling {
        Settling::Sealing(key) => key.attrs(),
        Settling::Rest | Settling::Filling(_) | Settling::Drawing(_) => None,
    };
    let (class, alias) = pulsed("g-seal", key);
    rsx! {
        div { class: "g-detail", div { class, "data-pulse": alias } }
    }
}

/// A speaker whose slash draws on and off (MorphGlyph Slash).
#[component]
pub fn SlashView(slashed: Slashed) -> Element {
    rsx! {
        div { class: "g-detail g-detail-glyph",
            MorphGlyph { icon: Icon::Volume, size: IconSize::Bar, style: MorphStyle::Slash, slashed }
        }
    }
}
