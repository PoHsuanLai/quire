//! LevelRing: a level as a stroked ring, for the Battery widget (design/04-COMPONENTS.md
//! "Widgets"; design/20 section 1.14; sill FINDINGS Q183). No ring for a level existed: the
//! SendPill's ring is a countdown drawn inside the pill.
//!
//! The ring's circle has a circumference of 100 units, so its dash is the level in percent. The
//! paint is SVG attributes on `currentColor` (CSS does not reach inside an SVG on Blitz, spike
//! S6), and the dash is an attribute Rust rewrites, with no transition (O-20). The level bumps
//! once when its percentage changes.

use crate::components::bump_on::{bump_attrs, use_bump_on};
use crate::components::text_runs::Text;
use crate::components::vocab::Fraction;
use dioxus::prelude::*;

/// The ring's radius for a circumference of 100 in its 36-unit box: `100 / 2π`.
const RADIUS: &str = "15.9155";

/// Whether the level is filling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RingMark {
    /// Just the level.
    #[default]
    Plain,
    /// Charging: a bolt at the top, and the ring never reads as low.
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

/// How full a ring reads, which picks its colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum RingTone {
    /// Above a fifth, or charging: `--ok`.
    Ok,
    /// A fifth or less: `--warn`.
    Low,
    /// A tenth or less: `--danger`.
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

/// The level's dash on the 100-unit circle: `42.5 100`; `None` at zero, where a round cap
/// would still draw a dot.
fn dash(level: Fraction) -> Option<String> {
    let permille = level.clamped().0;
    (permille > 0).then(|| match permille % 10 {
        0 => format!("{} 100", permille / 10),
        tenth => format!("{}.{tenth} 100", permille / 10),
    })
}

/// A level ring at `level` (permille), charging or not, named `label` for assistive
/// technology; `children` (a device's glyph) sit in the middle.
#[component]
pub fn LevelRing(
    level: Fraction,
    #[props(default)] mark: RingMark,
    #[props(into)] label: Text,
    children: Element,
) -> Element {
    let level = level.clamped();
    let percent = (level.0 + 5) / 10;
    let (class, alias) = bump_attrs("ds-ring", use_bump_on(percent));
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
            {ring(dash(level))}
            span { class: "ds-ring-centre", {children} }
            if mark == RingMark::Charging {
                span { class: "ds-ring-bolt", {bolt()} }
            }
        }
    }
}

/// The track and the level arc, turned so the level starts at twelve.
fn ring(dash: Option<String>) -> Element {
    rsx! {
        svg {
            class: "ds-ring-arc",
            "data-ds-svg": "ring",
            view_box: "0 0 36 36",
            "aria-hidden": "true",
            circle {
                cx: "18",
                cy: "18",
                r: RADIUS,
                fill: "none",
                stroke: "currentColor",
                "stroke-width": "3.5",
                opacity: ".2",
            }
            if let Some(dash) = dash {
                circle {
                    cx: "18",
                    cy: "18",
                    r: RADIUS,
                    fill: "none",
                    stroke: "currentColor",
                    "stroke-width": "3.5",
                    "stroke-linecap": "round",
                    "stroke-dasharray": dash,
                    transform: "rotate(-90 18 18)",
                }
            }
        }
    }
}

/// The charging bolt, on `currentColor`.
fn bolt() -> Element {
    rsx! {
        svg {
            class: "ds-ring-bolt-mark",
            "data-ds-svg": "bolt",
            view_box: "0 0 12 12",
            "aria-hidden": "true",
            path { d: "M6.8 1 2.8 6.6h2.9L5.2 11l4-5.6H6.3L6.8 1Z", fill: "currentColor" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RingMark, RingTone, dash};
    use crate::components::vocab::Fraction;

    #[test]
    fn the_dash_is_the_level_in_percent() {
        let cases = [
            (0, None),
            (5, Some("0.5 100")),
            (425, Some("42.5 100")),
            (1000, Some("100 100")),
            (1500, Some("100 100")),
        ];
        for (permille, want) in cases {
            assert_eq!(dash(Fraction(permille)).as_deref(), want, "{permille}");
        }
    }

    #[test]
    fn a_ring_reads_low_only_while_draining() {
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
