//! What asking for a helper came to.

use crate::capability::{Capability, Executable, PackageName};

/// What a request that found nothing was about, for the sheet's words: the capability asked for,
/// the first candidate package for this distro (what to look for in a software centre) and the
/// first probe program (what a package must provide). Either is `None` when the file gives none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Missing {
    /// The capability that was asked for.
    pub capability: Capability,
    /// The first candidate package for this distro.
    pub package: Option<PackageName>,
    /// The first program that proves the tool is there.
    pub program: Option<Executable>,
}

/// How a request to provide a capability ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The tool is on the machine now (or already was).
    Installed,
    /// The person, or the system's policy, said no: the password prompt was cancelled.
    Declined,
    /// No candidate package exists in the distro's repositories.
    NotFound(Missing),
    /// Nothing here can install it: an unknown distro, no entry for the distro, or no
    /// PackageKit.
    Unsupported(Missing),
    /// Something went wrong; the reason is the package manager's own words.
    Failed(String),
}
