//! 8-bit sRGB and linear light, and the sRGB transfer functions between them.

/// A colour as 8-bit, gamma-encoded sRGB channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Srgb(pub [u8; 3]);

/// A colour in linear-light sRGB. Unclamped: a colour converted from OKLab may lie outside
/// `0..=1`, and a gamut fit reads how far.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearRgb(pub [f64; 3]);

/// The sRGB transfer, decoding one 8-bit channel to linear light.
pub fn decode(channel: u8) -> f64 {
    let c = f64::from(channel) / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// The sRGB transfer, encoding one linear channel, clamped into `0..=1` first.
pub fn encode(linear: f64) -> f64 {
    let c = linear.clamp(0.0, 1.0);
    if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

impl From<Srgb> for LinearRgb {
    fn from(Srgb(channels): Srgb) -> Self {
        LinearRgb(channels.map(decode))
    }
}

impl From<LinearRgb> for Srgb {
    /// Clamped into the gamut and rounded to the nearest byte.
    fn from(LinearRgb(channels): LinearRgb) -> Self {
        // In range after `encode`'s clamp: 0..=255.
        Srgb(channels.map(|channel| (encode(channel) * 255.0).round() as u8))
    }
}

#[cfg(test)]
mod tests {
    use super::{LinearRgb, Srgb};

    #[test]
    fn bytes_round_trip_through_linear_light() {
        for byte in 0..=255u8 {
            let srgb = Srgb([byte, 255 - byte, byte / 2]);
            assert_eq!(Srgb::from(LinearRgb::from(srgb)), srgb);
        }
    }

    #[test]
    fn out_of_gamut_light_clamps() {
        assert_eq!(Srgb::from(LinearRgb([-0.2, 1.4, 0.5])).0[..2], [0, 255]);
    }
}
