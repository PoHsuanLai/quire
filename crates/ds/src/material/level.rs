//! The level control's inks (sill FINDINGS Q74, the user's brief of 2026-09-25): what a level
//! capsule paints on a chrome material. The filled part is the material's bright ink (white on
//! the dark vibrancy, a near-white on the light one), the unfilled part a translucent well with
//! an inner shade, and the glyph inside is two-toned: the well's ink on the well, a dark ink on
//! the fill. Declared per scheme on `.ds`, so a `Surface` forcing a scheme gets its own.

use crate::appearance::Scheme;
use crate::css::emit::{attr_selector, declaration, rule};
use crate::tokens::{Alpha, Colour, Hex, VarName};

/// `--m-level-fill`: the filled part and the knob.
pub(crate) const FILL: VarName = VarName("--m-level-fill");
/// `--m-level-well`: the unfilled part.
pub(crate) const WELL: VarName = VarName("--m-level-well");
/// `--m-level-shade`: the well's inner shade, a `box-shadow`.
pub(crate) const SHADE: VarName = VarName("--m-level-shade");
/// `--m-level-glyph`: the glyph where it lies on the well.
pub(crate) const GLYPH: VarName = VarName("--m-level-glyph");
/// `--m-level-glyph-fill`: the glyph where the fill covers it.
pub(crate) const GLYPH_FILL: VarName = VarName("--m-level-glyph-fill");
/// `--m-level-knob`: the knob's hairline and drop, a `box-shadow`.
pub(crate) const KNOB: VarName = VarName("--m-level-knob");
/// `--m-level-tick`: the fill edge's quiet mark when the level crosses a step.
pub(crate) const TICK: VarName = VarName("--m-level-tick");

/// Every level variable, for the lint's registry.
#[cfg_attr(not(feature = "lint"), allow(dead_code))] // Read by the lint's registry.
pub(crate) const LEVEL_VARS: [VarName; 7] = [FILL, WELL, SHADE, GLYPH, GLYPH_FILL, KNOB, TICK];

const WHITE: Hex = Hex([255, 255, 255]);
const BLACK: Hex = Hex([0, 0, 0]);

/// One scheme's level inks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelInks {
    fill: Colour,
    well: Colour,
    /// The inner shade's black alpha (`inset 0 1px 2px`).
    shade: Alpha,
    glyph: Colour,
    glyph_fill: Colour,
    /// The knob's outline and drop black alphas.
    knob: (Alpha, Alpha),
    tick: Colour,
}

/// The inks for `scheme`. Light: a near-white fill over a black .10 well, so the fill reads as the
/// brighter of the two on the light tint; dark: a white fill over a white .14 well on the dark
/// tint. The glyph on the fill is a dark ink in both, which is what makes it read as knocked out.
pub(crate) fn inks(scheme: Scheme) -> LevelInks {
    match scheme {
        Scheme::Light => LevelInks {
            fill: Colour::Alpha(Hex([253, 253, 251]), Alpha(970)),
            well: Colour::Alpha(BLACK, Alpha(100)),
            shade: Alpha(140),
            glyph: Colour::Alpha(BLACK, Alpha(620)),
            glyph_fill: Colour::Alpha(BLACK, Alpha(720)),
            knob: (Alpha(120), Alpha(220)),
            tick: Colour::Alpha(BLACK, Alpha(160)),
        },
        Scheme::Dark => LevelInks {
            fill: Colour::Alpha(WHITE, Alpha(940)),
            well: Colour::Alpha(WHITE, Alpha(140)),
            shade: Alpha(350),
            glyph: Colour::Alpha(WHITE, Alpha(860)),
            glyph_fill: Colour::Alpha(BLACK, Alpha(700)),
            knob: (Alpha(350), Alpha(450)),
            tick: Colour::Alpha(BLACK, Alpha(200)),
        },
    }
}

/// The two blocks: the light inks on `.ds`, the dark on `.ds[data-theme=dark]`.
pub(crate) fn level_css() -> String {
    Scheme::ALL
        .into_iter()
        .map(|scheme| {
            let selector = match scheme {
                Scheme::Light => ".ds".to_owned(),
                Scheme::Dark => format!(".ds{}", attr_selector("data-theme", scheme.slug())),
            };
            rule(&selector, &declarations(inks(scheme)))
        })
        .collect()
}

fn declarations(inks: LevelInks) -> Vec<String> {
    let black = |alpha: Alpha| Colour::Alpha(BLACK, alpha).css();
    let (outline, drop) = inks.knob;
    vec![
        declaration(FILL, &inks.fill.css()),
        declaration(WELL, &inks.well.css()),
        declaration(SHADE, &format!("inset 0 1px 2px {}", black(inks.shade))),
        declaration(GLYPH, &inks.glyph.css()),
        declaration(GLYPH_FILL, &inks.glyph_fill.css()),
        declaration(
            KNOB,
            &format!(
                "0 0 0 var(--hairline) {},0 1px 3px {}",
                black(outline),
                black(drop)
            ),
        ),
        declaration(TICK, &inks.tick.css()),
    ]
}

#[cfg(test)]
mod tests {
    use super::{LEVEL_VARS, level_css};

    #[test]
    fn both_schemes_declare_every_level_ink() {
        let css = level_css();
        let blocks: Vec<&str> = css
            .split('}')
            .filter(|block| !block.trim().is_empty())
            .collect();
        assert_eq!(blocks.len(), 2, "{css}");
        for block in blocks {
            for var in LEVEL_VARS {
                assert!(
                    block.contains(&format!("{}:", var.as_str())),
                    "{var:?} in {block}"
                );
            }
        }
        assert!(
            css.contains("--m-level-fill:rgba(255,255,255,.94);"),
            "{css}"
        );
    }
}
