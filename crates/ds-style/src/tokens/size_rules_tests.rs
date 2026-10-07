//! One table over every control size checking the ladder's rules (design/29-SIZING.md section 6,
//! R1-R8) and the numbers of design/30 section 1.6.

use super::control_size::ControlSize;
use super::size_scale::{HalfPx, KNOB_INSET, SizeScale, WholePx, on_grid};
use super::size_vars::{SizeToken, SizeVar};
use ds_core::word::Word;

/// Per size: height, radius, capsule radius (half px), knob, switch w x h, switch knob, slider
/// track and knob, segment and its radius, glyph, label inset, checkbox, spinner, progress bar,
/// label size.
type Row = (ControlSize, [u16; 17]);

const CASES: &[Row] = &[
    (
        ControlSize::Mini,
        [
            20, 6, 20, 18, 32, 18, 16, 3, 14, 18, 5, 12, 8, 12, 10, 4, 10,
        ],
    ),
    (
        ControlSize::Small,
        [
            24, 7, 24, 22, 38, 22, 20, 4, 16, 22, 6, 14, 10, 14, 16, 6, 11,
        ],
    ),
    (
        ControlSize::Regular,
        [
            28, 8, 28, 26, 44, 26, 24, 4, 22, 26, 7, 16, 12, 16, 32, 6, 13,
        ],
    ),
    (
        ControlSize::Large,
        [
            32, 16, 32, 30, 44, 26, 24, 4, 22, 30, 15, 18, 16, 16, 32, 6, 15,
        ],
    ),
    (
        ControlSize::ExtraLarge,
        [
            40, 20, 40, 38, 44, 26, 24, 4, 22, 38, 19, 20, 20, 16, 32, 6, 17,
        ],
    ),
];

fn numbers(scale: SizeScale) -> [u16; 17] {
    [
        scale.height.0,
        scale.radius.0,
        scale.capsule_radius().0,
        scale.knob().0,
        scale.switch_width().0,
        scale.switch_height.0,
        scale.switch_knob().0,
        scale.slider_track.0,
        scale.slider_knob.0,
        scale.segment().0,
        scale.segment_radius().0,
        scale.glyph.0,
        scale.pad_x.0,
        scale.checkbox.0,
        scale.spinner.0,
        scale.progress_bar.0,
        scale.font.0,
    ]
}

#[test]
fn every_size_obeys_the_rules() {
    let inset = KNOB_INSET.0;
    for (size, want) in CASES {
        let scale = size.scale();
        assert_eq!(numbers(scale), *want, "{size:?}: the settled numbers");
        // R2: a capsule is half its height.
        assert_eq!(
            scale.capsule_radius(),
            HalfPx(scale.height.0),
            "{size:?} R2"
        );
        assert_eq!(
            scale.switch_radius(),
            HalfPx(scale.switch_height.0),
            "{size:?} R2 switch"
        );
        // R3: a knob is its track less the inset each side.
        for (knob, track) in [
            (scale.knob(), scale.height),
            (scale.segment(), scale.height),
            (scale.switch_knob(), scale.switch_height),
        ] {
            assert_eq!(knob.0 + 2 * inset, track.0, "{size:?} R3");
        }
        assert_eq!(
            scale.switch_travel().0,
            scale.switch_width().0 - scale.switch_height.0,
            "{size:?} R3 travel"
        );
        // R4: the switch is round_even(1.73 h) wide.
        let exact = f32::from(scale.switch_height.0) * 1.73;
        assert_eq!(
            scale.switch_width().0,
            ((exact / 2.0).round() * 2.0) as u16,
            "{size:?} R4"
        );
        // R5: a rounded rectangle's radius is well short of a capsule's; Large and
        // ExtraLarge are capsules (design/34 section 2.2), their radius half the height (R2).
        let capsule = matches!(size, ControlSize::Large | ControlSize::ExtraLarge);
        assert!(
            if capsule {
                scale.radius.0 * 2 == scale.height.0
            } else {
                scale.radius.0 * 3 <= scale.height.0
            },
            "{size:?} R5"
        );
        // R6: the segment is concentric in its well; an inset child loses the inset.
        assert_eq!(
            scale.segment_radius().0 + inset,
            scale.well_radius.0,
            "{size:?} R6"
        );
        assert_eq!(
            scale.inner_radius(KNOB_INSET),
            WholePx(scale.radius.0 - inset),
            "{size:?} R6 inner"
        );
        // R7, R8: the inset is on the grid; the glyph fits inside the control.
        assert!(on_grid(scale.pad_x.0), "{size:?} R7");
        assert!(scale.glyph.0 < scale.height.0, "{size:?} R8");
    }
}

#[test]
fn the_ladder_ascends() {
    let heights: Vec<_> = ControlSize::ALL
        .iter()
        .map(|size| size.scale().height.0)
        .collect();
    let glyphs: Vec<_> = ControlSize::ALL
        .iter()
        .map(|size| size.scale().glyph.0)
        .collect();
    assert!(
        heights.windows(2).all(|pair| pair[0] < pair[1]),
        "{heights:?}"
    );
    assert!(
        glyphs.windows(2).all(|pair| pair[0] < pair[1]),
        "{glyphs:?}"
    );
}

#[test]
fn every_height_token_is_whole() {
    let heights = [
        SizeVar::Height,
        SizeVar::Knob,
        SizeVar::SwitchWidth,
        SizeVar::SwitchHeight,
        SizeVar::SwitchKnob,
        SizeVar::SliderKnob,
        SizeVar::CheckboxBox,
        SizeVar::Spinner,
        SizeVar::Segment,
    ];
    for size in ControlSize::ALL.iter().copied() {
        for var in heights {
            let css = var.css(size.scale());
            assert!(
                css.ends_with("px") && !css.contains('.'),
                "{var:?} at {size:?}: {css}"
            );
        }
    }
    let mini = ControlSize::Mini.scale();
    assert_eq!(
        SizeVar::SwitchRadius.css(mini),
        HalfPx(mini.switch_height.0).css()
    );
    assert_eq!(HalfPx(15).css(), "7.5px");
    assert_eq!(
        SizeToken::ALL.len(),
        SizeVar::ALL.len() * ControlSize::ALL.len() + 1
    );
}
