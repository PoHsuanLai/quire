//! The seam to whatever installs packages: the real PackageKit backend, and a scripted one for
//! tests and the gallery. An enum, not a trait object, so no async-trait is needed.

mod fake;
// The D-Bus backend, or its stand-in that reports every install `Unsupported`.
#[cfg_attr(feature = "quire-desktop", path = "packagekit.rs")]
#[cfg_attr(not(feature = "quire-desktop"), path = "packagekit_absent.rs")]
mod packagekit;

pub use fake::{FakeInstaller, StandIn};
pub use packagekit::PackageKit;

use crate::capability::PackageName;
use crate::family::Family;
use crate::outcome::{Missing, Outcome};

/// What to install: the alternatives for one tool, best first. The first that exists wins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// The distro family the names are for.
    pub family: Family,
    /// The package alternatives in preference order.
    pub candidates: Vec<PackageName>,
    /// What the request is about, handed back in [`Outcome::NotFound`] and
    /// [`Outcome::Unsupported`].
    pub missing: Missing,
}

/// An installer backend.
#[derive(Debug)]
pub enum Installer {
    /// The system's PackageKit daemon, which asks polkit for the password itself.
    PackageKit(PackageKit),
    /// A scripted installer.
    Fake(FakeInstaller),
}

impl Installer {
    /// Install the first candidate that exists. [`Outcome::Installed`] means the package is in
    /// place; the caller still probes for the tool.
    pub async fn install(&self, request: &Request) -> Outcome {
        match self {
            Installer::PackageKit(backend) => backend.install(request).await,
            Installer::Fake(backend) => backend.install(request),
        }
    }
}
