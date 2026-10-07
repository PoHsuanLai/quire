//! The glyph a control of a size draws (design/30 section 1.6): 12, 14, 16, 18 and 20 on the ladder.

use ds_style::icon::render::{IconPx, IconSize};
use ds_style::tokens::control_size::ControlSize;

/// The glyph size that goes with `size`, read from the ladder so the two cannot disagree.
pub(crate) fn glyph_size(size: ControlSize) -> IconSize {
    match size.scale().glyph.0 {
        12 => IconSize::Tiny,
        14 => IconSize::Compact,
        16 => IconSize::Base,
        px => IconSize::Px(IconPx(u8::try_from(px).unwrap_or(u8::MAX))),
    }
}

#[cfg(test)]
mod tests {
    use super::glyph_size;
    use ds_style::icon::render::{IconPx, IconSize};
    use ds_style::tokens::control_size::ControlSize;

    #[test]
    fn each_rung_draws_its_ladder_glyph() {
        const CASES: &[(ControlSize, IconSize)] = &[
            (ControlSize::Mini, IconSize::Tiny),
            (ControlSize::Small, IconSize::Compact),
            (ControlSize::Regular, IconSize::Base),
            (ControlSize::Large, IconSize::Px(IconPx(18))),
            (ControlSize::ExtraLarge, IconSize::Px(IconPx(20))),
        ];
        for &(size, want) in CASES {
            assert_eq!(glyph_size(size), want, "{size:?}");
        }
    }
}
