//! A person's choices plus the desktop's answer, resolved to the three attributes a `.ds` root
//! carries. Always explicit: a root never says "system" and never leans on a media query
//! (design/05-MOTION.md section 9 rule 11).
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use super::{Accent, Appearance, MotionLevel, Scheme, SystemPrefs, Theme};

/// What a `.ds` root is drawn as.
#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub struct Resolved {
    /// The palette.
    pub scheme: Scheme,
    /// The card's accent.
    pub accent: Accent,
    /// The motion level.
    pub motion: MotionLevel,
}

impl Resolved {
    /// `data-theme`, `data-accent` and `data-motion` with their values, in that order.
    pub fn attrs(&self) -> [(&'static str, &'static str); 3] {
        todo!()
    }
}

/// Resolve `app`'s choices against the Space's own theme and the desktop's preferences.
///
/// `look_theme` is the active [`crate::SpaceLook`]'s theme; `System` in either place defers to
/// `system`, and `Motion::System` becomes `Reduced` when the desktop asks for reduced motion.
pub fn resolve(app: Appearance, look_theme: Theme, system: SystemPrefs) -> Resolved {
    todo!()
}
