//! BatteryLevel: a battery's charge as a ring, for the Batteries widget (design/23-WIDGETS.md
//! section 4.1; design/04-COMPONENTS.md "Widgets"), drawn flat and bright as
//! the reference widget measures: a thick round-capped arc from twelve, clockwise as far as the
//! level, in `--battery-fill` (`--battery-low` at a fifth or less), over a full track in
//! `--battery-track` (the plate darkened); the device's glyph centred in the ring; while
//! charging, a bolt in a gap cut at twelve. The widget puts the percentage under it.
//!
//! The arcs are SVG paths Rust computes (`battery_ring.rs`), each on `currentColor` from its own
//! element (spike S6), with no CSS transition (O-20). The arc follows its level from Rust
//! ([`use_ring_share`]): it stands at the level on mount and moves to each new one linearly over
//! `--t-move` (design/30 section 1.3), recomputing its path each frame while it moves and never
//! at rest. The low red and `aria-valuenow` follow the true level from the first frame.

use crate::components::content::text_runs::TextLine;
use crate::motion::detail::tween::{TweenSpec, use_tween};
use crate::shell::battery::ring::{RingSpan, arc_path};
use dioxus::prelude::*;
use ds_core::vocab::Fraction;
use ds_core::word::Word;
use ds_style::tokens::{easing::EasingToken, timing::DurationToken};
use serde::{Deserialize, Serialize};

/// How the arc follows its level: linearly over `--t-move`.
const ARC: TweenSpec = TweenSpec {
    duration: DurationToken::Move,
    easing: EasingToken::Linear,
};

/// The share of a battery ring's arc drawn now, following `level` (clamped).
pub(crate) fn use_ring_share(level: Fraction) -> Fraction {
    use_tween(level.clamped(), ARC)
}

/// The charging bolt, in its own 10 x 16 box.
pub(crate) const BOLT: &str = "M7 0 0 9.6h4.6L3.2 16 10 6.4H5.4L7 0Z";

/// Whether the battery is filling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Word)]
pub enum RingMark {
    /// Just the level.
    #[default]
    Plain,
    /// Charging: a bolt in a gap at twelve, and never low.
    Charging,
}

impl RingMark {
    /// `data-mark`: written only while charging.
    pub(crate) fn attr(self) -> Option<&'static str> {
        (self != RingMark::Plain).then(|| self.slug())
    }
}

/// How full a battery reads, which picks its fill's colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
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
}

/// A battery ring at `level` (permille), charging or not, named `label` for assistive
/// technology. `children` (a device's glyph, optional) sit in the ring's middle. The arc stands
/// at its level on mount and moves to each new one.
#[component]
pub fn BatteryLevel(
    level: Fraction,
    #[props(default)] mark: RingMark,
    #[props(into)] label: TextLine,
    children: Element,
) -> Element {
    let level = level.clamped();
    let percent = level.whole_percent();
    let share = use_ring_share(level);
    let span = match mark {
        RingMark::Plain => RingSpan::FULL,
        RingMark::Charging => RingSpan::GAPPED,
    };
    rsx! {
        div {
            class: "ds-battery",
            "data-tone": RingTone::of(level, mark).slug(),
            "data-mark": mark.attr(),
            role: "progressbar",
            "aria-label": "{label.plain_text()}",
            "aria-valuemin": "0",
            "aria-valuemax": "100",
            "aria-valuenow": "{percent}",
            {ring(RingLayer::Track, arc_path(span))}
            {ring(RingLayer::Arc, arc_path(span.filled(share)))}
            if let Some(device) = given(children) {
                span { class: "ds-battery-device", {device} }
            }
            if mark == RingMark::Charging {
                svg { class: "ds-battery-bolt", "data-ds-svg": "battery", view_box: "0 0 10 16", "aria-hidden": "true",
                    path { d: BOLT, fill: "currentColor" }
                }
            }
        }
    }
}

/// One of the ring's two strokes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum RingLayer {
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
pub(crate) fn ring(layer: RingLayer, path: Option<String>) -> Element {
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
pub(crate) fn given(children: Element) -> Option<Element> {
    match &children {
        Ok(node) if *node == VNode::placeholder() => None,
        _ => Some(children),
    }
}

#[cfg(test)]
mod tests {
    use super::{RingMark, RingTone};
    use ds_core::vocab::Fraction;

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
