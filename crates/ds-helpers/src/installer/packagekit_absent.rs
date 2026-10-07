//! The PackageKit backend's stand-in for a build without `quire-desktop`: no D-Bus client, so
//! every install is [`Outcome::Unsupported`], the answer the app already handles.

use super::Request;
use crate::outcome::Outcome;

/// The PackageKit backend, absent: it installs nothing.
#[derive(Debug)]
pub struct PackageKit;

impl PackageKit {
    /// The system's PackageKit; without `quire-desktop` there is none to reach.
    pub fn system() -> PackageKit {
        PackageKit
    }

    pub(super) async fn install(&self, request: &Request) -> Outcome {
        Outcome::Unsupported(request.missing.clone())
    }
}
