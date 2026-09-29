//! What a band is: the bounds, per scheme, inside which [`super::accent_roles`] builds an accent.

use crate::style::appearance::theme::Scheme;
use crate::style::tokens::hex::Alpha;

/// An OKLCH lightness or chroma in thousandths: `Milli(560)` is 0.56.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Milli(pub u16);

impl Milli {
    /// The value as a fraction.
    pub fn fraction(self) -> f64 {
        f64::from(self.0) / 1000.0
    }
}

/// A hue on the OKLCH wheel, in whole degrees from 0 to 359.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Hue(pub u16);

impl Hue {
    /// The hue in degrees.
    pub fn degrees(self) -> f64 {
        f64::from(self.0 % 360)
    }
}

/// How far between the band's quiet and full chroma an accent sits, in thousandths.
///
/// A built-in accent is [`Weight::FULL`]; a Space lends its first dot's chroma (0 to 1 on the
/// editor's field), so a grey Space lends a grey-blue, never a vivid one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Weight(pub u16);

impl Weight {
    /// The whole of the band's chroma.
    pub const FULL: Weight = Weight(1000);

    /// The weight as a fraction, clamped to 0..=1.
    pub fn fraction(self) -> f64 {
        f64::from(self.0.min(1000)) / 1000.0
    }
}

/// What an accent is generated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccentPick {
    /// The hue.
    pub hue: Hue,
    /// How much of the band's chroma it takes.
    pub weight: Weight,
}

/// Which ink sits on the solid fill; the fill's lightness steps until that ink reads on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InkRule {
    /// White ink; the fill steps darker until white reaches the text floor.
    White,
    /// A near-black of the accent's own hue (OKLCH 0.22, 0.03); the fill steps lighter.
    Deep,
}

/// The chroma an accent takes at weight 0 and at full weight; gamut fitting may lower it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChromaSpan {
    /// Chroma at weight 0.
    pub quiet: Milli,
    /// Chroma at full weight: the band's ceiling.
    pub full: Milli,
}

impl ChromaSpan {
    /// The chroma `weight` asks for.
    pub fn at(self, weight: Weight) -> f64 {
        let quiet = self.quiet.fraction();
        quiet + (self.full.fraction() - quiet) * weight.fraction()
    }
}

/// One scheme's bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SchemeBand {
    /// Where the solid fill starts; it steps from here only as far as its ink needs.
    pub fill: Milli,
    /// The chroma bounds, shared by the fill and the text accent.
    pub chroma: ChromaSpan,
    /// The ink on the fill.
    pub ink: InkRule,
    /// Where the text accent starts; it steps away from the card until it reads on every ground.
    pub text: Milli,
    /// The wash's starting alpha; it rises only if the wash would not show on a ground.
    pub wash: Alpha,
    /// The focus ring's starting alpha; it rises until the ring stands off every ground.
    pub ring: Alpha,
}

/// A band: the light and the dark bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccentBand {
    /// Light.
    pub light: SchemeBand,
    /// Dark.
    pub dark: SchemeBand,
}

impl AccentBand {
    /// The bounds for `scheme`.
    pub fn scheme(&self, scheme: Scheme) -> &SchemeBand {
        match scheme {
            Scheme::Light => &self.light,
            Scheme::Dark => &self.dark,
        }
    }
}
