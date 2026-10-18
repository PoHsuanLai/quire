//! The size ladder as custom properties (design/29-SIZING.md section 10): each
//! [`ControlSize`]'s [`SizeScale`] written as `--ctl-*`, `--knob-*`, `--switch-*`, `--slider-*`
//! `--check-*`, `--spinner-*`, `--progress-*` and `--seg-*` tokens with an `-xs`, `-s`, `-m`, `-l` or `-xl`
//! suffix (Mini, Small, Regular, Large, ExtraLarge), so the sheets read the ladder and never
//! restate a number. The values come from `SizeScale`, so the rules hold in CSS as in Rust.

use super::control_size::ControlSize;
use super::name::VarName;
use super::size_scale::{KNOB_INSET, SizeScale};
use super::token::{CssValue, Token, TokenKind, TokenScope};
use ds_core::word::Word;
use std::sync::OnceLock;

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
    /// `--check-*`: a checkbox or radio button's box.
    CheckboxBox,
    /// `--spinner-*`: the spinner's box.
    Spinner,
    /// `--progress-*`: a progress bar's thickness.
    ProgressBar,
    /// `--seg-well-r-*`: the segmented well's radius.
    WellRadius,
    /// `--seg-h-*`: a selected segment's height.
    Segment,
    /// `--seg-r-*`: a selected segment's radius.
    SegmentRadius,
}

impl SizeVar {
    /// What its custom property starts with: `ctl-h` in `--ctl-h-xs`.
    fn stem(self) -> &'static str {
        match self {
            SizeVar::Height => "ctl-h",
            SizeVar::Radius => "ctl-r",
            SizeVar::CapsuleRadius => "ctl-cap-r",
            SizeVar::Glyph => "ctl-glyph",
            SizeVar::PadX => "ctl-pad",
            SizeVar::Font => "ctl-fs",
            SizeVar::Weight => "ctl-fw",
            SizeVar::Knob => "knob",
            SizeVar::SwitchWidth => "switch-w",
            SizeVar::SwitchHeight => "switch-h",
            SizeVar::SwitchRadius => "switch-r",
            SizeVar::SwitchKnob => "switch-knob",
            SizeVar::SliderTrack => "slider-track",
            SizeVar::SliderKnob => "slider-knob",
            SizeVar::CheckboxBox => "check",
            SizeVar::Spinner => "spinner",
            SizeVar::ProgressBar => "progress",
            SizeVar::WellRadius => "seg-well-r",
            SizeVar::Segment => "seg-h",
            SizeVar::SegmentRadius => "seg-r",
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
            SizeVar::SliderKnob => scale.slider_knob.css(),
            SizeVar::CheckboxBox => scale.checkbox.css(),
            SizeVar::Spinner => scale.spinner.css(),
            SizeVar::ProgressBar => scale.progress_bar.css(),
            SizeVar::WellRadius => scale.well_radius.css(),
            SizeVar::Segment => scale.segment().css(),
            SizeVar::SegmentRadius => scale.segment_radius().css(),
        }
    }
}

/// What a control size's custom properties end with: `xs` in `--ctl-h-xs`.
fn suffix(size: ControlSize) -> &'static str {
    match size {
        ControlSize::Mini => "xs",
        ControlSize::Small => "s",
        ControlSize::Regular => "m",
        ControlSize::Large => "l",
        ControlSize::ExtraLarge => "xl",
    }
}

/// One token of the size ladder: a quantity at a control size, or the knob inset every size
/// shares. The ladder is the product of the sizes and the quantities (each size's whole set, small
/// to large), then the inset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SizeToken {
    /// `--ctl-h-xs` and the rest: the quantity at the size.
    At(ControlSize, SizeVar),
    /// `--knob-inset`: what a knob or a selected segment keeps from its track's edge.
    KnobInset,
}

/// How many tokens the ladder has.
const LADDER_LEN: usize = ControlSize::ALL.len() * SizeVar::ALL.len() + 1;

/// Every token, in the ladder's order.
const LADDER: [SizeToken; LADDER_LEN] = ladder();

const fn ladder() -> [SizeToken; LADDER_LEN] {
    let mut all = [SizeToken::KnobInset; LADDER_LEN];
    let mut size = 0;
    while size < ControlSize::ALL.len() {
        let mut var = 0;
        while var < SizeVar::ALL.len() {
            all[size * SizeVar::ALL.len() + var] =
                SizeToken::At(ControlSize::ALL[size], SizeVar::ALL[var]);
            var += 1;
        }
        size += 1;
    }
    all
}

/// Where `member` stands in `all`.
fn place<T: Word>(all: &[T], member: T) -> usize {
    all.iter()
        .position(|other| *other == member)
        .unwrap_or_default()
}

impl SizeToken {
    /// Its place in [`Word::ALL`].
    fn index(self) -> usize {
        match self {
            SizeToken::At(size, var) => {
                place(ControlSize::ALL, size) * SizeVar::ALL.len() + place(SizeVar::ALL, var)
            }
            SizeToken::KnobInset => LADDER_LEN - 1,
        }
    }

    /// The custom property this token is declared as.
    pub fn var(self) -> VarName {
        static NAMES: OnceLock<Vec<&'static str>> = OnceLock::new();
        let names = NAMES.get_or_init(|| {
            LADDER
                .iter()
                .map(|token| {
                    let written = match token {
                        SizeToken::At(size, var) => format!("--{}-{}", var.stem(), suffix(*size)),
                        SizeToken::KnobInset => "--knob-inset".to_owned(),
                    };
                    &*Box::leak(written.into_boxed_str())
                })
                .collect()
        });
        VarName(names.get(self.index()).copied().unwrap_or_default())
    }
}

impl Word for SizeToken {
    const ALL: &'static [Self] = &LADDER;

    fn slug(self) -> &'static str {
        self.var().as_str().trim_start_matches("--")
    }

    fn label(self) -> &'static str {
        self.slug()
    }
}

impl Token for SizeToken {
    const PREFIX: &'static str = "";
    const KIND: TokenKind = TokenKind::Fixed;

    fn var(self) -> VarName {
        SizeToken::var(self)
    }

    fn css_value(self, _scope: TokenScope) -> CssValue {
        CssValue::computed(match self {
            SizeToken::At(size, var) => var.css(size.scale()),
            SizeToken::KnobInset => KNOB_INSET.css(),
        })
    }
}
