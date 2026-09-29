//! The `--f-*` variables a `.ds` root carries inline, computed in Rust for the resolved scheme
//! (design/03-COLOR.md sections 4.5 and 8, design/21-SPACES.md section 2).
//!
//! Replaces mailo's `ui/paint.rs` `push_palette` and `grain_opacity`; the `-l`/`-d` copies
//! and `SYSTEM_ACCENT` are deleted.
//!
//! The arithmetic is mailo's `push_palette`, `push_accent` and `grain_opacity` for an explicit
//! theme: the palette `derive` makes, the first stop as `--f-solid`, the gradient, the scheme's
//! `--f-line`, and the grain as an opacity. mailo's `--f-hover` is written `--f-pill-hover`.

use super::look::{CardAccent, Grain, SpaceLook};
use super::palette::{derive, gradient};
use crate::style::appearance::theme::Scheme;
use crate::style::tokens::accent_band::roles::AccentRoles;

/// Every frame variable one Space paints in one scheme, as CSS values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameVars {
    /// `--f-ink`: sidebar text.
    pub ink: String,
    /// `--f-ink-soft`: secondary sidebar text.
    pub ink_soft: String,
    /// `--f-ink-faint`: counts and meta.
    pub ink_faint: String,
    /// `--f-pill`: a selected pill.
    pub pill: String,
    /// `--f-pill-hover`: a row under the pointer.
    pub pill_hover: String,
    /// `--f-line`: hairlines on the frame.
    pub line: String,
    /// `--f-solid`: the first gradient stop, for surfaces that cannot paint the gradient.
    pub solid: String,
    /// `--f-grad`: the frame's `linear-gradient`.
    pub gradient: String,
    /// The gradient's stops, one colour per dot, left to right: what a `SpaceDot`
    /// hands its stylesheet as `--dot-c1..3`. Not written on the root.
    pub stops: Vec<String>,
    /// `--f-grain`: the grain tile's opacity, `grain / 100 x .20` light, `x .16` dark.
    pub grain_opacity: String,
    /// The accent roles when the card borrows the Space's hue; written as `--accent`,
    /// `--accent-ink`, `--accent-soft`, `--accent-text`, `--accent-text-material`,
    /// `--accent-ring` and `--seal`.
    pub accent: Option<AccentRoles>,
}

impl FrameVars {
    /// The frame variables `look` paints in `scheme`.
    pub fn of(look: &SpaceLook, scheme: Scheme) -> Self {
        let palette = derive(&look.dots, scheme);
        let gradient = gradient(&palette);
        let solid = palette.stops.first().cloned().unwrap_or_default();
        let line = match scheme {
            Scheme::Light => "rgba(0,0,0,.08)",
            Scheme::Dark => "rgba(255,255,255,.09)",
        };
        let accent = match look.card_accent {
            CardAccent::Postmark => None,
            CardAccent::SpaceHue => Some(palette.accent_roles),
        };
        FrameVars {
            ink: palette.ink,
            ink_soft: palette.soft,
            ink_faint: palette.faint,
            pill: palette.pill,
            pill_hover: palette.hover,
            line: line.to_owned(),
            solid,
            gradient,
            stops: palette.stops.clone(),
            grain_opacity: grain_opacity(look.grain, scheme),
            accent,
        }
    }

    /// The inline `style` attribute value: `--f-ink:#…;--f-ink-soft:#…;…`.
    pub fn style_attr(&self) -> String {
        self.pairs()
            .iter()
            .map(|(name, value)| format!("{name}:{value};"))
            .collect()
    }

    /// The variable names the attribute writes, in order (the lint registers them).
    #[cfg(feature = "lint")]
    pub(crate) fn names(&self) -> Vec<&'static str> {
        self.pairs().into_iter().map(|(name, _)| name).collect()
    }

    /// Every variable with its value, in the order the attribute writes them.
    fn pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = vec![
            ("--f-ink", self.ink.clone()),
            ("--f-ink-soft", self.ink_soft.clone()),
            ("--f-ink-faint", self.ink_faint.clone()),
            ("--f-pill", self.pill.clone()),
            ("--f-pill-hover", self.pill_hover.clone()),
            ("--f-line", self.line.clone()),
            ("--f-solid", self.solid.clone()),
            ("--f-grad", self.gradient.clone()),
            ("--f-grain", self.grain_opacity.clone()),
        ];
        if let Some(roles) = &self.accent {
            pairs.extend([
                ("--accent", roles.fill.css()),
                ("--accent-ink", roles.ink.css()),
                ("--accent-soft", roles.wash_colour().css()),
                ("--accent-text", roles.text.css()),
                ("--accent-text-material", roles.text_material.css()),
                ("--accent-ring", roles.ring_colour().css()),
                ("--seal", roles.fill.css()),
            ]);
        }
        pairs
    }
}

/// Grain as a CSS opacity: the stored 0-100 times .20 in light and .16 in dark
/// (design/03-COLOR.md section 8), written with no trailing zeros: 35 is `.07` light and
/// `.056` dark.
fn grain_opacity(grain: Grain, scheme: Scheme) -> String {
    let per_step: u32 = match scheme {
        Scheme::Light => 20,
        Scheme::Dark => 16,
    };
    // In ten-thousandths: grain 100 x 20 = 2000 is .2.
    let value = u32::from(grain.0) * per_step;
    let whole = value / 10_000;
    let fraction = value % 10_000;
    if fraction == 0 {
        return whole.to_string();
    }
    let digits = format!("{fraction:04}");
    let digits = digits.trim_end_matches('0');
    if whole == 0 {
        format!(".{digits}")
    } else {
        format!("{whole}.{digits}")
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameVars, grain_opacity};
    use crate::style::appearance::theme::{Scheme, Theme};
    use crate::style::space::look::{CardAccent, Grain, SpaceLook};
    use crate::style::space::palette::NEUTRAL_DOT;
    use crate::style::space::presets::PRESETS;

    #[test]
    fn grain_is_a_fraction_of_the_scheme_ceiling() {
        const CASES: &[(u8, Scheme, &str)] = &[
            (35, Scheme::Light, ".07"),
            (35, Scheme::Dark, ".056"),
            (55, Scheme::Light, ".11"),
            (0, Scheme::Dark, "0"),
            (100, Scheme::Light, ".2"),
            (100, Scheme::Dark, ".16"),
            (255, Scheme::Light, ".51"),
        ];
        for &(grain, scheme, want) in CASES {
            assert_eq!(
                grain_opacity(Grain(grain), scheme),
                want,
                "{grain} {scheme:?}"
            );
        }
    }

    #[test]
    fn the_style_attribute_names_every_variable_once() {
        let look = SpaceLook {
            dots: vec![NEUTRAL_DOT],
            grain: Grain(35),
            theme: crate::style::appearance::theme::Theme::System,
            card_accent: CardAccent::SpaceHue,
        };
        let vars = FrameVars::of(&look, Scheme::Light);
        let style = vars.style_attr();
        let names: Vec<&str> = style
            .split(';')
            .filter(|pair| !pair.is_empty())
            .filter_map(|pair| pair.split_once(':').map(|(name, _)| name))
            .collect();
        assert_eq!(
            names,
            [
                "--f-ink",
                "--f-ink-soft",
                "--f-ink-faint",
                "--f-pill",
                "--f-pill-hover",
                "--f-line",
                "--f-solid",
                "--f-grad",
                "--f-grain",
                "--accent",
                "--accent-ink",
                "--accent-soft",
                "--accent-text",
                "--accent-text-material",
                "--accent-ring",
                "--seal"
            ]
        );
        assert!(
            style.starts_with(&format!("--f-ink:{};", vars.ink)),
            "{style}"
        );
    }
    /// Each preset in each scheme, computed by the mockup's own JavaScript arithmetic
    /// (design/03-COLOR.md sections 4.3-4.4, ported line for line outside this crate):
    /// `[ink, soft, faint, hover, gradient]`. The Space accent is no longer the mockup's: it is
    /// the settled band's (design/03-COLOR.md section 20), checked against `derive` below.
    #[rustfmt::skip]
    const MOCKUP: &[(usize, Scheme, [&str; 5])] = &[
        (0, Scheme::Light, ["#12192f", "#3f475c", "#60697e", "#d3d9e9", "linear-gradient(135deg,#e1eafe 0%,#eee0f3 100%)"]),
        (0, Scheme::Dark, ["#e4e8f1", "#b8becc", "#8b92a2", "rgba(255,255,255,.06)", "linear-gradient(135deg,#131928 0%,#211924 100%)"]),
        (1, Scheme::Light, ["#0c1f12", "#3b4d40", "#5c6e60", "#d0ddd3", "linear-gradient(135deg,#dbf1df 0%,#f5e2d3 50%,#f3dcd8 100%)"]),
        (1, Scheme::Dark, ["#e3eae4", "#b5c1b8", "#89968c", "rgba(255,255,255,.06)", "linear-gradient(135deg,#101d13 0%,#251a11 50%,#291c1a 100%)"]),
        (2, Scheme::Light, ["#001f28", "#324c55", "#536e77", "#cadde3", "linear-gradient(135deg,#d0f0fb,#d0f0fb)"]),
        (2, Scheme::Dark, ["#dfeaee", "#b0c1c7", "#82969d", "rgba(255,255,255,.06)", "linear-gradient(135deg,#081d23,#081d23)"]),
        (3, Scheme::Light, ["#2a1313", "#584140", "#7b6261", "#e7d4d3", "linear-gradient(135deg,#fee2e1 0%,#f7e1d3 100%)"]),
        (3, Scheme::Dark, ["#f0e5e4", "#cab9b8", "#a08d8c", "rgba(255,255,255,.06)", "linear-gradient(135deg,#251414 0%,#261a11 100%)"]),
        (4, Scheme::Light, ["#02201e", "#354d4b", "#556f6d", "#cbdedc", "linear-gradient(135deg,#d3f1ee 0%,#d6e9f7 100%)"]),
        (4, Scheme::Dark, ["#e0eae9", "#b1c2c0", "#849795", "rgba(255,255,255,.06)", "linear-gradient(135deg,#0a1d1c 0%,#131e27 100%)"]),
        (5, Scheme::Light, ["#261420", "#53424d", "#75626e", "#e3d4dd", "linear-gradient(135deg,#f9e2f0 0%,#e5e3f7 100%)"]),
        (5, Scheme::Dark, ["#ede5ea", "#c6b9c1", "#9c8d96", "rgba(255,255,255,.06)", "linear-gradient(135deg,#21151d 0%,#1c1b26 100%)"]),
        (6, Scheme::Light, ["#1f1a09", "#4c4839", "#6d6959", "#dcd9cd", "linear-gradient(135deg,#efead7,#efead7)"]),
        (6, Scheme::Dark, ["#eae8e1", "#c0beb3", "#959286", "rgba(255,255,255,.06)", "linear-gradient(135deg,#1c190e,#1c190e)"]),
        (7, Scheme::Light, ["#191b1c", "#464849", "#68696b", "#d6d9dd", "linear-gradient(135deg,#e8eaec,#e8eaec)"]),
        (7, Scheme::Dark, ["#e7e8e8", "#bdbebf", "#919293", "rgba(255,255,255,.06)", "linear-gradient(135deg,#191a1b,#191a1b)"]),
    ];

    #[test]
    fn every_preset_paints_what_the_mockup_computes() {
        for &(index, scheme, [ink, soft, faint, hover, gradient]) in MOCKUP {
            let preset = PRESETS[index];
            let look = SpaceLook {
                dots: preset.dots.to_vec(),
                grain: Grain(preset.grain.unwrap_or(40)),
                theme: Theme::System,
                card_accent: CardAccent::SpaceHue,
            };
            let vars = FrameVars::of(&look, scheme);
            let case = format!("preset {index} {scheme:?}");
            assert_eq!(
                [vars.ink.as_str(), &vars.ink_soft, &vars.ink_faint],
                [ink, soft, faint],
                "{case}"
            );
            assert_eq!(vars.pill_hover, hover, "{case}");
            assert_eq!(vars.gradient, gradient, "{case}");
            let first = gradient["linear-gradient(135deg,".len()..]
                .split([' ', ','])
                .next();
            assert_eq!(
                Some(vars.solid.as_str()),
                first,
                "{case}: --f-solid is the first stop"
            );
            let (pill, line) = match scheme {
                Scheme::Light => ("rgba(255,255,255,.72)", "rgba(0,0,0,.08)"),
                Scheme::Dark => ("rgba(255,255,255,.10)", "rgba(255,255,255,.09)"),
            };
            assert_eq!([vars.pill.as_str(), &vars.line], [pill, line], "{case}");
            let lent = crate::style::space::palette::derive(&look.dots, scheme).accent_roles;
            assert_eq!(vars.accent, Some(lent), "{case}");
            let postmark = FrameVars::of(
                &SpaceLook {
                    card_accent: CardAccent::Postmark,
                    ..look
                },
                scheme,
            );
            assert_eq!(postmark.accent, None, "{case}: Postmark writes no accent");
        }
        assert_eq!(MOCKUP.len(), PRESETS.len() * 2);
    }
}
