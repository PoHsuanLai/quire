//! The squircle's constants (design/08-ICONS.md sections 2.1 and 4.1): the superellipse
//! exponent, how far a corner of nominal radius `r` reaches, and the circular radius its
//! shadows and hairlines are drawn from. The token table sizes corners from them and the icon
//! plate draws its masks from them.

/// The superellipse exponent (design/08 section 2.1, proposed; settled for the shell
/// 2026-09-24).
pub const EXPONENT: f64 = 5.0;

/// How far a squircle corner of nominal radius `r` reaches along each edge: `2 r`.
pub const EXTENT_PER_RADIUS: f64 = 2.0;

/// The circular radius that touches the squircle corner at its 45 degree point, as a share of the
/// nominal radius: `2 (1 - 2^(-1/5)) / (1 - 1/sqrt 2)`, about .884. The squircle lies inside this
/// circle everywhere and at most .03 r from it, so a squircle element's shadows and hairlines
/// are drawn from it (Blitz draws a `box-shadow` from the `border-radius`, never the mask).
pub fn shadow_radius_share() -> f64 {
    let diagonal = EXTENT_PER_RADIUS * (1.0 - 2f64.powf(-1.0 / EXPONENT));
    diagonal / (1.0 - std::f64::consts::FRAC_1_SQRT_2)
}

#[cfg(test)]
mod tests {
    use super::shadow_radius_share;

    #[test]
    fn the_shadow_radius_touches_the_corner_at_45_degrees() {
        assert!((shadow_radius_share() - 0.884).abs() < 0.001);
    }
}
