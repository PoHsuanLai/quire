//! The standard file-icon data: the lookup from a MIME type to a kind, the folders' emboss
//! glyphs and the extension label rule.

use ds_core::word::Word;
use ds_style::icon::Icon;
use ds_style::icon::file_kind::{ExtLabel, FileKind, FolderKind};
use ds_style::icon::mime::{MimeFacts, kind_for_mime};

fn kind(mime: &str, parents: &[&str], generic_icon: Option<&str>) -> (FileKind, &'static str) {
    let found = kind_for_mime(&MimeFacts {
        mime,
        parents,
        generic_icon,
    });
    (found.kind, found.description)
}

/// One lookup case: (mime, parents, generic icon, kind, description).
type Case = (
    &'static str,
    &'static [&'static str],
    Option<&'static str>,
    FileKind,
    &'static str,
);

#[test]
fn the_lookup_goes_exact_type_then_generic_icon_then_parents_then_major_then_other() {
    #[rustfmt::skip]
    let cases: &[Case] = &[
        // The exact type wins over a generic icon that says otherwise.
        ("application/pdf", &["application/octet-stream"], Some("package-x-generic"), FileKind::Pdf, "PDF document"),
        // A generic icon names the kind of a type nobody listed, even when its parent is a ZIP.
        ("application/x-unlisted-office", &["application/zip"], Some("x-office-document"), FileKind::Document, "Document"),
        ("application/x-unlisted-sheet", &[], Some("x-office-spreadsheet"), FileKind::Spreadsheet, "Spreadsheet"),
        ("application/x-unlisted-slides", &[], Some("x-office-presentation"), FileKind::Presentation, "Presentation"),
        ("application/x-unlisted-pack", &[], Some("package-x-generic"), FileKind::Archive, "Archive"),
        ("application/x-unlisted-song", &[], Some("audio-x-generic"), FileKind::Audio, "Audio"),
        ("application/x-unlisted-film", &[], Some("video-x-generic"), FileKind::Video, "Movie"),
        ("application/x-unlisted-pic", &[], Some("image-x-generic"), FileKind::Image, "Image"),
        ("application/x-unlisted-face", &[], Some("font-x-generic"), FileKind::Font, "Font"),
        ("application/x-unlisted-run", &[], Some("application-x-executable"), FileKind::Program, "Application"),
        // Shell scripts are plain text by parent and code by generic icon.
        ("application/x-shellscript", &["text/plain"], Some("text-x-script"), FileKind::Code, "Script"),
        // No generic icon: the nearest listed parent.
        ("application/x-unlisted-doc", &["application/x-unknown", "application/zip"], None, FileKind::Archive, "ZIP archive"),
        ("text/x-unlisted-notes", &["text/plain"], None, FileKind::Document, "Plain text document"),
        // An unlisted generic icon is skipped.
        ("image/x-unlisted", &[], Some("x-office-address-book"), FileKind::Image, "Image"),
        // The major type, then other.
        ("font/x-unlisted", &[], None, FileKind::Font, "Font"),
        ("application/x-unlisted", &[], None, FileKind::Other, "Document"),
    ];
    for &(mime, parents, generic, want, text) in cases {
        assert_eq!(kind(mime, parents, generic), (want, text), "{mime}");
    }
}

#[test]
fn only_other_has_no_mark_and_every_mark_is_a_glyph_of_the_set() {
    for &kind in FileKind::ALL {
        assert_eq!(kind.mark().is_none(), kind == FileKind::Other, "{kind:?}");
        if let Some(mark) = kind.mark() {
            assert!(Icon::ALL.contains(&mark), "{kind:?}");
        }
    }
}

#[test]
fn a_plain_folder_has_no_emboss_and_every_special_one_has_its_own() {
    let marks: Vec<Icon> = FolderKind::ALL.iter().filter_map(|k| k.emboss()).collect();
    assert_eq!(marks.len(), FolderKind::ALL.len() - 1);
    assert_eq!(FolderKind::Plain.emboss(), None);
    for (at, mark) in marks.iter().enumerate() {
        assert!(!marks[at + 1..].contains(mark), "{mark:?} twice");
    }
}

#[test]
fn an_extension_label_is_upper_case_and_at_most_four_characters() {
    // (extension, label)
    const CASES: &[(&str, Option<&str>)] = &[
        ("pdf", Some("PDF")),
        (".zip", Some("ZIP")),
        ("Docx", Some("DOCX")),
        ("7z", Some("7Z")),
        ("woff2", None),
        ("", None),
        (".", None),
        ("tar.gz", None),
        ("a b", None),
    ];
    for &(extension, label) in CASES {
        let got = ExtLabel::from_extension(extension);
        assert_eq!(got.as_ref().map(ExtLabel::as_str), label, "{extension:?}");
    }
}
