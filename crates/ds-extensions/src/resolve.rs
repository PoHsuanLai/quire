//! From a scan to the Extensions that can be used for one of this crate's words.
//!
//! Ranking happens after the owner's checks, because only the owner knows what is usable
//! (design: Extensions, section 4.3). Here the owner is this crate: it reads the raw entries of
//! the capability asked for, checks the files they name, and lets bayonet's registry set the
//! unusable ones aside and rank the rest (protocol, then the person's copy over the system's,
//! then the newer version, then the id).

use bayonet::manifest::ManifestError;
use bayonet::registry::{Candidate, Origin, Readiness, Registry, Unusable};
use bayonet::{Reparsed, reparse};
use ds_core::settings::ExtensionId;
use ds_core::word::Word;

use crate::capability::Capability;
use crate::fault::Fault;
use crate::provision::Provision;
use crate::search::{Found, PROTOCOL, names};

/// The Extensions that can serve one capability, and the ones that cannot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    capability: Capability,
    /// The usable Extensions in serving order, and the ones bayonet set aside after the checks
    /// ([`Registry::unusable`]): a missing file, a path out of the folder, a sheet over its cap,
    /// a newer entry format.
    pub registry: Registry<Provision>,
    /// Extensions with an entry for the capability that could not be read at all (a key the word
    /// does not have, a path that climbs out of the folder).
    pub unreadable: Vec<Unusable<Provision>>,
}

/// Why an Extension cannot be used, in the terms Settings explains it in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Trouble {
    /// A file it names is missing, too big, out of its folder or not what it should be.
    FilesUnreadable,
    /// It was written for a newer desktop.
    NeedsNewer,
}

/// An Extension listed in Settings with a reason it is not in use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// The Extension.
    pub id: ExtensionId,
    /// Whose copy it is: the person's own or the system's.
    pub origin: Origin,
    /// What is wrong.
    pub trouble: Trouble,
    capability: Capability,
}

impl Problem {
    /// The line Settings shows under the Extension, in plain words.
    pub fn words(&self) -> &'static str {
        match (self.trouble, self.capability) {
            (Trouble::NeedsNewer, _) => "This Extension needs a newer version of the desktop",
            (Trouble::FilesUnreadable, Capability::FileIcons) => {
                "Some pictures in this Extension could not be read"
            }
            (Trouble::FilesUnreadable, Capability::Look) => {
                "Some files in this Extension could not be read"
            }
        }
    }
}

impl Found {
    /// The Extensions that are on and have an entry for `capability`, checked and ranked. An
    /// Extension with an entry of the capability that cannot be used is in the result's
    /// problems, never half-used. Entries of words other owners have are left alone.
    pub fn resolve(&self, capability: Capability) -> Resolved {
        let mut candidates = Vec::new();
        let mut unreadable = Vec::new();
        for raw in self
            .candidates
            .iter()
            .filter(|raw| names(raw, capability.slug()))
        {
            match reparse::<Provision>(raw.clone()) {
                Ok(Reparsed {
                    candidate: Some(candidate),
                    ..
                }) => candidates.push(checked(candidate)),
                Ok(Reparsed {
                    candidate: None, ..
                }) => {}
                Err(reason) => unreadable.push(Unusable {
                    id: raw.manifest.id.clone(),
                    origin: raw.origin,
                    reason,
                }),
            }
        }
        Resolved {
            capability,
            registry: Registry::resolve(candidates, PROTOCOL),
            unreadable,
        }
    }
}

/// `candidate`, set aside if a file it names is not usable by this crate's checks (bayonet has
/// already made its own: the file is there and stays in the folder).
fn checked(mut candidate: Candidate<Provision>) -> Candidate<Provision> {
    if matches!(candidate.readiness, Readiness::Unready(_)) {
        return candidate;
    }
    if let Some(fault) = candidate
        .manifest
        .provides
        .iter()
        .find_map(Provision::fault)
    {
        candidate.readiness = Readiness::Unready(ManifestError::Provision(fault));
    }
    candidate
}

impl Resolved {
    /// The capability these were resolved for.
    pub fn capability(&self) -> Capability {
        self.capability
    }

    /// Every Extension that is installed for the capability and not usable, with the reason in
    /// plain words ([`Problem::words`]), by id.
    pub fn problems(&self) -> Vec<Problem> {
        let mut problems: Vec<Problem> = self
            .registry
            .unusable()
            .iter()
            .chain(self.unreadable.iter())
            .map(|unusable| Problem {
                id: ExtensionId(unusable.id.as_str().to_owned()),
                origin: unusable.origin,
                trouble: trouble_of(&unusable.reason),
                capability: self.capability,
            })
            .collect();
        problems.sort_by(|a, b| a.id.cmp(&b.id));
        problems
    }
}

fn trouble_of(reason: &ManifestError<Capability, Fault>) -> Trouble {
    if matches!(reason, ManifestError::ProtocolUnsupported { .. }) {
        Trouble::NeedsNewer
    } else {
        Trouble::FilesUnreadable
    }
}
