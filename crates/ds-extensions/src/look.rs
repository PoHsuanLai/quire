//! The `look` capability: a stylesheet that restyles every quire surface (design: Extensions,
//! section 7.7).
//!
//! ```toml
//! [[provides]]
//! capability = "look"
//! stylesheet = "look.css"       # in this folder; may name other files here
//! styles_for = 3                # the styling surface revision it was made against
//! ```
//!
//! This crate finds, checks and reads the sheet. It does not apply it: the root's style props
//! take the text ([`stylesheet_for`]), and the sheet's `url()` rule (a `data:` URL or a file in
//! its own folder, nothing from the network) is the style layer's.

use std::fs;
use std::path::{Path, PathBuf};

use bayonet::manifest::{Entry, Refusal};
use bayonet::registry::Registry;
use ds_helpers::{ReadCap, read_text_capped};
use ds_settings::schema::Provided;
use serde::Deserialize;

use crate::fault::Fault;
use crate::provision::Provision;

/// The most a look's stylesheet may be: 256 KiB. A larger one makes its Extension unusable.
pub const STYLESHEET_CAP_BYTES: u64 = 256 * 1024;

/// What a manifest's `look` entry says, as written.
#[derive(Debug, Deserialize)]
struct Wire {
    stylesheet: PathBuf,
    styles_for: u32,
}

/// A `look` entry: where its stylesheet is, and the styling surface it was written against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Look {
    stylesheet: PathBuf,
    folder: PathBuf,
    styles_for: u32,
}

impl Look {
    /// Reads the entry. The stylesheet path is made absolute inside the Extension's folder
    /// (`Entry::resolve`): an absolute path or a `..` is refused.
    pub(crate) fn from_entry(entry: Entry) -> Result<Look, Refusal<Fault>> {
        let wire: Wire = entry.clone().read::<Wire, Fault>()?;
        if !wire.stylesheet.extension().is_some_and(|ext| ext == "css") {
            return Err(Refusal::Fault(Fault::NotCss {
                path: wire.stylesheet,
            }));
        }
        let stylesheet = entry.resolve::<Fault>(wire.stylesheet)?;
        // The empty path resolves to the folder itself.
        let folder = entry.resolve::<Fault>(PathBuf::from("."))?;
        Ok(Look {
            stylesheet,
            folder,
            styles_for: wire.styles_for,
        })
    }

    /// The stylesheet, inside the Extension's folder.
    pub fn stylesheet(&self) -> &Path {
        &self.stylesheet
    }

    /// The Extension's folder: where the sheet's own `url()` files may be.
    pub fn folder(&self) -> &Path {
        &self.folder
    }

    /// The `ds::selectors::SURFACE` revision the sheet was made against.
    pub fn styles_for(&self) -> u32 {
        self.styles_for
    }

    /// The first problem with the files this entry names, if any: a sheet over the cap.
    pub(crate) fn fault(&self) -> Option<Fault> {
        let size = fs::metadata(&self.stylesheet).ok()?.len();
        (size > STYLESHEET_CAP_BYTES).then(|| Fault::TooBig {
            path: self.stylesheet.clone(),
            cap: STYLESHEET_CAP_BYTES,
        })
    }
}

/// A look's stylesheet, read: the text and the folder its `url()` files may come from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookSheet {
    /// The stylesheet's text.
    pub text: String,
    /// The Extension's folder.
    pub folder: PathBuf,
}

/// The stylesheet of the look `choice` names, from the usable Extensions in `installed` (the
/// registry of [`Found::resolve`](crate::Found::resolve) for
/// [`Capability::Look`](crate::Capability::Look)).
///
/// `None` for `standard`, for an id that is not installed or is off (it is not in `installed`),
/// and for a sheet that cannot be read now (gone, or over [`STYLESHEET_CAP_BYTES`] since the
/// scan): the desktop then looks as it ships.
///
/// The text is returned as the file holds it. The host puts it in `Styles::look` and `folder` in
/// `StyleHomes::look`; `Styles::drawn` then applies the reach rule (`ds_style::kit::reach`), so a
/// look's `url()`, `@import` and `@font-face` reach only `data:` and files in its own folder.
pub fn stylesheet_for(choice: &Provided, installed: &Registry<Provision>) -> Option<LookSheet> {
    let id = choice.chosen()?;
    let look = installed
        .installed()
        .iter()
        .find(|extension| extension.manifest.id.as_str() == id)?
        .manifest
        .provides
        .iter()
        .find_map(Provision::look)?;
    let text = read_text_capped(look.stylesheet(), ReadCap::bytes(STYLESHEET_CAP_BYTES)).ok()?;
    Some(LookSheet {
        text,
        folder: look.folder().to_path_buf(),
    })
}
