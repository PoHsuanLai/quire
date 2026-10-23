//! Solid forms of the file-manager glyphs (Home, the drives, Eject, the per-type file glyphs):
//! filled paths on the 24 grid, no stroke (design/08-ICONS.md section 1.2).
//!
//! Each constant says where its geometry comes from. Tabler and Phosphor are MIT, notices in
//! `assets/icons/LICENSE-tabler.txt` and `LICENSE-phosphor.txt`; a Phosphor path is scaled by
//! 3/32 from its 256 grid and rounded to three decimals.

use super::Icon;
use super::shape::Shape;

/// The solid children of a glyph this file holds; empty for any other.
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

/// quire's own `DriveExternal` as fills: the drive's body and the connector plug on top.
const DRIVE_EXTERNAL: &[Shape] = &[
    Shape::Solid("M4 11h16a2 2 0 0 1 2 2v6a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2v-6a2 2 0 0 1 2-2z"),
    Shape::Solid("M9 4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v7H9z"),
];

/// Tabler Icons 3.48.0 `filled/home` (MIT), verbatim.
const HOME: &[Shape] = &[Shape::Solid(
    "M12.707 2.293l9 9c.63 .63 .184 1.707 -.707 1.707h-1v6a3 3 0 0 1 -3 3h-1v-7a3 3 0 0 0 -2.824 -2.995l-.176 -.005h-2a3 3 0 0 0 -3 3v7h-1a3 3 0 0 1 -3 -3v-6h-1c-.89 0 -1.337 -1.077 -.707 -1.707l9 -9a1 1 0 0 1 1.414 0m.293 11.707a1 1 0 0 1 1 1v7h-4v-7a1 1 0 0 1 .883 -.993l.117 -.007z",
)];

/// Phosphor Icons 2.1.1 Fill `eject-fill` (MIT), scaled from its 256 grid to 24.
const EJECT: &[Shape] = &[Shape::Solid(
    "M3.123 11.835a1.462 1.462 0 0 1 0.291 -1.605h0l6.912 -7.267a2.321 2.321 0 0 1 3.349 0l6.912 7.267a1.462 1.462 0 0 1 0.291 1.605A1.517 1.517 0 0 1 19.477 12.75H4.522A1.517 1.517 0 0 1 3.123 11.835ZM19.5 14.25H4.5a1.5 1.5 0 0 0 -1.5 1.5v1.5a1.5 1.5 0 0 0 1.5 1.5H19.5a1.5 1.5 0 0 0 1.5 -1.5V15.75A1.5 1.5 0 0 0 19.5 14.25Z",
)];

/// Phosphor Icons 2.1.1 Fill `hard-drive-fill` (MIT), scaled from its 256 grid to 24.
const DRIVE_INTERNAL: &[Shape] = &[Shape::Solid(
    "M21 6H3A1.5 1.5 0 0 0 1.5 7.5v9a1.5 1.5 0 0 0 1.5 1.5H21a1.5 1.5 0 0 0 1.5 -1.5V7.5A1.5 1.5 0 0 0 21 6Zm-3.375 7.125a1.125 1.125 0 1 1 1.125 -1.125A1.125 1.125 0 0 1 17.625 13.125Z",
)];

/// Tabler Icons 3.48.0 `filled/disc` (MIT), verbatim.
const DISC: &[Shape] = &[Shape::Solid(
    "M17 3.34a10 10 0 1 1 -15 8.66l.005 -.324a10 10 0 0 1 14.995 -8.336m0 7.66a1 1 0 0 0 -1 1a4 4 0 0 1 -4 4a1 1 0 0 0 0 2a6 6 0 0 0 6 -6a1 1 0 0 0 -1 -1m-5 -1a2 2 0 0 0 -1.995 1.85l-.005 .15a2 2 0 1 0 2 -2m0 -4a6 6 0 0 0 -6 6a1 1 0 0 0 2 0a4 4 0 0 1 4 -4a1 1 0 0 0 0 -2",
)];

/// Tabler Icons 3.48.0 `filled/file-text` (MIT), verbatim.
const FILE_TEXT: &[Shape] = &[
    Shape::Solid(
        "M12 2l.117 .007a1 1 0 0 1 .876 .876l.007 .117v4l.005 .15a2 2 0 0 0 1.838 1.844l.157 .006h4l.117 .007a1 1 0 0 1 .876 .876l.007 .117v9a3 3 0 0 1 -2.824 2.995l-.176 .005h-10a3 3 0 0 1 -2.995 -2.824l-.005 -.176v-14a3 3 0 0 1 2.824 -2.995l.176 -.005zm3 14h-6a1 1 0 0 0 0 2h6a1 1 0 0 0 0 -2m0 -4h-6a1 1 0 0 0 0 2h6a1 1 0 0 0 0 -2m-5 -4h-1a1 1 0 1 0 0 2h1a1 1 0 0 0 0 -2",
    ),
    Shape::Solid("M19 7h-4l-.001 -4.001z"),
];

/// Tabler Icons 3.48.0 `filled/file-code` (MIT), verbatim.
const FILE_CODE: &[Shape] = &[
    Shape::Solid(
        "M12 2l.117 .007a1 1 0 0 1 .876 .876l.007 .117v4l.005 .15a2 2 0 0 0 1.838 1.844l.157 .006h4l.117 .007a1 1 0 0 1 .876 .876l.007 .117v9a3 3 0 0 1 -2.824 2.995l-.176 .005h-10a3 3 0 0 1 -2.995 -2.824l-.005 -.176v-14a3 3 0 0 1 2.824 -2.995l.176 -.005zm-1.553 11.106a1 1 0 0 0 -1.341 .447l-1 2a1 1 0 0 0 0 .894l1 2a1 1 0 0 0 1.341 .447l.102 -.058a1 1 0 0 0 .345 -1.283l-.776 -1.553l.776 -1.553a1 1 0 0 0 -.447 -1.341m4.447 .447a1 1 0 0 0 -1.341 -.447l-.102 .058a1 1 0 0 0 -.345 1.283l.774 1.553l-.775 1.553a1 1 0 0 0 1.788 .894l1 -2a1 1 0 0 0 0 -.894z",
    ),
    Shape::Solid("M19 7h-4l-.001 -4.001z"),
];

/// Tabler Icons 3.48.0 `filled/file-music` (MIT), verbatim.
const FILE_AUDIO: &[Shape] = &[
    Shape::Solid(
        "M12 2l.117 .007a1 1 0 0 1 .876 .876l.007 .117v4l.005 .15a2 2 0 0 0 1.838 1.844l.157 .006h4l.117 .007a1 1 0 0 1 .876 .876l.007 .117v9a3 3 0 0 1 -2.824 2.995l-.176 .005h-10a3 3 0 0 1 -2.995 -2.824l-.005 -.176v-14a3 3 0 0 1 2.824 -2.995l.176 -.005zm.447 9.106a1 1 0 0 0 -1.447 .894v3a2 2 0 0 0 -1.995 1.85l-.005 .15a2 2 0 1 0 4 0v-3.382l.553 .276a1 1 0 0 0 .894 -1.788z",
    ),
    Shape::Solid("M19 7h-4l-.001 -4.001z"),
];

/// Tabler Icons 3.48.0 `filled/file-settings` (MIT), verbatim.
const FILE_APP: &[Shape] = &[
    Shape::Solid(
        "M12 2l.117 .007a1 1 0 0 1 .876 .876l.007 .117v4l.005 .15a2 2 0 0 0 1.838 1.844l.157 .006h4l.117 .007a1 1 0 0 1 .876 .876l.007 .117v9a3 3 0 0 1 -2.824 2.995l-.176 .005h-10a3 3 0 0 1 -2.995 -2.824l-.005 -.176v-14a3 3 0 0 1 2.824 -2.995l.176 -.005zm0 8.5a1 1 0 0 0 -1 1v.67a3 3 0 0 0 -.909 .516l-.576 -.346a1 1 0 0 0 -1.03 1.714l.575 .346q -.043 .207 -.055 .424l-.005 .176q 0 .28 .05 .548l-.582 .336a1 1 0 0 0 1 1.732l.583 -.336c.277 .238 .598 .425 .95 .55l-.001 .67a1 1 0 0 0 2 0v-.671c.335 -.118 .641 -.294 .909 -.514l.576 .345a1 1 0 0 0 1.03 -1.714l-.575 -.346a3 3 0 0 0 .01 -1.148l.581 -.336a1 1 0 0 0 -1 -1.732l-.582 .335a3 3 0 0 0 -.948 -.548l-.001 -.671a1 1 0 0 0 -1 -1m0 3.5a1 1 0 1 1 0 2a1 1 0 0 1 0 -2",
    ),
    Shape::Solid("M19 7h-4l-.001 -4.001z"),
];

/// Phosphor Icons 2.1.1 Fill `file-image-fill` (MIT), scaled from its 256 grid to 24.
const FILE_IMAGE: &[Shape] = &[Shape::Solid(
    "M14.874 20.584A0.75 0.75 0 0 1 14.25 21.75H2.25a0.75 0.75 0 0 1 -0.631 -1.156l3.375 -5.25a0.75 0.75 0 0 1 1.262 0l0.915 1.423 1.955 -2.933a0.75 0.75 0 0 1 1.249 0ZM20.25 8.25V20.25a1.5 1.5 0 0 1 -1.5 1.5h-0.75a0.75 0.75 0 0 1 0 -1.5h0.75V9H14.25a0.75 0.75 0 0 1 -0.75 -0.75V3.75H5.25v8.25a0.75 0.75 0 0 1 -1.5 0V3.75A1.5 1.5 0 0 1 5.25 2.25h9a0.75 0.75 0 0 1 0.531 0.219l5.25 5.25A0.75 0.75 0 0 1 20.25 8.25Zm-5.25 -0.75h2.69L15 4.81Z",
)];

/// Phosphor Icons 2.1.1 Fill `file-video-fill` (MIT), scaled from its 256 grid to 24.
const FILE_VIDEO: &[Shape] = &[Shape::Solid(
    "M20.031 7.719l-5.25 -5.25A0.75 0.75 0 0 0 14.25 2.25H5.25A1.5 1.5 0 0 0 3.75 3.75v6.75a0.75 0.75 0 0 0 1.5 0V3.75h8.25V8.25a0.75 0.75 0 0 0 0.75 0.75h4.5V20.25h-0.75a0.75 0.75 0 0 0 0 1.5h0.75a1.5 1.5 0 0 0 1.5 -1.5V8.25A0.75 0.75 0 0 0 20.031 7.719ZM15 4.81 17.69 7.5H15ZM14.614 13.594a0.75 0.75 0 0 0 -0.761 0.021l-1.87 1.168A1.5 1.5 0 0 0 10.5 13.5H4.5a1.5 1.5 0 0 0 -1.5 1.5v4.5a1.5 1.5 0 0 0 1.5 1.5h6a1.5 1.5 0 0 0 1.482 -1.282l1.87 1.168A0.75 0.75 0 0 0 15 20.25V14.25A0.75 0.75 0 0 0 14.614 13.594ZM13.5 18.897l-1.5 -0.938V16.54l1.5 -0.938Z",
)];

/// Phosphor Icons 2.1.1 Fill `file-zip-fill` (MIT), scaled from its 256 grid to 24.
const FILE_ARCHIVE: &[Shape] = &[Shape::Solid(
    "M17.25 13.5H15.75a0.75 0.75 0 0 0 -0.75 0.75v5.225a0.766 0.766 0 0 0 0.7 0.773 0.75 0.75 0 0 0 0.8 -0.75v-0.75h0.694c1.429 0 2.638 -1.117 2.68 -2.545A2.625 2.625 0 0 0 17.25 13.5Zm-0.033 3.75H16.5V15h0.75A1.125 1.125 0 0 1 18.375 16.234 1.148 1.148 0 0 1 17.217 17.25ZM12.75 14.25v5.225a0.766 0.766 0 0 1 -0.7 0.773 0.75 0.75 0 0 1 -0.8 -0.75V14.275a0.766 0.766 0 0 1 0.7 -0.773A0.75 0.75 0 0 1 12.75 14.25ZM9 19.55A0.766 0.766 0 0 1 8.225 20.25H5.272a0.775 0.775 0 0 1 -0.562 -0.234A0.75 0.75 0 0 1 4.598 19.125l2.359 -4.125H5.275A0.766 0.766 0 0 1 4.5 14.3 0.75 0.75 0 0 1 5.25 13.5H8.228a0.775 0.775 0 0 1 0.562 0.234A0.75 0.75 0 0 1 8.906 14.625L6.543 18.75H8.25A0.75 0.75 0 0 1 9 19.55ZM20.031 7.719l-5.25 -5.25A0.75 0.75 0 0 0 14.25 2.25H5.25A1.5 1.5 0 0 0 3.75 3.75v7.125a0.375 0.375 0 0 0 0.375 0.375H19.875a0.375 0.375 0 0 0 0.375 -0.375V8.25A0.75 0.75 0 0 0 20.031 7.719ZM14.25 8.25V4.125l4.125 4.125Z",
)];

/// Phosphor Icons 2.1.1 Fill `file-pdf-fill` (MIT), scaled from its 256 grid to 24.
const FILE_PDF: &[Shape] = &[Shape::Solid(
    "M4.125 11.25H19.875a0.375 0.375 0 0 0 0.375 -0.375V8.25a0.75 0.75 0 0 0 -0.219 -0.531l-5.25 -5.25A0.75 0.75 0 0 0 14.25 2.25H5.25A1.5 1.5 0 0 0 3.75 3.75v7.125A0.375 0.375 0 0 0 4.125 11.25ZM14.25 4.125l4.125 4.125H14.25Zm6.75 10.175a0.767 0.767 0 0 1 -0.773 0.7H18v1.5h1.475a0.766 0.766 0 0 1 0.773 0.7 0.75 0.75 0 0 1 -0.75 0.8H18v1.475a0.766 0.766 0 0 1 -0.7 0.773 0.75 0.75 0 0 1 -0.8 -0.75V14.25a0.75 0.75 0 0 1 0.75 -0.75h3A0.75 0.75 0 0 1 21 14.3ZM6 13.5H4.5a0.75 0.75 0 0 0 -0.75 0.75v5.225A0.766 0.766 0 0 0 4.45 20.25 0.75 0.75 0 0 0 5.25 19.5v-0.75h0.694c1.429 0 2.638 -1.117 2.68 -2.545A2.625 2.625 0 0 0 6 13.5Zm-0.033 3.75H5.25V15h0.75a1.125 1.125 0 0 1 1.125 1.234A1.148 1.148 0 0 1 5.967 17.25ZM12 13.5H10.5a0.75 0.75 0 0 0 -0.75 0.75v5.25a0.75 0.75 0 0 0 0.75 0.75h1.436c1.843 0 3.395 -1.451 3.438 -3.293A3.375 3.375 0 0 0 12 13.5Zm-0.046 5.25H11.25V15h0.75a1.875 1.875 0 0 1 1.875 1.947C13.836 17.962 12.969 18.75 11.954 18.75Z",
)];

/// `Lines` as fills: three capsules 2 tall on the 24 grid, 18, 12 and 14 long.
const LINES: &[Shape] = &[
    Shape::Solid("M4 5h16a1 1 0 0 1 0 2H4a1 1 0 0 1 0-2z"),
    Shape::Solid("M4 11h10a1 1 0 0 1 0 2H4a1 1 0 0 1 0-2z"),
    Shape::Solid("M4 17h12a1 1 0 0 1 0 2H4a1 1 0 0 1 0-2z"),
];

/// `Pdf` as one filled letter P: the outer contour, then the counter wound the other way.
const PDF: &[Shape] = &[Shape::Solid(
    "M8 4h5.5a4.25 4.25 0 0 1 0 8.5H10.5V20H8zM10.5 6.5V10h3a1.75 1.75 0 0 0 0-3.5z",
)];

/// `Puzzle` as one filled piece: Lucide's outline, filled.
const PUZZLE: &[Shape] = &[Shape::Solid(
    "M15.39 4.39a1 1 0 0 0 1.68-.474 2.5 2.5 0 1 1 3.014 3.015 1 1 0 0 0-.474 1.68l1.683 1.682a2.414 2.414 0 0 1 0 3.414L19.61 15.39a1 1 0 0 1-1.68-.474 2.5 2.5 0 1 0-3.014 3.015 1 1 0 0 1 .474 1.68l-1.683 1.682a2.414 2.414 0 0 1-3.414 0L8.61 19.61a1 1 0 0 0-1.68.474 2.5 2.5 0 1 1-3.014-3.015 1 1 0 0 0 .474-1.68l-1.683-1.682a2.414 2.414 0 0 1 0-3.414L4.39 8.61a1 1 0 0 1 1.68.474 2.5 2.5 0 1 0 3.014-3.015 1 1 0 0 1-.474-1.68l1.683-1.682a2.414 2.414 0 0 1 3.414 0z",
)];
