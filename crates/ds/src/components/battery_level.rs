//! BatteryLevel: a battery's charge as a ring, for the Batteries widget (design/23-WIDGETS.md
//! section 4.1; design/04-COMPONENTS.md "Widgets"; sill FINDINGS Q183), drawn flat and bright as
//! the reference widget measures: a thick round-capped arc from twelve, clockwise as far as the
//! level, in `--battery-fill` (`--battery-low` at a fifth or less), over a full track in
//! `--battery-track` (the plate darkened); the device's glyph centred in the ring; while
//! charging, a bolt in a gap cut at twelve. The widget puts the percentage under it.
//!
//! The arcs are SVG paths Rust computes (`battery_ring.rs`), each on `currentColor` from its own
//! element (spike S6), with no transition (O-20). The ring bumps once when its percentage changes.

use crate::components::battery_ring::{Span, arc_path};
use crate::components::bump_on::{bump_attrs, use_bump_on};
use crate::components::text_runs::Text;
use crate::components::vocab::Fraction;
use dioxus::prelude::*;

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
/// technology. `children` (a device's glyph, optional) sit in the ring's middle.
#[component]
pub fn BatteryLevel(
    level: Fraction,
    #[props(default)] mark: RingMark,
    #[props(into)] label: Text,
    children: Element,
) -> Element {
    let level = level.clamped();
    let percent = (level.0 + 5) / 10;
    let (class, alias) = bump_attrs("ds-battery", use_bump_on(percent));
    let span = match mark {
        RingMark::Plain => Span::FULL,
        RingMark::Charging => Span::GAPPED,
    };
    rsx! {
        div {
            class,
            "data-pulse": alias,
            "data-tone": RingTone::of(level, mark).slug(),
            "data-mark": mark.slug(),
            role: "progressbar",
            "aria-label": "{label.plain_text()}",
            "aria-valuemin": "0",
            "aria-valuemax": "100",
            "aria-valuenow": "{percent}",
            {ring(RingLayer::Track, arc_path(span))}
            {ring(RingLayer::Arc, arc_path(span.filled(level)))}
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
