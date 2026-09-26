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
    /// `--widget-rim-top`: a dial bezel's lit top.
    RimTop,
    /// `--widget-rim-bottom`: a dial bezel's shaded foot.
    RimBottom,
    /// `--widget-sky-top`: a sky face at the zenith.
    SkyTop,
    /// `--widget-sky-bottom`: a sky face at the horizon.
    SkyBottom,
    /// `--widget-hand-shadow`: the shadow copy under a clock's hands.
    HandShadow,
    /// `--widget-boss`: a raised disc's lit top and seated foot.
    Boss,
}

impl WidgetPaint {
    /// Every widget paint, in stylesheet order.
    pub const ALL: [WidgetPaint; 12] = [
        WidgetPaint::Well,
        WidgetPaint::WellShade,
        WidgetPaint::Gloss,
        WidgetPaint::Spec,
        WidgetPaint::Sheen,
        WidgetPaint::Bevel,
        WidgetPaint::RimTop,
        WidgetPaint::RimBottom,
        WidgetPaint::SkyTop,
        WidgetPaint::SkyBottom,
        WidgetPaint::HandShadow,
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
            WidgetPaint::RimTop => "--widget-rim-top",
            WidgetPaint::RimBottom => "--widget-rim-bottom",
            WidgetPaint::SkyTop => "--widget-sky-top",
            WidgetPaint::SkyBottom => "--widget-sky-bottom",
            WidgetPaint::HandShadow => "--widget-hand-shadow",
            WidgetPaint::Boss => "--widget-boss",
        })
    }

    /// The value in `scheme`.
    pub fn css(self, scheme: Scheme) -> &'static str {
        let dark = scheme == Scheme::Dark;
        match self {
            WidgetPaint::Well if dark => "rgba(0,0,0,.28)",
            WidgetPaint::Well => "rgba(26,30,26,.08)",
            WidgetPaint::WellShade if dark => {
                "inset 0 1px 3px rgba(0,0,0,.55),inset 0 -1px 0 rgba(255,255,255,.06)"
            }
            WidgetPaint::WellShade => {
                "inset 0 1px 3px rgba(26,30,26,.22),inset 0 -1px 0 rgba(255,255,255,.6)"
            }
            WidgetPaint::Gloss if dark => "rgba(255,255,255,.26)",
            WidgetPaint::Gloss => "rgba(255,255,255,.38)",
            WidgetPaint::Spec if dark => "rgba(255,255,255,.35)",
            WidgetPaint::Spec => "rgba(255,255,255,.55)",
            WidgetPaint::Sheen if dark => "rgba(255,255,255,.05)",
            WidgetPaint::Sheen => "rgba(255,255,255,.22)",
            WidgetPaint::Bevel if dark => {
                "inset 0 var(--hair) 0 rgba(255,255,255,.14),inset 0 -1px 0 rgba(0,0,0,.22)"
            }
            WidgetPaint::Bevel => {
                "inset 0 var(--hair) 0 rgba(255,255,255,.55),inset 0 -1px 0 rgba(26,30,26,.06)"
            }
            WidgetPaint::RimTop if dark => "#4a5263",
            WidgetPaint::RimTop => "#ffffff",
            WidgetPaint::RimBottom if dark => "#10141c",
            WidgetPaint::RimBottom => "#c9cfc6",
            WidgetPaint::SkyTop if dark => "#0d1320",
            WidgetPaint::SkyTop => "#d3e0ea",
            WidgetPaint::SkyBottom if dark => "#2a3446",
            WidgetPaint::SkyBottom => "#fbfbf8",
            WidgetPaint::HandShadow if dark => "rgba(0,0,0,.5)",
            WidgetPaint::HandShadow => "rgba(26,30,26,.22)",
            WidgetPaint::Boss if dark => {
                "inset 0 var(--hair) 0 rgba(255,255,255,.12),0 1px 2px rgba(0,0,0,.55),0 2px 6px -2px rgba(0,0,0,.5)"
            }
            WidgetPaint::Boss => {
                "inset 0 var(--hair) 0 rgba(255,255,255,.9),0 1px 2px rgba(26,30,26,.18),0 2px 6px -2px rgba(26,30,26,.22)"
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
