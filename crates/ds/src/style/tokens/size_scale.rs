//! The size ladder's geometry (design/29-SIZING.md section 6): one [`SizeScale`] per
//! [`ControlSize`](super::ControlSize) holds the few numbers the reference gives (height,
//! rounded-rectangle radius, glyph, label, inset), and every other size is derived here by the
//! rules, so the stylesheet cannot disagree with them:
//!
//! - R1 heights are whole pixels and come from the ladder, never from padding;
//! - R2 a capsule's radius is half its height;
//! - R3 a knob fills its track less [`KNOB_INSET`] on each side;
//! - R4 a switch is `round_even(1.73 h)` wide;
//! - R5 a rounded rectangle's radius comes from its size;
//! - R6 an element inset `p` in a frame of radius `R` has radius `R - p`;
//! - R7 spacing is on a 4 px grid with a 2 px half step (`size_rules_tests`);
//! - R8 the glyph follows the size.

/// A length in whole logical pixels (R1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WholePx(pub u16);

impl WholePx {
    /// As CSS: `22px`.
    pub fn css(self) -> String {
        format!("{}px", self.0)
    }
}

/// A radius, which may fall on a half pixel (a 15 tall capsule's is 7.5), in half pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HalfPx(pub u16);

impl HalfPx {
    /// As CSS: `11px`, `7.5px`.
    pub fn css(self) -> String {
        match self.0 % 2 {
            0 => format!("{}px", self.0 / 2),
            _ => format!("{}.5px", self.0 / 2),
        }
    }
}

/// The inset a knob or a selected segment keeps from its track's edge (R3; settled 1,
/// design/29 section 13 decision 3).
pub const KNOB_INSET: WholePx = WholePx(1);

/// The switch's width over its height, in hundredths (R4): the reference's 26/15 and 38/22.
pub const SWITCH_RATIO: u16 = 173;

/// The numbers one control size is built from; everything else is a method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SizeScale {
    /// The control's height: a button, a field, a segmented well, a level capsule.
    pub height: WholePx,
    /// A rounded rectangle's radius at this height (R5).
    pub radius: WholePx,
    /// A segmented control's well radius; its segment is concentric inside (R6).
    pub well_radius: WholePx,
    /// A glyph's box inside the control (R8).
    pub glyph: WholePx,
    /// The horizontal inset of a label from the control's edge.
    pub pad_x: WholePx,
    /// The switch's track height (the reference's mini switch is 15 at Small).
    pub switch_height: WholePx,
    /// The slider's track thickness.
    pub slider_track: WholePx,
    /// The label's size.
    pub font: WholePx,
    /// The label's weight.
    pub weight: u16,
}

impl SizeScale {
    /// A capsule of this height's radius (R2).
    pub fn capsule_radius(self) -> HalfPx {
        HalfPx(self.height.0)
    }

    /// A knob riding a track of this height (R3): the level capsule's knob.
    pub fn knob(self) -> WholePx {
        inside(self.height)
    }

    /// The slider's round knob, as tall as the level's (R3).
    pub fn slider_knob(self) -> WholePx {
        inside(self.height)
    }

    /// The switch's track width (R4).
    pub fn switch_width(self) -> WholePx {
        round_even(self.switch_height.0 * SWITCH_RATIO, 100)
    }

    /// The switch's knob (R3).
    pub fn switch_knob(self) -> WholePx {
        inside(self.switch_height)
    }

    /// How far the switch's knob travels from off to on: the track less its two insets and
    /// the knob.
    pub fn switch_travel(self) -> WholePx {
        WholePx(self.switch_width().0 - 2 * KNOB_INSET.0 - self.switch_knob().0)
    }

    /// The switch track's radius (R2).
    pub fn switch_radius(self) -> HalfPx {
        HalfPx(self.switch_height.0)
    }

    /// A selected segment's height inside the well (R3).
    pub fn segment(self) -> WholePx {
        inside(self.height)
    }

    /// A selected segment's radius, concentric in the well (R6).
    pub fn segment_radius(self) -> WholePx {
        WholePx(self.well_radius.0 - KNOB_INSET.0)
    }

    /// The radius of anything inset `inset` inside a rounded rectangle of this size (R6).
    pub fn inner_radius(self, inset: WholePx) -> WholePx {
        WholePx(self.radius.0.saturating_sub(inset.0))
    }
}

/// A track's knob: the height less an inset each side.
fn inside(height: WholePx) -> WholePx {
    WholePx(height.0 - 2 * KNOB_INSET.0)
}

/// `numerator / denominator` rounded to the nearest even whole pixel.
fn round_even(numerator: u16, denominator: u16) -> WholePx {
    let halves = (u32::from(numerator) + u32::from(denominator)) / (2 * u32::from(denominator));
    WholePx((halves * 2) as u16)
}
