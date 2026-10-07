//! A scripted installer: answers what it was told, records what it was asked, and may drop
//! executables into a directory the caller gave it (a scratch `PATH` entry) when it "installs".

use super::Request;
use crate::outcome::Outcome;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

/// A stand-in tool: the executable's file name and the script text written into it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandIn {
    /// The executable's file name.
    pub name: String,
    /// What the file holds; start it with a `#!` line to make it runnable.
    pub body: String,
}

/// Where, and which, executables a successful fake install leaves behind.
#[derive(Debug, Clone)]
struct Drop {
    dir: PathBuf,
    tools: Vec<StandIn>,
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
    /// One that answers `outcome` to every request. The `Missing` inside a `NotFound` or
    /// `Unsupported` is replaced by the request's own, as a real backend would give it.
    pub fn new(outcome: Outcome) -> FakeInstaller {
        FakeInstaller {
            outcome,
            drop: None,
            asked: Arc::default(),
        }
    }

    /// On [`Outcome::Installed`], also create these executables in `dir`.
    pub fn leaving(self, dir: PathBuf, names: &[&str]) -> FakeInstaller {
        let empty = |name: &&str| StandIn {
            name: (*name).to_owned(),
            body: "#!/bin/sh\n".to_owned(),
        };
        self.leaving_with(dir, names.iter().map(empty).collect())
    }

    /// On [`Outcome::Installed`], also create these stand-in tools in `dir`, each with its own
    /// script text.
    pub fn leaving_with(mut self, dir: PathBuf, tools: Vec<StandIn>) -> FakeInstaller {
        self.drop = Some(Drop { dir, tools });
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
            (Outcome::NotFound(_), _) => Outcome::NotFound(request.missing.clone()),
            (Outcome::Unsupported(_), _) => Outcome::Unsupported(request.missing.clone()),
            (outcome, _) => outcome.clone(),
        }
    }
}

fn leave(drop: &Drop) -> std::io::Result<()> {
    std::fs::create_dir_all(&drop.dir)?;
    drop.tools.iter().try_for_each(|tool| {
        let file = drop.dir.join(&tool.name);
        std::fs::write(&file, &tool.body)?;
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755))
    })
}
