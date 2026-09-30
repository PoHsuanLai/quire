//! The widgets' paints (design/23-WIDGETS.md section 2, "Flat, bright, measured"): the battery
//! ring's fill, its low tone and its track, and the world clock's faces, their ink and the
//! seconds hand. Each has a light and a dark value, measured from the reference widgets
//! (design/23 section 1.1): a bright green ring on a track that is the plate darkened, a red
//! ring when low, white day dials and dark night dials with an orange seconds hand.

use crate::style::tokens::token::Token;
use ds_core::word::Word;

/// One widget paint token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "", kind = fixed)]
pub enum WidgetPaint {
    /// `--battery-fill`: the ring's arc at a healthy level, and while charging.
    #[token(light = "#28cd41", dark = "#32d74b")]
    BatteryFill,
    /// `--battery-low`: the arc at a fifth or less (low and critical alike).
    #[token(light = "#ff3b30", dark = "#ff453a")]
    BatteryLow,
    /// `--battery-track`: the ring's full circle under the arc, the plate darkened.
    #[token(light = "rgba(0,0,0,.12)", dark = "rgba(255,255,255,.14)")]
    BatteryTrack,
    /// `--clock-face-day`: a day dial.
    #[token(value = "#ffffff")]
    ClockFaceDay,
    /// `--clock-face-night`: a night dial.
    #[token(value = "#343436")]
    ClockFaceNight,
    /// `--clock-ink-day`: numerals, ticks and hands on a day dial.
    #[token(value = "#1c1c1e")]
    ClockInkDay,
    /// `--clock-ink-night`: numerals, ticks and hands on a night dial.
    #[token(value = "#ffffff")]
    ClockInkNight,
    /// `--clock-seconds`: the seconds hand and its ring.
    #[token(light = "#ff9500", dark = "#ff9f0a")]
    ClockSeconds,
}

#[cfg(test)]
mod tests {
    use super::WidgetPaint;
    use crate::style::appearance::theme::Scheme;
    use crate::style::tokens::token::{Token, TokenScope};
    use ds_core::word::Word;
    use std::collections::HashSet;

    #[test]
    fn every_paint_has_its_own_name_and_a_value_in_both_schemes() {
        let names: HashSet<&str> = WidgetPaint::ALL.iter().map(|p| p.var().0).collect();
        assert_eq!(names.len(), WidgetPaint::ALL.len());
        for paint in WidgetPaint::ALL.iter().copied() {
            for scheme in Scheme::ALL.iter().copied() {
                let value = paint.css_value(TokenScope::BASE.in_scheme(scheme));
                assert!(!value.as_str().is_empty(), "{paint:?} {scheme:?}");
            }
        }
    }
}
