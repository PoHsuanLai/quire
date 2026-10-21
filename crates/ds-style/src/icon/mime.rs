//! The glyph and Finder's plain wording for a MIME type: the file manager's icon column and its
//! "Kind" column. The exact type wins, then its major type (`image/*`), then the fallback, so a
//! type nobody listed still gets a sensible row.

use super::Icon;

/// What a file of some MIME type is drawn as and called.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MimeGlyph {
    /// The glyph of its kind.
    pub icon: Icon,
    /// The plain name of its kind: "PNG image", "ZIP archive", "Document".
    pub description: &'static str,
}

/// A type listed by its full `type/subtype`, lower case.
const EXACT: &[(&str, Icon, &str)] = &[
    ("image/png", Icon::FileImage, "PNG image"),
    ("image/jpeg", Icon::FileImage, "JPEG image"),
    ("image/gif", Icon::FileImage, "GIF image"),
    ("image/webp", Icon::FileImage, "WebP image"),
    ("image/heic", Icon::FileImage, "HEIC image"),
    ("image/tiff", Icon::FileImage, "TIFF image"),
    ("image/svg+xml", Icon::FileImage, "SVG image"),
    ("application/pdf", Icon::FilePdf, "PDF document"),
    ("text/plain", Icon::FileText, "Plain text document"),
    ("text/markdown", Icon::FileText, "Markdown document"),
    ("text/csv", Icon::FileText, "CSV document"),
    ("application/rtf", Icon::FileText, "Rich text document"),
    ("application/msword", Icon::FileText, "Word document"),
    ("text/html", Icon::FileCode, "HTML document"),
    ("application/json", Icon::FileCode, "JSON document"),
    ("text/x-python", Icon::FileCode, "Python script"),
    ("application/x-sh", Icon::FileCode, "Shell script"),
    ("application/zip", Icon::FileArchive, "ZIP archive"),
    ("application/gzip", Icon::FileArchive, "gzip archive"),
    ("application/x-tar", Icon::FileArchive, "tar archive"),
    ("application/x-7z-compressed", Icon::FileArchive, "7-Zip archive"),
    ("application/vnd.rar", Icon::FileArchive, "RAR archive"),
    ("audio/mpeg", Icon::FileAudio, "MP3 audio"),
    ("audio/flac", Icon::FileAudio, "FLAC audio"),
    ("audio/wav", Icon::FileAudio, "WAV audio"),
    ("audio/ogg", Icon::FileAudio, "Ogg audio"),
    ("video/mp4", Icon::FileVideo, "MPEG-4 movie"),
    ("video/quicktime", Icon::FileVideo, "QuickTime movie"),
    ("video/webm", Icon::FileVideo, "WebM movie"),
    ("application/x-iso9660-image", Icon::Disc, "Disc image"),
    ("application/x-executable", Icon::FileApp, "Unix executable"),
    ("application/vnd.appimage", Icon::FileApp, "Application"),
];

/// A major type with no listed subtype.
const MAJOR: &[(&str, Icon, &str)] = &[
    ("image", Icon::FileImage, "Image"),
    ("audio", Icon::FileAudio, "Audio"),
    ("video", Icon::FileVideo, "Movie"),
    ("text", Icon::FileText, "Text document"),
];

/// Anything else, and anything that is not a MIME type at all.
const FALLBACK: MimeGlyph = MimeGlyph {
    icon: Icon::File,
    description: "Document",
};

/// The glyph and description of `mime`. Parameters (`; charset=utf-8`) and case are ignored.
pub fn glyph_for_mime(mime: &str) -> MimeGlyph {
    let essence = mime
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    let major = essence.split('/').next().unwrap_or_default();
    let found = EXACT
        .iter()
        .find(|(listed, ..)| *listed == essence)
        .or_else(|| MAJOR.iter().find(|(listed, ..)| *listed == major));
    found.map_or(FALLBACK, |&(_, icon, description)| MimeGlyph {
        icon,
        description,
    })
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
    }
}
