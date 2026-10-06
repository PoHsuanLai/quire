//! What asking for a helper came to.

/// How a request to provide a capability ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The tool is on the machine now (or already was).
    Installed,
    /// The person, or the system's policy, said no: the password prompt was cancelled.
    Declined,
    /// No candidate package exists in the distro's repositories.
    NotFound,
    /// Nothing here can install it: an unknown distro, no entry for the distro, or no
    /// PackageKit.
    Unsupported,
    /// Something went wrong; the reason is the package manager's own words.
    Failed(String),
}
