//! The standard file icon as markup: the parts each choice draws, the label rule by size, the
//! monochrome paint, and that the stylesheet knows every kind and token the markup names.

use dioxus::core::VirtualDom;
use ds::components::content::file_icon::{FileIcon, FileIconChoice, FileIconProps, LABEL_MIN_PX};
use ds::prelude::*;
use ds_core::word::Word;
use ds_style::icon::render::IconPx;
use ds_style::icon::retint::Tint;

fn icon(choice: FileIconChoice, px: u8, style: FileIconStyle) -> String {
    let mut dom = VirtualDom::new_with_props(
        FileIcon,
        FileIconProps {
            choice,
            size: IconSize::Px(IconPx(px)),
            style,
            common: Common::default(),
        },
    );
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn standard(kind: FileKind, extension: &str, px: u8) -> String {
    let label = ExtLabel::from_extension(extension);
    icon(
        FileIconChoice::Standard { kind, label },
        px,
        FileIconStyle::Colour,
    )
}

#[test]
fn a_standard_page_has_paper_fold_edge_mark_and_label() {
    let html = standard(FileKind::Pdf, "pdf", 128);
    for part in [
        "data-choice=\"standard\"",
        "data-kind=\"pdf\"",
        "data-size=\"128\"",
        "data-label=\"shown\"",
        "ds-file-paper",
        "ds-file-flap",
        "ds-file-edge",
        "ds-file-shadow",
        "ds-file-mark",
        "ds-ic",
        "class=\"ds-file-label\"",
        ">PDF</span>",
        "role=\"img\"",
        "aria-label=\"PDF document\"",
    ] {
        assert!(html.contains(part), "{part}: {html}");
    }
    assert!(!html.contains("ds-file-folder"), "{html}");
}

#[test]
fn the_label_is_dropped_under_48_px_and_the_mark_stays() {
    assert_eq!(LABEL_MIN_PX, 48);
    for px in [16, 32] {
        let html = standard(FileKind::Archive, "zip", px);
        assert!(!html.contains("ds-file-label"), "{px}: {html}");
        assert!(html.contains("ds-file-mark"), "{px}: {html}");
        assert!(html.contains("data-label=\"hidden\""), "{px}: {html}");
    }
    assert!(standard(FileKind::Archive, "zip", 48).contains(">ZIP</span>"));
}

#[test]
fn a_kind_with_no_label_draws_the_mark_alone_and_other_draws_the_label_alone() {
    let bare = icon(
        FileIconChoice::Standard {
            kind: FileKind::Code,
            label: None,
        },
        64,
        FileIconStyle::Colour,
    );
    assert!(bare.contains("ds-file-mark") && !bare.contains("ds-file-label"));
    assert!(bare.contains("data-slot=\"bare\""), "{bare}");
    let other = standard(FileKind::Other, "xyz", 64);
    assert!(other.contains(">XYZ</span>") && !other.contains("ds-file-mark"));
    let small = standard(FileKind::Other, "xyz", 32);
    assert!(!small.contains("ds-file-mark") && !small.contains("ds-file-label"));
}

#[test]
fn every_kind_draws_its_own_mark_and_names_itself() {
    for kind in FileKind::ALL.iter().copied() {
        let html = standard(kind, "abc", 64);
        let kind_attr = format!("data-kind=\"{}\"", kind.slug());
        let name_attr = format!("aria-label=\"{}\"", kind.label());
        assert!(html.contains(&kind_attr), "{html}");
        assert!(html.contains(&name_attr), "{html}");
        let marked = html.contains("ds-file-mark");
        assert_eq!(marked, kind.mark().is_some(), "{html}");
    }
}

#[test]
fn a_folder_has_a_tab_and_a_front_and_a_special_one_its_glyph_twice() {
    let plain = icon(
        FileIconChoice::Folder(FolderKind::Plain),
        128,
        FileIconStyle::Colour,
    );
    for part in [
        "data-choice=\"folder\"",
        "ds-file-tab",
        "ds-file-body",
        "aria-label=\"Folder\"",
    ] {
        assert!(plain.contains(part), "{part}: {plain}");
    }
    assert!(!plain.contains("ds-file-emboss"), "{plain}");
    assert!(!plain.contains("ds-file-paper"), "{plain}");
    for kind in FolderKind::ALL
        .iter()
        .copied()
        .filter(|kind| *kind != FolderKind::Plain)
    {
        let html = icon(FileIconChoice::Folder(kind), 128, FileIconStyle::Colour);
        assert_eq!(
            html.matches("ds-file-emboss").count(),
            3,
            "{kind:?}: {html}"
        );
        assert!(html.contains("ds-file-emboss-light"), "{kind:?}: {html}");
    }
}

#[test]
fn a_picture_is_an_image_on_a_page_frame() {
    let html = icon(
        FileIconChoice::Picture(ImageSource("data:image/png;base64,AAAA".to_owned())),
        64,
        FileIconStyle::Colour,
    );
    for part in [
        "data-choice=\"picture\"",
        "<img",
        "src=\"data:image/png;base64,AAAA\"",
        "ds-file-photo",
    ] {
        assert!(html.contains(part), "{part}: {html}");
    }
}

#[test]
fn artwork_is_an_image_drawn_whole_with_no_page_frame() {
    let html = icon(
        FileIconChoice::Artwork(ImageSource("file:///packs/pdf.svg".to_owned())),
        64,
        FileIconStyle::Colour,
    );
    for part in [
        "data-choice=\"artwork\"",
        "<img",
        "src=\"file:///packs/pdf.svg\"",
        "ds-file-artwork",
    ] {
        assert!(html.contains(part), "{part}: {html}");
    }
    assert!(
        !html.contains("ds-file-photo") && !html.contains("ds-file-page"),
        "{html}"
    );
}

#[test]
fn monochrome_writes_the_tinted_folder_and_marks_and_colour_writes_nothing() {
    let tint = Tint {
        hue: 150.0,
        chroma: 0.05,
    };
    let mono = icon(
        FileIconChoice::Folder(FolderKind::Home),
        64,
        FileIconStyle::Monochrome(tint),
    );
    assert!(mono.contains("data-icon-style=\"monochrome\""), "{mono}");
    for var in [
        "--file-tab-l:",
        "--file-tab-d:",
        "--file-body-l:",
        "--file-body-d:",
        "--file-ink-l:",
        "--file-ink-d:",
        "--file-light-l:",
        "--file-light-d:",
        "--file-mark:",
    ] {
        assert!(mono.contains(var), "{var}: {mono}");
    }
    let colour = icon(
        FileIconChoice::Folder(FolderKind::Home),
        64,
        FileIconStyle::Colour,
    );
    assert!(
        !colour.contains("data-icon-style") && !colour.contains("--file-tab"),
        "{colour}"
    );
}

#[test]
fn the_stylesheet_styles_every_kind_and_declares_the_tokens() {
    let sheet = ds::stylesheet();
    for kind in FileKind::ALL.iter().copied() {
        let slug = kind.slug();
        assert!(sheet.contains(&format!("data-kind={slug}]")), "kind {slug}");
        assert!(
            sheet.contains(&format!("--file-kind-{slug}:")),
            "token {slug}"
        );
    }
    for token in [
        "--file-fold:",
        "--file-edge:",
        "--file-label:",
        "--file-folder-tab:",
        "--file-folder-body:",
        "--file-folder-ink:",
        "--file-folder-light:",
    ] {
        assert!(sheet.contains(token), "{token}");
    }
}
