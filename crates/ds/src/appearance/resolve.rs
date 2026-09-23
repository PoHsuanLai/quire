//! A person's choices plus the desktop's answer, resolved to the three attributes a `.ds` root
//! carries. Always explicit: a root never says "system" and never leans on a media query
//! (design/05-MOTION.md section 9 rule 11).

use super::{Accent, Appearance, Motion, MotionLevel, ReducedMotion, Scheme, SystemPrefs, Theme};

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
        [
            ("data-theme", self.scheme.slug()),
            ("data-accent", self.accent.slug()),
            ("data-motion", self.motion.slug()),
        ]
    }
}

/// Resolve `app`'s choices against the Space's own theme and the desktop's preferences.
///
/// `look_theme` is the active [`crate::SpaceLook`]'s theme; `System` in either place defers to
/// `system`, and `Motion::System` becomes `Reduced` when the desktop asks for reduced motion.
///
/// A Space that names its own theme wins over the app's: a Space's mode is `S`'s per-Space
/// rule and `system` there means "the viewer's scheme" (`S:1157-1159`), which is the app's
/// choice before it is the desktop's. Whether the shell honours a per-workspace theme is
/// design/03-COLOR.md open decision 13; this is the proposed reading.
pub fn resolve(app: Appearance, look_theme: Theme, system: SystemPrefs) -> Resolved {
    let scheme = explicit(look_theme)
        .or_else(|| explicit(app.theme))
        .unwrap_or(system.scheme);
    let motion = match app.motion {
        Motion::System => match system.motion {
            ReducedMotion::NoPreference => MotionLevel::Standard,
            ReducedMotion::Reduce => MotionLevel::Reduced,
        },
        Motion::Calm => MotionLevel::Calm,
        Motion::Standard => MotionLevel::Standard,
        Motion::Extra => MotionLevel::Extra,
        Motion::Reduced => MotionLevel::Reduced,
    };
    Resolved {
        scheme,
        accent: app.accent,
        motion,
    }
}

/// The scheme a theme names, or `None` for "follow".
fn explicit(theme: Theme) -> Option<Scheme> {
    match theme {
        Theme::System => None,
        Theme::Light => Some(Scheme::Light),
        Theme::Dark => Some(Scheme::Dark),
    }
}

#[cfg(test)]
mod tests {
    use super::{Resolved, resolve};
    use crate::appearance::{
        Accent, Appearance, Contrast, Motion, MotionLevel, ReducedMotion, Scheme, SystemPrefs,
        Theme,
    };

    const fn prefs(scheme: Scheme, motion: ReducedMotion) -> SystemPrefs {
        SystemPrefs {
            scheme,
            motion,
            contrast: Contrast::Normal,
        }
    }

    #[test]
    fn the_space_then_the_app_then_the_desktop_decide() {
        use ReducedMotion::{NoPreference, Reduce};
        #[rustfmt::skip]
        const CASES: &[(&str, Theme, Motion, Theme, SystemPrefs, Scheme, MotionLevel)] = &[
            ("all follow: desktop dark", Theme::System, Motion::System, Theme::System, prefs(Scheme::Dark, NoPreference), Scheme::Dark, MotionLevel::Standard),
            ("app light over a dark desktop", Theme::Light, Motion::System, Theme::System, prefs(Scheme::Dark, NoPreference), Scheme::Light, MotionLevel::Standard),
            ("space dark over an app light", Theme::Light, Motion::Calm, Theme::Dark, prefs(Scheme::Light, NoPreference), Scheme::Dark, MotionLevel::Calm),
            ("space light over a dark desktop", Theme::System, Motion::Extra, Theme::Light, prefs(Scheme::Dark, Reduce), Scheme::Light, MotionLevel::Extra),
            ("system motion under reduce", Theme::System, Motion::System, Theme::System, prefs(Scheme::Light, Reduce), Scheme::Light, MotionLevel::Reduced),
            ("explicit standard ignores reduce", Theme::Dark, Motion::Standard, Theme::System, prefs(Scheme::Light, Reduce), Scheme::Dark, MotionLevel::Standard),
            ("explicit reduced", Theme::System, Motion::Reduced, Theme::System, prefs(Scheme::Light, NoPreference), Scheme::Light, MotionLevel::Reduced),
        ];
        for &(name, theme, motion, look, system, scheme, level) in CASES {
            let app = Appearance {
                theme,
                accent: Accent::Green,
                motion,
            };
            let got = resolve(app, look, system);
            let want = Resolved {
                scheme,
                accent: Accent::Green,
                motion: level,
            };
            assert_eq!(got, want, "{name}");
        }
    }

    #[test]
    fn attrs_are_always_explicit() {
        let resolved = Resolved {
            scheme: Scheme::Dark,
            accent: Accent::Violet,
            motion: MotionLevel::Reduced,
        };
        assert_eq!(
            resolved.attrs(),
            [
                ("data-theme", "dark"),
                ("data-accent", "violet"),
                ("data-motion", "reduced")
            ]
        );
    }
}
