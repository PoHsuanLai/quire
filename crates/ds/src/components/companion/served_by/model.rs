//! A model the person may pick.

use crate::components::companion::answer::footer::ServedPlace;

/// Whether a model can answer now.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModelState {
    /// Loaded, or loads at once.
    Ready,
    /// Loading.
    Loading,
    /// Not on this computer yet.
    NeedsDownload {
        /// How big the download is, in words.
        size: String,
    },
    /// Cannot be used, and why.
    Unavailable(String),
}

/// One model in the chip's menu.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModelOption {
    /// The model's key, opaque.
    pub key: String,
    /// Its name.
    pub label: String,
    /// Where it runs.
    pub place: ServedPlace,
    /// Whether it can answer now.
    pub state: ModelState,
}
