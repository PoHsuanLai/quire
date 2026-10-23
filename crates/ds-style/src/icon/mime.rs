//! The kind and Finder's plain wording for a MIME type: the file manager's icon column, its
//! "Kind" column and the standard file icon (`super::file_kind`).
//!
//! The lookup takes the type, its parents and its generic-icon name as plain strings, so this
//! crate needs no MIME database; the caller reads them from shared-mime-info. The order is
//! 1. the exact type,
//! 2. the generic-icon name (`x-office-document`, `package-x-generic`, ...),
//! 3. the parents, nearest first, each by its exact type,
//! 4. the major type (`image/*`),
//! 5. [`FileKind::Other`].
//!
//! The generic icon comes before the parents because the parents of a specific type end in a
//! container or in plain text (a `.docx` is a ZIP archive, a shell script is plain text), while
//! its generic icon says what it is for.

use super::Icon;
use super::file_kind::FileKind;

/// A type and what the MIME database says about it, as strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MimeFacts<'a> {
    /// The type, `type/subtype`; parameters and case are ignored.
    pub mime: &'a str,
    /// Its `sub-class-of` parents, nearest first.
    pub parents: &'a [&'a str],
    /// Its `generic-icon` name, when the database has one.
    pub generic_icon: Option<&'a str>,
}

impl<'a> MimeFacts<'a> {
    /// A type with nothing known about it but its name.
    pub fn of(mime: &'a str) -> MimeFacts<'a> {
        MimeFacts {
            mime,
            parents: &[],
            generic_icon: None,
        }
    }
}

/// What a file of some MIME type is drawn as and called.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MimeKind {
    /// Which standard icon it gets.
    pub kind: FileKind,
    /// The plain name of its kind: "PNG image", "ZIP archive", "Document".
    pub description: &'static str,
}

/// What a file of some MIME type is drawn as in a list row, and called.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MimeGlyph {
    /// The glyph of its kind.
    pub icon: Icon,
    /// The plain name of its kind: "PNG image", "ZIP archive", "Document".
    pub description: &'static str,
}

type Row = (&'static str, FileKind, &'static str);

/// A type listed by its full `type/subtype`, lower case.
const EXACT: &[Row] = &[
    ("image/png", FileKind::Image, "PNG image"),
    ("image/jpeg", FileKind::Image, "JPEG image"),
    ("image/gif", FileKind::Image, "GIF image"),
    ("image/webp", FileKind::Image, "WebP image"),
    ("image/heic", FileKind::Image, "HEIC image"),
    ("image/tiff", FileKind::Image, "TIFF image"),
    ("image/svg+xml", FileKind::Image, "SVG image"),
    ("application/pdf", FileKind::Pdf, "PDF document"),
    ("text/plain", FileKind::Document, "Plain text document"),
    ("text/markdown", FileKind::Document, "Markdown document"),
    ("text/csv", FileKind::Spreadsheet, "CSV document"),
    ("application/rtf", FileKind::Document, "Rich text document"),
    ("application/msword", FileKind::Document, "Word document"),
    (
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        FileKind::Document,
        "Word document",
    ),
    (
        "application/vnd.oasis.opendocument.text",
        FileKind::Document,
        "OpenDocument text",
    ),
    (
        "application/vnd.ms-excel",
        FileKind::Spreadsheet,
        "Excel spreadsheet",
    ),
    (
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        FileKind::Spreadsheet,
        "Excel spreadsheet",
    ),
    (
        "application/vnd.oasis.opendocument.spreadsheet",
        FileKind::Spreadsheet,
        "OpenDocument spreadsheet",
    ),
    (
        "application/vnd.ms-powerpoint",
        FileKind::Presentation,
        "PowerPoint presentation",
    ),
    (
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        FileKind::Presentation,
        "PowerPoint presentation",
    ),
    (
        "application/vnd.oasis.opendocument.presentation",
        FileKind::Presentation,
        "OpenDocument presentation",
    ),
    ("text/html", FileKind::Code, "HTML document"),
    ("application/json", FileKind::Code, "JSON document"),
    ("text/x-python", FileKind::Code, "Python script"),
    ("application/x-sh", FileKind::Code, "Shell script"),
    ("application/zip", FileKind::Archive, "ZIP archive"),
    ("application/gzip", FileKind::Archive, "gzip archive"),
    ("application/x-tar", FileKind::Archive, "tar archive"),
    (
        "application/x-7z-compressed",
        FileKind::Archive,
        "7-Zip archive",
    ),
    ("application/vnd.rar", FileKind::Archive, "RAR archive"),
    ("audio/mpeg", FileKind::Audio, "MP3 audio"),
    ("audio/flac", FileKind::Audio, "FLAC audio"),
    ("audio/wav", FileKind::Audio, "WAV audio"),
    ("audio/ogg", FileKind::Audio, "Ogg audio"),
    ("video/mp4", FileKind::Video, "MPEG-4 movie"),
    ("video/quicktime", FileKind::Video, "QuickTime movie"),
    ("video/webm", FileKind::Video, "WebM movie"),
    ("font/ttf", FileKind::Font, "TrueType font"),
    ("font/otf", FileKind::Font, "OpenType font"),
    ("font/woff2", FileKind::Font, "WOFF2 font"),
    (
        "application/x-iso9660-image",
        FileKind::DiskImage,
        "Disc image",
    ),
    (
        "application/x-apple-diskimage",
        FileKind::DiskImage,
        "Disk image",
    ),
    (
        "application/x-executable",
        FileKind::Program,
        "Unix executable",
    ),
    ("application/vnd.appimage", FileKind::Program, "Application"),
];

/// A shared-mime-info generic-icon name.
const GENERIC: &[Row] = &[
    ("x-office-document", FileKind::Document, "Document"),
    ("x-office-spreadsheet", FileKind::Spreadsheet, "Spreadsheet"),
    (
        "x-office-presentation",
        FileKind::Presentation,
        "Presentation",
    ),
    ("x-office-drawing", FileKind::Image, "Drawing"),
    ("text-x-generic", FileKind::Document, "Text document"),
    ("text-x-script", FileKind::Code, "Script"),
    ("text-html", FileKind::Code, "HTML document"),
    ("package-x-generic", FileKind::Archive, "Archive"),
    ("audio-x-generic", FileKind::Audio, "Audio"),
    ("video-x-generic", FileKind::Video, "Movie"),
    ("image-x-generic", FileKind::Image, "Image"),
    ("font-x-generic", FileKind::Font, "Font"),
    ("application-x-cd-image", FileKind::DiskImage, "Disc image"),
    ("application-x-executable", FileKind::Program, "Application"),
    ("application-x-sharedlib", FileKind::Program, "Library"),
];

/// A major type with no listed subtype.
const MAJOR: &[Row] = &[
    ("image", FileKind::Image, "Image"),
    ("audio", FileKind::Audio, "Audio"),
    ("video", FileKind::Video, "Movie"),
    ("text", FileKind::Document, "Text document"),
    ("font", FileKind::Font, "Font"),
];

/// Anything else, and anything that is not a MIME type at all.
const FALLBACK: MimeKind = MimeKind {
    kind: FileKind::Other,
    description: "Document",
};

/// A type as the tables key it: parameters dropped, trimmed, lower case.
fn essence(mime: &str) -> String {
    mime.split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

fn find(table: &[Row], key: &str) -> Option<MimeKind> {
    table
        .iter()
        .find(|(listed, ..)| *listed == key)
        .map(|&(_, kind, description)| MimeKind { kind, description })
}

/// The kind and description of the type `facts` describe, by the order in the module docs.
pub fn kind_for_mime(facts: &MimeFacts<'_>) -> MimeKind {
    let exact = essence(facts.mime);
    let major = exact.split('/').next().unwrap_or_default();
    find(EXACT, &exact)
        .or_else(|| {
            facts
                .generic_icon
                .and_then(|name| find(GENERIC, &name.trim().to_ascii_lowercase()))
        })
        .or_else(|| {
            facts
                .parents
                .iter()
                .find_map(|parent| find(EXACT, &essence(parent)))
        })
        .or_else(|| find(MAJOR, major))
        .unwrap_or(FALLBACK)
}

/// The glyph and description of `mime` alone. Parameters (`; charset=utf-8`) and case are
/// ignored.
pub fn glyph_for_mime(mime: &str) -> MimeGlyph {
    let found = kind_for_mime(&MimeFacts::of(mime));
    MimeGlyph {
        icon: found.kind.file_glyph(),
        description: found.description,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mime_type_maps_to_its_glyph_and_kind() {
        // (mime, icon, description)
        #[rustfmt::skip]
        const CASES: &[(&str, Icon, &str)] = &[
            ("image/png", Icon::FileImage, "PNG image"),
            ("image/jpeg", Icon::FileImage, "JPEG image"),
            ("application/pdf", Icon::FilePdf, "PDF document"),
            ("text/plain", Icon::FileText, "Plain text document"),
            ("text/html", Icon::FileCode, "HTML document"),
            ("application/zip", Icon::FileArchive, "ZIP archive"),
            ("audio/mpeg", Icon::FileAudio, "MP3 audio"),
            ("video/mp4", Icon::FileVideo, "MPEG-4 movie"),
            ("application/x-iso9660-image", Icon::Disc, "Disc image"),
            ("application/x-executable", Icon::FileApp, "Unix executable"),
            // The exact type wins over its major type.
            ("text/markdown", Icon::FileText, "Markdown document"),
            // An unlisted subtype falls back to its major type.
            ("image/x-unlisted", Icon::FileImage, "Image"),
            ("audio/x-unlisted", Icon::FileAudio, "Audio"),
            ("video/x-unlisted", Icon::FileVideo, "Movie"),
            ("text/x-unlisted", Icon::FileText, "Text document"),
            // Parameters and case are not part of the type.
            ("text/plain; charset=utf-8", Icon::FileText, "Plain text document"),
            ("IMAGE/PNG", Icon::FileImage, "PNG image"),
            ("  application/pdf ", Icon::FilePdf, "PDF document"),
            // Unknown major types, an empty string and non-MIME text get the fallback.
            ("application/x-unlisted", Icon::File, "Document"),
            ("", Icon::File, "Document"),
            ("not a mime", Icon::File, "Document"),
        ];
        for &(mime, icon, description) in CASES {
            assert_eq!(
                glyph_for_mime(mime),
                MimeGlyph { icon, description },
                "{mime:?}"
            );
        }
    }

    #[test]
    fn every_listed_type_is_lower_case_and_has_a_distinct_name() {
        for (index, (mime, _, description)) in EXACT.iter().enumerate() {
            assert_eq!(*mime, mime.to_ascii_lowercase(), "{mime}");
            assert!(mime.contains('/'), "{mime}");
            assert!(
                EXACT[index + 1..].iter().all(|(other, ..)| other != mime),
                "{mime} is listed twice"
            );
            assert!(!description.is_empty(), "{mime}");
        }
        for (index, (name, ..)) in GENERIC.iter().enumerate() {
            assert!(
                GENERIC[index + 1..].iter().all(|(other, ..)| other != name),
                "{name} is listed twice"
            );
        }
    }
}
