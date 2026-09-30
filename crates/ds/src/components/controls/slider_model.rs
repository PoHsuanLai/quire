//! What a `Slider` looks like and where it stops (design/30 section 2.1, `NSSlider`): data and
//! the pure arithmetic of its tick marks.

use ds_core::vocab::Fraction;
use ds_core::word::Word;

/// How the slider is drawn, `data-look`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum SliderLook {
    /// A thin track under a round knob: the form slider of a settings row.
    #[default]
    Linear,
    /// A thick capsule whose fill is the material's bright ink, its glyph inside at the left end,
    /// two-toned where the fill covers it: the control center's volume and brightness.
    Capsule,
    /// The capsule with a separate round knob riding the fill's end, the glyph before it.
    CapsuleKnob,
}

/// The tick marks of a linear slider (`NSSlider.numberOfTickMarks`): none, or one every so much
/// of the range, at which the value snaps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Ticks {
    /// No marks; any value.
    #[default]
    None,
    /// A mark at every multiple of this share of the range, from the start to the end.
    Every(Fraction),
}

impl Ticks {
    /// The spacing of the marks, or `None` for no marks (and for a spacing of nothing).
    fn spacing(self) -> Option<u16> {
        match self {
            Ticks::None => None,
            Ticks::Every(share) => Some(share.clamped().0).filter(|spacing| *spacing > 0),
        }
    }

    /// `value` moved to the nearest mark, or as it is with no marks. Halfway rounds up.
    pub(crate) fn snap(self, value: Fraction) -> Fraction {
        let value = value.clamped().0;
        match self.spacing() {
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
    pub(crate) fn marks(self) -> Vec<Fraction> {
        match self.spacing() {
            None => Vec::new(),
            Some(spacing) => (0..=1000)
                .step_by(usize::from(spacing))
                .map(Fraction)
                .collect(),
        }
    }

    /// One key press's step: the marks' spacing, when there are marks.
    pub(crate) fn key_step(self) -> Option<Fraction> {
        self.spacing().map(Fraction)
    }
}

#[cfg(test)]
mod tests {
    use super::{SliderLook, Ticks};
    use ds_core::vocab::Fraction;
    use ds_core::word::Word;

    #[test]
    fn a_value_snaps_to_the_nearest_mark_and_halfway_goes_up() {
        // (ticks, value, snapped)
        const CASES: &[(Ticks, u16, u16)] = &[
            (Ticks::None, 333, 333),
            (Ticks::Every(Fraction(250)), 0, 0),
            (Ticks::Every(Fraction(250)), 124, 0),
            (Ticks::Every(Fraction(250)), 125, 250),
            (Ticks::Every(Fraction(250)), 610, 500),
            (Ticks::Every(Fraction(250)), 990, 1000),
            (Ticks::Every(Fraction(300)), 950, 900),
            (Ticks::Every(Fraction(300)), 990, 900),
            (Ticks::Every(Fraction(0)), 333, 333),
        ];
        for &(ticks, value, want) in CASES {
            assert_eq!(
                ticks.snap(Fraction(value)),
                Fraction(want),
                "{ticks:?} {value}"
            );
        }
    }

    #[test]
    fn marks_run_from_the_start_and_never_past_the_end() {
        let marks = |ticks: Ticks| ticks.marks().iter().map(|mark| mark.0).collect::<Vec<_>>();
        assert_eq!(marks(Ticks::None), Vec::<u16>::new());
        assert_eq!(marks(Ticks::Every(Fraction(250))), [0, 250, 500, 750, 1000]);
        assert_eq!(marks(Ticks::Every(Fraction(300))), [0, 300, 600, 900]);
        assert_eq!(marks(Ticks::Every(Fraction(0))), Vec::<u16>::new());
    }

    #[test]
    fn a_key_moves_one_mark() {
        assert_eq!(Ticks::None.key_step(), None);
        assert_eq!(Ticks::Every(Fraction(250)).key_step(), Some(Fraction(250)));
    }

    #[test]
    fn every_look_has_its_word() {
        for look in SliderLook::ALL {
            assert_eq!(SliderLook::parse(look.slug()), Some(*look));
        }
        assert_eq!(SliderLook::CapsuleKnob.slug(), "capsule-knob");
    }
}
