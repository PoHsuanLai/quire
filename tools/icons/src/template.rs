/// A fraction in 0..=1 (of a plate side, of a canvas), kept apart from pixel counts.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Fraction(pub f32);

/// The template values design/08-ICONS.md marks "proposed" (2.1, 2.2, 2.4, 2.5, 3.7). One
/// struct so the bake-off can vary them and the settings wiring later has one place to read.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Template {
    /// Lamé exponent n of the squircle (2.1).
    pub exponent: f32,
    /// Plate side over canvas side at 64 px and up (2.2: 824 / 1024 = 80.5 %).
    pub plate: Fraction,
    /// Safe square side over plate side (2.4).
    pub safe: Fraction,
    /// The object's longer bounding-box axis over plate side (2.4 allows 64-80 %).
    pub fill: Fraction,
    /// Upward shift of the object's centroid, over plate side (2.4).
    pub lift: Fraction,
    /// Background key: largest OKLab chroma distance (a, b) from the local background a ground
    /// or shadow pixel may have; shadows darken, they do not tint.
    pub key_chroma: f32,
    /// Background key: how much lighter (OKLab L) than the local background ground may be.
    pub key_lighter: f32,
    /// Background key: how much darker (OKLab L) a shadow on the ground may be.
    pub key_darker: f32,
    /// Background key: largest ΔE_OK between neighbours inside the ground (a drawn edge is more).
    pub key_step: f32,
    /// Background key: blur sigma of the object's edge in render px (3.7's 2 px feather).
    pub key_feather: f32,
    /// Background key: object components smaller than this fraction of the largest are ground.
    pub key_speck: f32,
    /// Shadow matte: darkening below this fraction is treated as ground noise.
    pub shadow_floor: f32,
    /// Shadow matte: the most opaque the kept model shadow may get.
    pub shadow_max: f32,
}

impl Default for Template {
    fn default() -> Self {
        Self {
            exponent: 5.0,
            plate: Fraction(824.0 / 1024.0),
            safe: Fraction(0.80),
            fill: Fraction(0.72),
            lift: Fraction(0.02),
            key_chroma: 0.025,
            key_lighter: 0.08,
            key_darker: 0.35,
            key_step: 0.012,
            key_feather: 1.0,
            key_speck: 0.02,
            shadow_floor: 0.03,
            shadow_max: 0.45,
        }
    }
}
