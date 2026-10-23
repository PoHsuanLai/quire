//! Choosing among Extensions: the list a menu shows, and what a stored choice means once the
//! installed Extensions are known (design: Extensions, section 5).
//!
//! A `Provided` settings key stores `standard` or an Extension id and lists no choices
//! (`ds_settings::schema::KeyKind::Provided`). The caller that draws it builds [`Choices`] from the
//! usable Extensions of the key's capability and fills the key with [`Choices::fill`]; the program
//! that reads it calls [`chosen`].

use bayonet::manifest::Provides;
use bayonet::registry::Installed;
use ds_core::settings::ExtensionId;
use ds_settings::InvalidKey;
use ds_settings::schema::{KeyPath, KeySpec, Provided, STANDARD, WordLabels};

use crate::capability::Capability;
use crate::provision::Provision;
use crate::your_changes::YOUR_CHANGES_ID;

/// What a person calls the built-in choice.
const STANDARD_LABEL: &str = "Standard";

/// The choices for one capability: `standard` first, then each usable Extension that provides it,
/// in serving order, with the names a menu shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choices {
    words: Vec<String>,
    labels: WordLabels,
}

/// The choices for `capability` among `installed`, the usable Extensions in serving order
/// ([`Resolved::registry`](crate::Resolved)`.installed()`). Pure: nothing is read. "Your changes"
/// ([`YOUR_CHANGES_ID`]) is never on offer: it is always applied, on top of the choice.
pub fn choices_for(capability: Capability, installed: &[Installed<Provision>]) -> Choices {
    let mut words = vec![STANDARD.to_owned()];
    let mut labels = vec![(STANDARD.to_owned(), STANDARD_LABEL.to_owned())];
    for extension in installed
        .iter()
        .filter(|extension| extension.manifest.id.as_str() != YOUR_CHANGES_ID)
    {
        let provides = extension
            .manifest
            .provides
            .iter()
            .any(|provision| provision.capability() == capability);
        if provides {
            let id = extension.manifest.id.as_str().to_owned();
            labels.push((id.clone(), extension.manifest.name.clone()));
            words.push(id);
        }
    }
    Choices {
        words,
        labels: WordLabels(labels.into_iter().collect()),
    }
}

impl Choices {
    /// The stored words, `standard` first.
    pub fn words(&self) -> &[String] {
        &self.words
    }

    /// The label of each word: "Standard", and each Extension's name.
    pub fn labels(&self) -> &WordLabels {
        &self.labels
    }

    /// Whether `id` is one of the Extensions on offer.
    pub fn offers(&self, id: &str) -> bool {
        self.words.iter().skip(1).any(|word| word == id)
    }

    /// `key` with these choices in it, drawn by the `menu` control (`KeySpec::offering`).
    pub fn fill(&self, key: &KeySpec) -> KeySpec {
        key.offering(self.words.clone(), self.labels.clone())
    }
}

/// A stored choice that names an Extension that is not installed, or is turned off.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stale {
    /// The id as stored.
    pub stored: String,
}

impl Stale {
    /// The report for `Loaded`'s diagnostics: the key, and that Standard is used instead.
    pub fn invalid_key(&self, key: &KeyPath) -> InvalidKey {
        InvalidKey {
            path: key.0.clone(),
            reason: format!(
                "{:?} is not installed or is turned off, so Standard is used",
                self.stored
            ),
        }
    }
}

/// What a stored choice means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chosen {
    /// The Extension to use, or `None` for the standard one.
    pub id: Option<ExtensionId>,
    /// Set when the stored id could not be used and `standard` was read instead. Report it once,
    /// when the file is loaded, not on every read.
    pub stale: Option<Stale>,
}

/// The choice `stored` means among `choices`: an id that is on offer is used; `standard` is the
/// standard one; any other id reads as `standard` and is returned as [`Stale`].
pub fn chosen(stored: &Provided, choices: &Choices) -> Chosen {
    match stored.chosen() {
        None => Chosen {
            id: None,
            stale: None,
        },
        Some(id) if choices.offers(id) => Chosen {
            id: Some(ExtensionId(id.to_owned())),
            stale: None,
        },
        Some(id) => Chosen {
            id: None,
            stale: Some(Stale {
                stored: id.to_owned(),
            }),
        },
    }
}
