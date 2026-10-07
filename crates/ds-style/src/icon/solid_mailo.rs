//! Solid forms of the mailo set (design/08-ICONS.md section 1.1): filled paths on the 24 grid, no stroke (design/08-ICONS.md section 1.2).
//!
//! Each constant says where its geometry comes from. Tabler and Phosphor are MIT, notices in
//! `assets/icons/LICENSE-tabler.txt` and `LICENSE-phosphor.txt`; a Phosphor path is scaled by
//! 3/32 from its 256 grid and rounded to three decimals.

use super::Icon;
use super::shape::Shape;

/// The solid children of a glyph this file holds; empty for any other.
pub(super) fn shapes(icon: Icon) -> &'static [Shape] {
    match icon {
        Icon::Inbox => INBOX,
        Icon::Star => STAR,
        Icon::Archive => ARCHIVE,
        Icon::Clock => CLOCK,
        Icon::Trash => TRASH,
        Icon::Mail => MAIL,
        Icon::MailOpen => MAIL_OPEN,
        Icon::Tag => TAG,
        Icon::Refresh => REFRESH,
        Icon::Send => SEND,
        Icon::Command => COMMAND,
        Icon::Columns => COLUMNS,
        Icon::Group => GROUP,
        Icon::Panel => PANEL,
        Icon::Square => SQUARE,
        Icon::Maximize => MAXIMIZE,
        Icon::Corner => CORNER,
        Icon::Undo => UNDO,
        Icon::Check => CHECK,
        Icon::X => X,
        Icon::Paperclip => PAPERCLIP,
        Icon::Pen => PEN,
        Icon::Key => KEY,
        Icon::FilePen => FILE_PEN,
        Icon::OctagonAlert => OCTAGON_ALERT,
        Icon::Pin => PIN,
        Icon::Reply => REPLY,
        Icon::ReplyAll => REPLY_ALL,
        Icon::Forward => FORWARD,
        Icon::Search => SEARCH,
        Icon::Settings => SETTINGS,
        Icon::PanelLeft => PANEL_LEFT,
        Icon::Plus => PLUS,
        Icon::Minus => MINUS,
        _ => &[],
    }
}

/// Phosphor Icons 2.1.1 Fill `tray-fill` (MIT), scaled from its 256 grid to 24.
const INBOX: &[Shape] = &[Shape::Solid(
    "M19.5 3 H4.5 A1.5 1.5 0 0 0 3 4.5 V19.5 a1.5 1.5 0 0 0 1.5 1.5 H19.5 a1.5 1.5 0 0 0 1.5 -1.5 V4.5 A1.5 1.5 0 0 0 19.5 3 Z m0 16.5 H4.5 V15.75 H7.19 L9 17.561 A1.49 1.49 0 0 0 10.06 18 h3.879 A1.487 1.487 0 0 0 15 17.56 L16.81 15.75 H19.5 v3.75 Z",
)];

/// Tabler Icons 3.48.0 `filled/star` (MIT), verbatim.
const STAR: &[Shape] = &[Shape::Solid(
    "M8.243 7.34l-6.38 .925l-.113 .023a1 1 0 0 0 -.44 1.684l4.622 4.499l-1.09 6.355l-.013 .11a1 1 0 0 0 1.464 .944l5.706 -3l5.693 3l.1 .046a1 1 0 0 0 1.352 -1.1l-1.091 -6.355l4.624 -4.5l.078 -.085a1 1 0 0 0 -.633 -1.62l-6.38 -.926l-2.852 -5.78a1 1 0 0 0 -1.794 0l-2.853 5.78z",
)];

/// Tabler Icons 3.48.0 `filled/archive` (MIT), verbatim.
const ARCHIVE: &[Shape] = &[
    Shape::Solid("M2 5a2 2 0 0 1 2 -2h16a2 2 0 0 1 2 2a2 2 0 0 1 -2 2h-16a2 2 0 0 1 -2 -2z"),
    Shape::Solid(
        "M19 9c.513 0 .936 .463 .993 1.06l.007 .14v7.2c0 1.917 -1.249 3.484 -2.824 3.594l-.176 .006h-10c-1.598 0 -2.904 -1.499 -2.995 -3.388l-.005 -.212v-7.2c0 -.663 .448 -1.2 1 -1.2h14zm-5 2h-4l-.117 .007a1 1 0 0 0 0 1.986l.117 .007h4l.117 -.007a1 1 0 0 0 0 -1.986l-.117 -.007z",
    ),
];

/// Tabler Icons 3.48.0 `filled/clock` (MIT), verbatim.
const CLOCK: &[Shape] = &[Shape::Solid(
    "M17 3.34a10 10 0 1 1 -14.995 8.984l-.005 -.324l.005 -.324a10 10 0 0 1 14.995 -8.336zm-5 2.66a1 1 0 0 0 -.993 .883l-.007 .117v5l.009 .131a1 1 0 0 0 .197 .477l.087 .1l3 3l.094 .082a1 1 0 0 0 1.226 0l.094 -.083l.083 -.094a1 1 0 0 0 0 -1.226l-.083 -.094l-2.707 -2.708v-4.585l-.007 -.117a1 1 0 0 0 -.993 -.883z",
)];

/// Tabler Icons 3.48.0 `filled/trash` (MIT), verbatim.
const TRASH: &[Shape] = &[
    Shape::Solid(
        "M20 6a1 1 0 0 1 .117 1.993l-.117 .007h-.081l-.919 11a3 3 0 0 1 -2.824 2.995l-.176 .005h-8c-1.598 0 -2.904 -1.249 -2.992 -2.75l-.005 -.167l-.923 -11.083h-.08a1 1 0 0 1 -.117 -1.993l.117 -.007zm-10 4a1 1 0 0 0 -1 1v6a1 1 0 0 0 2 0v-6a1 1 0 0 0 -1 -1m4 0a1 1 0 0 0 -1 1v6a1 1 0 0 0 2 0v-6a1 1 0 0 0 -1 -1",
    ),
    Shape::Solid(
        "M14 2a2 2 0 0 1 2 2a1 1 0 0 1 -1.993 .117l-.007 -.117h-4l-.007 .117a1 1 0 0 1 -1.993 -.117a2 2 0 0 1 1.85 -1.995l.15 -.005z",
    ),
];

/// Tabler Icons 3.48.0 `filled/mail` (MIT), verbatim.
const MAIL: &[Shape] = &[
    Shape::Solid(
        "M22 7.535v9.465a3 3 0 0 1 -2.824 2.995l-.176 .005h-14a3 3 0 0 1 -2.995 -2.824l-.005 -.176v-9.465l9.445 6.297l.116 .066a1 1 0 0 0 .878 0l.116 -.066l9.445 -6.297z",
    ),
    Shape::Solid(
        "M19 4c1.08 0 2.027 .57 2.555 1.427l-9.555 6.37l-9.555 -6.37a2.999 2.999 0 0 1 2.354 -1.42l.201 -.007h14z",
    ),
];

/// Tabler Icons 3.48.0 `filled/mail-opened` (MIT), verbatim.
const MAIL_OPEN: &[Shape] = &[
    Shape::Solid(
        "M14.872 14.287l6.522 6.52a2.996 2.996 0 0 1 -2.218 1.188l-.176 .005h-14a2.995 2.995 0 0 1 -2.394 -1.191l6.521 -6.522l2.318 1.545l.116 .066a1 1 0 0 0 .878 0l.116 -.066l2.317 -1.545z",
    ),
    Shape::Solid("M2 9.535l5.429 3.62l-5.429 5.43z"),
    Shape::Solid("M22 9.535v9.05l-5.43 -5.43z"),
    Shape::Solid(
        "M12.44 2.102l.115 .066l8.444 5.629l-8.999 6l-9 -6l8.445 -5.63a1 1 0 0 1 .994 -.065z",
    ),
];

/// Tabler Icons 3.48.0 `filled/tag` (MIT), verbatim.
const TAG: &[Shape] = &[Shape::Solid(
    "M11.172 2a3 3 0 0 1 2.121 .879l7.71 7.71a3.41 3.41 0 0 1 0 4.822l-5.592 5.592a3.41 3.41 0 0 1 -4.822 0l-7.71 -7.71a3 3 0 0 1 -.879 -2.121v-5.172a4 4 0 0 1 4 -4zm-3.672 3.5a2 2 0 0 0 -1.995 1.85l-.005 .15a2 2 0 1 0 2 -2",
)];

/// Phosphor Icons 2.1.1 Fill `arrows-clockwise-fill` (MIT), scaled from its 256 grid to 24.
const REFRESH: &[Shape] = &[Shape::Solid(
    "M21 4.5 V9 a0.75 0.75 0 0 1 -0.75 0.75 H15.75 a0.75 0.75 0 0 1 -0.531 -1.281 L16.936 6.75 a7.451 7.451 0 0 0 -5.13 -2.071 h-0.042 A7.455 7.455 0 0 0 6.524 6.817 A0.75 0.75 0 0 1 5.476 5.744 A9 9 0 0 1 18 5.691 l1.721 -1.721 A0.75 0.75 0 0 1 21 4.5 Z M17.476 17.183 A7.5 7.5 0 0 1 7.064 17.25 l1.717 -1.717 A0.75 0.75 0 0 0 8.25 14.25 H3.75 a0.75 0.75 0 0 0 -0.75 0.75 v4.5 a0.75 0.75 0 0 0 1.281 0.531 L6 18.309 a8.946 8.946 0 0 0 6.188 2.509 h0.05 a8.94 8.94 0 0 0 6.288 -2.562 a0.75 0.75 0 0 0 -1.048 -1.073 Z",
)];

/// Tabler Icons 3.48.0 `filled/send` (MIT), verbatim.
const SEND: &[Shape] = &[Shape::Solid(
    "M21.864 3.549l-6.454 17.868a1.55 1.55 0 0 1 -1.41 .903a1.54 1.54 0 0 1 -1.394 -.874l-2.88 -5.759zm-1.414 -1.414l-12.139 12.138l-5.728 -2.864a1.55 1.55 0 0 1 -.903 -1.409c0 -.606 .353 -1.157 .981 -1.44z",
)];

/// Phosphor Icons 2.1.1 Fill `command-fill` (MIT), scaled from its 256 grid to 24.
const COMMAND: &[Shape] = &[Shape::Solid(
    "M10.875 10.875 h2.25 v2.25 H10.875 Z M8.062 6.75 a1.312 1.312 0 0 0 0 2.625 h1.312 V8.062 A1.312 1.312 0 0 0 8.062 6.75 Z m9.188 1.312 a1.312 1.312 0 0 0 -2.625 0 v1.312 h1.312 A1.312 1.312 0 0 0 17.25 8.062 Z M6.75 15.938 a1.312 1.312 0 0 0 2.625 0 V14.625 H8.062 A1.312 1.312 0 0 0 6.75 15.938 Z M21 4.5 V19.5 a1.5 1.5 0 0 1 -1.5 1.5 H4.5 a1.5 1.5 0 0 1 -1.5 -1.5 V4.5 A1.5 1.5 0 0 1 4.5 3 H19.5 A1.5 1.5 0 0 1 21 4.5 Z m-6.375 8.625 V10.875 h1.312 a2.812 2.812 0 1 0 -2.812 -2.812 v1.312 H10.875 V8.062 a2.812 2.812 0 1 0 -2.812 2.812 h1.312 v2.25 H8.062 a2.812 2.812 0 1 0 2.812 2.812 V14.625 h2.25 v1.312 a2.812 2.812 0 1 0 2.812 -2.812 Z m0 2.812 a1.312 1.312 0 1 0 1.312 -1.312 H14.625 Z",
)];

/// Tabler Icons 3.48.0 `filled/columns-3` (MIT), verbatim.
const COLUMNS: &[Shape] = &[
    Shape::Solid("M4 2h2a1 1 0 0 1 1 1v18a1 1 0 0 1 -1 1h-2a2 2 0 0 1 -2 -2v-16a2 2 0 0 1 2 -2"),
    Shape::Solid("M9 3a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v18a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1z"),
    Shape::Solid("M18 2h2a2 2 0 0 1 2 2v16a2 2 0 0 1 -2 2h-2a1 1 0 0 1 -1 -1v-18a1 1 0 0 1 1 -1"),
];

/// Tabler Icons 3.48.0 `filled/layout-board` (MIT), verbatim.
const GROUP: &[Shape] = &[
    Shape::Solid("M5 3h5a1 1 0 0 1 1 1v3a1 1 0 0 1 -1 1h-6a1 1 0 0 1 -1 -1v-2a2 2 0 0 1 2 -2"),
    Shape::Solid("M14 3h5a2 2 0 0 1 2 2v8a1 1 0 0 1 -1 1h-6a1 1 0 0 1 -1 -1v-9a1 1 0 0 1 1 -1"),
    Shape::Solid("M14 16h6a1 1 0 0 1 1 1v2a2 2 0 0 1 -2 2h-5a1 1 0 0 1 -1 -1v-3a1 1 0 0 1 1 -1"),
    Shape::Solid("M4 10h6a1 1 0 0 1 1 1v9a1 1 0 0 1 -1 1h-5a2 2 0 0 1 -2 -2v-8a1 1 0 0 1 1 -1"),
];

/// Tabler Icons 3.48.0 `filled/layout-sidebar-right` (MIT), verbatim.
const PANEL: &[Shape] = &[Shape::Solid(
    "M6 21a3 3 0 0 1 -3 -3v-12a3 3 0 0 1 3 -3h12a3 3 0 0 1 3 3v12a3 3 0 0 1 -3 3zm8 -16h-8a1 1 0 0 0 -1 1v12a1 1 0 0 0 1 1h8z",
)];

/// Tabler Icons 3.48.0 `filled/square` (MIT), verbatim.
const SQUARE: &[Shape] = &[Shape::Solid(
    "M19 2h-14a3 3 0 0 0 -3 3v14a3 3 0 0 0 3 3h14a3 3 0 0 0 3 -3v-14a3 3 0 0 0 -3 -3z",
)];

/// Phosphor Icons 2.1.1 Fill `corners-out-fill` (MIT), scaled from its 256 grid to 24.
const MAXIMIZE: &[Shape] = &[Shape::Solid(
    "M8.781 18.969 A0.75 0.75 0 0 1 8.25 20.25 H4.5 a0.75 0.75 0 0 1 -0.75 -0.75 V15.75 a0.75 0.75 0 0 1 1.281 -0.531 Z M8.25 3.75 H4.5 a0.75 0.75 0 0 0 -0.75 0.75 V8.25 a0.75 0.75 0 0 0 1.281 0.531 l3.75 -3.75 A0.75 0.75 0 0 0 8.25 3.75 Z M19.787 15.057 a0.75 0.75 0 0 0 -0.818 0.162 l-3.75 3.75 A0.75 0.75 0 0 0 15.75 20.25 h3.75 a0.75 0.75 0 0 0 0.75 -0.75 V15.75 A0.75 0.75 0 0 0 19.787 15.057 Z M19.5 3.75 H15.75 a0.75 0.75 0 0 0 -0.531 1.281 l3.75 3.75 A0.75 0.75 0 0 0 20.25 8.25 V4.5 A0.75 0.75 0 0 0 19.5 3.75 Z",
)];

/// Phosphor Icons 2.1.1 Fill `arrow-elbow-down-left-fill` (MIT), scaled from its 256 grid to 24.
const CORNER: &[Shape] = &[Shape::Solid(
    "M18.75 3 V16.5 a0.75 0.75 0 0 1 -0.75 0.75 H9.75 v3.75 a0.75 0.75 0 0 1 -1.281 0.531 l-4.5 -4.5 a0.75 0.75 0 0 1 0 -1.061 l4.5 -4.5 A0.75 0.75 0 0 1 9.75 12 v3.75 h7.5 V3 a0.75 0.75 0 0 1 1.5 0 Z",
)];

/// Phosphor Icons 2.1.1 Fill `arrow-u-up-left-fill` (MIT), scaled from its 256 grid to 24.
const UNDO: &[Shape] = &[Shape::Solid(
    "M21.75 13.5 a6.007 6.007 0 0 1 -6 6 H7.5 a0.75 0.75 0 0 1 0 -1.5 h8.25 a4.5 4.5 0 0 0 0 -9 H8.25 v3.75 a0.75 0.75 0 0 1 -1.281 0.531 l-4.5 -4.5 a0.75 0.75 0 0 1 0 -1.061 l4.5 -4.5 A0.75 0.75 0 0 1 8.25 3.75 V7.5 h7.5 A6.007 6.007 0 0 1 21.75 13.5 Z",
)];

/// Tabler Icons 3.48.0 `filled/check` (MIT), verbatim.
const CHECK: &[Shape] = &[Shape::Solid(
    "M20.707 6.293a1 1 0 0 1 0 1.414l-10 10a1 1 0 0 1 -1.414 0l-5 -5a1 1 0 0 1 1.414 -1.414l4.293 4.293l9.293 -9.293a1 1 0 0 1 1.414 0",
)];

/// Tabler Icons 3.48.0 `filled/x` (MIT), verbatim.
const X: &[Shape] = &[Shape::Solid(
    "M6.707 5.293l5.293 5.292l5.293 -5.292a1 1 0 0 1 1.414 1.414l-5.292 5.293l5.292 5.293a1 1 0 0 1 -1.414 1.414l-5.293 -5.292l-5.293 5.292a1 1 0 1 1 -1.414 -1.414l5.292 -5.293l-5.292 -5.293a1 1 0 0 1 1.414 -1.414",
)];

/// Phosphor Icons 2.1.1 Fill `paperclip-fill` (MIT), scaled from its 256 grid to 24.
const PAPERCLIP: &[Shape] = &[Shape::Solid(
    "M12 2.25 A9.75 9.75 0 1 0 21.75 12 A9.76 9.76 0 0 0 12 2.25 Z m3.531 4.719 a0.75 0.75 0 0 0 -1.061 0 L8.165 13.406 A2.25 2.25 0 1 0 11.344 16.594 l4.624 -4.718 a0.75 0.75 0 1 1 1.071 1.05 l-4.628 4.723 a3.75 3.75 0 1 1 -5.308 -5.298 L13.406 5.915 A2.25 2.25 0 1 1 16.594 9.094 L10.285 15.525 a0.75 0.75 0 1 1 -1.071 -1.05 L15.525 8.035 a0.75 0.75 0 0 0 0.006 -1.066 Z",
)];

/// Tabler Icons 3.48.0 `filled/pencil` (MIT), verbatim.
const PEN: &[Shape] = &[Shape::Solid(
    "M12.085 6.5l5.415 5.415l-8.793 8.792a1 1 0 0 1 -.707 .293h-4a1 1 0 0 1 -1 -1v-4a1 1 0 0 1 .293 -.707zm5.406 -2.698a3.828 3.828 0 0 1 1.716 6.405l-.292 .293l-5.415 -5.415l.293 -.292a3.83 3.83 0 0 1 3.698 -.991",
)];

/// Tabler Icons 3.48.0 `filled/key` (MIT), verbatim.
const KEY: &[Shape] = &[Shape::Solid(
    "M14.52 2c1.029 0 2.015 .409 2.742 1.136l3.602 3.602a3.877 3.877 0 0 1 0 5.483l-2.643 2.643a3.88 3.88 0 0 1 -4.941 .452l-.105 -.078l-5.882 5.883a3 3 0 0 1 -1.68 .843l-.22 .027l-.221 .009h-1.172c-1.014 0 -1.867 -.759 -1.991 -1.823l-.009 -.177v-1.172c0 -.704 .248 -1.386 .73 -1.96l.149 -.161l.414 -.414a1 1 0 0 1 .707 -.293h1v-1a1 1 0 0 1 .883 -.993l.117 -.007h1v-1a1 1 0 0 1 .206 -.608l.087 -.1l1.468 -1.469l-.076 -.103a3.9 3.9 0 0 1 -.678 -1.963l-.007 -.236c0 -1.029 .409 -2.015 1.136 -2.742l2.643 -2.643a3.88 3.88 0 0 1 2.741 -1.136m.495 5h-.02a2 2 0 1 0 0 4h.02a2 2 0 1 0 0 -4",
)];

/// Tabler Icons 3.48.0 `filled/file-pencil` (MIT), verbatim.
const FILE_PEN: &[Shape] = &[
    Shape::Solid(
        "M12 2l.117 .007a1 1 0 0 1 .876 .876l.007 .117v4l.005 .15a2 2 0 0 0 1.838 1.844l.157 .006h4l.117 .007a1 1 0 0 1 .876 .876l.007 .117v9a3 3 0 0 1 -2.824 2.995l-.176 .005h-10a3 3 0 0 1 -2.995 -2.824l-.005 -.176v-14a3 3 0 0 1 2.824 -2.995l.176 -.005zm1 10l-5 5v2h2l5 -5a1.414 1.414 0 0 0 -2 -2",
    ),
    Shape::Solid("M19 7h-4l-.001 -4.001z"),
];

/// Tabler Icons 3.48.0 `filled/alert-octagon` (MIT), verbatim.
const OCTAGON_ALERT: &[Shape] = &[Shape::Solid(
    "M14.897 1a4 4 0 0 1 2.664 1.016l.165 .156l4.1 4.1a4 4 0 0 1 1.168 2.605l.006 .227v5.794a4 4 0 0 1 -1.016 2.664l-.156 .165l-4.1 4.1a4 4 0 0 1 -2.603 1.168l-.227 .006h-5.795a3.999 3.999 0 0 1 -2.664 -1.017l-.165 -.156l-4.1 -4.1a4 4 0 0 1 -1.168 -2.604l-.006 -.227v-5.794a4 4 0 0 1 1.016 -2.664l.156 -.165l4.1 -4.1a4 4 0 0 1 2.605 -1.168l.227 -.006h5.793zm-2.887 14l-.127 .007a1 1 0 0 0 0 1.986l.117 .007l.127 -.007a1 1 0 0 0 0 -1.986l-.117 -.007zm-.01 -8a1 1 0 0 0 -.993 .883l-.007 .117v4l.007 .117a1 1 0 0 0 1.986 0l.007 -.117v-4l-.007 -.117a1 1 0 0 0 -.993 -.883z",
)];

/// Tabler Icons 3.48.0 `filled/pin` (MIT), verbatim.
const PIN: &[Shape] = &[Shape::Solid(
    "M15.113 3.21l.094 .083l5.5 5.5a1 1 0 0 1 -1.175 1.59l-3.172 3.171l-1.424 3.797a1 1 0 0 1 -.158 .277l-.07 .08l-1.5 1.5a1 1 0 0 1 -1.32 .082l-.095 -.083l-2.793 -2.792l-3.793 3.792a1 1 0 0 1 -1.497 -1.32l.083 -.094l3.792 -3.793l-2.792 -2.793a1 1 0 0 1 -.083 -1.32l.083 -.094l1.5 -1.5a1 1 0 0 1 .258 -.187l.098 -.042l3.796 -1.425l3.171 -3.17a1 1 0 0 1 1.497 -1.26z",
)];

/// Phosphor Icons 2.1.1 Fill `arrow-bend-up-left-fill` (MIT), scaled from its 256 grid to 24.
const REPLY: &[Shape] = &[Shape::Solid(
    "M21.75 18.75 a0.75 0.75 0 0 1 -1.5 0 a8.259 8.259 0 0 0 -8.25 -8.25 H8.25 v3.75 a0.75 0.75 0 0 1 -1.281 0.531 l-4.5 -4.5 a0.75 0.75 0 0 1 0 -1.061 l4.5 -4.5 A0.75 0.75 0 0 1 8.25 5.25 V9 h3.75 A9.76 9.76 0 0 1 21.75 18.75 Z",
)];

/// Phosphor Icons 2.1.1 Fill `arrow-bend-double-up-left-fill` (MIT), scaled from its 256 grid to 24.
const REPLY_ALL: &[Shape] = &[Shape::Solid(
    "M8.031 13.719 a0.75 0.75 0 0 1 -1.061 1.061 l-4.5 -4.5 a0.75 0.75 0 0 1 0 -1.061 l4.5 -4.5 A0.75 0.75 0 0 1 8.031 5.781 L4.06 9.75 Z M12.75 9.028 V5.25 a0.75 0.75 0 0 0 -1.281 -0.531 l-4.5 4.5 a0.75 0.75 0 0 0 0 1.061 l4.5 4.5 A0.75 0.75 0 0 0 12.75 14.25 V10.535 A8.26 8.26 0 0 1 20.25 18.75 a0.75 0.75 0 0 0 1.5 0 A9.764 9.764 0 0 0 12.75 9.028 Z",
)];

/// Phosphor Icons 2.1.1 Fill `arrow-bend-up-right-fill` (MIT), scaled from its 256 grid to 24.
const FORWARD: &[Shape] = &[Shape::Solid(
    "M21.531 10.281 l-4.5 4.5 A0.75 0.75 0 0 1 15.75 14.25 V10.5 H12 a8.259 8.259 0 0 0 -8.25 8.25 a0.75 0.75 0 0 1 -1.5 0 A9.76 9.76 0 0 1 12 9 h3.75 V5.25 a0.75 0.75 0 0 1 1.281 -0.531 l4.5 4.5 A0.75 0.75 0 0 1 21.531 10.281 Z",
)];

/// drawn here, filled.
const SEARCH: &[Shape] = &[
    Shape::Solid(
        "M11 3a8 8 0 1 0 0 16 8 8 0 0 0 0-16zm0 2.5a5.5 5.5 0 1 1 0 11 5.5 5.5 0 0 1 0-11z",
    ),
    Shape::Solid("M16.3 17.9l1.6-1.6 3.4 3.4a1.13 1.13 0 0 1-1.6 1.6z"),
];

/// Tabler Icons 3.48.0 `filled/settings` (MIT), verbatim.
const SETTINGS: &[Shape] = &[Shape::Solid(
    "M14.647 4.081a.724 .724 0 0 0 1.08 .448c2.439 -1.485 5.23 1.305 3.745 3.744a.724 .724 0 0 0 .447 1.08c2.775 .673 2.775 4.62 0 5.294a.724 .724 0 0 0 -.448 1.08c1.485 2.439 -1.305 5.23 -3.744 3.745a.724 .724 0 0 0 -1.08 .447c-.673 2.775 -4.62 2.775 -5.294 0a.724 .724 0 0 0 -1.08 -.448c-2.439 1.485 -5.23 -1.305 -3.745 -3.744a.724 .724 0 0 0 -.447 -1.08c-2.775 -.673 -2.775 -4.62 0 -5.294a.724 .724 0 0 0 .448 -1.08c-1.485 -2.439 1.305 -5.23 3.744 -3.745a.722 .722 0 0 0 1.08 -.447c.673 -2.775 4.62 -2.775 5.294 0zm-2.647 4.919a3 3 0 1 0 0 6a3 3 0 0 0 0 -6",
)];

/// Tabler Icons 3.48.0 `filled/layout-sidebar` (MIT), verbatim.
const PANEL_LEFT: &[Shape] = &[Shape::Solid(
    "M6 21a3 3 0 0 1 -3 -3v-12a3 3 0 0 1 3 -3h12a3 3 0 0 1 3 3v12a3 3 0 0 1 -3 3zm12 -16h-8v14h8a1 1 0 0 0 1 -1v-12a1 1 0 0 0 -1 -1",
)];

/// Tabler Icons 3.48.0 `filled/plus` (MIT), verbatim.
const PLUS: &[Shape] = &[Shape::Solid(
    "M12 4a1 1 0 0 1 1 1v6h6a1 1 0 0 1 0 2h-6v6a1 1 0 0 1 -2 0v-6h-6a1 1 0 0 1 0 -2h6v-6a1 1 0 0 1 1 -1",
)];

/// drawn here, filled.
const MINUS: &[Shape] = &[Shape::Solid("M5 11h14a1 1 0 0 1 0 2H5a1 1 0 0 1 0-2z")];
