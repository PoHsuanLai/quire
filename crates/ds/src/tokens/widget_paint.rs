//! The widget language's paints (design/23-WIDGETS.md section 3.3): the well a gauge is pressed
//! into, the gloss on a level's fill, the lit plate's bevel and sheen, a dial's bezel and sky,
//! and the hands' shadow. Each has a light and a dark value; a clock dial is drawn in its
//! phase's scheme, so a dial token's dark value is its night value on any desktop.

use super::name::VarName;
use crate::appearance::Scheme;

/// One widget paint token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WidgetPaint {
    /// `--widget-well`: the ground of a well pressed into the plate.
    Well,
    /// `--widget-well-shade`: a well's shade under its top edge and light along its foot.
    WellShade,
    /// `--widget-gloss`: the lighter band on a level's fill.
    Gloss,
    /// `--widget-spec`: the specular line along a fill's top, a lit bolt's halo.
    Spec,
    /// `--widget-sheen`: the lit plate's sheen down its top.
    Sheen,
    /// `--widget-bevel`: the lit plate's top highlight and foot.
    Bevel,
    /// `--widget-lift`: the lit plate's warmer, brighter wash.
    Lift,
    /// `--widget-foot`: the lit plate's shade band along its foot.
    Foot,
    /// `--widget-rim-top`: a dial bezel's lit top.
    RimTop,
    /// `--widget-rim-bottom`: a dial bezel's shaded foot.
    RimBottom,
    /// `--widget-sky-top`: a sky face at the zenith.
    SkyTop,
    /// `--widget-sky-bottom`: a sky face at the horizon.
    SkyBottom,
    /// `--widget-glass`: the highlight across the top of a dial's face.
    Glass,
    /// `--widget-hand-shadow`: the shadow copy under a clock's hands.
    HandShadow,
    /// `--widget-liquid-shade`: the darker foot of a level's liquid.
    LiquidShade,
    /// `--widget-boss`: a raised disc's lit top and seated foot.
    Boss,
}

impl WidgetPaint {
    /// Every widget paint, in stylesheet order.
    pub const ALL: [WidgetPaint; 16] = [
        WidgetPaint::Well,
        WidgetPaint::WellShade,
        WidgetPaint::Gloss,
        WidgetPaint::Spec,
        WidgetPaint::Sheen,
        WidgetPaint::Bevel,
        WidgetPaint::Lift,
        WidgetPaint::Foot,
        WidgetPaint::RimTop,
        WidgetPaint::RimBottom,
        WidgetPaint::SkyTop,
        WidgetPaint::SkyBottom,
        WidgetPaint::Glass,
        WidgetPaint::HandShadow,
        WidgetPaint::LiquidShade,
        WidgetPaint::Boss,
    ];

    /// The custom property.
    pub fn var(self) -> VarName {
        VarName(match self {
            WidgetPaint::Well => "--widget-well",
            WidgetPaint::WellShade => "--widget-well-shade",
            WidgetPaint::Gloss => "--widget-gloss",
            WidgetPaint::Spec => "--widget-spec",
            WidgetPaint::Sheen => "--widget-sheen",
            WidgetPaint::Bevel => "--widget-bevel",
            WidgetPaint::Lift => "--widget-lift",
            WidgetPaint::Foot => "--widget-foot",
            WidgetPaint::RimTop => "--widget-rim-top",
            WidgetPaint::RimBottom => "--widget-rim-bottom",
            WidgetPaint::SkyTop => "--widget-sky-top",
            WidgetPaint::SkyBottom => "--widget-sky-bottom",
            WidgetPaint::Glass => "--widget-glass",
            WidgetPaint::HandShadow => "--widget-hand-shadow",
            WidgetPaint::LiquidShade => "--widget-liquid-shade",
            WidgetPaint::Boss => "--widget-boss",
        })
    }

    /// The value in `scheme`.
    pub fn css(self, scheme: Scheme) -> &'static str {
        let dark = scheme == Scheme::Dark;
        match self {
            WidgetPaint::Well if dark => "rgba(0,0,0,.34)",
            WidgetPaint::Well => "rgba(26,30,26,.10)",
            WidgetPaint::WellShade if dark => {
                "inset 0 2px 5px rgba(0,0,0,.75),inset 0 1px 1px rgba(0,0,0,.5),inset 0 -1px 0 rgba(255,255,255,.10),0 1px 0 rgba(255,255,255,.08)"
            }
            WidgetPaint::WellShade => {
                "inset 0 2px 4px rgba(26,30,26,.30),inset 0 1px 1px rgba(26,30,26,.20),inset 0 -1px 0 rgba(255,255,255,.85),0 1px 0 rgba(255,255,255,.7)"
            }
            WidgetPaint::Gloss if dark => "rgba(255,255,255,.38)",
            WidgetPaint::Gloss => "rgba(255,255,255,.60)",
            WidgetPaint::Spec if dark => "rgba(255,255,255,.45)",
            WidgetPaint::Spec => "rgba(255,255,255,.85)",
            WidgetPaint::Sheen if dark => "rgba(255,255,255,.09)",
            WidgetPaint::Sheen => "rgba(255,255,255,.55)",
            WidgetPaint::Bevel if dark => {
                "inset 0 var(--hair) 0 rgba(255,255,255,.22),inset 0 0 0 var(--hair) rgba(255,255,255,.04),inset 0 -2px 3px -1px rgba(0,0,0,.45),0 14px 30px -12px rgba(0,0,0,.7)"
            }
            WidgetPaint::Bevel => {
                "inset 0 var(--hair) 0 rgba(255,255,255,.95),inset 0 0 0 var(--hair) rgba(255,255,255,.22),inset 0 -2px 3px -1px rgba(26,30,26,.14),0 1px 0 rgba(26,30,26,.10),0 12px 28px -12px rgba(26,30,26,.40)"
            }
            WidgetPaint::Lift if dark => "rgba(255,250,240,.035)",
            WidgetPaint::Lift => "rgba(255,251,242,.30)",
            WidgetPaint::Foot if dark => "rgba(0,0,0,.25)",
            WidgetPaint::Foot => "rgba(26,30,26,.12)",
            WidgetPaint::RimTop if dark => "#5a6376",
            WidgetPaint::RimTop => "#ffffff",
            WidgetPaint::RimBottom if dark => "#0a0d13",
            WidgetPaint::RimBottom => "#a9b2a6",
            WidgetPaint::SkyTop if dark => "#060a13",
            WidgetPaint::SkyTop => "#a9c8e2",
            WidgetPaint::SkyBottom if dark => "#2c3a54",
            WidgetPaint::SkyBottom => "#f7fafc",
            WidgetPaint::Glass if dark => "rgba(255,255,255,.14)",
            WidgetPaint::Glass => "rgba(255,255,255,.55)",
            WidgetPaint::HandShadow if dark => "rgba(0,0,0,.65)",
            WidgetPaint::HandShadow => "rgba(26,30,26,.30)",
            WidgetPaint::LiquidShade if dark => "rgba(0,0,0,.30)",
            WidgetPaint::LiquidShade => "rgba(0,0,0,.18)",
            WidgetPaint::Boss if dark => {
                "inset 0 var(--hair) 0 rgba(255,255,255,.16),inset 0 -1px 1px rgba(0,0,0,.4),0 1px 2px rgba(0,0,0,.7),0 4px 10px -3px rgba(0,0,0,.7)"
            }
            WidgetPaint::Boss => {
                "inset 0 var(--hair) 0 rgba(255,255,255,1),inset 0 -1px 1px rgba(26,30,26,.08),0 1px 2px rgba(26,30,26,.25),0 4px 10px -3px rgba(26,30,26,.35)"
            }
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
