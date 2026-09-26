//! Each material's tint, edge, shadow and radius per scheme: the `--m-*` tokens
//! (design/03-COLOR.md section 17.2, every value proposed, tuned in the gallery). Four
//! translucent alphas were raised in the smallest .02 steps that hold the card's ink at 4.5:1
//! over a pure black or white backdrop (`tests/legibility.rs`), and, since 2026-09-24, six more
//! (settled: the dark bar, both dock schemes, the dark OSD and both widget schemes) were raised
//! the same way to clear 4.5:1 over blur too, closing the shortfall FINDINGS "Bar gaps" and
//! design/21-SPACES.md section 3 recorded (`tests/legibility.rs`,
//! `the_tinted_chrome_holds_its_ink_over_blur`).
//!
//! The tint alpha over blur is a settings key (`appearance.material_tint_alpha`, default 80),
//! so the recipe takes it rather than hard-coding `.80`. Section 17.2 gives each material its
//! own alpha (the bar .70, the dock .59, …), and one key cannot name eight numbers, so the key
//! scales them: at the default 80 every tint is section 17.2's value, and at 100 each is 1.25
//! times it (capped at opaque). That reading is this wave's, and it is proposed.

use super::layer::{Layer, LayerAlpha, Side, joined};
use super::material::Material;
use super::stack::{
    HAIRLINE_DARK, HAIRLINE_LIGHT, HIGHLIGHT_DARK, HIGHLIGHT_LIGHT, SHADOW_STRENGTH,
};
use super::vibrancy::boosted;
use crate::appearance::Scheme;
use crate::tokens::Radius;
use crate::tokens::hex::{Alpha, Colour, Hex};

/// The settings key's default, `appearance.material_tint_alpha = 80` (design/22-SETTINGS.md
/// section 3.1, proposed): the alpha at which every tint is section 17.2's own.
pub(crate) const DEFAULT_TINT_ALPHA: Alpha = Alpha(800);

/// The solid fallback's alpha when blur is unavailable (section 17.1, settled: "alpha at least
/// 0.94"). The key does not move it: it is the floor that keeps text legible on any backdrop.
pub(crate) const SOLID_ALPHA: Alpha = Alpha(940);

/// The `--m-*` values one material paints in one scheme, as CSS values at the settings keys'
/// defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialRecipe {
    /// `--m-tint`: the translucent tint over blur, the vibrancy boost baked in.
    pub tint: String,
    /// `--m-tint-solid`: the same tint at alpha .94 or above, when blur is unavailable.
    pub tint_solid: String,
    /// `--m-edge`: the inner hairline.
    pub edge: String,
    /// `--m-shadow`: the ambient drop shadow (`--m-shadow-ambient`), or `none`.
    pub shadow: String,
    /// `--m-radius`.
    pub radius: String,
    /// `--m-highlight`: the 1 px inner top highlight, or `none` (stack v2).
    pub highlight: String,
    /// `--m-hairline`: the 0.5 px dark outer hairline, or `none` (stack v2).
    pub hairline: String,
    /// `--m-shadow-contact`: the tight contact shadow under the card, or `none` (stack v2).
    pub shadow_contact: String,
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
    let stack = layers(material, scheme);
    let one = |layer: Option<Layer>| joined(&Vec::from_iter(layer), Layer::css);
    MaterialRecipe {
        tint,
        tint_solid,
        edge: joined(&stack.edge, Layer::css),
        shadow: one(stack.ambient),
        radius: radius(material).to_owned(),
        highlight: one(stack.highlight),
        hairline: one(stack.hairline),
        shadow_contact: one(stack.contact),
    }
}

/// The tint's colour with the vibrancy boost baked in, and its alpha at the default key, or
/// `None` for the window.
pub(crate) fn tint(material: Material, scheme: Scheme) -> Option<(Hex, Alpha)> {
    flat_tint(material, scheme).map(|(hex, alpha)| (boosted(hex, scheme), alpha))
}

/// Section 17.2's own tint colour, before the vibrancy boost, and its alpha at the default key,
/// or `None` for the window.
///
/// Light tints are `--surface` (`rgba(248,249,246,…)`) and the popover's `--raise` white; dark
/// tints are `--paper` (`rgba(21,24,20,…)`) and the popover's dark `--raise`.
pub(crate) fn flat_tint(material: Material, scheme: Scheme) -> Option<(Hex, Alpha)> {
    const SURFACE: Hex = Hex([248, 249, 246]);
    const WHITE: Hex = Hex([255, 255, 255]);
    const PAPER_DARK: Hex = Hex([21, 24, 20]);
    const RAISE_DARK: Hex = Hex([42, 47, 40]);
    // The light widget's neutral grey (design/23 section 1.1): boosted, the reference's plate
    // `#e4e4e4`.
    const WIDGET_GREY: Hex = Hex([224, 224, 224]);
    let light = scheme == Scheme::Light;
    let (hex, alpha) = match material {
        Material::Window => return None,
        Material::Bar if light => (SURFACE, 700),
        // .58 -> .66 (wave 1, over an opaque ground) -> .68 (settled 2026-09-24, over blur:
        // held the dark ink at 4.36:1 over white at .66, short of 4.5).
        Material::Bar => (PAPER_DARK, 680),
        // .55 -> .59 (settled 2026-09-24, over blur: held 4.11:1 over black at .55).
        Material::Dock if light => (SURFACE, 590),
        // .50 -> .66 (wave 1, as the dark bar) -> .68 (settled 2026-09-24, over blur).
        Material::Dock => (PAPER_DARK, 680),
        Material::Popover if light => (WHITE, 780),
        Material::Popover => (RAISE_DARK, 780),
        Material::Sheet if light => (SURFACE, 820),
        Material::Sheet => (PAPER_DARK, 780),
        Material::Toast if light => (SURFACE, 800),
        Material::Toast => (PAPER_DARK, 740),
        Material::Osd if light => (SURFACE, 720),
        // .66 -> .68 (settled 2026-09-24, over blur, as the dark bar and dock).
        Material::Osd => (PAPER_DARK, 680),
        // The widget adds grain over its tint; the grain is the component's, not the recipe's.
        // .50 -> .54 (wave 1, light, over black) -> .60 (settled 2026-09-24, over blur: held
        // 3.91:1 over black at .54, the worst of the six). The reference's card measures .48
        // of a near-white over a Gaussian blur of sigma 22 (design/23-WIDGETS.md section 1.1,
        // M26-M30), under this floor; at the floor, the tint that best fits the measured
        // composite is a neutral grey that lands on the reference's opaque plate, 228 once
        // boosted (the vibrancy pass, 2026-09-26, proposed).
        Material::Widget if light => (WIDGET_GREY, 600),
        // .45 -> .65 (wave 1, dark, over white) -> .67 (settled 2026-09-24, over blur).
        Material::Widget => (PAPER_DARK, 670),
    };
    Some((hex, Alpha(alpha)))
}

/// `base` scaled by the key: `base x tint_alpha / 80%`, capped at opaque.
fn scaled(base: Alpha, tint_alpha: Alpha) -> Alpha {
    let value = u32::from(base.0) * u32::from(tint_alpha.0) / u32::from(DEFAULT_TINT_ALPHA.0);
    Alpha(u16::try_from(value.min(1000)).unwrap_or(1000))
}

/// A material's layers, outside in: the outer hairline, the two shadows, the inner edge and
/// the highlight (stack v2).
pub(crate) struct Layers {
    pub hairline: Option<Layer>,
    pub contact: Option<Layer>,
    pub ambient: Option<Layer>,
    pub edge: Vec<Layer>,
    pub highlight: Option<Layer>,
}

const BLACK: Hex = Hex([0, 0, 0]);
const WHITE: Hex = Hex([255, 255, 255]);

/// The stack of `material` in `scheme`. The window has none; the bar has its inner bottom edge
/// and, below it, the outer bottom hairline; every card has all five.
pub(crate) fn layers(material: Material, scheme: Scheme) -> Layers {
    let light = scheme == Scheme::Light;
    let (highlight_input, highlight_alpha) = if light {
        (HIGHLIGHT_LIGHT, 300)
    } else {
        (HIGHLIGHT_DARK, 120)
    };
    let (hairline_input, hairline_alpha) = if light {
        (HAIRLINE_LIGHT, 140)
    } else {
        (HAIRLINE_DARK, 600)
    };
    // The inner hairline: the `--f-line` values (section 4).
    let (inner, inner_alpha) = if light { (BLACK, 80) } else { (WHITE, 90) };
    let outer = |geometry| Layer {
        side: Side::Outer,
        geometry,
        colour: BLACK,
        alpha: LayerAlpha::Input(hairline_input, Alpha(hairline_alpha)),
    };
    let shadow = |geometry, colour, alpha| Layer {
        side: Side::Outer,
        geometry,
        colour,
        alpha: LayerAlpha::Scaled(Alpha(alpha), SHADOW_STRENGTH),
    };
    let inset = |geometry, colour, alpha| Layer {
        side: Side::Inner,
        geometry,
        colour,
        alpha: LayerAlpha::Fixed(Alpha(alpha)),
    };
    let card = |ambient: Layer, edge: Layer| Layers {
        hairline: Some(outer("0 0 0 var(--hairline)")),
        contact: Some(shadow("0 1px 2px", BLACK, if light { 100 } else { 300 })),
        ambient: Some(ambient),
        edge: vec![edge],
        highlight: Some(Layer {
            side: Side::Inner,
            geometry: "0 var(--hair) 0",
            colour: WHITE,
            alpha: LayerAlpha::Input(highlight_input, Alpha(highlight_alpha)),
        }),
    };
    let hairline_edge = inset("0 0 0 var(--hairline)", inner, inner_alpha);
    let pop = |light_alpha, dark_alpha| {
        shadow(
            "0 12px 40px -12px",
            BLACK,
            if light { light_alpha } else { dark_alpha },
        )
    };
    match material {
        Material::Window => Layers {
            hairline: None,
            contact: None,
            ambient: None,
            edge: Vec::new(),
            highlight: None,
        },
        Material::Bar => Layers {
            hairline: Some(outer("0 var(--hairline) 0")),
            contact: None,
            ambient: None,
            edge: vec![inset("0 calc(-1 * var(--hairline)) 0", inner, inner_alpha)],
            highlight: None,
        },
        Material::Dock => card(
            shadow("0 10px 30px -10px", BLACK, if light { 350 } else { 500 }),
            inset("0 0 0 var(--hairline)", WHITE, 550),
        ),
        Material::Popover | Material::Toast | Material::Osd => card(pop(280, 550), hairline_edge),
        Material::Sheet => card(
            if light {
                shadow("0 24px 60px -18px", BLACK, 400)
            } else {
                shadow("0 30px 70px -20px", BLACK, 650)
            },
            hairline_edge,
        ),
        // Measured on the reference's desktop card (design/23-WIDGETS.md section 1.1, M32-M34):
        // a one-point dark rim outside, a lighter rim inside, and a short soft drop. The dark
        // scheme was not measured: it takes the one-point rim and keeps its edge and drop.
        Material::Widget => Layers {
            hairline: Some(outer("0 0 0 var(--hair)")),
            ..card(
                if light {
                    shadow("0 2px 8px", BLACK, 120)
                } else {
                    shadow("0 8px 20px -8px", BLACK, 500)
                },
                if light {
                    inset("0 0 0 var(--hair)", WHITE, 100)
                } else {
                    hairline_edge
                },
            )
        },
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
    use super::{DEFAULT_TINT_ALPHA, flat_tint, recipe};
    use crate::appearance::Scheme;
    use crate::material::Material;
    use crate::tokens::hex::{Alpha, Hex};

    #[test]
    fn the_flat_tints_are_section_17_2() {
        #[rustfmt::skip]
        const CASES: &[(Material, Scheme, [u8; 3], u16)] = &[
            (Material::Bar, Scheme::Light, [248, 249, 246], 700),
            (Material::Bar, Scheme::Dark, [21, 24, 20], 680),
            (Material::Dock, Scheme::Light, [248, 249, 246], 590),
            (Material::Popover, Scheme::Light, [255, 255, 255], 780),
            (Material::Popover, Scheme::Dark, [42, 47, 40], 780),
            (Material::Sheet, Scheme::Light, [248, 249, 246], 820),
            (Material::Toast, Scheme::Dark, [21, 24, 20], 740),
            (Material::Osd, Scheme::Light, [248, 249, 246], 720),
            (Material::Widget, Scheme::Dark, [21, 24, 20], 670),
        ];
        for &(material, scheme, hex, alpha) in CASES {
            assert_eq!(
                flat_tint(material, scheme),
                Some((Hex(hex), Alpha(alpha))),
                "{material:?} {scheme:?}"
            );
        }
        assert_eq!(flat_tint(Material::Window, Scheme::Light), None);
    }

    #[test]
    fn the_default_key_paints_the_boosted_tint_at_section_17_2s_alphas() {
        #[rustfmt::skip]
        const CASES: &[(Material, Scheme, &str, &str, &str)] = &[
            (Material::Bar, Scheme::Light, "rgba(252,253,249,.7)", "rgba(252,253,249,.94)", "0"),
            (Material::Bar, Scheme::Dark, "rgba(20,24,19,.68)", "rgba(20,24,19,.94)", "0"),
            (Material::Dock, Scheme::Light, "rgba(252,253,249,.59)", "rgba(252,253,249,.94)", "22px"),
            (Material::Popover, Scheme::Light, "rgba(255,255,255,.78)", "rgba(255,255,255,.94)", "14px"),
            (Material::Popover, Scheme::Dark, "rgba(41,48,38,.78)", "rgba(41,48,38,.94)", "14px"),
            (Material::Sheet, Scheme::Light, "rgba(252,253,249,.82)", "rgba(252,253,249,.94)", "18px"),
            (Material::Widget, Scheme::Dark, "rgba(20,24,19,.67)", "rgba(20,24,19,.94)", "20px"),
            (Material::Window, Scheme::Light, "var(--f-grad)", "var(--f-grad)", "0"),
        ];
        for &(material, scheme, tint, solid, radius) in CASES {
            let got = recipe(material, scheme, DEFAULT_TINT_ALPHA);
            assert_eq!(got.tint, tint, "{material:?} {scheme:?}");
            assert_eq!(got.tint_solid, solid, "{material:?} {scheme:?}");
            assert_eq!(got.radius, radius, "{material:?} {scheme:?}");
        }
    }

    /// The vibrancy pass (design/23-WIDGETS.md section 1.1, M26-M34): the light card is a
    /// neutral grey at the gated .60, rimmed one point dark outside and light inside, with a
    /// short drop; the dark card keeps its tint, edge and drop and takes the one-point rim.
    #[test]
    fn the_widget_card_is_the_measured_grey_rim_and_drop() {
        let light = recipe(Material::Widget, Scheme::Light, DEFAULT_TINT_ALPHA);
        assert_eq!(light.tint, "rgba(228,228,228,.6)");
        assert_eq!(light.tint_solid, "rgba(228,228,228,.94)");
        assert_eq!(light.hairline, "0 0 0 var(--hair) rgba(0,0,0,.14)");
        assert_eq!(light.edge, "inset 0 0 0 var(--hair) rgba(255,255,255,.1)");
        assert_eq!(light.shadow, "0 2px 8px rgba(0,0,0,.12)");
        let dark = recipe(Material::Widget, Scheme::Dark, DEFAULT_TINT_ALPHA);
        assert_eq!(dark.hairline, "0 0 0 var(--hair) rgba(0,0,0,.6)");
        assert_eq!(
            dark.edge,
            "inset 0 0 0 var(--hairline) rgba(255,255,255,.09)"
        );
        assert_eq!(dark.shadow, "0 8px 20px -8px rgba(0,0,0,.5)");
    }

    #[test]
    fn the_key_scales_the_tint_and_never_the_solid_floor() {
        let low = recipe(Material::Sheet, Scheme::Light, Alpha(400));
        assert_eq!(low.tint, "rgba(252,253,249,.41)");
        assert_eq!(low.tint_solid, "rgba(252,253,249,.94)");
        let high = recipe(Material::Sheet, Scheme::Light, Alpha(1000));
        assert_eq!(high.tint, "rgba(252,253,249,1)");
    }

    #[test]
    fn every_card_stacks_hairline_highlight_edge_and_two_shadows() {
        let light = recipe(Material::Popover, Scheme::Light, DEFAULT_TINT_ALPHA);
        let dark = recipe(Material::Popover, Scheme::Dark, DEFAULT_TINT_ALPHA);
        assert_eq!(light.edge, "inset 0 0 0 var(--hairline) rgba(0,0,0,.08)");
        assert_eq!(
            dark.edge,
            "inset 0 0 0 var(--hairline) rgba(255,255,255,.09)"
        );
        assert_eq!(
            light.highlight,
            "inset 0 var(--hair) 0 rgba(255,255,255,.3)"
        );
        assert_eq!(
            dark.highlight,
            "inset 0 var(--hair) 0 rgba(255,255,255,.12)"
        );
        assert_eq!(light.hairline, "0 0 0 var(--hairline) rgba(0,0,0,.14)");
        assert_eq!(dark.hairline, "0 0 0 var(--hairline) rgba(0,0,0,.6)");
        assert_eq!(light.shadow_contact, "0 1px 2px rgba(0,0,0,.1)");
        assert_eq!(light.shadow, "0 12px 40px -12px rgba(0,0,0,.28)");
        for material in [
            Material::Dock,
            Material::Popover,
            Material::Sheet,
            Material::Toast,
            Material::Osd,
            Material::Widget,
        ] {
            for scheme in Scheme::ALL {
                let got = recipe(material, scheme, DEFAULT_TINT_ALPHA);
                for layer in [
                    &got.highlight,
                    &got.hairline,
                    &got.shadow_contact,
                    &got.shadow,
                ] {
                    assert_ne!(layer, "none", "{material:?} {scheme:?}");
                }
            }
        }
        let bar = recipe(Material::Bar, Scheme::Light, DEFAULT_TINT_ALPHA);
        assert_eq!(
            bar.edge,
            "inset 0 calc(-1 * var(--hairline)) 0 rgba(0,0,0,.08)"
        );
        assert_eq!(bar.hairline, "0 var(--hairline) 0 rgba(0,0,0,.14)");
        assert_eq!(
            (bar.shadow.as_str(), bar.highlight.as_str()),
            ("none", "none")
        );
    }
}
