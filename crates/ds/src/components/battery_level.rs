//! BatteryLevel: a battery's charge as a battery glyph filled to the level, for the Battery
//! widget (design/23-WIDGETS.md section 4.1; design/04-COMPONENTS.md "Widgets"; sill FINDINGS
//! Q183). The widget puts the percentage beside it in the display face; the glyph is the
//! gauge, drawn in the soft language (design/23 section 2): a rounded body of the plate's own
//! colour pushed out of it, a channel pressed into the body, a solid matte fill in the channel
//! as long as the level, a small terminal cap, and a bolt over the body while charging.
//!
//! The body, channel, fill and cap are boxes (the soft shadows are `box-shadow`s, which an SVG
//! cannot take); the fill's length is a custom property Rust writes, with no transition (O-20).
//! The bolt is SVG on `currentColor` (spike S6), twice: its knockout in the plate's colour and
//! the bolt itself. The level bumps once when its percentage changes.

use crate::components::bump_on::{bump_attrs, use_bump_on};
use crate::components::text_runs::Text;
use crate::components::vocab::Fraction;
use dioxus::prelude::*;

/// The charging bolt in the glyph's box, centred on the case.
const BOLT: &str = "M15.6 1.6 9.2 9.1h4.3l-1.1 5.3 6.4-7.5h-4.3l1.1-5.3Z";

/// Whether the battery is filling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RingMark {
    /// Just the level.
    #[default]
    Plain,
    /// Charging: a bolt over the case, the fill in `--ok`, and never low.
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
    /// Above a fifth, or charging: `--ink` (`--ok` while charging).
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

/// A battery glyph at `level` (permille), charging or not, named `label` for assistive
/// technology. `children` (a device's glyph, optional) sit before the battery.
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
            if let Some(device) = given(children) {
                span { class: "ds-battery-device", {device} }
            }
            span { class: "ds-battery-glyph",
                span { class: "ds-battery-body",
                    span { class: "ds-battery-channel",
                        if level.0 > 0 {
                            span { class: "ds-battery-fill", style: "--f:{level.css()}" }
                        }
                    }
                }
                span { class: "ds-battery-cap" }
                if mark == RingMark::Charging {
                    {bolt(BoltLayer::Knockout)}
                    {bolt(BoltLayer::Mark)}
                }
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
/// props are the same, the drawing is the battery glyph.
pub use BatteryLevel as LevelRing;

/// One of the bolt's two layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BoltLayer {
    /// Its outline in the plate's colour, cutting the bolt out of the case and the fill.
    Knockout,
    /// The bolt itself.
    Mark,
}

impl BoltLayer {
    fn class(self) -> &'static str {
        match self {
            BoltLayer::Knockout => "ds-battery-knockout",
            BoltLayer::Mark => "ds-battery-bolt",
        }
    }

    /// The outline's width: only the knockout has one.
    fn stroke(self) -> &'static str {
        match self {
            BoltLayer::Knockout => "3",
            BoltLayer::Mark => "0",
        }
    }
}

/// The charging bolt's `layer`.
fn bolt(layer: BoltLayer) -> Element {
    rsx! {
        svg { class: layer.class(), "data-ds-svg": "battery", view_box: "0 0 32 16", "aria-hidden": "true",
            path { d: BOLT, fill: "currentColor", stroke: "currentColor", "stroke-width": layer.stroke(), "stroke-linejoin": "round" }
        }
    }
}

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
