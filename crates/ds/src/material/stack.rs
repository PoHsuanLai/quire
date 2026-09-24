//! Material stack v2 (the macOS polish pass, 2026-09-24; design/03-COLOR.md section 17.4): the
//! layers macOS stacks on every chrome material and quire lacked, and the settings that tune
//! them. From the outside in: a 0.5 px dark outer hairline (`--m-hairline`), the tight contact
//! shadow (`--m-shadow-contact`) and the wide ambient one (`--m-shadow-ambient`), then inside
//! the 1 px top highlight (`--m-highlight`) over the tint, whose colour carries the vibrancy
//! boost (`vibrancy.rs`). Each is written as an input on the root ([`crate::Ds`]'s `stack`) or
//! on any element around a surface.

use super::layer::fraction;
use crate::tokens::{Alpha, VarName};

/// `--m-highlight-light`: the top highlight's white alpha in the light scheme.
pub(crate) const HIGHLIGHT_LIGHT: VarName = VarName("--m-highlight-light");
/// `--m-highlight-dark`: the same in the dark scheme.
pub(crate) const HIGHLIGHT_DARK: VarName = VarName("--m-highlight-dark");
/// `--m-hairline-light`: the outer hairline's black alpha in the light scheme.
pub(crate) const HAIRLINE_LIGHT: VarName = VarName("--m-hairline-light");
/// `--m-hairline-dark`: the same in the dark scheme.
pub(crate) const HAIRLINE_DARK: VarName = VarName("--m-hairline-dark");
/// `--m-shadow-strength`: both shadows' alphas are multiplied by it (1 at the default).
pub(crate) const SHADOW_STRENGTH: VarName = VarName("--m-shadow-strength");
/// `--m-vibrancy`: how much of the baked boost the tint shows (1 at the default, 0 flat).
pub(crate) const VIBRANCY: VarName = VarName("--m-vibrancy");

/// Every input, for the lint's registry.
#[cfg_attr(not(feature = "lint"), allow(dead_code))] // Read by the lint's registry.
pub(crate) const STACK_INPUTS: [VarName; 6] = [
    HIGHLIGHT_LIGHT,
    HIGHLIGHT_DARK,
    HAIRLINE_LIGHT,
    HAIRLINE_DARK,
    SHADOW_STRENGTH,
    VIBRANCY,
];

/// The material stack's settings (design/22-SETTINGS.md `appearance.material_*`, proposed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialStack {
    /// The top highlight's alpha, light scheme (`appearance.material_highlight_light`, .30).
    pub highlight_light: Alpha,
    /// The top highlight's alpha, dark scheme (`appearance.material_highlight_dark`, .12).
    pub highlight_dark: Alpha,
    /// The outer hairline's alpha, light scheme (`appearance.material_hairline_light`, .14).
    pub hairline_light: Alpha,
    /// The outer hairline's alpha, dark scheme (`appearance.material_hairline_dark`, .60).
    pub hairline_dark: Alpha,
    /// Both shadows' strength (`appearance.material_shadow_strength`, 1.00).
    pub shadow_strength: Alpha,
    /// The vibrancy boost's share of the tint (`appearance.material_vibrancy`, 1.00).
    pub vibrancy: Alpha,
}

impl Default for MaterialStack {
    /// The keys' defaults.
    fn default() -> Self {
        MaterialStack {
            highlight_light: Alpha(300),
            highlight_dark: Alpha(120),
            hairline_light: Alpha(140),
            hairline_dark: Alpha(600),
            shadow_strength: Alpha(1000),
            vibrancy: Alpha(1000),
        }
    }
}

impl MaterialStack {
    /// Every input, inline: `--m-highlight-light:.3;…`.
    pub fn style_attr(&self) -> String {
        [
            (HIGHLIGHT_LIGHT, self.highlight_light),
            (HIGHLIGHT_DARK, self.highlight_dark),
            (HAIRLINE_LIGHT, self.hairline_light),
            (HAIRLINE_DARK, self.hairline_dark),
            (SHADOW_STRENGTH, self.shadow_strength),
            (VIBRANCY, self.vibrancy),
        ]
        .into_iter()
        .map(|(var, value)| format!("{}:{};", var.as_str(), fraction(value)))
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::MaterialStack;
    use crate::tokens::Alpha;

    #[test]
    fn the_stack_writes_every_input() {
        assert_eq!(
            MaterialStack::default().style_attr(),
            "--m-highlight-light:.3;--m-highlight-dark:.12;--m-hairline-light:.14;\
             --m-hairline-dark:.6;--m-shadow-strength:1;--m-vibrancy:1;"
        );
        let flat = MaterialStack {
            vibrancy: Alpha(0),
            ..MaterialStack::default()
        };
        assert!(flat.style_attr().ends_with("--m-vibrancy:0;"));
    }
}
