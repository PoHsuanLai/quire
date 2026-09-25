//! anyrender layers as krilla's nested state: a layer is up to three pushes (clip, blend mode,
//! opacity), and its pop undoes exactly those.

use krilla::blend::BlendMode as KrillaBlend;
use peniko::{BlendMode, Mix};

/// How many surface pushes one open layer made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Pushes(pub(crate) u8);

impl Pushes {
    /// One more push.
    pub(crate) fn and_one(self) -> Self {
        Pushes(self.0 + 1)
    }
}

/// The krilla blend mode for `blend`'s mix; `None` for normal, which needs no push. The
/// compositing operator has no PDF form and paints as source-over.
pub(crate) fn blend(blend: BlendMode) -> Option<KrillaBlend> {
    let mode = match blend.mix {
        Mix::Normal => return None,
        Mix::Multiply => KrillaBlend::Multiply,
        Mix::Screen => KrillaBlend::Screen,
        Mix::Overlay => KrillaBlend::Overlay,
        Mix::Darken => KrillaBlend::Darken,
        Mix::Lighten => KrillaBlend::Lighten,
        Mix::ColorDodge => KrillaBlend::ColorDodge,
        Mix::ColorBurn => KrillaBlend::ColorBurn,
        Mix::HardLight => KrillaBlend::HardLight,
        Mix::SoftLight => KrillaBlend::SoftLight,
        Mix::Difference => KrillaBlend::Difference,
        Mix::Exclusion => KrillaBlend::Exclusion,
        Mix::Hue => KrillaBlend::Hue,
        Mix::Saturation => KrillaBlend::Saturation,
        Mix::Color => KrillaBlend::Color,
        Mix::Luminosity => KrillaBlend::Luminosity,
    };
    Some(mode)
}
