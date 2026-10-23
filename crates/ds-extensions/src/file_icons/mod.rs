//! File icons: the `file_icons` capability and the resolver that turns a file into the icon to
//! draw (design: Extensions, sections 4.2 and 6).
//!
//! A pack is an Extension with one `file_icons` entry, a folder of pictures named the way
//! shared-mime-info names icons:
//!
//! ```text
//! file-icons/
//!     application-pdf.svg      a type's own icon name
//!     x-office-document.svg    a generic icon
//!     image.svg                a major type
//!     other.svg                anything else
//!     folder.svg  folder-downloads.svg  ...
//! ```
//!
//! Only `.svg` and `.png` are read (an `.svg` wins over a `.png` of the same name), each under
//! [`PICTURE_CAP_BYTES`]. A pack may cover ten types or a thousand: whatever it lacks falls to the
//! layer below.
//!
//! # Which icon a file gets
//!
//! [`FileIcons::of`] asks the layers in this order, and the first with a picture wins:
//!
//! 1. **Your changes** ([`YourChanges`](crate::YourChanges)): the pictures the person set for
//!    single types.
//! 2. **The chosen pack** (`appearance.file_icons`).
//! 3. **The standard set**: the kind and extension label quire draws
//!    (`ds_style::icon::mime::kind_for_mime`).
//!
//! Inside a layer, a file's names are tried in this order: the type's own icon name, its generic
//! icon, its `sub-class-of` parents (nearest first), its major type, then `other`. The generic
//! icon comes before the parents on purpose, the same order the standard set uses for kinds: a
//! `.docx` has the parent `application/zip`, and its generic icon (`x-office-document`) says
//! more than that. (The design's section 6.2 words it as parents first; this is the order the
//! standard set already has, so a pack and the standard set agree.) A folder tries
//! `folder-<kind>` (`folder-downloads`) and then `folder`.
//!
//! # Layering
//!
//! This crate may not name `ds` (the components), so [`FileIcons::of`] answers with its own small
//! [`IconPick`], made of `ds-style`'s [`FileKind`], [`FolderKind`] and [`ExtLabel`] and a path.
//! An app maps it to `ds::FileIconChoice` in three arms (CONSUMING.md, "File icons").

mod folder;
mod pictures;
mod resolver;

pub use folder::IconFolder;
pub use pictures::{PICTURE_CAP_BYTES, Pictures};
pub use resolver::{Built, FileFacts, FileIcons, FolderFacts, IconPick};

#[cfg(doc)]
use ds_style::icon::file_kind::{ExtLabel, FileKind, FolderKind};
