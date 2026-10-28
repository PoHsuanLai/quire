//! What type a file is, which apps open it, which one is the default, and its icon name, as the
//! freedesktop.org standards define them: shared-mime-info, `mimeapps.list` and `mimeinfo.cache`.
//!
//! Every parser takes text and is pure; the [`loader`] module is the only code that reads files, and
//! it takes the folders as an argument instead of reading the environment.
//!
//! Content sniffing (the `magic` file) is not here: a caller that needs the type of a file whose
//! name says nothing reads the file's bytes with its own sniffer.
//!
//! The names [`MimeType`] and [`FileName`] (with their errors [`MimeError`] and [`NameError`]) are
//! the two small types the database speaks in. The id of an app is
//! [`DesktopId`](crate::DesktopId) of the desktop-entry layer.
//!
//! ```
//! use ds_helpers::mime::{FileName, MimeDb, MimeLayer, MimeType, NameGuess};
//!
//! let layer = MimeLayer { globs2: "50:application/pdf:*.pdf\n", ..MimeLayer::default() };
//! let db = MimeDb::from_layers(&[layer]);
//! let name = FileName::new("Report.PDF")?;
//! assert_eq!(db.by_name(&name), NameGuess::Type(MimeType::new("application/pdf")?));
//! assert_eq!(db.icon_name(&MimeType::new("application/pdf")?).specific(), "application-pdf");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod assoc;
mod db;
mod error;
mod file_name;
mod fnmatch;
mod globs;
pub mod loader;
mod mime_type;
mod mimeapps;
mod tables;
mod text;

pub use crate::desktop_entries::{DesktopId, DesktopIdError};
pub use assoc::Associations;
pub use db::{IconName, MimeDb, MimeLayer, Relation};
pub use error::{LoadError, MimeError, NameError};
pub use file_name::FileName;
pub use globs::{Case, GlobEntry, Globs, NameGuess, Weight};
pub use loader::{Loaded, XdgDirs};
pub use mime_type::MimeType;
pub use mimeapps::{MimeApps, MimeCache};
pub use tables::{Aliases, IconNames, Subclasses};
