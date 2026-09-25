//! Lucide geometry for the control center, the power menu and Now Playing (sill FINDINGS
//! Q81), transcribed from `lucide-static` 1.47.0
//! (<https://unpkg.com/lucide-static@1.47.0/icons/>), ISC licence; see `super`. A Lucide
//! `<line>` is written as the equivalent path `M x1 y1 L x2 y2`, since [`Shape`] has no line;
//! every other child is as published. `Power` was already in the shell set and is not repeated.

use super::Icon;
use super::shape::Shape;

/// The children of a control glyph; any other glyph draws nothing here (`Icon::shapes` routes
/// only this set's glyphs to it).
pub(super) fn shapes(icon: Icon) -> &'static [Shape] {
    match icon {
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
        _ => &[],
    }
}

/// Lucide `play`.
const PLAY: &[Shape] = &[Shape::Path(
    "M5 5a2 2 0 0 1 3.008-1.728l11.997 6.998a2 2 0 0 1 .003 3.458l-12 7A2 2 0 0 1 5 19z",
)];

/// Lucide `pause`.
const PAUSE: &[Shape] = &[
    Shape::Rect {
        x: "14",
        y: "3",
        width: "5",
        height: "18",
        rx: "1",
    },
    Shape::Rect {
        x: "5",
        y: "3",
        width: "5",
        height: "18",
        rx: "1",
    },
];

/// Lucide `skip-back`.
const SKIP_BACK: &[Shape] = &[
    Shape::Path(
        "M17.971 4.285A2 2 0 0 1 21 6v12a2 2 0 0 1-3.029 1.715l-9.997-5.998a2 2 0 0 1-.003-3.432z",
    ),
    Shape::Path("M3 20V4"),
];

/// Lucide `skip-forward`.
const SKIP_FORWARD: &[Shape] = &[
    Shape::Path("M21 4v16"),
    Shape::Path(
        "M6.029 4.285A2 2 0 0 0 3 6v12a2 2 0 0 0 3.029 1.715l9.997-5.998a2 2 0 0 0 .003-3.432z",
    ),
];

/// Lucide `log-out`.
const LOG_OUT: &[Shape] = &[
    Shape::Path("m16 17 5-5-5-5"),
    Shape::Path("M21 12H9"),
    Shape::Path("M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"),
];

/// Lucide `rotate-ccw`: the power menu's Restart.
const RESTART: &[Shape] = &[
    Shape::Path("M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"),
    Shape::Path("M3 3v5h5"),
];

/// Lucide `headphones`.
const HEADPHONES: &[Shape] = &[Shape::Path(
    "M3 14h3a2 2 0 0 1 2 2v3a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-7a9 9 0 0 1 18 0v7a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3",
)];

/// Lucide `speaker`.
const SPEAKER: &[Shape] = &[
    Shape::Rect {
        x: "4",
        y: "2",
        width: "16",
        height: "20",
        rx: "2",
    },
    Shape::Path("M12 6h.01"),
    Shape::Circle {
        cx: "12",
        cy: "14",
        r: "4",
    },
    Shape::Path("M12 14h.01"),
];

/// Lucide `mouse`.
const MOUSE: &[Shape] = &[
    Shape::Rect {
        x: "5",
        y: "2",
        width: "14",
        height: "20",
        rx: "7",
    },
    Shape::Path("M12 6v4"),
];

/// Lucide `gamepad`.
const GAMEPAD: &[Shape] = &[
    Shape::Path("M6 12L10 12"),
    Shape::Path("M8 10L8 14"),
    Shape::Path("M15 13L15.01 13"),
    Shape::Path("M18 11L18.01 11"),
    Shape::Rect {
        x: "2",
        y: "6",
        width: "20",
        height: "12",
        rx: "2",
    },
];

/// Lucide `smartphone`.
const PHONE: &[Shape] = &[
    Shape::Rect {
        x: "5",
        y: "2",
        width: "14",
        height: "20",
        rx: "2",
    },
    Shape::Path("M12 18h.01"),
];
