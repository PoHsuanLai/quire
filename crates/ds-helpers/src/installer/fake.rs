//! A scripted installer: answers what it was told, records what it was asked, and may drop
//! executables into a directory the caller gave it (a scratch `PATH` entry) when it "installs".

use super::Request;
use crate::outcome::Outcome;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

/// Where, and which, executables a successful fake install leaves behind.
#[derive(Debug, Clone)]
struct Drop {
    dir: PathBuf,
    names: Vec<String>,
}

/// An installer that never touches a package manager. Clones share the record of requests, so a
/// test keeps one and hands the other to [`crate::Helpers`].
#[derive(Debug, Clone)]
pub struct FakeInstaller {
    outcome: Outcome,
    drop: Option<Drop>,
    asked: Arc<Mutex<Vec<Request>>>,
}

impl FakeInstaller {
    /// One that answers `outcome` to every request.
    pub fn new(outcome: Outcome) -> FakeInstaller {
        FakeInstaller {
            outcome,
            drop: None,
            asked: Arc::default(),
        }
    }

    /// On [`Outcome::Installed`], also create these executables in `dir`.
    pub fn leaving(mut self, dir: PathBuf, names: &[&str]) -> FakeInstaller {
        self.drop = Some(Drop {
            dir,
            names: names.iter().map(|name| (*name).to_owned()).collect(),
        });
        self
    }

    /// Every request received so far.
    pub fn asked(&self) -> Vec<Request> {
        self.asked
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub(super) fn install(&self, request: &Request) -> Outcome {
        self.asked
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(request.clone());
        match (&self.outcome, &self.drop) {
            (Outcome::Installed, Some(drop)) => match leave(drop) {
                Ok(()) => Outcome::Installed,
                Err(error) => Outcome::Failed(error.to_string()),
            },
            (outcome, _) => outcome.clone(),
        }
    }
}

fn leave(drop: &Drop) -> std::io::Result<()> {
    std::fs::create_dir_all(&drop.dir)?;
    drop.names.iter().try_for_each(|name| {
        let file = drop.dir.join(name);
        std::fs::write(&file, "#!/bin/sh\n")?;
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755))
    })
}
