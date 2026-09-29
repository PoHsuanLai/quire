//! DeviceBattery: one device's battery in the control center's Battery module (design/26-DETAILS.md
//! 5.2.9): the battery ring with its percentage under it, on the grammar's own primitives.
//! On a center just opened (`FirstShow::Animate`) the arc sweeps from empty over `--t-sweep` and
//! the number counts up in step with it (the user's ask of 2026-09-26); a later level sweeps from
//! where the arc is over `--t-quick`, counting only a change of more than a point (R12); the same
//! percentage restated is no moment (R2); Reduced shows the level at once (R7). While charging,
//! the bolt fades in once the sweep has landed.

use crate::components::content::text_runs::TextLine;
use crate::core::vocab::Fraction;
use crate::core::word::Word;
use crate::motion::detail::{
    count_up::{CountPace, use_count_up},
    detailed::Detailed,
    first_show::FirstShow,
    moment::Moment,
    sweep::use_sweep,
    touch::Touch,
    use_detail::use_detail,
};
use crate::shell::battery::level::{BOLT, RingLayer, RingMark, RingTone, given, percent_of, ring};
use crate::shell::battery::ring::{RingSpan, arc_path};
use dioxus::prelude::*;

/// What a device's battery shows: the whole percent and the mark (R2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Reading {
    pub(crate) percent: u16,
    pub(crate) mark: RingMark,
}

impl Detailed for Reading {
    fn moment(from: &Self, to: &Self) -> Moment {
        if from == to {
            Moment::Rest
        } else {
            Moment::Change
        }
    }

    /// A level shown for the first time on a surface just opened sweeps in (design/26 3.1).
    fn first(_: &Self) -> Moment {
        Moment::Appear
    }
}

/// The ring at `level` (permille), charging or not, named `label`, with the percentage under it;
/// `children` (the device's glyph, optional) sit in the ring's middle. `first` is `Animate` on
/// a control center just opened (the default `Still` draws it at its level).
#[component]
pub fn DeviceBattery(
    level: Fraction,
    #[props(default)] mark: RingMark,
    #[props(into)] label: TextLine,
    #[props(default)] first: FirstShow,
    children: Element,
) -> Element {
    let level = level.clamped();
    let reading = Reading {
        percent: percent_of(level),
        mark,
    };
    let detail = use_detail(reading, first, Touch::Remote);
    let sweep = use_sweep(level, detail.cue());
    let shown = use_count_up(
        i64::from(reading.percent),
        detail.cue(),
        CountPace::InStep(sweep),
    )
    .shown();
    let span = match mark {
        RingMark::Plain => RingSpan::FULL,
        RingMark::Charging => RingSpan::GAPPED,
    };
    let landed = sweep.tween().landed();
    rsx! {
        div { class: "ds-device-battery",
            div {
                class: "ds-battery",
                "data-tone": RingTone::of(level, mark).slug(),
                "data-mark": mark.attr(),
                role: "progressbar",
                "aria-label": "{label.plain_text()}",
                "aria-valuemin": "0",
                "aria-valuemax": "100",
                "aria-valuenow": "{reading.percent}",
                {ring(RingLayer::Track, arc_path(span))}
                {ring(RingLayer::Arc, arc_path(span.filled(sweep.share())))}
                if let Some(device) = given(children) {
                    span { class: "ds-battery-device", {device} }
                }
                if mark == RingMark::Charging {
                    svg {
                        class: "ds-battery-bolt",
                        "data-ds-svg": "battery",
                        "data-show": if landed { "on" } else { "off" },
                        view_box: "0 0 10 16",
                        "aria-hidden": "true",
                        path { d: BOLT, fill: "currentColor" }
                    }
                }
            }
            span { class: "ds-battery-figure", "aria-hidden": "true", "{shown}%" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Reading;
    use crate::motion::detail::{
        detailed::{first_table, moment_table},
        moment::Moment,
    };
    use crate::shell::battery::level::RingMark;

    #[test]
    fn a_ring_moves_only_for_what_it_prints() {
        let at = |percent, mark| Reading { percent, mark };
        moment_table(&[
            (
                at(93, RingMark::Plain),
                at(93, RingMark::Plain),
                Moment::Rest,
            ),
            (
                at(93, RingMark::Plain),
                at(92, RingMark::Plain),
                Moment::Change,
            ),
            (
                at(93, RingMark::Plain),
                at(93, RingMark::Charging),
                Moment::Change,
            ),
        ]);
        first_table(&[(at(93, RingMark::Plain), Moment::Appear)]);
    }
}
