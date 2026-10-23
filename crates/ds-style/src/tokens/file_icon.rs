//! The colours of the standard file icon (`ds::FileIcon`): the page's fold and label, the blue
//! folder of the reference's era, and the colour of each kind's mark.
//!
//! The page stays light in the dark scheme, as the reference's does, so the kind colours are the
//! system hues of the light scheme in both. The folder takes its dark tones in the dark.

use super::hex::Hex;
use super::token::{CssValue, Token, TokenScope};
use crate::appearance::system::Contrast;
use crate::appearance::theme::Scheme;
use crate::icon::file_kind::FileKind;
use ds_core::word::Word;

/// One colour of the file icon's own parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "file-", kind = fixed, css = file_css)]
#[non_exhaustive]
pub enum FileIconColour {
    /// `--file-fold`: the page's folded corner, a shade under the paper.
    Fold,
    /// `--file-edge`: the page's hairline edge.
    Edge,
    /// `--file-label`: the extension label's ink.
    Label,
    /// `--file-folder-tab`: the folder's back and tab, the darker tone.
    FolderTab,
    /// `--file-folder-body`: the folder's front, the lighter tone.
    FolderBody,
    /// `--file-folder-ink`: the glyph embossed on the folder.
    FolderInk,
    /// `--file-folder-light`: the bevel highlight under the embossed glyph.
    FolderLight,
}

impl FileIconColour {
    /// The colour in `scheme`.
    pub const fn hex(self, scheme: Scheme) -> Hex {
        let (light, dark): (u32, u32) = match self {
            FileIconColour::Fold => (0xDCDDE1, 0xDCDDE1),
            FileIconColour::Edge => (0xC2C4CA, 0xC2C4CA),
            FileIconColour::Label => (0x6C6C70, 0x6C6C70),
            FileIconColour::FolderTab => (0x3FA3E8, 0x2A78BC),
            FileIconColour::FolderBody => (0x6CC4F8, 0x4AA6E4),
            FileIconColour::FolderInk => (0x2C84C2, 0x1D5C92),
            FileIconColour::FolderLight => (0xA6DEFB, 0x7CC2F0),
        };
        let value = match scheme {
            Scheme::Light => light,
            Scheme::Dark => dark,
        };
        Hex([(value >> 16) as u8, (value >> 8) as u8, value as u8])
    }
}

fn file_css(token: FileIconColour, scope: TokenScope) -> CssValue {
    CssValue::computed(token.hex(scope.scheme).css())
}

/// The colour of a kind's mark (`--file-kind-pdf`): the kind's system hue, light scheme in both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "file-kind-", kind = fixed, css = kind_css)]
#[non_exhaustive]
pub enum FileKindColour {
    /// `--file-kind-document`.
    Document,
    /// `--file-kind-pdf`.
    Pdf,
    /// `--file-kind-spreadsheet`.
    Spreadsheet,
    /// `--file-kind-presentation`.
    Presentation,
    /// `--file-kind-image`.
    Image,
    /// `--file-kind-audio`.
    Audio,
    /// `--file-kind-video`.
    Video,
    /// `--file-kind-archive`.
    Archive,
    /// `--file-kind-code`.
    Code,
    /// `--file-kind-font`.
    Font,
    /// `--file-kind-disk-image`.
    DiskImage,
    /// `--file-kind-program`.
    Program,
    /// `--file-kind-other`.
    Other,
}

impl FileKindColour {
    /// The token of `kind`'s mark.
    pub fn of(kind: FileKind) -> FileKindColour {
        match kind {
            FileKind::Document => FileKindColour::Document,
            FileKind::Pdf => FileKindColour::Pdf,
            FileKind::Spreadsheet => FileKindColour::Spreadsheet,
            FileKind::Presentation => FileKindColour::Presentation,
            FileKind::Image => FileKindColour::Image,
            FileKind::Audio => FileKindColour::Audio,
            FileKind::Video => FileKindColour::Video,
            FileKind::Archive => FileKindColour::Archive,
            FileKind::Code => FileKindColour::Code,
            FileKind::Font => FileKindColour::Font,
            FileKind::DiskImage => FileKindColour::DiskImage,
            FileKind::Program => FileKindColour::Program,
            FileKind::Other => FileKindColour::Other,
        }
    }

    /// The kind this token colours.
    pub fn kind(self) -> FileKind {
        match self {
            FileKindColour::Document => FileKind::Document,
            FileKindColour::Pdf => FileKind::Pdf,
            FileKindColour::Spreadsheet => FileKind::Spreadsheet,
            FileKindColour::Presentation => FileKind::Presentation,
            FileKindColour::Image => FileKind::Image,
            FileKindColour::Audio => FileKind::Audio,
            FileKindColour::Video => FileKind::Video,
            FileKindColour::Archive => FileKind::Archive,
            FileKindColour::Code => FileKind::Code,
            FileKindColour::Font => FileKind::Font,
            FileKindColour::DiskImage => FileKind::DiskImage,
            FileKindColour::Program => FileKind::Program,
            FileKindColour::Other => FileKind::Other,
        }
    }

    /// The colour of the mark: the kind's tag hue in the light scheme.
    pub fn hex(self) -> Hex {
        let hue = self.kind().colour().hue();
        hue.value(Scheme::Light, Contrast::Normal)
    }
}

fn kind_css(token: FileKindColour, _scope: TokenScope) -> CssValue {
    CssValue::computed(token.hex().css())
}
