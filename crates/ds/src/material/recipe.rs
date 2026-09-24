//! Each material's tint, edge, shadow and radius per scheme: the `--m-*` tokens
//! (design/03-COLOR.md section 17.2, every value proposed, tuned in the gallery). Four
//! translucent alphas were raised in the smallest .02 steps that hold the card's ink at 4.5:1
//! over a pure black or white backdrop (`tests/legibility.rs`).
//!
//! The tint alpha over blur is a settings key (`appearance.material_tint_alpha`, default 80),
//! so the recipe takes it rather than hard-coding `.80`. Section 17.2 gives each material its
//! own alpha (the bar .70, the dock .55, …), and one key cannot name eight numbers, so the key
//! scales them: at the default 80 every tint is section 17.2's value, and at 100 each is 1.25
//! times it (capped at opaque). That reading is this wave's, and it is proposed.

use super::material::Material;
use crate::appearance::Scheme;
use crate::tokens::hex::{Alpha, Colour, Hex};
use crate::tokens::{Radius, Shadow};

/// The settings key's default, `appearance.material_tint_alpha = 80` (design/22-SETTINGS.md
/// section 3.1, proposed): the alpha at which every tint is section 17.2's own.
pub(crate) const DEFAULT_TINT_ALPHA: Alpha = Alpha(800);

/// The solid fallback's alpha when blur is unavailable (section 17.1, settled: "alpha at least
/// 0.94"). The key does not move it: it is the floor that keeps text legible on any backdrop.
pub(crate) const SOLID_ALPHA: Alpha = Alpha(940);

/// The five `--m-*` values one material paints in one scheme, as CSS values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialRecipe {
    /// `--m-tint`: the translucent tint over blur.
    pub tint: String,
    /// `--m-tint-solid`: the same tint at alpha .94 or above, when blur is unavailable.
    pub tint_solid: String,
    /// `--m-edge`: hairline and highlight.
    pub edge: String,
    /// `--m-shadow`: the drop shadow, or `none`.
    pub shadow: String,
    /// `--m-radius`.
    pub radius: String,
}

/// What `material` paints in `scheme`, with its tint at `tint_alpha` over blur.
pub fn recipe(material: Material, scheme: Scheme, tint_alpha: Alpha) -> MaterialRecipe {
    let (tint, tint_solid) = match tint(material, scheme) {
        Some((hex, base)) => {
            let over_blur = scaled(base, tint_alpha);
            (
                Colour::Alpha(hex, over_blur).css(),
                Colour::Alpha(hex, SOLID_ALPHA).css(),
            )
        }
        // The window paints the Space gradient with its layers and grain, never a tint.
        None => ("var(--f-grad)".to_owned(), "var(--f-grad)".to_owned()),
    };
    MaterialRecipe {
        tint,
        tint_solid,
        edge: edge(material, scheme),
        shadow: shadow(material, scheme).to_owned(),
        radius: radius(material).to_owned(),
    }
}

/// The tint's colour and its alpha at the default key, or `None` for the window.
///
/// Light tints are `--surface` (`rgba(248,249,246,…)`) and the popover's `--raise` white; dark
/// tints are `--paper` (`rgba(21,24,20,…)`) and the popover's dark `--raise`.
pub(crate) fn tint(material: Material, scheme: Scheme) -> Option<(Hex, Alpha)> {
    const SURFACE: Hex = Hex([248, 249, 246]);
    const WHITE: Hex = Hex([255, 255, 255]);
    const PAPER_DARK: Hex = Hex([21, 24, 20]);
    const RAISE_DARK: Hex = Hex([42, 47, 40]);
    let light = scheme == Scheme::Light;
    let (hex, alpha) = match material {
        Material::Window => return None,
        Material::Bar if light => (SURFACE, 700),
        // Raised from .58 so the dark ink holds 4.5:1 over a white backdrop (wave 1).
        Material::Bar => (PAPER_DARK, 660),
        Material::Dock if light => (SURFACE, 550),
        // Raised from .50, as the dark bar (wave 1).
        Material::Dock => (PAPER_DARK, 660),
        Material::Popover if light => (WHITE, 780),
        Material::Popover => (RAISE_DARK, 780),
        Material::Sheet if light => (SURFACE, 820),
        Material::Sheet => (PAPER_DARK, 780),
        Material::Toast if light => (SURFACE, 800),
        Material::Toast => (PAPER_DARK, 740),
        Material::Osd if light => (SURFACE, 720),
        Material::Osd => (PAPER_DARK, 660),
        // The widget adds grain over its tint; the grain is the component's, not the recipe's.
        // Raised from .50 (light, over black) and .45 (dark, over white) for 4.5:1 (wave 1).
        Material::Widget if light => (SURFACE, 540),
        Material::Widget => (PAPER_DARK, 650),
    };
    Some((hex, Alpha(alpha)))
}

/// `base` scaled by the key: `base x tint_alpha / 80%`, capped at opaque.
fn scaled(base: Alpha, tint_alpha: Alpha) -> Alpha {
    let value = u32::from(base.0) * u32::from(tint_alpha.0) / u32::from(DEFAULT_TINT_ALPHA.0);
    Alpha(u16::try_from(value.min(1000)).unwrap_or(1000))
}

/// The hairline (the `--f-line` values) and the highlight (the `--shadow-1` inset).
fn edge(material: Material, scheme: Scheme) -> String {
    let (hairline, highlight) = match scheme {
        Scheme::Light => ("rgba(0,0,0,.08)", "rgba(255,255,255,.6)"),
        Scheme::Dark => ("rgba(255,255,255,.09)", "rgba(255,255,255,.05)"),
    };
    match material {
        Material::Window => "none".to_owned(),
        Material::Bar => format!("inset 0 -.5px 0 {hairline}"),
        Material::Dock => {
            format!("inset 0 0 0 .5px rgba(255,255,255,.55),inset 0 1px 0 {highlight}")
        }
        Material::Popover
        | Material::Sheet
        | Material::Toast
        | Material::Osd
        | Material::Widget => format!("inset 0 0 0 .5px {hairline},inset 0 1px 0 {highlight}"),
    }
}

fn shadow(material: Material, scheme: Scheme) -> &'static str {
    match material {
        Material::Window | Material::Bar => "none",
        Material::Dock => "0 10px 30px -10px rgba(0,0,0,.35)",
        Material::Popover | Material::Toast | Material::Osd => Shadow::Pop.css(scheme),
        Material::Sheet => Shadow::Sheet.css(scheme),
        // "soft": the `--shadow-2` drop without its inset.
        Material::Widget => "0 6px 16px -6px rgba(26,30,26,.3)",
    }
}

fn radius(material: Material) -> &'static str {
    match material {
        // The card inside a window keeps its own 12/12/12/4.
        Material::Window | Material::Bar => "0",
        Material::Dock => "22px",
        Material::Popover => Radius::Panel.css(),
        Material::Sheet | Material::Osd => "18px",
        Material::Toast => "16px",
        Material::Widget => "20px",
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_TINT_ALPHA, recipe};
    use crate::appearance::Scheme;
    use crate::material::Material;
    use crate::tokens::hex::Alpha;

    #[test]
    fn the_default_key_paints_section_17_2() {
        #[rustfmt::skip]
        const CASES: &[(Material, Scheme, &str, &str, &str)] = &[
            (Material::Bar, Scheme::Light, "rgba(248,249,246,.7)", "rgba(248,249,246,.94)", "0"),
            (Material::Bar, Scheme::Dark, "rgba(21,24,20,.66)", "rgba(21,24,20,.94)", "0"),
            (Material::Dock, Scheme::Light, "rgba(248,249,246,.55)", "rgba(248,249,246,.94)", "22px"),
            (Material::Popover, Scheme::Light, "rgba(255,255,255,.78)", "rgba(255,255,255,.94)", "14px"),
            (Material::Popover, Scheme::Dark, "rgba(42,47,40,.78)", "rgba(42,47,40,.94)", "14px"),
            (Material::Sheet, Scheme::Light, "rgba(248,249,246,.82)", "rgba(248,249,246,.94)", "18px"),
            (Material::Toast, Scheme::Dark, "rgba(21,24,20,.74)", "rgba(21,24,20,.94)", "16px"),
            (Material::Osd, Scheme::Light, "rgba(248,249,246,.72)", "rgba(248,249,246,.94)", "18px"),
            (Material::Widget, Scheme::Dark, "rgba(21,24,20,.65)", "rgba(21,24,20,.94)", "20px"),
            (Material::Window, Scheme::Light, "var(--f-grad)", "var(--f-grad)", "0"),
        ];
        for &(material, scheme, tint, solid, radius) in CASES {
            let got = recipe(material, scheme, DEFAULT_TINT_ALPHA);
            assert_eq!(got.tint, tint, "{material:?} {scheme:?}");
            assert_eq!(got.tint_solid, solid, "{material:?} {scheme:?}");
            assert_eq!(got.radius, radius, "{material:?} {scheme:?}");
        }
    }

    #[test]
    fn the_key_scales_the_tint_and_never_the_solid_floor() {
        let low = recipe(Material::Sheet, Scheme::Light, Alpha(400));
        assert_eq!(low.tint, "rgba(248,249,246,.41)");
        assert_eq!(low.tint_solid, "rgba(248,249,246,.94)");
        let high = recipe(Material::Sheet, Scheme::Light, Alpha(1000));
        assert_eq!(high.tint, "rgba(248,249,246,1)");
        assert_eq!(high.tint_solid, "rgba(248,249,246,.94)");
    }

    #[test]
    fn edges_and_shadows_follow_the_scheme() {
        let light = recipe(Material::Popover, Scheme::Light, DEFAULT_TINT_ALPHA);
        let dark = recipe(Material::Popover, Scheme::Dark, DEFAULT_TINT_ALPHA);
        assert_eq!(
            light.edge,
            "inset 0 0 0 .5px rgba(0,0,0,.08),inset 0 1px 0 rgba(255,255,255,.6)"
        );
        assert_eq!(
            dark.edge,
            "inset 0 0 0 .5px rgba(255,255,255,.09),inset 0 1px 0 rgba(255,255,255,.05)"
        );
        assert_eq!(light.shadow, "0 18px 40px -16px rgba(0,0,0,.45)");
        let sheet = recipe(Material::Sheet, Scheme::Dark, DEFAULT_TINT_ALPHA);
        assert_eq!(sheet.shadow, "0 30px 60px -20px rgba(0,0,0,.7)");
        let bar = recipe(Material::Bar, Scheme::Light, DEFAULT_TINT_ALPHA);
        assert_eq!(bar.edge, "inset 0 -.5px 0 rgba(0,0,0,.08)");
        assert_eq!(bar.shadow, "none");
    }
}
