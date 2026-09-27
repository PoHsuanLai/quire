//! A connected device's battery at the end of its settings row (design/26-DETAILS.md 5.2.3 G21):
//! the battery glyph and its percentage. When it first shows on a pane just opened, or arrives on
//! a row already showing (the device just connected), the fill sweeps in from empty over
//! `--t-sweep` with the number counting in step; a pane re-mounted in place shows it still (R1); a
//! later level sweeps from where it is, counting only a change of more than a point (R12);
//! Reduced shows the level at once (R7).

use crate::components::status::{BatteryGlyph, BatteryState};
use crate::components::vocab::Fraction;
use crate::detail::{
    CountPace, Detailed, FirstShow, Moment, Touch, use_count_up, use_detail, use_sweep,
};
use crate::icon::render::IconSize;
use dioxus::prelude::*;

/// A battery's level as the row prints it: whole percent (R2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Percent(pub(crate) u16);

impl Percent {
    /// The whole percent of `level`, rounded.
    pub(crate) fn of(level: Fraction) -> Percent {
        Percent((level.clamped().0 + 5) / 10)
    }
}

impl Detailed for Percent {
    fn moment(from: &Self, to: &Self) -> Moment {
        if from == to {
            Moment::Rest
        } else {
            Moment::Change
        }
    }

    /// A value shown for the first time on a surface just opened (design/26 3.1 Appear).
    fn first(_: &Self) -> Moment {
        Moment::Appear
    }
}

/// The glyph and the counting percentage, `84%` in tabular figures; `first` is `Animate` when
/// the battery arrives with a surface just opened or after the row was already showing.
#[component]
pub(crate) fn RowBattery(level: Fraction, first: FirstShow) -> Element {
    let percent = Percent::of(level);
    let detail = use_detail(percent, first, Touch::Remote);
    // The glyph's own fill runs the same sweep; this one only paces the count, so it lands with it.
    let sweep = use_sweep(level.clamped(), detail.cue());
    let shown = use_count_up(i64::from(percent.0), detail.cue(), CountPace::InStep(sweep)).shown();
    let state = BatteryState {
        level: level.clamped(),
        ..BatteryState::default()
    };
    rsx! {
        BatteryGlyph { state, size: IconSize::Compact, first }
        span { class: "ds-settings-row-figure", "{shown}%" }
    }
}

#[cfg(test)]
mod tests {
    use super::Percent;
    use crate::components::vocab::Fraction;
    use crate::detail::{Moment, first_table, moment_table};

    #[test]
    fn the_battery_counts_what_it_prints() {
        assert_eq!(Percent::of(Fraction(844)), Percent(84));
        assert_eq!(Percent::of(Fraction(845)), Percent(85));
        moment_table(&[
            (Percent(84), Percent(84), Moment::Rest),
            (Percent(84), Percent(60), Moment::Change),
        ]);
        first_table(&[(Percent(84), Moment::Appear)]);
    }
}
