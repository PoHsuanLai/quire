//! The widgets' paints (design/23-WIDGETS.md section 2, "Flat, bright, measured"): the battery
//! ring's fill, its low tone and its track, and the world clock's faces, their ink and the
//! seconds hand. Each has a light and a dark value, measured from the reference widgets
//! (design/23 section 1.1): a bright green ring on a track that is the plate darkened, a red
//! ring when low, white day dials and dark night dials with an orange seconds hand.

use super::name::VarName;
use crate::appearance::Scheme;

/// One widget paint token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WidgetPaint {
    /// `--battery-fill`: the ring's arc at a healthy level, and while charging.
    BatteryFill,
    /// `--battery-low`: the arc at a fifth or less (low and critical alike).
    BatteryLow,
    /// `--battery-track`: the ring's full circle under the arc, the plate darkened.
    BatteryTrack,
    /// `--clock-face-day`: a day dial.
    ClockFaceDay,
    /// `--clock-face-night`: a night dial.
    ClockFaceNight,
    /// `--clock-ink-day`: numerals, ticks and hands on a day dial.
    ClockInkDay,
    /// `--clock-ink-night`: numerals, ticks and hands on a night dial.
    ClockInkNight,
    /// `--clock-seconds`: the seconds hand and its ring.
    ClockSeconds,
}

impl WidgetPaint {
    /// Every widget paint, in stylesheet order.
    pub const ALL: [WidgetPaint; 8] = [
        WidgetPaint::BatteryFill,
        WidgetPaint::BatteryLow,
        WidgetPaint::BatteryTrack,
        WidgetPaint::ClockFaceDay,
        WidgetPaint::ClockFaceNight,
        WidgetPaint::ClockInkDay,
        WidgetPaint::ClockInkNight,
        WidgetPaint::ClockSeconds,
    ];

    /// The custom property.
    pub fn var(self) -> VarName {
        VarName(match self {
            WidgetPaint::BatteryFill => "--battery-fill",
            WidgetPaint::BatteryLow => "--battery-low",
            WidgetPaint::BatteryTrack => "--battery-track",
            WidgetPaint::ClockFaceDay => "--clock-face-day",
            WidgetPaint::ClockFaceNight => "--clock-face-night",
            WidgetPaint::ClockInkDay => "--clock-ink-day",
            WidgetPaint::ClockInkNight => "--clock-ink-night",
            WidgetPaint::ClockSeconds => "--clock-seconds",
        })
    }

    /// The value in `scheme`.
    pub fn css(self, scheme: Scheme) -> &'static str {
        let dark = scheme == Scheme::Dark;
        match self {
            WidgetPaint::BatteryFill if dark => "#32d74b",
            WidgetPaint::BatteryFill => "#28cd41",
            WidgetPaint::BatteryLow if dark => "#ff453a",
            WidgetPaint::BatteryLow => "#ff3b30",
            WidgetPaint::BatteryTrack if dark => "rgba(255,255,255,.14)",
            WidgetPaint::BatteryTrack => "rgba(0,0,0,.12)",
            WidgetPaint::ClockFaceDay => "#ffffff",
            WidgetPaint::ClockFaceNight => "#343436",
            WidgetPaint::ClockInkDay => "#1c1c1e",
            WidgetPaint::ClockInkNight => "#ffffff",
            WidgetPaint::ClockSeconds if dark => "#ff9f0a",
            WidgetPaint::ClockSeconds => "#ff9500",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WidgetPaint;
    use crate::appearance::Scheme;
    use std::collections::HashSet;

    #[test]
    fn every_paint_has_its_own_name_and_a_value_in_both_schemes() {
        let names: HashSet<&str> = WidgetPaint::ALL.iter().map(|p| p.var().0).collect();
        assert_eq!(names.len(), WidgetPaint::ALL.len());
        for paint in WidgetPaint::ALL {
            for scheme in Scheme::ALL {
                assert!(!paint.css(scheme).is_empty(), "{paint:?} {scheme:?}");
            }
        }
    }
}
