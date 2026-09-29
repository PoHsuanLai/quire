//! One table over every control size checking the ladder's rules (design/29-SIZING.md section 6,
//! R1-R8) and the settled numbers of option A (section 7).

use super::control_center::CONTROL_CENTER;
use super::control_size::ControlSize;
use super::shell_scale::SHELL_SCALE;
use super::size_scale::{HalfPx, KNOB_INSET, SizeScale, WholePx};
use super::size_vars::{SizeVar, size_tokens};
use crate::core::word::Word;

/// The spacing steps new sizes are drawn from (R7): a 4 px grid with a 2 px half step.
const SPACING_GRID: [u16; 10] = [2, 4, 6, 8, 10, 12, 16, 20, 24, 32];

/// Whether `px` is a step of [`SPACING_GRID`].
fn on_grid(px: u16) -> bool {
    SPACING_GRID.contains(&px)
}

/// Per size: height, radius, capsule radius (half px), knob, switch w x h, switch knob, slider
/// track and knob, segment and its radius, glyph.
type Row = (ControlSize, [u16; 13]);

const CASES: &[Row] = &[
    (
        ControlSize::Small,
        [16, 4, 16, 14, 26, 15, 13, 4, 14, 14, 3, 12, 6],
    ),
    (
        ControlSize::Regular,
        [22, 5, 22, 20, 38, 22, 20, 4, 20, 20, 5, 16, 10],
    ),
    (
        ControlSize::Large,
        [28, 6, 28, 26, 48, 28, 26, 6, 26, 26, 6, 18, 12],
    ),
];

fn numbers(scale: SizeScale) -> [u16; 13] {
    [
        scale.height.0,
        scale.radius.0,
        scale.capsule_radius().0,
        scale.knob().0,
        scale.switch_width().0,
        scale.switch_height.0,
        scale.switch_knob().0,
        scale.slider_track.0,
        scale.slider_knob().0,
        scale.segment().0,
        scale.segment_radius().0,
        scale.glyph.0,
        scale.pad_x.0,
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
            (scale.slider_knob(), scale.height),
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
        // R5: a rounded rectangle's radius is well short of a capsule's.
        assert!(scale.radius.0 * 4 <= scale.height.0, "{size:?} R5");
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
        SizeVar::Segment,
    ];
    for size in ControlSize::ALL.iter().copied() {
        for var in heights {
            let css = var.css(size.scale());
            assert!(
                css.ends_with("px") && !css.contains('.'),
                "{}: {css}",
                var.var(size).as_str()
            );
        }
    }
    assert_eq!(
        SizeVar::SwitchRadius.css(ControlSize::Small.scale()),
        "7.5px"
    );
    assert_eq!(size_tokens().len(), SizeVar::ALL.len() * 3 + 1);
}

#[test]
fn the_shell_surfaces_nest_concentrically() {
    let cc = CONTROL_CENTER;
    // R6: the panel is its modules' radius plus the padding round them.
    assert_eq!(cc.panel_radius(), WholePx(18));
    assert_eq!(cc.level_module(), WholePx(64));
    assert_eq!(cc.tile.0, 2 * ControlSize::Large.scale().height.0);
    for step in [cc.padding, cc.gap, cc.module_radius, cc.head_gap] {
        assert!(on_grid(step.0), "{step:?} R7");
    }
    // The bar's items centre in it with the same space above and below.
    assert_eq!(SHELL_SCALE.item(), WholePx(22));
    assert_eq!((SHELL_SCALE.bar.0 - SHELL_SCALE.item().0) % 2, 0);
}
