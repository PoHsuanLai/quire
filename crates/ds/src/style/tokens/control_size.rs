//! A control's size on the ladder (design/30-CATALOGUE.md section 1.6, macOS's
//! `NSControl.ControlSize`): Mini 16, Small 19, Regular 22, Large 28. Each size's whole geometry
//! comes from [`SizeScale`], computed from its height by the design/29-SIZING.md section 6 rules.

use super::size_scale::{SizeScale, WholePx};
use crate::core::word::Word;

/// How big a control is drawn: its height and everything derived from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum ControlSize {
    /// 16 tall: a settings row's switch, a dense toolbar.
    Mini,
    /// 19 tall: a compact inspector's controls.
    Small,
    /// 22 tall: the default push button, field, menu row and bar item.
    #[default]
    Regular,
    /// 28 tall: a prominent button, a module disc.
    Large,
}

impl ControlSize {
    /// This size's geometry.
    pub fn scale(self) -> SizeScale {
        match self {
            ControlSize::Mini => SizeScale {
                height: WholePx(16),
                radius: WholePx(4),
                well_radius: WholePx(4),
                glyph: WholePx(12),
                pad_x: WholePx(6),
                switch_height: WholePx(15),
                slider_track: WholePx(3),
                slider_knob: WholePx(12),
                checkbox: WholePx(10),
                spinner: WholePx(10),
                progress_bar: WholePx(4),
                font: WholePx(9),
                weight: 600,
            },
            ControlSize::Small => SizeScale {
                height: WholePx(19),
                radius: WholePx(5),
                well_radius: WholePx(6),
                glyph: WholePx(14),
                pad_x: WholePx(8),
                switch_height: WholePx(18),
                slider_track: WholePx(4),
                slider_knob: WholePx(14),
                checkbox: WholePx(12),
                spinner: WholePx(16),
                progress_bar: WholePx(6),
                font: WholePx(11),
                weight: 500,
            },
            ControlSize::Regular => SizeScale {
                height: WholePx(22),
                radius: WholePx(5),
                well_radius: WholePx(6),
                glyph: WholePx(16),
                pad_x: WholePx(10),
                switch_height: WholePx(22),
                slider_track: WholePx(4),
                slider_knob: WholePx(20),
                checkbox: WholePx(14),
                spinner: WholePx(32),
                progress_bar: WholePx(6),
                font: WholePx(13),
                weight: 500,
            },
            ControlSize::Large => SizeScale {
                height: WholePx(28),
                radius: WholePx(5),
                well_radius: WholePx(6),
                glyph: WholePx(20),
                pad_x: WholePx(12),
                switch_height: WholePx(22),
                slider_track: WholePx(4),
                slider_knob: WholePx(20),
                checkbox: WholePx(14),
                spinner: WholePx(32),
                progress_bar: WholePx(6),
                font: WholePx(15),
                weight: 600,
            },
        }
    }
}
