//! Where the keymap comes from, as the toolkit holds it: chordkit's source and the platform it
//! loads for. The launcher injects it (`ds_blitz::AppConfig::with_keymap_source`); without one a
//! document takes our desktop's conventions, so a test reads no real system.

use chordkit::{
    ChangeCallback, ConventionSource, Desktop, Keymap, KeymapSource, Overrides, Platform,
    SourceError, Watch,
};
use std::fmt;
use std::sync::Arc;

/// A keymap source and the platform it serves. `Clone`, `Send` and `Sync` so the launcher can
/// hand a copy to every window as a root context.
#[derive(Clone)]
pub struct KeySource {
    platform: Platform,
    source: Arc<dyn KeymapSource + Send + Sync>,
    overrides: Option<Arc<Overrides>>,
}

impl KeySource {
    /// `source`, loading for `platform`.
    pub fn new(platform: Platform, source: Box<dyn KeymapSource + Send + Sync>) -> Self {
        KeySource {
            platform,
            source: Arc::from(source),
            overrides: None,
        }
    }

    /// This source with an app's own changes laid on top of whatever it loads (system detection,
    /// keycap), so an app with its own rebinding file folds into the shared keymap without
    /// reimplementing detection. Applied again after every reload; a change it cannot apply is
    /// kept in `Keys::problems`. A second call replaces the first.
    pub fn with_overrides(mut self, overrides: Overrides) -> Self {
        self.overrides = Some(Arc::new(overrides));
        self
    }

    /// `platform`'s conventions and nothing else: the fixed-platform source tests use.
    pub fn conventions(platform: Platform) -> Self {
        KeySource::new(platform, Box::new(ConventionSource))
    }

    /// The platform the keymap is for.
    pub fn platform(&self) -> Platform {
        self.platform
    }

    pub(super) fn overrides(&self) -> Option<&Overrides> {
        self.overrides.as_deref()
    }

    /// The keymap the source gives for its platform, before the app's overrides.
    pub fn load(&self) -> Result<Keymap, SourceError> {
        self.source.load(self.platform)
    }

    pub(super) fn watch(&self, on_change: ChangeCallback) -> Result<Watch, SourceError> {
        self.source.watch(on_change)
    }
}

impl Default for KeySource {
    /// Our desktop's conventions: the Mac's, with Command arriving as Super.
    fn default() -> Self {
        KeySource::conventions(Platform::Linux {
            desktop: Desktop::Ours,
        })
    }
}

impl fmt::Debug for KeySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeySource")
            .field("platform", &self.platform)
            .finish_non_exhaustive()
    }
}
