//! Why this crate refuses an entry it owns: bayonet's `ManifestError::Provision` payload.

use std::path::PathBuf;

/// An entry of an owned word that is well formed and still not usable.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum Fault {
    /// The entry names a word this crate does not own. Reached only by parsing an entry
    /// directly; a scan hands over just the owned words ([`Provides::owns`]).
    ///
    /// [`Provides::owns`]: bayonet::manifest::Provides::owns
    #[error("{word:?} is not a capability this crate owns")]
    NotOwned {
        /// The word as written.
        word: String,
    },
    /// A look's stylesheet is not a `.css` file.
    #[error("the stylesheet {path:?} is not a .css file")]
    NotCss {
        /// The path as written.
        path: PathBuf,
    },
    /// A file is larger than the cap for its kind.
    #[error("{path:?} is larger than {cap} bytes")]
    TooBig {
        /// The file.
        path: PathBuf,
        /// The cap it passed.
        cap: u64,
    },
    /// A path that must be a folder is not one.
    #[error("{path:?} is not a folder")]
    NotAFolder {
        /// The path.
        path: PathBuf,
    },
    /// A path leaves the Extension's folder through a link.
    #[error("{path:?} leads out of the Extension's folder")]
    LeadsOut {
        /// The path.
        path: PathBuf,
    },
}
