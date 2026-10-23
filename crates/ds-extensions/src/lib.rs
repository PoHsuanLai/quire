//! Extensions: things a person adds to the desktop, found in folders and described by one
//! manifest each. This crate is the folder scan, the envelope checks, the on and off state, and
//! the data capabilities (a look, file icons). It is a library with no UI, no async runtime and
//! no bus, so sill, files, mailo, anyview, detent and casement can all use it.
//!
//! # Finding them
//!
//! An Extension is `<data>/quire/extensions/<id>/extension.toml` beside the files it names, in
//! the person's data directory and in each system one. [`Search`] takes those roots as values
//! ([`Search::new`]; [`Search::from_env`] reads the XDG variables for a program), so a test or a
//! portable install passes its own. [`Search::scan`] returns every copy of every id **unranked**
//! ([`Found`]), with the turned-off ids apart and named ([`Off`]), so a host can say "off"
//! where it would have said "missing".
//!
//! Ranking waits for the owner of the word, because only the owner knows what is usable: a
//! missing program, a newer protocol, a file out of the folder. For this crate's words that is
//! [`Found::resolve`]. Unusable Extensions are set aside first; the rest are ordered by protocol,
//! then the person's copy over the system's, then the newer version, then the id (bayonet's
//! registry does it). An Extension that fails is listed with a reason in plain words
//! ([`Problem::words`]), and is never half-used.
//!
//! # Whose word is whose
//!
//! Each capability word has one owner, the crate that parses its entries. This crate owns the
//! data words in [`Capability`] ([`Look`], [`IconFolder`]; the rest of the list is later). Other
//! owners (anyview-plugin owns `probe`, `peek`, `thumbnail`, `decode`, `export` and `play`)
//! read their own words from the same scan: the entries a scan keeps are raw, and the owner turns
//! the candidates into its own type with `bayonet::reparse`, which sets aside the words it does
//! not own instead of failing on them. Then it asks its own checks and ranks with
//! `bayonet::registry::Registry::resolve`.
//!
//! ```
//! use std::path::Path;
//!
//! use ds_extensions::Search;
//! use ds_extensions::bayonet::manifest::{Entry, Provides, Refusal};
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! enum Word {
//!     Thumbnail,
//! }
//!
//! // The owner's own type: it owns `thumbnail` and defines what its entry says.
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! struct Thumbnail;
//!
//! impl Provides for Thumbnail {
//!     type Capability = Word;
//!     type Fault = std::convert::Infallible;
//!
//!     fn parse(_entry: Entry) -> Result<Thumbnail, Refusal<Self::Fault>> {
//!         Ok(Thumbnail)
//!     }
//!
//!     fn capability(&self) -> Word {
//!         Word::Thumbnail
//!     }
//!
//!     // Claim only your own words; the rest come back as `skipped`, not as an error.
//!     fn owns(word: &str) -> bool {
//!         word == "thumbnail"
//!     }
//! }
//!
//! let found = Search::new(Path::new("/nonexistent/data"), &[]).scan(&[]);
//! for raw in found.candidates {
//!     if let Ok(read) = ds_extensions::bayonet::reparse::<Thumbnail>(raw) {
//!         // `read.candidate` is this owner's view of the Extension, if it has an entry of
//!         // its words; `read.skipped` are the other owners' entries.
//!         drop((read.candidate, read.skipped));
//!     }
//! }
//! ```
//!
//! # Choosing among them
//!
//! A settings key of kind `Provided` stores `standard` or an Extension id. [`choices_for`] turns
//! the usable Extensions of a capability into the menu's words and labels, `standard` first
//! ([`Choices::fill`] puts them in the key), and [`chosen`] says what a stored value means: an id
//! that is not installed or is off reads as `standard` and is returned as [`Stale`] to report
//! once. [`look::stylesheet_for`] reads the chosen look's stylesheet.
//!
//! # File icons
//!
//! [`FileIcons`] turns a file (its type, name and whether it is a folder) into the icon to draw:
//! Your changes first ([`YourChanges`]), then the chosen pack (`appearance.file_icons`), then
//! the standard set. [`FileIcons::build`] makes it from a scan and the stored choice; build a new
//! one when the watch fires. The module `file_icons` has the lookup order.
//!
//! # On and off
//!
//! [`ExtensionsFile`] is `quire/extensions.toml`, a schema'd settings file whose `extensions.off`
//! lists the ids turned off ([`extensions_schema`] is its schema). On is the default.

mod capability;
mod choices;
mod extensions_file;
mod fault;
pub mod file_icons;
pub mod look;
mod provision;
mod resolve;
mod search;
mod your_changes;

pub use bayonet;
pub use capability::Capability;
pub use choices::{Choices, Chosen, Stale, choices_for, chosen};
pub use ds_core::settings::ExtensionId;
pub use extensions_file::{ExtensionsFile, ExtensionsSettings, OffRow, Power, extensions_schema};
pub use fault::Fault;
pub use file_icons::{
    Built, FileFacts, FileIcons, FolderFacts, IconFolder, IconPick, PICTURE_CAP_BYTES, Pictures,
};
pub use look::{Look, LookSheet, STYLESHEET_CAP_BYTES};
pub use provision::Provision;
pub use resolve::{Problem, Resolved, Trouble};
pub use search::{Found, Off, PROTOCOL, Search};
pub use your_changes::{ChangeError, Removal, YOUR_CHANGES_ID, YOUR_CHANGES_NAME, YourChanges};
