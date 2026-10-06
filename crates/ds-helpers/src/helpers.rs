//! The app-facing handle: probe, ask, and hear about changes.

use crate::capability::Capability;
use crate::catalog::{Catalog, Entry};
use crate::family::Family;
use crate::installer::{Installer, Request};
use crate::outcome::Outcome;
use crate::probe::{Environment, Presence, presence};
use crate::watch::{Log, Subscription};
use std::sync::Arc;

#[derive(Debug)]
struct Inner {
    catalog: Catalog,
    environment: Environment,
    family: Option<Family>,
    installer: Installer,
    log: Log,
}

/// An app's missing-helpers handle. Cheap to clone; clones share one availability feed.
#[derive(Debug, Clone)]
pub struct Helpers(Arc<Inner>);

impl Helpers {
    /// A handle over `catalog`, probing `environment` and installing through `installer`. The
    /// distro family is read from `environment.os_release` once, here.
    pub fn new(catalog: Catalog, environment: Environment, installer: Installer) -> Helpers {
        let family = std::fs::read_to_string(&environment.os_release)
            .ok()
            .and_then(|text| Family::detect(&text));
        Helpers(Arc::new(Inner {
            catalog,
            environment,
            family,
            installer,
            log: Log::default(),
        }))
    }

    /// What the file says about `capability`: the tool's name and purpose, for the sheet.
    pub fn entry(&self, capability: &Capability) -> Option<&Entry> {
        self.0.catalog.get(capability)
    }

    /// The distro family of this machine, `None` when unknown.
    pub fn family(&self) -> Option<Family> {
        self.0.family
    }

    /// Is the tool there now? An undeclared capability is [`Presence::Missing`].
    pub fn available(&self, capability: &Capability) -> Presence {
        self.entry(capability).map_or(Presence::Missing, |entry| {
            presence(&self.0.environment.path, entry)
        })
    }

    /// Probe again and announce the result to subscribers if it changed (a tool the person
    /// installed some other way, or removed).
    pub fn refresh(&self, capability: &Capability) -> Presence {
        let now = self.available(capability);
        self.0.log.record(capability, now);
        now
    }

    /// Changes in availability from now on.
    pub fn subscribe(&self) -> Subscription {
        self.0.log.subscribe()
    }

    /// Ask for the tool: nothing happens when it is already there; otherwise the installer
    /// is asked for the first candidate package that exists. After an install the tool is
    /// probed again, and subscribers hear of it. Call it when the person tries to use the
    /// feature, after they have said yes to the sheet, never at launch.
    pub async fn provide(&self, capability: &Capability) -> Outcome {
        let Some(entry) = self.entry(capability) else {
            return Outcome::Unsupported;
        };
        if self.refresh(capability) == Presence::Present {
            return Outcome::Installed;
        }
        let candidates = self.0.family.map(|f| (f, entry.candidates(f)));
        let Some((family, candidates)) = candidates.filter(|(_, c)| !c.is_empty()) else {
            return Outcome::Unsupported;
        };
        let request = Request {
            family,
            candidates: candidates.to_vec(),
        };
        match self.0.installer.install(&request).await {
            Outcome::Installed => match self.refresh(capability) {
                Presence::Present => Outcome::Installed,
                Presence::Missing => Outcome::Failed(format!(
                    "{} was installed, but it is still not on your PATH.",
                    entry.tool
                )),
            },
            other => other,
        }
    }
}
