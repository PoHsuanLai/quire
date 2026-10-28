//! What a `Slider` looks like and where it stops (design/30 section 2.1, `NSSlider`): data and
//! the pure arithmetic of its tick marks.

use ds_core::vocab::Fraction;
use ds_core::word::Word;

/// How the slider is drawn, `data-look`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum SliderLook {
    /// A thin track under a round knob, the fill in the accent: `NSSlider`, the form slider of a
    /// settings row. The default for anything a person sets with a pointer.
    #[default]
    Linear,
    /// The Control Center's level slider (Display, Sound, Keyboard Brightness): a 22 high
    /// capsule whose fill ends under a round white knob, the level's glyph inside the knob. The
    /// fill, the well and the knob are the material's level inks. Click anywhere jumps the knob,
    /// a drag moves it, the arrows step a sixteenth (design/29 section 16, design/34 section 3.4).
    Level,
}

// The tick marks of a linear slider (`NSSlider.numberOfTickMarks`) are `Option<Fraction>`:
// `None` for no marks (any value), or a mark at every multiple of that share of the range, from
// the start to the end, at which the value snaps.

/// The spacing of the marks in thousandths, or `None` for no marks (and for a spacing of nothing).
fn spacing(ticks: Option<Fraction>) -> Option<u16> {
    ticks
        .map(|share| share.clamped().0)
        .filter(|spacing| *spacing > 0)
}

/// `value` moved to the nearest mark, or as it is with no marks. Halfway rounds up.
pub(crate) fn snap(ticks: Option<Fraction>, value: Fraction) -> Fraction {
    let value = value.clamped().0;
    match spacing(ticks) {
        None => Fraction(value),
        Some(spacing) => {
            let marks = (u32::from(value) + u32::from(spacing) / 2) / u32::from(spacing);
            Fraction(
                u16::try_from(marks * u32::from(spacing))
                    .unwrap_or(1000)
                    .min(1000),
            )
        }
    }
}

/// Where the marks are, in order, from the start to the last one within the range.
pub(crate) fn marks(ticks: Option<Fraction>) -> Vec<Fraction> {
    match spacing(ticks) {
        None => Vec::new(),
        Some(spacing) => (0..=1000)
            .step_by(usize::from(spacing))
            .map(Fraction)
            .collect(),
    }
}

/// One key press's step: the marks' spacing, when there are marks.
pub(crate) fn key_step(ticks: Option<Fraction>) -> Option<Fraction> {
    spacing(ticks).map(Fraction)
}

#[cfg(test)]
mod tests {
    use super::{SliderLook, key_step, marks, snap};
    use ds_core::vocab::Fraction;
    use ds_core::word::Word;

    #[test]
    fn a_value_snaps_to_the_nearest_mark_and_halfway_goes_up() {
        // (ticks, value, snapped)
        const CASES: &[(Option<Fraction>, u16, u16)] = &[
            (None, 333, 333),
            (Some(Fraction(250)), 0, 0),
            (Some(Fraction(250)), 124, 0),
            (Some(Fraction(250)), 125, 250),
            (Some(Fraction(250)), 610, 500),
            (Some(Fraction(250)), 990, 1000),
            (Some(Fraction(300)), 950, 900),
            (Some(Fraction(300)), 990, 900),
            (Some(Fraction(0)), 333, 333),
        ];
        for &(ticks, value, want) in CASES {
            assert_eq!(
                snap(ticks, Fraction(value)),
                Fraction(want),
                "{ticks:?} {value}"
            );
        }
    }

    #[test]
    fn marks_run_from_the_start_and_never_past_the_end() {
        let listed =
            |ticks: Option<Fraction>| marks(ticks).iter().map(|mark| mark.0).collect::<Vec<_>>();
        assert_eq!(listed(None), Vec::<u16>::new());
        assert_eq!(listed(Some(Fraction(250))), [0, 250, 500, 750, 1000]);
        assert_eq!(listed(Some(Fraction(300))), [0, 300, 600, 900]);
        assert_eq!(listed(Some(Fraction(0))), Vec::<u16>::new());
    }

    #[test]
    fn a_key_moves_one_mark() {
        assert_eq!(key_step(None), None);
        assert_eq!(key_step(Some(Fraction(250))), Some(Fraction(250)));
    }

    #[test]
    fn every_look_has_its_word() {
        for look in SliderLook::ALL {
            assert_eq!(SliderLook::parse(look.slug()), Some(*look));
        }
        assert_eq!(SliderLook::Level.slug(), "level");
    }
}
