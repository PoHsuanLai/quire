//! Which keymap source a launch uses: the one the environment names, or the one the app gave.

use crate::keymap_detect::{detected, platform_of};
use chordkit::{ChangeCallback, Keymap, KeymapSource, Platform, SourceError, Watch};
use ds::keys::KeySource;
use std::fmt;
use std::sync::Arc;

type SharedSource = Arc<dyn KeymapSource + Send + Sync>;

/// Where an app's keymap comes from.
#[derive(Clone, Default)]
pub(crate) enum KeymapChoice {
    /// The platform's own: chordkit's detection and the sources it has for that platform.
    #[default]
    Detected,
    /// The app's source (our desktop's keycap, say), for the detected platform.
    Given(SharedSource),
    /// The app's source and the platform it serves.
    GivenFor(Platform, SharedSource),
}

impl KeymapChoice {
    /// The source to launch with, reading the environment through `var`.
    pub(crate) fn resolve(self, var: &impl Fn(&str) -> Option<String>) -> KeySource {
        match self {
            KeymapChoice::Detected => detected(var),
            KeymapChoice::Given(source) => {
                KeySource::new(platform_of(var), Box::new(Shared(source)))
            }
            KeymapChoice::GivenFor(platform, source) => {
                KeySource::new(platform, Box::new(Shared(source)))
            }
        }
    }
}

impl fmt::Debug for KeymapChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeymapChoice::Detected => f.write_str("Detected"),
            KeymapChoice::Given(_) => f.write_str("Given"),
            KeymapChoice::GivenFor(platform, _) => write!(f, "GivenFor({platform:?})"),
        }
    }
}

/// A shared source as a source, so a clonable config can hand a window its own box.
struct Shared(SharedSource);

impl KeymapSource for Shared {
    fn load(&self, platform: Platform) -> Result<Keymap, SourceError> {
        self.0.load(platform)
    }

    fn watch(&self, on_change: ChangeCallback) -> Result<Watch, SourceError> {
        self.0.watch(on_change)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chordkit::{ConventionSource, Desktop};

    #[test]
    fn a_given_source_takes_the_detected_platform_or_the_one_named() {
        let given: SharedSource = Arc::new(ConventionSource);
        let env = |name: &str| (name == "CHORDKIT_PLATFORM").then(|| "windows".to_owned());
        let ours = Platform::Linux {
            desktop: Desktop::Ours,
        };
        let by_env = KeymapChoice::Given(Arc::clone(&given)).resolve(&env);
        let named = KeymapChoice::GivenFor(ours, given).resolve(&env);
        assert_eq!(by_env.platform(), Platform::Windows);
        assert_eq!(named.platform(), ours);
        assert!(named.load().is_ok());
    }
}
