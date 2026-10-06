//! The seam to whatever installs packages: the real PackageKit backend, and a scripted one for
//! tests and the gallery. An enum, not a trait object, so no async-trait is needed.

mod fake;
mod packagekit;

pub use fake::FakeInstaller;
pub use packagekit::PackageKit;

use crate::capability::PackageName;
use crate::family::Family;
use crate::outcome::Outcome;

/// What to install: the alternatives for one tool, best first. The first that exists wins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// The distro family the names are for.
    pub family: Family,
    /// The package alternatives in preference order.
    pub candidates: Vec<PackageName>,
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
