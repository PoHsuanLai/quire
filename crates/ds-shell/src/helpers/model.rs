//! What the sheet shows, as plain data the host maps its install state to.

/// Where a helper request stands. The host owns it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum HelperPhase {
    /// The question: install it now, or not.
    #[default]
    Ask,
    /// The system is installing it (it may ask for the password itself). Nothing to press.
    Installing,
    /// The install failed; `reason` is the package manager's own words.
    Failed {
        /// What went wrong.
        reason: String,
    },
    /// The software sources have no package for it; `package` is what to look for.
    NotFound {
        /// The package name to look for in a software centre.
        package: String,
    },
    /// This system cannot install it from here; `program` is what the person needs on the path.
    Unsupported {
        /// The program a package must provide.
        program: String,
    },
}
