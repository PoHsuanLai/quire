//! `FileIcons`: from a file's facts to the icon to draw.

use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Arc;

use bayonet::registry::{Installed, Origin};
use ds_core::word::Word;
use ds_helpers::mime::{FileName, MimeDb, MimeType};
use ds_settings::schema::Provided;
use ds_style::icon::file_kind::{ExtLabel, FileKind, FolderKind};
use ds_style::icon::mime::{MimeFacts, kind_for_mime};

use crate::capability::Capability;
use crate::choices::{Stale, choices_for, chosen};
use crate::provision::Provision;
use crate::search::{Found, Off};
use crate::your_changes::YOUR_CHANGES_ID;

use super::pictures::Pictures;

/// The pack name for a type nothing else matched.
const OTHER: &str = "other";
/// The pack name for a folder.
const FOLDER: &str = "folder";

/// Whether a file is a folder, and which.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FolderFacts {
    /// A file.
    NotAFolder,
    /// A folder: [`FolderKind::Plain`], or one of the special folders.
    Folder(FolderKind),
}

/// What the resolver needs to know about a file.
#[derive(Debug, Clone, Copy)]
pub struct FileFacts<'a> {
    /// Its type, from `MimeDb::by_name` or the caller's sniffing.
    pub mime: &'a MimeType,
    /// Its name, for the extension label.
    pub name: &'a FileName,
    /// Whether it is a folder.
    pub folder: FolderFacts,
}

/// The icon to draw. The app maps it to `ds::FileIconChoice` (`Standard`, `Folder`, `Artwork`).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum IconPick {
    /// The standard page: the kind's mark and, when it has one that fits, the extension.
    Standard {
        /// Which mark and colour.
        kind: FileKind,
        /// The extension label.
        label: Option<ExtLabel>,
    },
    /// The standard Mac folder.
    Folder(FolderKind),
    /// A picture from a pack or from Your changes: an absolute path to an `.svg` or `.png` inside
    /// the Extension's folder, drawn as it is.
    Artwork(PathBuf),
}

#[derive(Debug)]
struct Inner {
    db: Arc<MimeDb>,
    /// The pictures to ask, top first.
    layers: Vec<Pictures>,
}

/// The file-icon resolver. Cheap to clone; read-only. Build a new one when the Extensions change
/// (the app's existing watch on the extensions folders and `extensions.toml`).
#[derive(Debug, Clone)]
pub struct FileIcons {
    inner: Arc<Inner>,
}

/// A resolver, and what its stored choice turned out to mean.
#[derive(Debug, Clone)]
pub struct Built {
    /// The resolver.
    pub icons: FileIcons,
    /// Set when `appearance.file_icons` names an Extension that is not installed or is turned
    /// off, and `standard` was used instead. Report it once, when the file is loaded.
    pub stale: Option<Stale>,
    /// Set when that Extension is installed and turned off ("turned off is not missing"): the
    /// host may offer Turn On… instead of treating it as gone.
    pub off: Option<Off>,
}

impl FileIcons {
    /// A resolver with only the standard set.
    pub fn standard(db: Arc<MimeDb>) -> FileIcons {
        FileIcons::layered(db, Vec::new())
    }

    /// The resolver for what is installed: Your changes on top, then the pack `stored`
    /// (`appearance.file_icons`) names, then the standard set. A stored id that is not installed
    /// or is off reads as `standard` ([`Built::stale`], and [`Built::off`] for an off one).
    pub fn build(db: Arc<MimeDb>, found: &Found, stored: &Provided) -> Built {
        let resolved = found.resolve(Capability::FileIcons);
        let installed = resolved.registry.installed();
        let choices = choices_for(Capability::FileIcons, installed);
        let picked = chosen(stored, &choices);
        let off = picked.stale.as_ref().and_then(|stale| {
            found
                .off
                .iter()
                .find(|off| off.id.0 == stale.stored && off.serves(Capability::FileIcons.slug()))
                .cloned()
        });
        let layers = [
            pictures_of(installed, YOUR_CHANGES_ID, Some(Origin::User)),
            picked.id.and_then(|id| pictures_of(installed, &id.0, None)),
        ]
        .into_iter()
        .flatten()
        .collect();
        Built {
            icons: FileIcons::layered(db, layers),
            stale: picked.stale,
            off,
        }
    }

    fn layered(db: Arc<MimeDb>, layers: Vec<Pictures>) -> FileIcons {
        FileIcons {
            inner: Arc::new(Inner { db, layers }),
        }
    }

    /// The icon for a file. Reads no file: the pictures were listed when the resolver was built.
    pub fn of(&self, facts: &FileFacts<'_>) -> IconPick {
        let (names, standard) = match facts.folder {
            FolderFacts::Folder(kind) => (folder_names(kind), IconPick::Folder(kind)),
            FolderFacts::NotAFolder => (
                file_names(&self.inner.db, facts.mime),
                self.standard_file(facts),
            ),
        };
        self.inner
            .layers
            .iter()
            .find_map(|layer| names.iter().find_map(|name| layer.get(name)))
            .map_or(standard, |path| IconPick::Artwork(path.to_path_buf()))
    }

    /// The standard set's icon for a file: the kind of its type and the extension's label.
    fn standard_file(&self, facts: &FileFacts<'_>) -> IconPick {
        let db = &self.inner.db;
        let mime = db.canonical(facts.mime);
        let icon = db.icon_name(&mime);
        let ancestors = db.ancestors(&mime);
        let parents: Vec<&str> = ancestors.iter().map(MimeType::as_str).collect();
        let kind = kind_for_mime(&MimeFacts {
            mime: mime.as_str(),
            parents: &parents,
            generic_icon: Some(icon.generic()),
        })
        .kind;
        let label = facts
            .name
            .extension()
            .and_then(OsStr::to_str)
            .and_then(ExtLabel::from_extension);
        IconPick::Standard { kind, label }
    }
}

/// The names a pack may hold a file's picture under, in the order they are tried: the type's own
/// icon name, its generic icon, its parents (nearest first), its major type, then `other`.
fn file_names(db: &MimeDb, mime: &MimeType) -> Vec<String> {
    let mime = db.canonical(mime);
    let icon = db.icon_name(&mime);
    let parents = db
        .ancestors(&mime)
        .into_iter()
        .map(|parent| db.icon_name(&parent).specific().to_owned());
    [icon.specific(), icon.generic()]
        .into_iter()
        .map(str::to_owned)
        .chain(parents)
        .chain([mime.major().to_owned(), OTHER.to_owned()])
        .collect()
}

/// The names for a folder: `folder-downloads` for a special one, then `folder`.
fn folder_names(kind: FolderKind) -> Vec<String> {
    let special = (kind != FolderKind::Plain).then(|| format!("{FOLDER}-{}", kind.slug()));
    special.into_iter().chain([FOLDER.to_owned()]).collect()
}

/// The pictures of the pack folder of Extension `id`, if it is installed with a `file_icons`
/// entry. With `from`, only the copy found in that root counts.
fn pictures_of(
    installed: &[Installed<Provision>],
    id: &str,
    from: Option<Origin>,
) -> Option<Pictures> {
    let extension = installed.iter().find(|extension| {
        extension.manifest.id.as_str() == id && from.is_none_or(|origin| origin == extension.origin)
    })?;
    let folder = extension
        .manifest
        .provides
        .iter()
        .find_map(Provision::file_icons)?;
    Some(Pictures::read(folder.dir()))
}
