//! The app switcher (design/13 §13.3.5, §13.4; 03 §1.3): Command-Tab over applications, not
//! windows. A quick tap switches to the previous app with no panel; held past the show delay the
//! panel appears; Tab, Grave and the arrows move the selection; Q quits and H hides the selected
//! app; Down or Up opens App Exposé for it; Escape cancels; the chord's release activates.
//!
//! Generic over the app key `K`, so the compositor runs it over its own app ids and the shell
//! over its launcher's. The caller owns the chord's grab, the release and the panel's drawing; it
//! passes the MRU app list (the current app first) to every [`step`].

mod model;
mod step;
#[cfg(test)]
mod tests;

pub use model::{SwIn, SwKey, SwOut, Switcher, SwitcherParams};
pub use step::step;
