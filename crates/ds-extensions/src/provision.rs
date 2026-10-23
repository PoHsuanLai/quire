//! The entries of the words this crate owns, as one bayonet `Provides` type.

use std::path::Path;

use bayonet::manifest::{Entry, PathRole, Provides, Refusal};
use ds_core::word::Word;
use serde::Deserialize;

use crate::capability::Capability;
use crate::fault::Fault;
use crate::file_icons::FileIcons;
use crate::look::Look;

/// One `[[provides]]` entry of a word this crate owns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provision {
    /// `file_icons`.
    FileIcons(FileIcons),
    /// `look`.
    Look(Look),
}

/// The word of an entry, read before the rest.
#[derive(Debug, Deserialize)]
struct Head {
    capability: String,
}

impl Provision {
    /// The look, if this entry is one.
    pub fn look(&self) -> Option<&Look> {
        match self {
            Provision::Look(look) => Some(look),
            Provision::FileIcons(_) => None,
        }
    }

    /// The file icons, if this entry is that.
    pub fn file_icons(&self) -> Option<&FileIcons> {
        match self {
            Provision::FileIcons(icons) => Some(icons),
            Provision::Look(_) => None,
        }
    }

    /// The first problem with the files this entry names that bayonet's own checks do not make
    /// (a size over its cap, a folder that is not one).
    pub(crate) fn fault(&self) -> Option<Fault> {
        match self {
            Provision::FileIcons(icons) => icons.fault(),
            Provision::Look(look) => look.fault(),
        }
    }
}

impl Provides for Provision {
    type Capability = Capability;
    type Fault = Fault;

    fn parse(entry: Entry) -> Result<Provision, Refusal<Fault>> {
        let head = entry.clone().read::<Head, Fault>()?;
        match Capability::parse(&head.capability) {
            Some(Capability::FileIcons) => FileIcons::from_entry(entry).map(Provision::FileIcons),
            Some(Capability::Look) => Look::from_entry(entry).map(Provision::Look),
            None => Err(Refusal::Fault(Fault::NotOwned {
                word: head.capability,
            })),
        }
    }

    fn capability(&self) -> Capability {
        match self {
            Provision::FileIcons(_) => Capability::FileIcons,
            Provision::Look(_) => Capability::Look,
        }
    }

    /// Only the words of [`Capability`]: every other word belongs to another owner, who reads
    /// it from the raw entries of the scan.
    fn owns(capability: &str) -> bool {
        Capability::parse(capability).is_some()
    }

    /// Data words carry no program.
    fn needs_program(&self) -> bool {
        false
    }

    fn paths(&self) -> Vec<(&Path, PathRole)> {
        match self {
            Provision::Look(look) => vec![(look.stylesheet(), PathRole::Library)],
            Provision::FileIcons(_) => Vec::new(),
        }
    }
}
