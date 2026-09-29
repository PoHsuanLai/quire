//! The size ladder as custom properties (design/29-SIZING.md section 10): each
//! [`ControlSize`]'s [`SizeScale`] written as `--ctl-*`, `--knob-*`, `--switch-*`, `--slider-*`
//! and `--seg-*` tokens with an `-s`, `-m` or `-l` suffix, so the sheets read the ladder and never
//! restate a number. The values come from `SizeScale`, so the rules hold in CSS as in Rust.

use super::control_size::ControlSize;
use super::size_scale::{KNOB_INSET, SizeScale};
use super::token::{CssValue, Token, TokenScope};
use crate::core::word::Word;

/// One quantity every size has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
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

/// One control size's geometry as a token, in the ladder's order (each size's whole set, small to
/// large), then the knob inset every size shares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "", kind = fixed, css = size_css)]
pub enum SizeToken {
    /// `--ctl-h-s`: Height at Small.
    #[token(name = "ctl-h-s")]
    SmallHeight,
    /// `--ctl-r-s`: Radius at Small.
    #[token(name = "ctl-r-s")]
    SmallRadius,
    /// `--ctl-cap-r-s`: CapsuleRadius at Small.
    #[token(name = "ctl-cap-r-s")]
    SmallCapsuleRadius,
    /// `--ctl-glyph-s`: Glyph at Small.
    #[token(name = "ctl-glyph-s")]
    SmallGlyph,
    /// `--ctl-pad-s`: PadX at Small.
    #[token(name = "ctl-pad-s")]
    SmallPadX,
    /// `--ctl-fs-s`: Font at Small.
    #[token(name = "ctl-fs-s")]
    SmallFont,
    /// `--ctl-fw-s`: Weight at Small.
    #[token(name = "ctl-fw-s")]
    SmallWeight,
    /// `--knob-s`: Knob at Small.
    #[token(name = "knob-s")]
    SmallKnob,
    /// `--switch-w-s`: SwitchWidth at Small.
    #[token(name = "switch-w-s")]
    SmallSwitchWidth,
    /// `--switch-h-s`: SwitchHeight at Small.
    #[token(name = "switch-h-s")]
    SmallSwitchHeight,
    /// `--switch-r-s`: SwitchRadius at Small.
    #[token(name = "switch-r-s")]
    SmallSwitchRadius,
    /// `--switch-knob-s`: SwitchKnob at Small.
    #[token(name = "switch-knob-s")]
    SmallSwitchKnob,
    /// `--slider-track-s`: SliderTrack at Small.
    #[token(name = "slider-track-s")]
    SmallSliderTrack,
    /// `--slider-knob-s`: SliderKnob at Small.
    #[token(name = "slider-knob-s")]
    SmallSliderKnob,
    /// `--seg-well-r-s`: WellRadius at Small.
    #[token(name = "seg-well-r-s")]
    SmallWellRadius,
    /// `--seg-h-s`: Segment at Small.
    #[token(name = "seg-h-s")]
    SmallSegment,
    /// `--seg-r-s`: SegmentRadius at Small.
    #[token(name = "seg-r-s")]
    SmallSegmentRadius,
    /// `--ctl-h-m`: Height at Regular.
    #[token(name = "ctl-h-m")]
    RegularHeight,
    /// `--ctl-r-m`: Radius at Regular.
    #[token(name = "ctl-r-m")]
    RegularRadius,
    /// `--ctl-cap-r-m`: CapsuleRadius at Regular.
    #[token(name = "ctl-cap-r-m")]
    RegularCapsuleRadius,
    /// `--ctl-glyph-m`: Glyph at Regular.
    #[token(name = "ctl-glyph-m")]
    RegularGlyph,
    /// `--ctl-pad-m`: PadX at Regular.
    #[token(name = "ctl-pad-m")]
    RegularPadX,
    /// `--ctl-fs-m`: Font at Regular.
    #[token(name = "ctl-fs-m")]
    RegularFont,
    /// `--ctl-fw-m`: Weight at Regular.
    #[token(name = "ctl-fw-m")]
    RegularWeight,
    /// `--knob-m`: Knob at Regular.
    #[token(name = "knob-m")]
    RegularKnob,
    /// `--switch-w-m`: SwitchWidth at Regular.
    #[token(name = "switch-w-m")]
    RegularSwitchWidth,
    /// `--switch-h-m`: SwitchHeight at Regular.
    #[token(name = "switch-h-m")]
    RegularSwitchHeight,
    /// `--switch-r-m`: SwitchRadius at Regular.
    #[token(name = "switch-r-m")]
    RegularSwitchRadius,
    /// `--switch-knob-m`: SwitchKnob at Regular.
    #[token(name = "switch-knob-m")]
    RegularSwitchKnob,
    /// `--slider-track-m`: SliderTrack at Regular.
    #[token(name = "slider-track-m")]
    RegularSliderTrack,
    /// `--slider-knob-m`: SliderKnob at Regular.
    #[token(name = "slider-knob-m")]
    RegularSliderKnob,
    /// `--seg-well-r-m`: WellRadius at Regular.
    #[token(name = "seg-well-r-m")]
    RegularWellRadius,
    /// `--seg-h-m`: Segment at Regular.
    #[token(name = "seg-h-m")]
    RegularSegment,
    /// `--seg-r-m`: SegmentRadius at Regular.
    #[token(name = "seg-r-m")]
    RegularSegmentRadius,
    /// `--ctl-h-l`: Height at Large.
    #[token(name = "ctl-h-l")]
    LargeHeight,
    /// `--ctl-r-l`: Radius at Large.
    #[token(name = "ctl-r-l")]
    LargeRadius,
    /// `--ctl-cap-r-l`: CapsuleRadius at Large.
    #[token(name = "ctl-cap-r-l")]
    LargeCapsuleRadius,
    /// `--ctl-glyph-l`: Glyph at Large.
    #[token(name = "ctl-glyph-l")]
    LargeGlyph,
    /// `--ctl-pad-l`: PadX at Large.
    #[token(name = "ctl-pad-l")]
    LargePadX,
    /// `--ctl-fs-l`: Font at Large.
    #[token(name = "ctl-fs-l")]
    LargeFont,
    /// `--ctl-fw-l`: Weight at Large.
    #[token(name = "ctl-fw-l")]
    LargeWeight,
    /// `--knob-l`: Knob at Large.
    #[token(name = "knob-l")]
    LargeKnob,
    /// `--switch-w-l`: SwitchWidth at Large.
    #[token(name = "switch-w-l")]
    LargeSwitchWidth,
    /// `--switch-h-l`: SwitchHeight at Large.
    #[token(name = "switch-h-l")]
    LargeSwitchHeight,
    /// `--switch-r-l`: SwitchRadius at Large.
    #[token(name = "switch-r-l")]
    LargeSwitchRadius,
    /// `--switch-knob-l`: SwitchKnob at Large.
    #[token(name = "switch-knob-l")]
    LargeSwitchKnob,
    /// `--slider-track-l`: SliderTrack at Large.
    #[token(name = "slider-track-l")]
    LargeSliderTrack,
    /// `--slider-knob-l`: SliderKnob at Large.
    #[token(name = "slider-knob-l")]
    LargeSliderKnob,
    /// `--seg-well-r-l`: WellRadius at Large.
    #[token(name = "seg-well-r-l")]
    LargeWellRadius,
    /// `--seg-h-l`: Segment at Large.
    #[token(name = "seg-h-l")]
    LargeSegment,
    /// `--seg-r-l`: SegmentRadius at Large.
    #[token(name = "seg-r-l")]
    LargeSegmentRadius,
    /// `--knob-inset`: what a knob or a selected segment keeps from its track's edge.
    #[token(name = "knob-inset")]
    KnobInset,
}

impl SizeToken {
    /// The variable and control size this token is, or `None` for the shared knob inset.
    fn parts(self) -> Option<(SizeVar, ControlSize)> {
        let index = Self::ALL.iter().position(|token| *token == self)?;
        let vars = SizeVar::ALL.len();
        Some((
            *SizeVar::ALL.get(index % vars)?,
            *ControlSize::ALL.get(index / vars)?,
        ))
    }
}

/// A ladder token as the stylesheet writes it.
fn size_css(token: SizeToken, _scope: TokenScope) -> CssValue {
    CssValue::computed(match token.parts() {
        Some((var, size)) => var.css(size.scale()),
        None => KNOB_INSET.css(),
    })
}
