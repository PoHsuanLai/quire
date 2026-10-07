//! Solid forms of the actions, control, own and format glyphs: filled paths on the 24 grid, no stroke (design/08-ICONS.md section 1.2).
//!
//! Each constant says where its geometry comes from. Tabler and Phosphor are MIT, notices in
//! `assets/icons/LICENSE-tabler.txt` and `LICENSE-phosphor.txt`; a Phosphor path is scaled by
//! 3/32 from its 256 grid and rounded to three decimals.

use super::Icon;
use super::shape::Shape;

/// The solid children of a glyph this file holds; empty for any other.
pub(super) fn shapes(icon: Icon) -> &'static [Shape] {
    match icon {
        Icon::Link => LINK,
        Icon::Sparkles => SPARKLES,
        Icon::Gauge => GAUGE,
        Icon::Brightness => BRIGHTNESS,
        Icon::Printer => PRINTER,
        Icon::FolderInput => FOLDER_INPUT,
        Icon::Play => PLAY,
        Icon::Pause => PAUSE,
        Icon::SkipBack => SKIP_BACK,
        Icon::SkipForward => SKIP_FORWARD,
        Icon::LogOut => LOG_OUT,
        Icon::Restart => RESTART,
        Icon::Headphones => HEADPHONES,
        Icon::Speaker => SPEAKER,
        Icon::Mouse => MOUSE,
        Icon::Gamepad => GAMEPAD,
        Icon::Phone => PHONE,
        Icon::Ellipsis => ELLIPSIS,
        Icon::EllipsisVertical => ELLIPSIS_VERTICAL,
        Icon::Switches => SWITCHES,
        Icon::ArrowRight => ARROW_RIGHT,
        Icon::CapsLock => CAPS_LOCK,
        Icon::Clipboard => CLIPBOARD,
        Icon::Smile => SMILE,
        Icon::Globe => GLOBE,
        Icon::MoonFilled => MOON_FILLED,
        Icon::Bold => BOLD,
        Icon::Italic => ITALIC,
        Icon::Underline => UNDERLINE,
        Icon::Strike => STRIKE,
        Icon::Code => CODE,
        Icon::Info => INFO,
        Icon::CircleCheck => CIRCLE_CHECK,
        Icon::TriangleAlert => TRIANGLE_ALERT,
        Icon::Heart => HEART,
        _ => &[],
    }
}

/// Tabler Icons 3.48.0 `filled/link` (MIT), verbatim.
const LINK: &[Shape] = &[
    Shape::Solid(
        "M15.707 8.293a1 1 0 0 1 0 1.414l-6 6a1 1 0 1 1 -1.414 -1.414l6 -6a1 1 0 0 1 1.414 0",
    ),
    Shape::Solid(
        "M19.242 4.757c2.343 2.344 2.342 6.143 -.052 8.534l-.534 .464a1 1 0 1 1 -1.312 -1.51l.483 -.416a4 4 0 0 0 0 -5.657c-1.562 -1.563 -4.095 -1.563 -5.607 -.054l-.463 .536a1 1 0 1 1 -1.514 -1.308l.513 -.59a6 6 0 0 1 8.486 .001",
    ),
    Shape::Solid(
        "M6.75 10.338a1 1 0 0 1 -.088 1.411l-.483 .425a3.97 3.97 0 0 0 0 5.649a4.064 4.064 0 0 0 5.678 .038l.34 -.458a1 1 0 1 1 1.606 1.194l-.397 .534l-.1 .114a6.07 6.07 0 0 1 -8.533 0a5.97 5.97 0 0 1 -1.773 -4.247c0 -1.595 .638 -3.124 1.814 -4.284l.524 -.463a1 1 0 0 1 1.411 .087",
    ),
];

/// Tabler Icons 3.48.0 `filled/sparkles` (MIT), verbatim.
const SPARKLES: &[Shape] = &[
    Shape::Solid(
        "M16 19a1 1 0 0 1 0 -2a1 1 0 0 0 1 -1c0 -1.333 2 -1.333 2 0a1 1 0 0 0 1 1c1.333 0 1.333 2 0 2a1 1 0 0 0 -1 1c0 1.333 -2 1.333 -2 0a1 1 0 0 0 -1 -1",
    ),
    Shape::Solid(
        "M3 11a5 5 0 0 0 5 -5c0 -1.333 2 -1.333 2 0a5 5 0 0 0 5 5c1.333 0 1.333 2 0 2a5 5 0 0 0 -5 5a1 1 0 0 1 -2 0a5 5 0 0 0 -5 -5c-1.333 0 -1.333 -2 0 -2",
    ),
    Shape::Solid(
        "M16 7a1 1 0 0 1 0 -2a1 1 0 0 0 1 -1c0 -1.333 2 -1.333 2 0a1 1 0 0 0 1 1c1.333 0 1.333 2 0 2a1 1 0 0 0 -1 1c0 1.333 -2 1.333 -2 0a1 1 0 0 0 -1 -1",
    ),
];

/// Tabler Icons 3.48.0 `filled/gauge` (MIT), verbatim.
const GAUGE: &[Shape] = &[Shape::Solid(
    "M17 3.34a10 10 0 1 1 -14.995 8.984l-.005 -.324l.005 -.324a10 10 0 0 1 14.995 -8.336zm-.293 3.953a1 1 0 0 0 -1.414 0l-2.59 2.59l-.083 .094l-.068 .1a2.001 2.001 0 0 0 -2.547 1.774l-.005 .149l.005 .15a2 2 0 1 0 3.917 -.701a.968 .968 0 0 0 .195 -.152l2.59 -2.59l.083 -.094a1 1 0 0 0 -.083 -1.32zm-4.707 -1.293a6 6 0 0 0 -6 6a1 1 0 0 0 2 0a4 4 0 0 1 4 -4a1 1 0 0 0 0 -2z",
)];

/// Tabler Icons 3.48.0 `filled/brightness` (MIT), verbatim.
const BRIGHTNESS: &[Shape] = &[Shape::Solid(
    "M17 3.34a10 10 0 1 1 -15 8.66l.005 -.324a10 10 0 0 1 14.995 -8.336m-9 1.732a8 8 0 0 0 4.001 14.928l-.001 -16a8 8 0 0 0 -4 1.072",
)];

/// Phosphor Icons 2.1.1 Fill `printer-fill` (MIT), scaled from its 256 grid to 24.
const PRINTER: &[Shape] = &[Shape::Solid(
    "M22.5 9 v7.5 a0.75 0.75 0 0 1 -0.75 0.75 H18.75 v3 a0.75 0.75 0 0 1 -0.75 0.75 H6 a0.75 0.75 0 0 1 -0.75 -0.75 V17.25 H2.25 a0.75 0.75 0 0 1 -0.75 -0.75 V9 c0 -1.24 1.065 -2.25 2.375 -2.25 H5.25 V3.75 a0.75 0.75 0 0 1 0.75 -0.75 H18 a0.75 0.75 0 0 1 0.75 0.75 V6.75 h1.375 C21.435 6.75 22.5 7.76 22.5 9 Z M6.75 6.75 H17.25 V4.5 H6.75 Z m10.5 8.25 H6.75 v4.5 H17.25 Z m1.5 -4.125 a1.125 1.125 0 1 0 -1.125 1.125 A1.125 1.125 0 0 0 18.75 10.875 Z",
)];

/// Phosphor Icons 2.1.1 Fill `tray-arrow-down-fill` (MIT), scaled from its 256 grid to 24.
const FOLDER_INPUT: &[Shape] = &[Shape::Solid(
    "M19.5 3 H4.5 A1.5 1.5 0 0 0 3 4.5 V19.5 a1.5 1.5 0 0 0 1.5 1.5 H19.5 a1.5 1.5 0 0 0 1.5 -1.5 V4.5 A1.5 1.5 0 0 0 19.5 3 Z M8.469 10.719 a0.75 0.75 0 0 1 1.061 0 L11.25 12.44 V6.75 a0.75 0.75 0 0 1 1.5 0 v5.69 l1.719 -1.72 a0.75 0.75 0 0 1 1.061 1.061 l-3 3 a0.75 0.75 0 0 1 -1.061 0 l-3 -3 A0.75 0.75 0 0 1 8.469 10.719 Z M19.5 19.5 H4.5 V15.75 H7.19 L9 17.561 A1.49 1.49 0 0 0 10.06 18 h3.879 A1.487 1.487 0 0 0 15 17.56 L16.81 15.75 H19.5 v3.75 Z",
)];

/// Tabler Icons 3.48.0 `filled/player-play` (MIT), verbatim.
const PLAY: &[Shape] = &[Shape::Solid(
    "M6 4v16a1 1 0 0 0 1.524 .852l13 -8a1 1 0 0 0 0 -1.704l-13 -8a1 1 0 0 0 -1.524 .852z",
)];

/// Tabler Icons 3.48.0 `filled/player-pause` (MIT), verbatim.
const PAUSE: &[Shape] = &[
    Shape::Solid("M9 4h-2a2 2 0 0 0 -2 2v12a2 2 0 0 0 2 2h2a2 2 0 0 0 2 -2v-12a2 2 0 0 0 -2 -2z"),
    Shape::Solid("M17 4h-2a2 2 0 0 0 -2 2v12a2 2 0 0 0 2 2h2a2 2 0 0 0 2 -2v-12a2 2 0 0 0 -2 -2z"),
];

/// Tabler Icons 3.48.0 `filled/player-skip-back` (MIT), verbatim.
const SKIP_BACK: &[Shape] = &[
    Shape::Solid(
        "M19.496 4.136l-12 7a1 1 0 0 0 0 1.728l12 7a1 1 0 0 0 1.504 -.864v-14a1 1 0 0 0 -1.504 -.864z",
    ),
    Shape::Solid(
        "M4 4a1 1 0 0 1 .993 .883l.007 .117v14a1 1 0 0 1 -1.993 .117l-.007 -.117v-14a1 1 0 0 1 1 -1z",
    ),
];

/// Tabler Icons 3.48.0 `filled/player-skip-forward` (MIT), verbatim.
const SKIP_FORWARD: &[Shape] = &[
    Shape::Solid(
        "M3 5v14a1 1 0 0 0 1.504 .864l12 -7a1 1 0 0 0 0 -1.728l-12 -7a1 1 0 0 0 -1.504 .864z",
    ),
    Shape::Solid(
        "M20 4a1 1 0 0 1 .993 .883l.007 .117v14a1 1 0 0 1 -1.993 .117l-.007 -.117v-14a1 1 0 0 1 1 -1z",
    ),
];

/// Phosphor Icons 2.1.1 Fill `sign-out-fill` (MIT), scaled from its 256 grid to 24.
const LOG_OUT: &[Shape] = &[Shape::Solid(
    "M11.25 20.25 a0.75 0.75 0 0 1 -0.75 0.75 H4.5 a0.75 0.75 0 0 1 -0.75 -0.75 V3.75 a0.75 0.75 0 0 1 0.75 -0.75 h6 a0.75 0.75 0 0 1 0 1.5 H5.25 V19.5 h5.25 A0.75 0.75 0 0 1 11.25 20.25 Z m10.281 -8.781 l-3.75 -3.75 A0.75 0.75 0 0 0 16.5 8.25 v3 H10.5 a0.75 0.75 0 0 0 0 1.5 h6 v3 a0.75 0.75 0 0 0 1.281 0.531 l3.75 -3.75 A0.75 0.75 0 0 0 21.531 11.469 Z",
)];

/// Phosphor Icons 2.1.1 Fill `arrow-counter-clockwise-fill` (MIT), scaled from its 256 grid to 24.
const RESTART: &[Shape] = &[Shape::Solid(
    "M21 12 a9 9 0 0 1 -8.879 9 H12 A8.942 8.942 0 0 1 5.822 18.544 a0.75 0.75 0 0 1 1.031 -1.09 A7.5 7.5 0 1 0 6.697 6.693 a0.288 0.288 0 0 1 -0.024 0.023 L5.684 7.621 l1.594 1.594 A0.75 0.75 0 0 1 6.75 10.5 H2.25 a0.75 0.75 0 0 1 -0.75 -0.75 V5.25 A0.75 0.75 0 0 1 2.781 4.716 L4.623 6.562 L5.648 5.625 A9 9 0 0 1 21 12 Z",
)];

/// Tabler Icons 3.48.0 `filled/headphones` (MIT), verbatim.
const HEADPHONES: &[Shape] = &[Shape::Solid(
    "M21 18a3 3 0 0 1 -2.824 2.995l-.176 .005h-1a3 3 0 0 1 -2.995 -2.824l-.005 -.176v-3a3 3 0 0 1 2.824 -2.995l.176 -.005h1c.351 0 .688 .06 1 .171v-.171a7 7 0 0 0 -13.996 -.24l-.004 .24v.17c.25 -.088 .516 -.144 .791 -.163l.209 -.007h1a3 3 0 0 1 2.995 2.824l.005 .176v3a3 3 0 0 1 -2.824 2.995l-.176 .005h-1a3 3 0 0 1 -2.995 -2.824l-.005 -.176v-6a9 9 0 0 1 17.996 -.265l.004 .265v6z",
)];

/// Tabler Icons 3.48.0 `filled/device-speaker` (MIT), verbatim.
const SPEAKER: &[Shape] = &[Shape::Solid(
    "M17 2a3 3 0 0 1 3 3v14a3 3 0 0 1 -3 3h-10a3 3 0 0 1 -3 -3v-14a3 3 0 0 1 3 -3zm-5 9a4 4 0 0 0 -3.995 3.8l-.005 .2a4 4 0 1 0 4 -4m0 -5a1 1 0 0 0 -1 1v.01a1 1 0 0 0 2 0v-.01a1 1 0 0 0 -1 -1",
)];

/// Tabler Icons 3.48.0 `filled/mouse` (MIT), verbatim.
const MOUSE: &[Shape] = &[Shape::Solid(
    "M14 2a5 5 0 0 1 5 5v10a5 5 0 0 1 -5 5h-4a5 5 0 0 1 -5 -5v-10a5 5 0 0 1 5 -5zm-2 4a1 1 0 0 0 -1 1v4l.007 .117a1 1 0 0 0 1.993 -.117v-4l-.007 -.117a1 1 0 0 0 -.993 -.883z",
)];

/// Tabler Icons 3.48.0 `filled/device-gamepad` (MIT), verbatim.
const GAMEPAD: &[Shape] = &[Shape::Solid(
    "M20 5a3 3 0 0 1 3 3v8a3 3 0 0 1 -3 3h-16a3 3 0 0 1 -3 -3v-8a3 3 0 0 1 3 -3zm-12 4l-.117 .007a1 1 0 0 0 -.883 .993v1h-1a1 1 0 0 0 -1 1l.007 .117a1 1 0 0 0 .993 .883h1v1a1 1 0 0 0 1 1l.117 -.007a1 1 0 0 0 .883 -.993v-1h1a1 1 0 0 0 1 -1l-.007 -.117a1 1 0 0 0 -.993 -.883h-1v-1a1 1 0 0 0 -1 -1m10 3a1 1 0 0 0 -1 1v.01a1 1 0 0 0 2 0v-.01a1 1 0 0 0 -1 -1m-3 -2a1 1 0 0 0 -1 1v.01a1 1 0 0 0 2 0v-.01a1 1 0 0 0 -1 -1",
)];

/// Tabler Icons 3.48.0 `filled/device-mobile` (MIT), verbatim.
const PHONE: &[Shape] = &[Shape::Solid(
    "M16 2a3 3 0 0 1 2.995 2.824l.005 .176v14a3 3 0 0 1 -2.824 2.995l-.176 .005h-8a3 3 0 0 1 -2.995 -2.824l-.005 -.176v-14a3 3 0 0 1 2.824 -2.995l.176 -.005h8zm-4 14a1 1 0 0 0 -.993 .883l-.007 .117l.007 .127a1 1 0 0 0 1.986 0l.007 -.117l-.007 -.127a1 1 0 0 0 -.993 -.883zm1 -12h-2l-.117 .007a1 1 0 0 0 0 1.986l.117 .007h2l.117 -.007a1 1 0 0 0 0 -1.986l-.117 -.007z",
)];

/// Tabler Icons 3.48.0 `filled/dots` (MIT), verbatim.
const ELLIPSIS: &[Shape] = &[
    Shape::Solid(
        "M7 12a2 2 0 1 1 -4 0q 0 -.053 .005 -.102a1.996 1.996 0 0 1 1.995 -1.898a2 2 0 0 1 2 2",
    ),
    Shape::Solid(
        "M14 12a2 2 0 1 1 -4 0q 0 -.053 .005 -.102a1.996 1.996 0 0 1 1.995 -1.898a2 2 0 0 1 2 2",
    ),
    Shape::Solid(
        "M21 12a2 2 0 1 1 -4 0q 0 -.053 .005 -.102a1.996 1.996 0 0 1 1.995 -1.898a2 2 0 0 1 2 2",
    ),
];

/// Tabler Icons 3.48.0 `filled/dots-vertical` (MIT), verbatim.
const ELLIPSIS_VERTICAL: &[Shape] = &[
    Shape::Solid(
        "M14 12a2 2 0 1 1 -4 0q 0 -.053 .005 -.102a1.996 1.996 0 0 1 1.995 -1.898a2 2 0 0 1 2 2",
    ),
    Shape::Solid(
        "M14 19a2 2 0 1 1 -4 0q 0 -.052 .005 -.102a1.996 1.996 0 0 1 1.995 -1.898a2 2 0 0 1 2 2",
    ),
    Shape::Solid(
        "M14 5a2 2 0 1 1 -4 0q 0 -.053 .005 -.102a1.996 1.996 0 0 1 1.995 -1.898a2 2 0 0 1 2 2",
    ),
];

/// drawn here, filled.
const SWITCHES: &[Shape] = &[
    Shape::Solid("M6 2h12a4 4 0 0 1 0 8H6A4 4 0 0 1 6 2zm0 2a2 2 0 1 0 0 4 2 2 0 0 0 0-4z"),
    Shape::Solid("M6 14h12a4 4 0 0 1 0 8H6a4 4 0 0 1 0-8zm12 2a2 2 0 1 0 0 4 2 2 0 0 0 0-4z"),
];

/// Phosphor Icons 2.1.1 Fill `arrow-right-fill` (MIT), scaled from its 256 grid to 24.
const ARROW_RIGHT: &[Shape] = &[Shape::Solid(
    "M20.781 12.531 l-6.75 6.75 A0.75 0.75 0 0 1 12.75 18.75 V12.75 H3.75 a0.75 0.75 0 0 1 0 -1.5 h9 V5.25 a0.75 0.75 0 0 1 1.281 -0.531 l6.75 6.75 A0.75 0.75 0 0 1 20.781 12.531 Z",
)];

/// Tabler Icons 3.48.0 `filled/arrow-big-up-line` (MIT), verbatim.
const CAPS_LOCK: &[Shape] = &[
    Shape::Solid(
        "M10.586 3l-6.586 6.586a2 2 0 0 0 -.434 2.18l.068 .145a2 2 0 0 0 1.78 1.089h2.586v5a1 1 0 0 0 1 1h6l.117 -.007a1 1 0 0 0 .883 -.993l-.001 -5h2.587a2 2 0 0 0 1.414 -3.414l-6.586 -6.586a2 2 0 0 0 -2.828 0z",
    ),
    Shape::Solid("M15 20a1 1 0 0 1 .117 1.993l-.117 .007h-6a1 1 0 0 1 -.117 -1.993l.117 -.007h6z"),
];

/// Tabler Icons 3.48.0 `filled/clipboard` (MIT), verbatim.
const CLIPBOARD: &[Shape] = &[Shape::Solid(
    "M17.997 4.17a3 3 0 0 1 2.003 2.83v12a3 3 0 0 1 -3 3h-10a3 3 0 0 1 -3 -3v-12a3 3 0 0 1 2.003 -2.83a4 4 0 0 0 3.997 3.83h4a4 4 0 0 0 3.98 -3.597zm-3.997 -2.17a2 2 0 1 1 0 4h-4a2 2 0 1 1 0 -4z",
)];

/// Tabler Icons 3.48.0 `filled/mood-smile` (MIT), verbatim.
const SMILE: &[Shape] = &[Shape::Solid(
    "M17 3.34a10 10 0 1 1 -14.995 8.984l-.005 -.324l.005 -.324a10 10 0 0 1 14.995 -8.336zm-1.8 10.946a1 1 0 0 0 -1.414 .014a2.5 2.5 0 0 1 -3.572 0a1 1 0 0 0 -1.428 1.4a4.5 4.5 0 0 0 6.428 0a1 1 0 0 0 -.014 -1.414zm-6.19 -5.286l-.127 .007a1 1 0 0 0 .117 1.993l.127 -.007a1 1 0 0 0 -.117 -1.993zm6 0l-.127 .007a1 1 0 0 0 .117 1.993l.127 -.007a1 1 0 0 0 -.117 -1.993z",
)];

/// Tabler Icons 3.48.0 `filled/globe` (MIT), verbatim.
const GLOBE: &[Shape] = &[
    Shape::Solid("M11 4a5 5 0 1 1 -4.995 5.217l-.005 -.217l.005 -.217a5 5 0 0 1 4.995 -4.783z"),
    Shape::Solid(
        "M14.133 1.502a1 1 0 0 1 1.365 -.369a9.015 9.015 0 1 1 -10.404 14.622a1 1 0 1 1 1.312 -1.51a7.015 7.015 0 1 0 8.096 -11.378a1 1 0 0 1 -.369 -1.365z",
    ),
    Shape::Solid(
        "M11 16a1 1 0 0 1 .993 .883l.007 .117v4a1 1 0 0 1 -1.993 .117l-.007 -.117v-4a1 1 0 0 1 1 -1z",
    ),
    Shape::Solid("M15 20a1 1 0 0 1 .117 1.993l-.117 .007h-8a1 1 0 0 1 -.117 -1.993l.117 -.007h8z"),
];

/// Tabler Icons 3.48.0 `filled/moon` (MIT), verbatim.
const MOON_FILLED: &[Shape] = &[Shape::Solid(
    "M12 1.992a10 10 0 1 0 9.236 13.838c.341 -.82 -.476 -1.644 -1.298 -1.31a6.5 6.5 0 0 1 -6.864 -10.787l.077 -.08c.551 -.63 .113 -1.653 -.758 -1.653h-.266l-.068 -.006l-.06 -.002z",
)];

/// Phosphor Icons 2.1.1 Fill `text-b-fill` (MIT), scaled from its 256 grid to 24.
const BOLD: &[Shape] = &[Shape::Solid(
    "M15.75 14.625 a1.875 1.875 0 0 1 -1.875 1.875 H9 V12.75 h4.875 A1.875 1.875 0 0 1 15.75 14.625 Z M21 4.5 V19.5 a1.5 1.5 0 0 1 -1.5 1.5 H4.5 a1.5 1.5 0 0 1 -1.5 -1.5 V4.5 A1.5 1.5 0 0 1 4.5 3 H19.5 A1.5 1.5 0 0 1 21 4.5 Z M17.25 14.625 a3.375 3.375 0 0 0 -1.688 -2.92 A3.375 3.375 0 0 0 13.125 6 H8.25 a0.75 0.75 0 0 0 -0.75 0.75 V17.25 a0.75 0.75 0 0 0 0.75 0.75 h5.625 A3.375 3.375 0 0 0 17.25 14.625 Z m-2.25 -5.25 a1.875 1.875 0 0 0 -1.875 -1.875 H9 v3.75 h4.125 A1.875 1.875 0 0 0 15 9.375 Z",
)];

/// Phosphor Icons 2.1.1 Fill `text-italic-fill` (MIT), scaled from its 256 grid to 24.
const ITALIC: &[Shape] = &[Shape::Solid(
    "M19.5 3 H4.5 A1.5 1.5 0 0 0 3 4.5 V19.5 a1.5 1.5 0 0 0 1.5 1.5 H19.5 a1.5 1.5 0 0 0 1.5 -1.5 V4.5 A1.5 1.5 0 0 0 19.5 3 Z M16.5 7.5 H14.404 l-3.215 9 H12.75 a0.75 0.75 0 0 1 0 1.5 H7.5 a0.75 0.75 0 0 1 0 -1.5 h2.096 l3.215 -9 H11.25 a0.75 0.75 0 0 1 0 -1.5 h5.25 a0.75 0.75 0 0 1 0 1.5 Z",
)];

/// Phosphor Icons 2.1.1 Fill `text-underline-fill` (MIT), scaled from its 256 grid to 24.
const UNDERLINE: &[Shape] = &[Shape::Solid(
    "M19.5 3 H4.5 A1.5 1.5 0 0 0 3 4.5 V19.5 a1.5 1.5 0 0 0 1.5 1.5 H19.5 a1.5 1.5 0 0 0 1.5 -1.5 V4.5 A1.5 1.5 0 0 0 19.5 3 Z M7.5 6.75 a0.75 0.75 0 0 1 1.5 0 v4.5 a3 3 0 0 0 6 0 V6.75 a0.75 0.75 0 0 1 1.5 0 v4.5 a4.5 4.5 0 0 1 -9 0 Z m9 12 H7.5 a0.75 0.75 0 0 1 0 -1.5 h9 a0.75 0.75 0 0 1 0 1.5 Z",
)];

/// Phosphor Icons 2.1.1 Fill `text-strikethrough-fill` (MIT), scaled from its 256 grid to 24.
const STRIKE: &[Shape] = &[Shape::Solid(
    "M19.5 3 H4.5 A1.5 1.5 0 0 0 3 4.5 V19.5 a1.5 1.5 0 0 0 1.5 1.5 H19.5 a1.5 1.5 0 0 0 1.5 -1.5 V4.5 A1.5 1.5 0 0 0 19.5 3 Z M7.754 8.867 C8.062 7.178 9.804 6 12 6 c1.706 0 3.149 0.695 3.861 1.859 a0.75 0.75 0 1 1 -1.281 0.782 C14.151 7.937 13.162 7.5 12 7.5 c-1.434 0 -2.6 0.687 -2.77 1.633 A0.75 0.75 0 0 1 8.493 9.75 a0.728 0.728 0 0 1 -0.134 -0.012 A0.75 0.75 0 0 1 7.754 8.867 Z M18 12.75 H15.777 A2.667 2.667 0 0 1 16.5 14.625 c0 1.893 -1.976 3.375 -4.5 3.375 c-2.24 0 -4.109 -1.198 -4.447 -2.85 a0.75 0.75 0 1 1 1.469 -0.3 c0.188 0.925 1.5 1.65 2.978 1.65 c1.627 0 3 -0.859 3 -1.875 c0 -0.857 -0.634 -1.353 -2.411 -1.875 H6 a0.75 0.75 0 0 1 0 -1.5 H18 a0.75 0.75 0 0 1 0 1.5 Z",
)];

/// Phosphor Icons 2.1.1 Fill `code-fill` (MIT), scaled from its 256 grid to 24.
const CODE: &[Shape] = &[Shape::Solid(
    "M20.25 3.75 H3.75 A1.5 1.5 0 0 0 2.25 5.25 V18.75 a1.5 1.5 0 0 0 1.5 1.5 H20.25 a1.5 1.5 0 0 0 1.5 -1.5 V5.25 A1.5 1.5 0 0 0 20.25 3.75 Z M8.7 13.65 a0.75 0.75 0 1 1 -0.9 1.2 l-3 -2.25 a0.75 0.75 0 0 1 0 -1.2 l3 -2.25 a0.75 0.75 0 0 1 0.9 1.2 L6.5 12 Z m5.521 -6.694 l-3 10.5 a0.75 0.75 0 1 1 -1.442 -0.413 l3 -10.5 a0.75 0.75 0 0 1 1.442 0.413 Z m4.979 5.644 l-3 2.25 a0.75 0.75 0 0 1 -0.9 -1.2 L17.5 12 L15.3 10.35 a0.75 0.75 0 1 1 0.9 -1.2 l3 2.25 a0.75 0.75 0 0 1 0 1.2 Z",
)];

/// Tabler Icons 3.48.0 `filled/info-circle` (MIT), verbatim.
const INFO: &[Shape] = &[Shape::Solid(
    "M12 2c5.523 0 10 4.477 10 10a10 10 0 0 1 -19.995 .324l-.005 -.324l.004 -.28c.148 -5.393 4.566 -9.72 9.996 -9.72zm0 9h-1l-.117 .007a1 1 0 0 0 0 1.986l.117 .007v3l.007 .117a1 1 0 0 0 .876 .876l.117 .007h1l.117 -.007a1 1 0 0 0 .876 -.876l.007 -.117l-.007 -.117a1 1 0 0 0 -.764 -.857l-.112 -.02l-.117 -.006v-3l-.007 -.117a1 1 0 0 0 -.876 -.876l-.117 -.007zm.01 -3l-.127 .007a1 1 0 0 0 0 1.986l.117 .007l.127 -.007a1 1 0 0 0 0 -1.986l-.117 -.007z",
)];

/// Tabler Icons 3.48.0 `filled/circle-check` (MIT), verbatim.
const CIRCLE_CHECK: &[Shape] = &[Shape::Solid(
    "M17 3.34a10 10 0 1 1 -14.995 8.984l-.005 -.324l.005 -.324a10 10 0 0 1 14.995 -8.336zm-1.293 5.953a1 1 0 0 0 -1.32 -.083l-.094 .083l-3.293 3.292l-1.293 -1.292l-.094 -.083a1 1 0 0 0 -1.403 1.403l.083 .094l2 2l.094 .083a1 1 0 0 0 1.226 0l.094 -.083l4 -4l.083 -.094a1 1 0 0 0 -.083 -1.32z",
)];

/// Tabler Icons 3.48.0 `filled/alert-triangle` (MIT), verbatim.
const TRIANGLE_ALERT: &[Shape] = &[Shape::Solid(
    "M12 1.67c.955 0 1.845 .467 2.39 1.247l.105 .16l8.114 13.548a2.914 2.914 0 0 1 -2.307 4.363l-.195 .008h-16.225a2.914 2.914 0 0 1 -2.582 -4.2l.099 -.185l8.11 -13.538a2.914 2.914 0 0 1 2.491 -1.403zm.01 13.33l-.127 .007a1 1 0 0 0 0 1.986l.117 .007l.127 -.007a1 1 0 0 0 0 -1.986l-.117 -.007zm-.01 -7a1 1 0 0 0 -.993 .883l-.007 .117v4l.007 .117a1 1 0 0 0 1.986 0l.007 -.117v-4l-.007 -.117a1 1 0 0 0 -.993 -.883z",
)];

/// Tabler Icons 3.48.0 `filled/heart` (MIT), verbatim.
const HEART: &[Shape] = &[Shape::Solid(
    "M6.979 3.074a6 6 0 0 1 4.988 1.425l.037 .033l.034 -.03a6 6 0 0 1 4.733 -1.44l.246 .036a6 6 0 0 1 3.364 10.008l-.18 .185l-.048 .041l-7.45 7.379a1 1 0 0 1 -1.313 .082l-.094 -.082l-7.493 -7.422a6 6 0 0 1 3.176 -10.215z",
)];
