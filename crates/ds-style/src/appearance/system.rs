//! What the desktop says, read from the settings portal by `ds-settings`
//! (`org.freedesktop.appearance`: color-scheme, reduced motion, contrast).

use crate::appearance::theme::Scheme;
use serde::{Deserialize, Serialize};

/// The desktop's reduced-motion preference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReducedMotion {
    /// The desktop has no preference.
    #[default]
    NoPreference,
    /// The desktop asks for reduced motion.
    Reduce,
}

/// The desktop's contrast preference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum Contrast {
    /// Ordinary contrast.
    #[default]
    Normal,
    /// The desktop asks for higher contrast.
    High,
}

/// Everything the desktop contributes to resolving an [`crate::Appearance`].
///
/// The default is what a desktop with no portal answers: light, no motion preference, normal
/// contrast.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default)]
#[serde(default)]
#[non_exhaustive]
pub struct SystemPrefs {
    /// The desktop's colour scheme.
    pub scheme: Scheme,
    /// The desktop's reduced-motion preference.
    pub motion: ReducedMotion,
    /// The desktop's contrast preference.
    pub contrast: Contrast,
}

impl SystemPrefs {
    /// These preferences with the colour scheme `scheme`.
    #[must_use]
    pub const fn with_scheme(mut self, scheme: Scheme) -> Self {
        self.scheme = scheme;
        self
    }

    /// These preferences with the reduced-motion preference `motion`.
    #[must_use]
    pub const fn with_motion(mut self, motion: ReducedMotion) -> Self {
        self.motion = motion;
        self
    }

    /// These preferences with the contrast preference `contrast`.
    #[must_use]
    pub const fn with_contrast(mut self, contrast: Contrast) -> Self {
        self.contrast = contrast;
        self
    }
}
