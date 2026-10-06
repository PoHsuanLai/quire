//! The missing-helper sheet (design/27 permission-prompt rules): an app tried to use something
//! that needs a distro tool which is not installed, and asks, at that moment and never at launch,
//! whether to install it. "Photos needs mpv to play videos." with Not Now and Install...
//!
//! Controlled: the host owns the [`model::HelperPhase`] and moves it as `ds-helpers` reports
//! (Ask, then Installing while `Helpers::provide` runs, then nothing on success or one of the
//! end phases). No `ds-helpers` type is named here; a host maps its outcome to a phase.

pub mod model;
pub(crate) mod sheet;
pub(crate) mod wording;
