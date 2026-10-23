//! Finding Extensions: the folder scan, unranked, with the turned-off ones set apart.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use bayonet::manifest::Raw;
use bayonet::registry::{Candidate, Origin};
use bayonet::{Layout, Rejected};
use ds_core::settings::ExtensionId;
use ds_settings::xdg;

/// The folder under a data directory where Extensions live: `<data>/quire/extensions/<id>/`.
const APP: &str = "quire";
const FOLDER: &str = "extensions";
const MANIFEST: &str = "extension.toml";

/// The newest entry-format version this crate understands. For the data capabilities the
/// manifest's `protocol` is this number; an Extension that asks for a newer one is set aside.
pub const PROTOCOL: u32 = 1;

/// Where to look. The roots are values: a program passes the environment's through
/// [`Search::from_env`], and a test or a portable install passes its own.
///
/// A root is a data directory, the shape of `$XDG_DATA_HOME`: Extensions are in
/// `<root>/quire/extensions/<id>/extension.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Search {
    person: Option<PathBuf>,
    system: Vec<PathBuf>,
}

impl Search {
    /// A search of `person`'s data directory and each of the `system` ones, most important first.
    pub fn new(person: &Path, system: &[PathBuf]) -> Search {
        Search {
            person: Some(person.to_path_buf()),
            system: system.to_vec(),
        }
    }

    /// A search over the XDG variables as a program has them: `$XDG_DATA_HOME` (else
    /// `$HOME/.local/share`) and `$XDG_DATA_DIRS`.
    pub fn from_env() -> Search {
        Search::from_dirs(
            std::env::var_os("XDG_DATA_HOME").as_deref(),
            std::env::var_os("XDG_DATA_DIRS").as_deref(),
            std::env::var_os("HOME").as_deref(),
        )
    }

    /// [`Search::from_env`] with the variables handed in, under the XDG rules (an empty or
    /// relative value is ignored). A person with no home has no personal root.
    pub fn from_dirs(
        xdg_data_home: Option<&OsStr>,
        xdg_data_dirs: Option<&OsStr>,
        home: Option<&OsStr>,
    ) -> Search {
        Search {
            person: xdg::base_dir(xdg_data_home, home, xdg::DATA_HOME_UNDER_HOME),
            system: xdg::search_dirs(xdg_data_dirs),
        }
    }

    /// Reads every Extension manifest under the roots and returns what it found, unranked: every
    /// copy of every id, with where it came from. `off` are the ids turned off
    /// ([`ExtensionsFile::off_ids`](crate::ExtensionsFile::off_ids)); they come back apart, in
    /// [`Found::off`], with their names.
    ///
    /// Nothing here fails: a manifest that cannot be read is in [`Found::rejected`].
    pub fn scan(&self, off: &[ExtensionId]) -> Found {
        let roots = self
            .person
            .iter()
            .map(|root| (Origin::User, root))
            .chain(self.system.iter().map(|root| (Origin::System, root)));
        let mut candidates = Vec::new();
        let mut rejected = Vec::new();
        for (origin, root) in roots {
            let (found, refused) = scan_root(root, origin);
            candidates.extend(found);
            rejected.extend(refused);
        }
        let (candidates, off) = set_apart(candidates, off);
        Found {
            candidates,
            rejected,
            off,
        }
    }
}

/// The manifests of one root, marked as coming from `origin`.
fn scan_root(root: &Path, origin: Origin) -> (Vec<Candidate<Raw>>, Vec<Rejected<Raw>>) {
    let layout = Layout::Folders {
        dir: FOLDER,
        manifest: MANIFEST,
    };
    // One root at a time, so each keeps its own origin and the order of the roots stays the order
    // of the candidates (bayonet's scan marks the first root it is given as the person's).
    let search = bayonet::Search::new(APP, root, &[], PROTOCOL).with_layout(layout);
    let scan = bayonet::scan::<Raw>(&search);
    let candidates = scan
        .candidates
        .into_iter()
        .map(|mut candidate| {
            candidate.origin = origin;
            candidate
        })
        .collect();
    (candidates, scan.rejected)
}

/// What a scan found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// Every manifest of an Extension that is on, with where it was found and its entries kept
    /// raw. Two copies of one id are both here, and nothing is ranked: the owner of a word
    /// checks its entries first, since only the owner knows what is usable, and ranks after
    /// ([`Found::resolve`] for this crate's words; `bayonet::reparse` and
    /// `bayonet::registry::Registry::resolve` for another owner's).
    pub candidates: Vec<Candidate<Raw>>,
    /// Manifest files that could not be read, with why.
    pub rejected: Vec<Rejected<Raw>>,
    /// The Extensions that are installed and turned off, by id. A host that finds nothing serving
    /// a request asks these before it says "missing": an Extension that serves it may be off.
    pub off: Vec<Off>,
}

/// An installed Extension that is turned off.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Off {
    /// Its id.
    pub id: ExtensionId,
    /// What a person calls it.
    pub name: String,
    /// Every copy of it that was found, so a host can read its entries.
    pub candidates: Vec<Candidate<Raw>>,
}

impl Off {
    /// Whether the Extension has an entry for the capability word `word`.
    pub fn serves(&self, word: &str) -> bool {
        self.candidates
            .iter()
            .any(|candidate| names(candidate, word))
    }
}

impl Found {
    /// The turned-off Extensions that have an entry for the capability word `word`: what
    /// "Turn On…" would offer instead of "missing".
    pub fn off_serving<'a>(&'a self, word: &'a str) -> impl Iterator<Item = &'a Off> {
        self.off.iter().filter(move |off| off.serves(word))
    }
}

/// Whether `candidate` has an entry for the capability word `word`.
pub(crate) fn names(candidate: &Candidate<Raw>, word: &str) -> bool {
    candidate
        .manifest
        .provides
        .iter()
        .any(|entry| entry.word() == word)
}

/// `candidates` without the ones whose id is in `off`, and those as [`Off`] entries by id.
fn set_apart(
    candidates: Vec<Candidate<Raw>>,
    off: &[ExtensionId],
) -> (Vec<Candidate<Raw>>, Vec<Off>) {
    let mut on = Vec::new();
    let mut apart: BTreeMap<String, Off> = BTreeMap::new();
    for candidate in candidates {
        let id = candidate.manifest.id.as_str().to_owned();
        if !off.iter().any(|listed| listed.0 == id) {
            on.push(candidate);
            continue;
        }
        let name = candidate.manifest.name.clone();
        apart
            .entry(id.clone())
            .or_insert_with(|| Off {
                id: ExtensionId(id),
                name,
                candidates: Vec::new(),
            })
            .candidates
            .push(candidate);
    }
    (on, apart.into_values().collect())
}
