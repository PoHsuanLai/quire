//! A control's size on the ladder (design/30-CATALOGUE.md section 1.6, macOS's
//! `NSControl.ControlSize`): Mini 20, Small 24, Regular 28, Large 32, ExtraLarge 40 (design/34-MODERN-LOOK.md section 2.2,
//! step 2, Tahoe; the heights are an M-low estimate, section 6 decision 1). Each size's whole geometry
//! comes from [`SizeScale`], computed from its height by the design/29-SIZING.md section 6 rules.

use super::size_scale::{SizeScale, WholePx};
use ds_core::word::Word;

/// How big a control is drawn: its height and everything derived from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum ControlSize {
    /// 20 tall: a settings row's switch, a dense toolbar.
    Mini,
    /// 24 tall: a compact inspector's controls.
    Small,
    /// 28 tall: the default push button, field, menu row and bar item.
    #[default]
    Regular,
    /// 32 tall, a capsule: a prominent button, a module disc.
    Large,
    /// 40 tall, a capsule: a hero button or search field.
    ExtraLarge,
}

/// How tall a source list's rows are (design/30 section 1.5): the sidebar's own ladder, chosen by
/// `appearance.sidebar_size`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum SidebarSize {
    /// 28 tall.
    Small,
    /// 32 tall.
    #[default]
    Medium,
    /// 36 tall.
    Large,
}

impl SidebarSize {
    /// A row's height at this size.
    pub fn row_height(self) -> WholePx {
        WholePx(match self {
            SidebarSize::Small => 28,
            SidebarSize::Medium => 32,
            SidebarSize::Large => 36,
        })
    }
}

impl ControlSize {
    /// This size's geometry.
    pub fn scale(self) -> SizeScale {
        match self {
            ControlSize::Mini => SizeScale {
                height: WholePx(20),
                radius: WholePx(6),
                well_radius: WholePx(6),
                glyph: WholePx(12),
                pad_x: WholePx(8),
                switch_height: WholePx(18),
                slider_track: WholePx(3),
                slider_knob: WholePx(14),
                checkbox: WholePx(12),
                spinner: WholePx(10),
                progress_bar: WholePx(4),
                font: WholePx(10),
                weight: 600,
            },
            ControlSize::Small => SizeScale {
                height: WholePx(24),
                radius: WholePx(7),
                well_radius: WholePx(7),
                glyph: WholePx(14),
                pad_x: WholePx(10),
                switch_height: WholePx(22),
                slider_track: WholePx(4),
                slider_knob: WholePx(16),
                checkbox: WholePx(14),
                spinner: WholePx(16),
                progress_bar: WholePx(6),
                font: WholePx(11),
                weight: 500,
            },
            ControlSize::Regular => SizeScale {
                height: WholePx(28),
                radius: WholePx(8),
                well_radius: WholePx(8),
                glyph: WholePx(16),
                pad_x: WholePx(12),
                switch_height: WholePx(26),
                slider_track: WholePx(4),
                slider_knob: WholePx(22),
                checkbox: WholePx(16),
                spinner: WholePx(32),
                progress_bar: WholePx(6),
                font: WholePx(13),
                weight: 500,
            },
            ControlSize::Large => SizeScale {
                height: WholePx(32),
                radius: WholePx(16),
                well_radius: WholePx(16),
                glyph: WholePx(18),
                pad_x: WholePx(16),
                switch_height: WholePx(26),
                slider_track: WholePx(4),
                slider_knob: WholePx(22),
                checkbox: WholePx(16),
                spinner: WholePx(32),
                progress_bar: WholePx(6),
                font: WholePx(15),
                weight: 600,
            },
            ControlSize::ExtraLarge => SizeScale {
                height: WholePx(40),
                radius: WholePx(20),
                well_radius: WholePx(20),
                glyph: WholePx(20),
                pad_x: WholePx(20),
                switch_height: WholePx(26),
                slider_track: WholePx(4),
                slider_knob: WholePx(22),
                checkbox: WholePx(16),
                spinner: WholePx(32),
                progress_bar: WholePx(6),
                font: WholePx(17),
                weight: 600,
            },
        }
    }
}
