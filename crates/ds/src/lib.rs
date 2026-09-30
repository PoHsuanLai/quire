//! quire's renderer-free design system: the host seams, the overlay stack, the root, the generic
//! components, and the assembly of the stylesheet and `Ds`. It sits over three crates (`ds-core`,
//! `ds-style`, `ds-motion`); its tests lint with `ds-lint`. `ds-shell` builds the shell's own parts
//! on top of this crate. Inside the crate a layer names only the layers below it
//! (`scripts/check-boundary.sh`). A consumer writes `use ds::prelude::*` for the names it draws
//! with and names everything else by its home path; the root holds only `prelude` and the
//! stylesheet assembly (`kits`, `stylesheet`, `component_sheets`, `KIT`, `selectors`). DESIGN.md
//! maps each module to the design doc section it implements.

pub mod assembly;
pub mod components;
pub mod detail;
pub mod edit;
pub mod file_drop;
pub mod focus;
pub mod host;
pub mod icon;
pub mod motion;
pub mod root;
pub mod spell;
pub mod stack;
pub mod time;
pub mod window;

pub mod prelude;

// The curated roots: the stylesheet assembly a host installs. Every other name is in
// `prelude` or at its home path.
pub use crate::assembly::selectors;
pub use crate::assembly::{
    kit::{KIT, kits},
    stylesheet::{component_sheets, stylesheet},
};
