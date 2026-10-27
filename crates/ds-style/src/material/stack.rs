//! Material stack v2 (the macOS polish pass, 2026-09-24; design/03-COLOR.md section 17.4): the
//! layers macOS stacks on every chrome material and quire lacked, and the tokens that tune
//! them. From the outside in: a 0.5 px dark outer hairline (`--m-hairline`), the tight contact
//! shadow (`--m-shadow-contact`) and the wide ambient one (`--m-shadow-ambient`), then inside
//! the 1 px top highlight (`--m-highlight`) over the tint, whose colour carries the vibrancy
//! boost (`vibrancy.rs`). Each input is a stylesheet token with its default in the material
//! blocks; a look or the person's own sheet sets it on the root or on any element around a surface.

use crate::tokens::name::VarName;

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
pub(crate) const STACK_INPUTS: [VarName; 6] = [
    HIGHLIGHT_LIGHT,
    HIGHLIGHT_DARK,
    HAIRLINE_LIGHT,
    HAIRLINE_DARK,
    SHADOW_STRENGTH,
    VIBRANCY,
];
