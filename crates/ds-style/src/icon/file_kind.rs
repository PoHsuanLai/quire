//! The standard file-icon set as data (design/08-ICONS.md section 2.12, the customization
//! design's section 6.1): the closed list of file kinds with a mark and a colour each, the
//! special folders with the glyph embossed on the folder, and the extension label.
//!
//! Pure data. `ds::FileIcon` draws it; the lookup from a MIME type to a kind is in [`super::mime`].

use super::Icon;
use crate::appearance::system::Contrast;
use crate::appearance::theme::Scheme;
use crate::tokens::hex::Hex;
use crate::tokens::system_hue::SystemHue;
use crate::tokens::token::{CssValue, Token, TokenScope};
use ds_core::word::Word;

/// What a file is, for drawing its icon. As a token it is the colour of the kind's mark
/// (`--file-kind-pdf`): the kind's system hue, light scheme in both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "file-kind-", kind = fixed, css = kind_css)]
#[non_exhaustive]
pub enum FileKind {
    /// Text, rich text and word-processor files.
    #[word(label = "Document")]
    Document,
    /// A PDF.
    #[word(label = "PDF document")]
    Pdf,
    /// A sheet of cells.
    #[word(label = "Spreadsheet")]
    Spreadsheet,
    /// Slides.
    #[word(label = "Presentation")]
    Presentation,
    /// A picture.
    #[word(label = "Image")]
    Image,
    /// Sound.
    #[word(label = "Audio")]
    Audio,
    /// A movie.
    #[word(label = "Movie")]
    Video,
    /// Files packed into one.
    #[word(label = "Archive")]
    Archive,
    /// Source code and markup.
    #[word(label = "Source code")]
    Code,
    /// A typeface.
    #[word(label = "Font")]
    Font,
    /// A disc or disk image.
    #[word(label = "Disc image")]
    DiskImage,
    /// Something that runs.
    #[word(label = "Application")]
    Program,
    /// Anything else: the page carries the extension label alone.
    #[word(label = "Document")]
    Other,
}

impl FileKind {
    /// The mark drawn on the page, from the glyph set; `None` for [`FileKind::Other`], whose
    /// only mark is its extension label.
    pub fn mark(self) -> Option<Icon> {
        match self {
            FileKind::Document => Some(Icon::Lines),
            FileKind::Pdf => Some(Icon::Pdf),
            FileKind::Spreadsheet => Some(Icon::Table),
            FileKind::Presentation => Some(Icon::Monitor),
            FileKind::Image => Some(Icon::Image),
            FileKind::Audio => Some(Icon::Music),
            FileKind::Video => Some(Icon::Play),
            FileKind::Archive => Some(Icon::Archive),
            FileKind::Code => Some(Icon::Code),
            FileKind::Font => Some(Icon::Type),
            FileKind::DiskImage => Some(Icon::Disc),
            FileKind::Program => Some(Icon::Settings),
            FileKind::Other => None,
        }
    }

    /// The tag colour the mark is drawn in (`--tag-*`, so it follows the scheme). `Other` is gray.
    pub fn colour(self) -> SystemHue {
        match self {
            FileKind::Document => SystemHue::Blue,
            FileKind::Pdf => SystemHue::Red,
            FileKind::Spreadsheet => SystemHue::Green,
            FileKind::Presentation => SystemHue::Orange,
            FileKind::Image => SystemHue::Purple,
            FileKind::Audio => SystemHue::Pink,
            FileKind::Video => SystemHue::Indigo,
            FileKind::Archive => SystemHue::Brown,
            FileKind::Code => SystemHue::Teal,
            FileKind::Font | FileKind::Other => SystemHue::Gray,
            FileKind::DiskImage => SystemHue::Cyan,
            FileKind::Program => SystemHue::Mint,
        }
    }

    /// The colour of the mark: the kind's tag hue in the light scheme.
    pub fn hex(self) -> Hex {
        self.colour().value(Scheme::Light, Contrast::Normal)
    }

    /// The per-file glyph a list row draws for this kind (the page-shaped glyphs of the actions
    /// set), for a consumer that wants a glyph and not the full icon.
    pub fn file_glyph(self) -> Icon {
        match self {
            FileKind::Document | FileKind::Spreadsheet | FileKind::Presentation => Icon::FileText,
            FileKind::Pdf => Icon::FilePdf,
            FileKind::Image => Icon::FileImage,
            FileKind::Audio => Icon::FileAudio,
            FileKind::Video => Icon::FileVideo,
            FileKind::Archive => Icon::FileArchive,
            FileKind::Code => Icon::FileCode,
            FileKind::DiskImage => Icon::Disc,
            FileKind::Program => Icon::FileApp,
            FileKind::Font | FileKind::Other => Icon::File,
        }
    }
}

fn kind_css(token: FileKind, _scope: TokenScope) -> CssValue {
    CssValue::computed(token.hex().css())
}

/// What a folder is, for drawing its icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
#[non_exhaustive]
pub enum FolderKind {
    /// Any folder.
    #[default]
    Plain,
    /// The person's home folder.
    Home,
    /// The Desktop folder.
    Desktop,
    /// The Documents folder.
    Documents,
    /// The Downloads folder.
    Downloads,
    /// The Pictures folder.
    Pictures,
    /// The Music folder.
    Music,
    /// The Movies (Videos) folder.
    Movies,
    /// The Trash.
    Trash,
}

impl FolderKind {
    /// The glyph embossed on the folder; `None` for a plain one.
    pub fn emboss(self) -> Option<Icon> {
        match self {
            FolderKind::Plain => None,
            FolderKind::Home => Some(Icon::Home),
            FolderKind::Desktop => Some(Icon::Monitor),
            FolderKind::Documents => Some(Icon::FileText),
            FolderKind::Downloads => Some(Icon::Download),
            FolderKind::Pictures => Some(Icon::Image),
            FolderKind::Music => Some(Icon::Music),
            FolderKind::Movies => Some(Icon::Play),
            FolderKind::Trash => Some(Icon::Trash),
        }
    }
}

/// The most characters an extension label holds.
pub const EXT_LABEL_MAX: usize = 4;

/// The extension a page carries near its bottom: upper case, one to four letters or digits.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExtLabel(String);

impl ExtLabel {
    /// The label of `extension` (with or without its dot, any case): `pdf` is `PDF`. `None` when
    /// it is empty, longer than [`EXT_LABEL_MAX`] or not letters and digits, since a label that
    /// had to be cut would name another type.
    pub fn from_extension(extension: &str) -> Option<ExtLabel> {
        let bare = extension.trim().trim_start_matches('.');
        let fits = !bare.is_empty()
            && bare.chars().count() <= EXT_LABEL_MAX
            && bare.chars().all(|c| c.is_ascii_alphanumeric());
        fits.then(|| ExtLabel(bare.to_ascii_uppercase()))
    }

    /// The label text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
