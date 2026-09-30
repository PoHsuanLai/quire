//! LevelIndicator: `NSLevelIndicator` (design/30 section 2.9): a read-only level, continuous or in
//! sixteen steps, with an optional glyph that follows it and warning and critical bands: the
//! volume and brightness bar of the OSD, a battery or storage row.
//! Markup: `div.ds-level-indicator[role=meter][data-style][data-band][data-size]`, the level as
//! `--f`; parts `track`, `fill`, `icon`.

use crate::components::content::level_glyph::vocab::LevelSource;
use crate::components::controls::level_draw::{Drawn, GlyphAt, INDICATOR, Shape, body, lead};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::Fraction;
use ds_core::word::Word;
use ds_style::tokens::control_size::ControlSize;

/// How the level is drawn (`NSLevelIndicator.Style`), `data-style`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum LevelStyle {
    /// A capsule filled to the level, the glyph inside at its left end.
    #[default]
    Continuous,
    /// Sixteen rounded squares that fill in one by one, the glyph before them.
    Discrete,
}

/// Where a level starts to warn (`NSLevelIndicator.warningValue` and `criticalValue`): a level at
/// or under `warning` reads as a warning, at or under `critical` as critical. Both of nothing
/// (the default) never warn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Bands {
    /// At or under this the fill is the warning colour.
    pub warning: Fraction,
    /// At or under this the fill is the critical colour.
    pub critical: Fraction,
}

/// Which band a level is in, `data-band`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub(crate) enum Band {
    /// Above every band: the fill's ordinary colour.
    Normal,
    /// At or under the warning level.
    Warning,
    /// At or under the critical level.
    Critical,
}

impl Bands {
    /// The band `value` is in. A band of nothing is off: a level of zero is not critical unless
    /// a critical level above zero was asked for.
    pub(crate) fn of(self, value: Fraction) -> Band {
        let value = value.clamped();
        let at_or_under = |limit: Fraction| limit.0 > 0 && value <= limit;
        if at_or_under(self.critical) {
            Band::Critical
        } else if at_or_under(self.warning) {
            Band::Warning
        } else {
            Band::Normal
        }
    }
}

/// A read-only level. `glyph` is a `LevelGlyph` that follows `value`, or a `VolumeState` (both
/// convert). The fill follows a new value linearly over `--t-move` (a level set from outside
/// slides; nothing sweeps in on first show).
#[component]
pub fn LevelIndicator(
    label: String,
    value: Fraction,
    #[props(default)] style: LevelStyle,
    #[props(default)] glyph: Option<LevelSource>,
    #[props(default)] bands: Bands,
    #[props(default)] size: ControlSize,
    #[props(default)] common: Common,
) -> Element {
    let value = value.clamped();
    let shape = match style {
        LevelStyle::Continuous => Shape::Capsule,
        LevelStyle::Discrete => Shape::Segments,
    };
    let drawn = Drawn {
        shape,
        parts: INDICATOR,
        value,
        glyph: glyph.map(|source| {
            let (glyph, level) = source.drawn(value);
            GlyphAt { glyph, level }
        }),
    };
    let leading = lead(&drawn);
    let class = common.class("ds-level-indicator");
    let data = common.data_attributes();
    rsx! {
        div {
            id: common.id.clone(),
            class,
            role: "meter",
            "data-style": style.slug(),
            "data-band": bands.of(value).slug(),
            "data-size": size.slug(),
            "aria-label": common.aria_label.clone().unwrap_or_else(|| label.clone()),
            "aria-valuemin": "0",
            "aria-valuemax": "100",
            "aria-valuenow": "{value.whole_percent()}",
            style: "--f:{value.css()}",
            onmounted: move |event| common.mounted(event),
            ..data,
            {leading}
            div { class: "ds-level-indicator-rail", {body(drawn)} }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Band, Bands};
    use ds_core::vocab::Fraction;

    #[test]
    fn a_level_falls_into_the_lowest_band_it_reaches() {
        let bands = Bands {
            warning: Fraction(200),
            critical: Fraction(100),
        };
        // (value, band)
        const CASES: &[(u16, Band)] = &[
            (1000, Band::Normal),
            (201, Band::Normal),
            (200, Band::Warning),
            (101, Band::Warning),
            (100, Band::Critical),
            (0, Band::Critical),
        ];
        for &(value, want) in CASES {
            assert_eq!(bands.of(Fraction(value)), want, "{value}");
        }
    }

    #[test]
    fn no_bands_never_warn() {
        for value in [0, 500, 1000] {
            assert_eq!(
                Bands::default().of(Fraction(value)),
                Band::Normal,
                "{value}"
            );
        }
    }
}
