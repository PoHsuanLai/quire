//! The app switcher (design/13 §13.3.5, §13.4; 03 §1.3): Command-Tab over applications, not
//! windows. A quick tap switches to the previous app with no panel; held past the show delay the
//! panel appears; Tab, Grave and the arrows move the selection; Q quits and H hides the selected
//! app; Down or Up opens App Exposé for it; Escape cancels; the chord's release activates.
//!
//! The machine is the [`Phase`]. It names apps by index into the MRU list (the current app first),
//! so the compositor runs it over its own app ids and the shell over its launcher's. The caller
//! owns the chord's grab, the release and the panel's drawing; it provides the list's length as
//! the machine's context at every step and maps the indexes in the outputs back through its list.
//! The panel's show delay is a deadline in the `Armed` phase, so the machine's wake is its one
//! timer.

mod model;
mod step;
#[cfg(test)]
mod tests;

pub use model::{Phase, SwIn, SwKey, SwOut, SwitcherParams};
