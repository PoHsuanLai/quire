//! Geometry for a file manager's glyphs: Home, the drives, Eject and the per-type file glyphs.
//! Lucide, transcribed from `lucide-static` 1.47.0
//! (<https://unpkg.com/lucide-static@1.47.0/icons/>), ISC licence; see `super`, except the
//! marks Lucide has no icon for, which are quire's own on Lucide's grid (24, a 2 px round
//! stroke, no fills):
//!
//! - `DriveExternal`: a drive's flat body (a pill-cornered box 20 x 10 with an activity dot)
//!   and a connector plug standing on it, so it reads as a drive on a cable and not as
//!   `DriveInternal`'s slanted case. Lucide's `usb` is the trident symbol and says "USB", not
//!   "drive"; `hard-drive-download` carries a download arrow.
//! - `FilePdf`: Lucide's `file` with a letter P inside (a stem and a bowl); Lucide has no PDF
//!   file icon.
//! - `Pdf`: the letter P of `FilePdf` without its page (a stem and a bowl), the mark of a PDF
//!   on the standard file icon.
//! - `Lines`: Lucide `align-left`, the mark of a document on the standard file icon.
//! - `Puzzle`: Lucide `puzzle`, the Extensions page's glyph.
//!
//! A Lucide `<line>` is written as the equivalent path, as in `geometry_shell`.

use super::Icon;
use super::shape::Shape;

/// The outline children of a file-manager glyph; empty for any other, which `Icon::shapes`
/// never sends here.
pub(super) fn shapes(icon: Icon) -> &'static [Shape] {
    match icon {
        Icon::Home => HOME,
        Icon::Eject => EJECT,
        Icon::DriveInternal => DRIVE_INTERNAL,
        Icon::DriveExternal => DRIVE_EXTERNAL,
        Icon::Disc => DISC,
        Icon::FileText => FILE_TEXT,
        Icon::FileImage => FILE_IMAGE,
        Icon::FileAudio => FILE_AUDIO,
        Icon::FileVideo => FILE_VIDEO,
        Icon::FileArchive => FILE_ARCHIVE,
        Icon::FileCode => FILE_CODE,
        Icon::FilePdf => FILE_PDF,
        Icon::FileApp => FILE_APP,
        Icon::Lines => LINES,
        Icon::Pdf => PDF,
        Icon::Puzzle => PUZZLE,
        _ => &[],
    }
}

/// Lucide `house`.
const HOME: &[Shape] = &[
    Shape::Path("M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"),
    Shape::Path(
        "M3 10a2 2 0 0 1 .709-1.528l7-6a2 2 0 0 1 2.582 0l7 6A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z",
    ),
];

/// Lucide `eject`: a triangle over a bar.
const EJECT: &[Shape] = &[
    Shape::Path(
        "M4 13a1 1 0 0 1-.72-1.695l7.257-7.668a2 2 0 0 1 2.926 0l7.256 7.668A1 1 0 0 1 20 13z",
    ),
    Shape::Rect {
        x: "3",
        y: "17",
        width: "18",
        height: "4",
        rx: "1",
    },
];

/// Lucide `hard-drive`: a drive inside the machine.
const DRIVE_INTERNAL: &[Shape] = &[
    Shape::Path("M10 16h.01"),
    Shape::Path(
        "M2.212 11.577a2 2 0 0 0-.212.896V18a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-5.527a2 2 0 0 0-.212-.896L18.55 5.11A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z",
    ),
    Shape::Path("M21.946 12.013H2.054"),
    Shape::Path("M6 16h.01"),
];

/// quire's own: a flat drive body with a connector plug standing on it.
const DRIVE_EXTERNAL: &[Shape] = &[
    Shape::Rect {
        x: "2",
        y: "11",
        width: "20",
        height: "10",
        rx: "2",
    },
    Shape::Path("M6 16h.01"),
    Shape::Path("M9 11V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v7"),
];

/// Lucide `disc`: an optical disc.
const DISC: &[Shape] = &[
    Shape::Circle {
        cx: "12",
        cy: "12",
        r: "10",
    },
    Shape::Circle {
        cx: "12",
        cy: "12",
        r: "2",
    },
];

/// Lucide `file-text`.
const FILE_TEXT: &[Shape] = &[
    Shape::Path(
        "M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z",
    ),
    Shape::Path("M14 2v5a1 1 0 0 0 1 1h5"),
    Shape::Path("M10 9H8"),
    Shape::Path("M16 13H8"),
    Shape::Path("M16 17H8"),
];

/// Lucide `file-image`.
const FILE_IMAGE: &[Shape] = &[
    Shape::Path(
        "M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z",
    ),
    Shape::Path("M14 2v5a1 1 0 0 0 1 1h5"),
    Shape::Circle {
        cx: "10",
        cy: "12",
        r: "2",
    },
    Shape::Path("m20 17-1.296-1.296a2.41 2.41 0 0 0-3.408 0L9 22"),
];

/// Lucide `file-audio`.
const FILE_AUDIO: &[Shape] = &[
    Shape::Path(
        "M4 6.835V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.706.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2h-.343",
    ),
    Shape::Path("M14 2v5a1 1 0 0 0 1 1h5"),
    Shape::Path(
        "M2 19a2 2 0 0 1 4 0v1a2 2 0 0 1-4 0v-4a6 6 0 0 1 12 0v4a2 2 0 0 1-4 0v-1a2 2 0 0 1 4 0",
    ),
];

/// Lucide `file-video`.
const FILE_VIDEO: &[Shape] = &[
    Shape::Path(
        "M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z",
    ),
    Shape::Path("M14 2v5a1 1 0 0 0 1 1h5"),
    Shape::Path(
        "M15.033 13.44a.647.647 0 0 1 0 1.12l-4.065 2.352a.645.645 0 0 1-.968-.56v-4.704a.645.645 0 0 1 .967-.56z",
    ),
];

/// Lucide `file-archive`.
const FILE_ARCHIVE: &[Shape] = &[
    Shape::Path(
        "M13.659 22H18a2 2 0 0 0 2-2V8a2.4 2.4 0 0 0-.706-1.706l-3.588-3.588A2.4 2.4 0 0 0 14 2H6a2 2 0 0 0-2 2v11.5",
    ),
    Shape::Path("M14 2v5a1 1 0 0 0 1 1h5"),
    Shape::Path("M8 12v-1"),
    Shape::Path("M8 18v-2"),
    Shape::Path("M8 7V6"),
    Shape::Circle {
        cx: "8",
        cy: "20",
        r: "2",
    },
];

/// Lucide `file-code`.
const FILE_CODE: &[Shape] = &[
    Shape::Path(
        "M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z",
    ),
    Shape::Path("M14 2v5a1 1 0 0 0 1 1h5"),
    Shape::Path("M10 12.5 8 15l2 2.5"),
    Shape::Path("m14 12.5 2 2.5-2 2.5"),
];

/// quire's own: Lucide `file` with a letter P inside.
const FILE_PDF: &[Shape] = &[
    Shape::Path(
        "M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z",
    ),
    Shape::Path("M14 2v5a1 1 0 0 0 1 1h5"),
    Shape::Path("M9 18v-6h2a2 2 0 0 1 0 4H9"),
];

/// Lucide `file-cog`: a program.
const FILE_APP: &[Shape] = &[
    Shape::Path("M15 8a1 1 0 0 1-1-1V2a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8z"),
    Shape::Path("M20 8v12a2 2 0 0 1-2 2h-4.182"),
    Shape::Path("m3.305 19.53.923-.382"),
    Shape::Path("M4 10.592V4a2 2 0 0 1 2-2h8"),
    Shape::Path("m4.228 16.852-.924-.383"),
    Shape::Path("m5.852 15.228-.383-.923"),
    Shape::Path("m5.852 20.772-.383.924"),
    Shape::Path("m8.148 15.228.383-.923"),
    Shape::Path("m8.53 21.696-.382-.924"),
    Shape::Path("m9.773 16.852.922-.383"),
    Shape::Path("m9.773 19.148.922.383"),
    Shape::Circle {
        cx: "7",
        cy: "18",
        r: "3",
    },
];

/// Lucide `align-left`: three lines of text.
const LINES: &[Shape] = &[
    Shape::Path("M21 6H3"),
    Shape::Path("M15 12H3"),
    Shape::Path("M17 18H3"),
];

/// quire's own: the letter P of `FILE_PDF`, alone.
const PDF: &[Shape] = &[Shape::Path("M9 20V4h4.5a4 4 0 0 1 0 8H9")];

/// Lucide `puzzle`: one piece with a knob on each side.
const PUZZLE: &[Shape] = &[Shape::Path(
    "M15.39 4.39a1 1 0 0 0 1.68-.474 2.5 2.5 0 1 1 3.014 3.015 1 1 0 0 0-.474 1.68l1.683 1.682a2.414 2.414 0 0 1 0 3.414L19.61 15.39a1 1 0 0 1-1.68-.474 2.5 2.5 0 1 0-3.014 3.015 1 1 0 0 1 .474 1.68l-1.683 1.682a2.414 2.414 0 0 1-3.414 0L8.61 19.61a1 1 0 0 0-1.68.474 2.5 2.5 0 1 1-3.014-3.015 1 1 0 0 0 .474-1.68l-1.683-1.682a2.414 2.414 0 0 1 0-3.414L4.39 8.61a1 1 0 0 1 1.68.474 2.5 2.5 0 1 0 3.014-3.015 1 1 0 0 1-.474-1.68l1.683-1.682a2.414 2.414 0 0 1 3.414 0z",
)];
