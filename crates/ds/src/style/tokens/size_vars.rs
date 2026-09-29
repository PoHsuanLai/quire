//! The size ladder as custom properties (design/29-SIZING.md section 10): each
//! [`ControlSize`]'s [`SizeScale`] written as `--ctl-*`, `--knob-*`, `--switch-*`, `--slider-*`
//! and `--seg-*` tokens with an `-s`, `-m` or `-l` suffix, so the sheets read the ladder and never
//! restate a number. The values come from `SizeScale`, so the rules hold in CSS as in Rust.

use super::control_size::ControlSize;
use super::name::VarName;
use super::size_scale::{KNOB_INSET, SizeScale};
use crate::core::word::Word;

/// One quantity every size has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SizeVar {
    /// `--ctl-h-*`: the control's height.
    Height,
    /// `--ctl-r-*`: a rounded rectangle's radius.
    Radius,
    /// `--ctl-cap-r-*`: a capsule's radius, half the height.
    CapsuleRadius,
    /// `--ctl-glyph-*`: the glyph's box.
    Glyph,
    /// `--ctl-pad-*`: the label's horizontal inset.
    PadX,
    /// `--ctl-fs-*`: the label's size.
    Font,
    /// `--ctl-fw-*`: the label's weight.
    Weight,
    /// `--knob-*`: a knob in a track of the control's height.
    Knob,
    /// `--switch-w-*`: the switch's width.
    SwitchWidth,
    /// `--switch-h-*`: the switch's height.
    SwitchHeight,
    /// `--switch-r-*`: the switch's radius.
    SwitchRadius,
    /// `--switch-knob-*`: the switch's knob.
    SwitchKnob,
    /// `--slider-track-*`: the slider's track thickness.
    SliderTrack,
    /// `--slider-knob-*`: the slider's round knob.
    SliderKnob,
    /// `--seg-well-r-*`: the segmented well's radius.
    WellRadius,
    /// `--seg-h-*`: a selected segment's height.
    Segment,
    /// `--seg-r-*`: a selected segment's radius.
    SegmentRadius,
}

impl SizeVar {
    /// Every quantity, in stylesheet order.
    pub const ALL: [SizeVar; 17] = [
        SizeVar::Height,
        SizeVar::Radius,
        SizeVar::CapsuleRadius,
        SizeVar::Glyph,
        SizeVar::PadX,
        SizeVar::Font,
        SizeVar::Weight,
        SizeVar::Knob,
        SizeVar::SwitchWidth,
        SizeVar::SwitchHeight,
        SizeVar::SwitchRadius,
        SizeVar::SwitchKnob,
        SizeVar::SliderTrack,
        SizeVar::SliderKnob,
        SizeVar::WellRadius,
        SizeVar::Segment,
        SizeVar::SegmentRadius,
    ];

    /// The custom property at `size`: `--ctl-h-m`.
    pub fn var(self, size: ControlSize) -> VarName {
        let [small, regular, large] = self.names();
        VarName(match size {
            ControlSize::Small => small,
            ControlSize::Regular => regular,
            ControlSize::Large => large,
        })
    }

    fn names(self) -> [&'static str; 3] {
        match self {
            SizeVar::Height => ["--ctl-h-s", "--ctl-h-m", "--ctl-h-l"],
            SizeVar::Radius => ["--ctl-r-s", "--ctl-r-m", "--ctl-r-l"],
            SizeVar::CapsuleRadius => ["--ctl-cap-r-s", "--ctl-cap-r-m", "--ctl-cap-r-l"],
            SizeVar::Glyph => ["--ctl-glyph-s", "--ctl-glyph-m", "--ctl-glyph-l"],
            SizeVar::PadX => ["--ctl-pad-s", "--ctl-pad-m", "--ctl-pad-l"],
            SizeVar::Font => ["--ctl-fs-s", "--ctl-fs-m", "--ctl-fs-l"],
            SizeVar::Weight => ["--ctl-fw-s", "--ctl-fw-m", "--ctl-fw-l"],
            SizeVar::Knob => ["--knob-s", "--knob-m", "--knob-l"],
            SizeVar::SwitchWidth => ["--switch-w-s", "--switch-w-m", "--switch-w-l"],
            SizeVar::SwitchHeight => ["--switch-h-s", "--switch-h-m", "--switch-h-l"],
            SizeVar::SwitchRadius => ["--switch-r-s", "--switch-r-m", "--switch-r-l"],
            SizeVar::SwitchKnob => ["--switch-knob-s", "--switch-knob-m", "--switch-knob-l"],
            SizeVar::SliderTrack => ["--slider-track-s", "--slider-track-m", "--slider-track-l"],
            SizeVar::SliderKnob => ["--slider-knob-s", "--slider-knob-m", "--slider-knob-l"],
            SizeVar::WellRadius => ["--seg-well-r-s", "--seg-well-r-m", "--seg-well-r-l"],
            SizeVar::Segment => ["--seg-h-s", "--seg-h-m", "--seg-h-l"],
            SizeVar::SegmentRadius => ["--seg-r-s", "--seg-r-m", "--seg-r-l"],
        }
    }

    /// Its CSS value in `scale`: `22px`, `7.5px`, `500`.
    pub fn css(self, scale: SizeScale) -> String {
        match self {
            SizeVar::Height => scale.height.css(),
            SizeVar::Radius => scale.radius.css(),
            SizeVar::CapsuleRadius => scale.capsule_radius().css(),
            SizeVar::Glyph => scale.glyph.css(),
            SizeVar::PadX => scale.pad_x.css(),
            SizeVar::Font => scale.font.css(),
            SizeVar::Weight => scale.weight.to_string(),
            SizeVar::Knob => scale.knob().css(),
            SizeVar::SwitchWidth => scale.switch_width().css(),
            SizeVar::SwitchHeight => scale.switch_height.css(),
            SizeVar::SwitchRadius => scale.switch_radius().css(),
            SizeVar::SwitchKnob => scale.switch_knob().css(),
            SizeVar::SliderTrack => scale.slider_track.css(),
            SizeVar::SliderKnob => scale.slider_knob().css(),
            SizeVar::WellRadius => scale.well_radius.css(),
            SizeVar::Segment => scale.segment().css(),
            SizeVar::SegmentRadius => scale.segment_radius().css(),
        }
    }
}

/// `--knob-inset`: the inset every knob and selected segment keeps (R3).
pub const KNOB_INSET_VAR: VarName = VarName("--knob-inset");

/// Every size token and its value, size by size: `(--ctl-h-s, 16px)`, …, then the knob inset.
pub fn size_tokens() -> Vec<(VarName, String)> {
    ControlSize::ALL
        .iter()
        .copied()
        .flat_map(|size| {
            SizeVar::ALL
                .into_iter()
                .map(move |var| (var.var(size), var.css(size.scale())))
        })
        .chain([(KNOB_INSET_VAR, KNOB_INSET.css())])
        .collect()
}
