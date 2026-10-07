//! The debug probe (feature `debug-probe`): a window's controls by role and accessible name, with
//! their boxes, written to a file after each frame, so a scenario outside the process presses a
//! control by what it is called and never by where it was last drawn.
//!
//! Off unless the feature is built in and `QUIRE_DEBUG_PROBE=<directory>` is set in the app's
//! environment. Then each window writes `<directory>/<pid>-<n>.tsv` (n counts the process's
//! windows from 0) whenever what it shows changes, one control per line, tab separated:
//!
//! ```text
//! role  x  y  w  h  state  name
//! ```
//!
//! The box is the border box in the window's logical pixels from its top-left corner, `state` is
//! `-` or space-separated words (`checked`, `pressed`, `selected`, `expanded`, `disabled`), and
//! `name` is `aria-label`, else an image's `alt`, else the element's visible text. The roles are
//! the ARIA role attribute, else the one the element implies (`button`, `link`, `textbox`,
//! `checkbox`, `radio`, `combobox`, `heading`, `option`, `img`).
//!
//! A file, not a D-Bus object: the document lives on the UI thread, and a file asks nothing of
//! the core (no zbus, no thread), so the portable build is the same with the feature on.

#[cfg(feature = "debug-probe")]
mod name;
#[cfg(feature = "debug-probe")]
mod on;
#[cfg(feature = "debug-probe")]
mod role;
#[cfg(feature = "debug-probe")]
mod snapshot;

#[cfg(feature = "debug-probe")]
pub(crate) use on::Probe;
#[cfg(feature = "debug-probe")]
pub use snapshot::snapshot;

#[cfg(not(feature = "debug-probe"))]
mod off;
#[cfg(not(feature = "debug-probe"))]
pub(crate) use off::Probe;
