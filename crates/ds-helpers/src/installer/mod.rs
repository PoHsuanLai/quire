//! The seam to whatever installs packages: [`Install`], with the real PackageKit backend and a
//! scripted one for tests and the gallery. The backend is a trait object so the handle that holds
//! it, [`crate::Helpers`], needs no type parameter: an app picks the backend once, where it builds
//! the handle, and everything that holds the handle is the same type either way.

mod fake;
// The D-Bus backend, or its stand-in that reports every install `Unsupported`.
#[cfg_attr(feature = "quire-desktop", path = "packagekit.rs")]
#[cfg_attr(not(feature = "quire-desktop"), path = "packagekit_absent.rs")]
mod packagekit;

#[cfg(feature = "quire-desktop")]
mod transaction;

pub use fake::{FakeInstaller, StandIn};
pub use packagekit::PackageKit;

use crate::capability::PackageName;
use crate::family::Family;
use crate::outcome::{Missing, Outcome};
use ds_peer::Deadline;
use std::future::Future;
use std::pin::Pin;

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

/// An installer backend: whatever installs a package for a [`Request`].
///
/// The future is boxed so a `dyn Install` can sit in [`crate::Helpers`].
pub trait Install: std::fmt::Debug + Send + Sync {
    /// Install the first candidate that exists. [`Outcome::Installed`] means the package is in
    /// place; the caller still probes for the tool.
    fn install<'a>(&'a self, request: &'a Request) -> BoxFuture<'a, Outcome>;
}

/// A boxed future that may run on another thread.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// How long each wait on the package manager may take before it counts as not answering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Timeouts {
    /// The quick calls, and a lookup of a package's name with its answer.
    pub step: Deadline,
    /// An installation: the password prompt, the download and the install. Long, because the
    /// person may be typing and a download may be slow; a package manager that vanishes ends
    /// the wait at once, whatever this says.
    pub install: Deadline,
}

impl Timeouts {
    /// Timeouts of `step` for the quick calls and `install` for an installation.
    pub const fn new(step: Deadline, install: Deadline) -> Timeouts {
        Timeouts { step, install }
    }
}

impl Default for Timeouts {
    fn default() -> Timeouts {
        Timeouts {
            step: Deadline::ACTION,
            install: Deadline::PROMPT,
        }
    }
}
