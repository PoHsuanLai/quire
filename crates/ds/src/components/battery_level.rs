//! BatteryLevel: a battery's charge as a ring, for the Batteries widget (design/23-WIDGETS.md
//! section 4.1; design/04-COMPONENTS.md "Widgets"; sill FINDINGS Q183), drawn flat and bright as
//! the reference widget measures: a thick round-capped arc from twelve, clockwise as far as the
//! level, in `--battery-fill` (`--battery-low` at a fifth or less), over a full track in
//! `--battery-track` (the plate darkened); the device's glyph centred in the ring; while
//! charging, a bolt in a gap cut at twelve. The widget puts the percentage under it.
//!
//! The arcs are SVG paths Rust computes (`battery_ring.rs`), each on `currentColor` from its own
//! element (spike S6), with no CSS transition (O-20). The arc fills from Rust instead
//! ([`use_battery_fill`]): on mount and on each new `wake` it sweeps from empty to the level, on
//! each new level from the old one to the new, recomputing its path each frame while it moves
//! and never at rest; a charging bolt fades in once the sweep has arrived. The low red and
//! `aria-valuenow` follow the true level from the first frame.

use crate::components::battery_ring::{Span, arc_path};
use crate::components::text_runs::Text;
use crate::components::vocab::Fraction;
use crate::motion::{RunFrame, RunTokens, WakeStamp, use_level_run};
use crate::tokens::{DurationToken, EasingToken};
use dioxus::prelude::*;

/// The fill's timing: `--t-fill` at `--e-out`, then the bolt's fade over `--t-quick`.
pub const FILL: RunTokens = RunTokens {
    duration: DurationToken::Fill,
    easing: EasingToken::Out,
    tail: DurationToken::Quick,
};

/// The frame of a battery ring's fill toward `level`: from empty on mount and on each new
/// `wake`, from the last level drawn on each new `level`, at once under Reduced motion.
/// [`BatteryLevel`] draws its arc from it and [`crate::use_battery_figure`] its count, so a ring
/// and a percentage given the same `level` and `wake` move in step.
pub fn use_battery_fill(level: Fraction, wake: WakeStamp) -> RunFrame {
    use_level_run(level.clamped(), wake, FILL)
}

/// The charging bolt, in its own 10 x 16 box.
const BOLT: &str = "M7 0 0 9.6h4.6L3.2 16 10 6.4H5.4L7 0Z";

/// Whether the battery is filling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RingMark {
    /// Just the level.
    #[default]
    Plain,
    /// Charging: a bolt in a gap at twelve, and never low.
    Charging,
}

impl RingMark {
    /// `data-mark`: written only while charging.
    fn slug(self) -> Option<&'static str> {
        match self {
            RingMark::Plain => None,
            RingMark::Charging => Some("charging"),
        }
    }
}

/// How full a battery reads, which picks its fill's colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum RingTone {
    /// Above a fifth, or charging: `--battery-fill`.
    Ok,
    /// A fifth or less: `--battery-low`.
    Low,
    /// A tenth or less: `--battery-low` too (the reference has no separate critical colour).
    Critical,
}

impl RingTone {
    /// The tone of `level` with `mark`: a charging battery is never low.
    pub(crate) fn of(level: Fraction, mark: RingMark) -> Self {
        match (mark, level.clamped().0) {
            (RingMark::Charging, _) => RingTone::Ok,
            (RingMark::Plain, 0..=100) => RingTone::Critical,
            (RingMark::Plain, 101..=200) => RingTone::Low,
            (RingMark::Plain, _) => RingTone::Ok,
        }
    }

    /// The `data-tone` word.
    fn slug(self) -> &'static str {
        match self {
            RingTone::Ok => "ok",
            RingTone::Low => "low",
            RingTone::Critical => "critical",
        }
    }
}

/// A battery ring at `level` (permille), charging or not, named `label` for assistive
/// technology. `children` (a device's glyph, optional) sit in the ring's middle. The arc fills
/// on mount and again on each new `wake` (a host passes `WakeStamp::next` when its widgets come
/// into view), and sweeps to each new level; `data-pulse` is `a` while it moves.
#[component]
pub fn BatteryLevel(
    level: Fraction,
    #[props(default)] mark: RingMark,
    #[props(into)] label: Text,
    #[props(default)] wake: WakeStamp,
    children: Element,
) -> Element {
    let level = level.clamped();
    let percent = percent_of(level);
    let frame = use_battery_fill(level, wake);
    let alias = moving(frame, level);
    let span = match mark {
        RingMark::Plain => Span::FULL,
        RingMark::Charging => Span::GAPPED,
    };
    rsx! {
        div {
            class: "ds-battery",
            "data-pulse": alias,
            "data-tone": RingTone::of(level, mark).slug(),
            "data-mark": mark.slug(),
            role: "progressbar",
            "aria-label": "{label.plain_text()}",
            "aria-valuemin": "0",
            "aria-valuemax": "100",
            "aria-valuenow": "{percent}",
            {ring(RingLayer::Track, arc_path(span))}
            {ring(RingLayer::Arc, arc_path(span.filled(frame.shown)))}
            if let Some(device) = given(children) {
                span { class: "ds-battery-device", {device} }
            }
            if mark == RingMark::Charging && frame.tail.0 > 0 {
                svg { class: "ds-battery-bolt", "data-ds-svg": "battery", view_box: "0 0 10 16", "aria-hidden": "true",
                    style: fading(frame.tail),
                    path { d: BOLT, fill: "currentColor" }
                }
            }
        }
    }
}

/// A level in whole percent, rounded: what `aria-valuenow` reads, and the count a figure shows
/// for the level drawn in a frame (the fill's count-at-progress: `percent_of(level_at(run, p))`).
pub fn percent_of(level: Fraction) -> u16 {
    (level.clamped().0 + 5) / 10
}

/// `data-pulse` while the fill moves (`a`), absent at rest: kept for hosts that select a
/// moving ring by it, as they did its bump.
fn moving(frame: RunFrame, level: Fraction) -> Option<&'static str> {
    if frame == RunFrame::rest(level) {
        None
    } else {
        Some("a")
    }
}

/// The bolt's opacity while it fades in; nothing once it is whole.
fn fading(tail: Fraction) -> Option<String> {
    if tail.0 >= 1000 {
        None
    } else {
        Some(format!("opacity:{:.3}", f32::from(tail.0) / 1000.0))
    }
}

/// One of the ring's two strokes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RingLayer {
    /// The full circle under the level.
    Track,
    /// The level.
    Arc,
}

impl RingLayer {
    fn class(self) -> &'static str {
        match self {
            RingLayer::Track => "ds-battery-track",
            RingLayer::Arc => "ds-battery-arc",
        }
    }
}

/// `layer` along `path`, or nothing for an empty arc.
fn ring(layer: RingLayer, path: Option<String>) -> Element {
    let Some(d) = path else {
        return rsx! {};
    };
    rsx! {
        svg { class: layer.class(), "data-ds-svg": "battery", view_box: "0 0 100 100", "aria-hidden": "true",
            path {
                d,
                fill: "none",
                stroke: "currentColor",
                "stroke-width": "9.3",
                "stroke-linecap": "round",
            }
        }
    }
}

/// `children`, or `None` when the caller passed none: an omitted `children` is the shared
/// placeholder node.
fn given(children: Element) -> Option<Element> {
    match &children {
        Ok(node) if *node == VNode::placeholder() => None,
        _ => Some(children),
    }
}

/// The old name of [`BatteryLevel`], kept for callers written against the level ring: the
/// props are the same.
pub use BatteryLevel as LevelRing;

#[cfg(test)]
mod tests {
    use super::{RingMark, RingTone};
    use crate::components::vocab::Fraction;

    #[test]
    fn a_battery_reads_low_only_while_draining() {
        let cases = [
            (50, RingMark::Plain, RingTone::Critical),
            (100, RingMark::Plain, RingTone::Critical),
            (101, RingMark::Plain, RingTone::Low),
            (200, RingMark::Plain, RingTone::Low),
            (201, RingMark::Plain, RingTone::Ok),
            (50, RingMark::Charging, RingTone::Ok),
        ];
        for (permille, mark, want) in cases {
            assert_eq!(
                RingTone::of(Fraction(permille), mark),
                want,
                "{permille} {mark:?}"
            );
        }
    }
}
