//! The File icons page: the standard file-icon set (`ds::FileIcon`), every kind with and without
//! its extension label, every folder, a picture, and the monochrome style, at 16, 32, 64 and
//! 128 px, in the light scheme and the dark. The page stays light in the dark and the folder
//! takes its dark tones.

use crate::pages::content::picture::wide_image;
use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::prelude::*;
use ds::style::icon::render::IconPx;
use ds::style::icon::retint::{IconStyle, Tint};
use ds::style::space::presets::PRESETS;

/// The sizes every specimen is drawn at.
const SIZES: [u8; 4] = [16, 32, 64, 128];

/// An extension each kind's page carries in the specimens.
fn example(kind: FileKind) -> &'static str {
    match kind {
        FileKind::Document => "txt",
        FileKind::Pdf => "pdf",
        FileKind::Spreadsheet => "xlsx",
        FileKind::Presentation => "pptx",
        FileKind::Image => "png",
        FileKind::Audio => "mp3",
        FileKind::Video => "mp4",
        FileKind::Archive => "zip",
        FileKind::Code => "rs",
        FileKind::Font => "ttf",
        FileKind::DiskImage => "iso",
        FileKind::Program => "run",
        _ => "xyz",
    }
}

/// One row of `choice` at every size, bottom-aligned.
fn ladder(choice: FileIconChoice, style: FileIconStyle) -> Element {
    rsx! {
        div { class: "g-row",
            for px in SIZES {
                FileIcon { choice: choice.clone(), size: IconSize::Px(IconPx(px)), style }
            }
        }
    }
}

/// Every kind, once with its label and once without.
fn kinds(style: FileIconStyle) -> Element {
    rsx! {
        div { class: "g-row g-row-top g-stage-pad",
            for kind in FileKind::ALL.iter().copied() {
                Specimen { name: kind.slug().to_owned(),
                    {ladder(FileIconChoice::Standard { kind, label: ExtLabel::from_extension(example(kind)) }, style)}
                    {ladder(FileIconChoice::Standard { kind, label: None }, style)}
                }
            }
        }
    }
}

/// Every folder.
fn folders(style: FileIconStyle) -> Element {
    rsx! {
        div { class: "g-row g-row-top g-stage-pad",
            for kind in FolderKind::ALL.iter().copied() {
                Specimen { name: kind.slug().to_owned(), {ladder(FileIconChoice::Folder(kind), style)} }
            }
        }
    }
}

/// The page.
#[component]
pub fn FileIconsPage() -> Element {
    let work = FileIconStyle::of(IconStyle::Monochrome, Tint::space(PRESETS[0].dots));
    rsx! {
        for scheme in Scheme::ALL.iter().copied() {
            Section { title: format!("Kinds ({})", scheme.slug()), note: "FileIcon {{ choice: Standard {{ kind, label }} }}: a white page with a folded corner, the kind's mark in its colour and the extension in a small bold face. First row with the label, second without. The label is dropped under 48 px and the mark grows.",
                Surface { material: Material::Popover, theme: Some(scheme), {kinds(FileIconStyle::Colour)} }
            }
            Section { title: format!("Folders ({})", scheme.slug()), note: "FileIcon {{ choice: Folder(kind) }}: the Mac folder in two tones of blue, a special folder with its glyph embossed on the front. The tones follow the scheme; the page above does not.",
                Surface { material: Material::Popover, theme: Some(scheme), {folders(FileIconStyle::Colour)} }
            }
        }
        Section { title: "Picture", note: "FileIcon {{ choice: Picture(source) }}: the file's own thumbnail cropped to a page-shaped frame with the page's edge and shadow.",
            Surface { material: Material::Popover, theme: Some(Scheme::Light),
                div { class: "g-row g-row-top g-stage-pad",
                    {ladder(FileIconChoice::Picture(wide_image()), FileIconStyle::Colour)}
                }
            }
        }
        for scheme in Scheme::ALL.iter().copied() {
            Section { title: format!("Monochrome ({})", scheme.slug()), note: "icons.style = monochrome with the Work Space's tint: the folder and the marks take one hue, as the app icons do.",
                Surface { material: Material::Popover, theme: Some(scheme),
                    {kinds(work)}
                    {folders(work)}
                }
            }
        }
    }
}
