//! Lucide geometry for the shell set (design/08-ICONS.md section 1.6), one constant per glyph,
//! transcribed from `lucide-static` 1.47.0 (<https://unpkg.com/lucide-static@1.47.0/icons/>),
//! ISC licence; see `super`. `Brightness` is Tabler Icons 3.48.0 `outline/brightness-half`
//! (MIT, <https://tabler.io/icons>), imported unchanged except for Tabler's invisible bounding
//! path (design/08-ICONS.md section 1.2 rule 6). A Lucide `<line>` is written as the equivalent
//! path `M x1 y1 L x2 y2`, since [`Shape`] has no line; every other child is as published.

use super::Icon;
use super::shape::Shape;

/// The children of a shell-set glyph. The mailo set never reaches here: `Icon::shapes`
/// matches it first, so a mailo glyph here is a broken invariant and draws nothing.
pub(super) fn shapes(icon: Icon) -> &'static [Shape] {
    match icon {
        Icon::Wifi => WIFI,
        Icon::WifiLow => WIFI_LOW,
        Icon::WifiHigh => WIFI_HIGH,
        Icon::WifiOff => WIFI_OFF,
        Icon::Ethernet => ETHERNET,
        Icon::Battery => BATTERY,
        Icon::BatteryLow => BATTERY_LOW,
        Icon::BatteryMedium => BATTERY_MEDIUM,
        Icon::BatteryFull => BATTERY_FULL,
        Icon::BatteryCharging => BATTERY_CHARGING,
        Icon::BatteryWarning => BATTERY_WARNING,
        Icon::Bluetooth => BLUETOOTH,
        Icon::BluetoothConnected => BLUETOOTH_CONNECTED,
        Icon::BluetoothOff => BLUETOOTH_OFF,
        Icon::Volume => VOLUME,
        Icon::Volume1 => VOLUME1,
        Icon::Volume2 => VOLUME2,
        Icon::VolumeX => VOLUME_X,
        Icon::Mic => MIC,
        Icon::MicOff => MIC_OFF,
        Icon::Sun => SUN,
        Icon::Moon => MOON,
        Icon::SunMoon => SUN_MOON,
        Icon::Power => POWER,
        Icon::Lock => LOCK,
        Icon::Bell => BELL,
        Icon::BellOff => BELL_OFF,
        Icon::Grid => GRID,
        Icon::Window => WINDOW,
        Icon::Monitor => MONITOR,
        Icon::Keyboard => KEYBOARD,
        Icon::ChevronLeft => CHEVRON_LEFT,
        Icon::ChevronRight => CHEVRON_RIGHT,
        Icon::ChevronUp => CHEVRON_UP,
        Icon::ChevronDown => CHEVRON_DOWN,
        Icon::Folder => FOLDER,
        Icon::File => FILE,
        Icon::Image => IMAGE,
        Icon::Terminal => TERMINAL,
        Icon::StickyNote => STICKY_NOTE,
        Icon::Camera => CAMERA,
        Icon::Download => DOWNLOAD,
        Icon::Upload => UPLOAD,
        Icon::Copy => COPY,
        Icon::Link => LINK,
        Icon::Sparkles => SPARKLES,
        Icon::Gauge => GAUGE,
        Icon::Brightness => BRIGHTNESS,
        _ => &[],
    }
}

/// Lucide `wifi`.
const WIFI: &[Shape] = &[
    Shape::Path("M12 20h.01"),
    Shape::Path("M2 8.82a15 15 0 0 1 20 0"),
    Shape::Path("M5 12.859a10 10 0 0 1 14 0"),
    Shape::Path("M8.5 16.429a5 5 0 0 1 7 0"),
];

/// Lucide `wifi-low`.
const WIFI_LOW: &[Shape] = &[
    Shape::Path("M12 20h.01"),
    Shape::Path("M8.5 16.429a5 5 0 0 1 7 0"),
];

/// Lucide `wifi-high`.
const WIFI_HIGH: &[Shape] = &[
    Shape::Path("M12 20h.01"),
    Shape::Path("M5 12.859a10 10 0 0 1 14 0"),
    Shape::Path("M8.5 16.429a5 5 0 0 1 7 0"),
];

/// Lucide `wifi-off`.
const WIFI_OFF: &[Shape] = &[
    Shape::Path("M12 20h.01"),
    Shape::Path("M8.5 16.429a5 5 0 0 1 7 0"),
    Shape::Path("M5 12.859a10 10 0 0 1 5.17-2.69"),
    Shape::Path("M19 12.859a10 10 0 0 0-2.007-1.523"),
    Shape::Path("M2 8.82a15 15 0 0 1 4.177-2.643"),
    Shape::Path("M22 8.82a15 15 0 0 0-11.288-3.764"),
    Shape::Path("m2 2 20 20"),
];

const ETHERNET: &[Shape] = &[
    Shape::Path("M10 8v1"),
    Shape::Path("M14 8v1"),
    Shape::Path("M18 8v1"),
    Shape::Path(
        "M19 17a2 2 0 00-1.765 1.059l-.47.882A2 2 0 0115 20H9a2 2 0 01-1.765-1.059l-.47-.882A2 2 0 005 17H4a2 2 0 01-2-2V6a2 2 0 012-2h16a2 2 0 012 2v9a2 2 0 01-2 2z",
    ),
    Shape::Path("M6 8v1"),
];

/// Lucide `battery`.
const BATTERY: &[Shape] = &[
    Shape::Path("M 22 14 L 22 10"),
    Shape::Rect {
        x: "2",
        y: "6",
        width: "16",
        height: "12",
        rx: "2",
    },
];

/// Lucide `battery-low`.
const BATTERY_LOW: &[Shape] = &[
    Shape::Path("M22 14v-4"),
    Shape::Path("M6 14v-4"),
    Shape::Rect {
        x: "2",
        y: "6",
        width: "16",
        height: "12",
        rx: "2",
    },
];

/// Lucide `battery-medium`.
const BATTERY_MEDIUM: &[Shape] = &[
    Shape::Path("M10 14v-4"),
    Shape::Path("M22 14v-4"),
    Shape::Path("M6 14v-4"),
    Shape::Rect {
        x: "2",
        y: "6",
        width: "16",
        height: "12",
        rx: "2",
    },
];

/// Lucide `battery-full`.
const BATTERY_FULL: &[Shape] = &[
    Shape::Path("M10 10v4"),
    Shape::Path("M14 10v4"),
    Shape::Path("M22 14v-4"),
    Shape::Path("M6 10v4"),
    Shape::Rect {
        x: "2",
        y: "6",
        width: "16",
        height: "12",
        rx: "2",
    },
];

/// Lucide `battery-charging`.
const BATTERY_CHARGING: &[Shape] = &[
    Shape::Path("m11 7-3 5h4l-3 5"),
    Shape::Path("M14.856 6H16a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2h-2.935"),
    Shape::Path("M22 14v-4"),
    Shape::Path("M5.14 18H4a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h2.936"),
];

/// Lucide `battery-warning`.
const BATTERY_WARNING: &[Shape] = &[
    Shape::Path("M10 17h.01"),
    Shape::Path("M10 7v6"),
    Shape::Path("M14 6h2a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2h-2"),
    Shape::Path("M22 14v-4"),
    Shape::Path("M6 18H4a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h2"),
];

/// Lucide `bluetooth`.
const BLUETOOTH: &[Shape] = &[Shape::Path("m7 7 10 10-5 5V2l5 5L7 17")];

/// Lucide `bluetooth-connected`.
const BLUETOOTH_CONNECTED: &[Shape] = &[
    Shape::Path("m7 7 10 10-5 5V2l5 5L7 17"),
    Shape::Path("M18 12L21 12"),
    Shape::Path("M3 12L6 12"),
];

/// Lucide `bluetooth-off`.
const BLUETOOTH_OFF: &[Shape] = &[
    Shape::Path("m17 17-5 5V12l-5 5"),
    Shape::Path("m2 2 20 20"),
    Shape::Path("M14.5 9.5 17 7l-5-5v4.5"),
];

/// Lucide `volume`.
const VOLUME: &[Shape] = &[Shape::Path(
    "M11 4.702a.705.705 0 0 0-1.203-.498L6.413 7.587A1.4 1.4 0 0 1 5.416 8H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2.416a1.4 1.4 0 0 1 .997.413l3.383 3.384A.705.705 0 0 0 11 19.298z",
)];

/// Lucide `volume-1`.
const VOLUME1: &[Shape] = &[
    Shape::Path(
        "M11 4.702a.705.705 0 0 0-1.203-.498L6.413 7.587A1.4 1.4 0 0 1 5.416 8H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2.416a1.4 1.4 0 0 1 .997.413l3.383 3.384A.705.705 0 0 0 11 19.298z",
    ),
    Shape::Path("M16 9a5 5 0 0 1 0 6"),
];

/// Lucide `volume-2`.
const VOLUME2: &[Shape] = &[
    Shape::Path(
        "M11 4.702a.705.705 0 0 0-1.203-.498L6.413 7.587A1.4 1.4 0 0 1 5.416 8H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2.416a1.4 1.4 0 0 1 .997.413l3.383 3.384A.705.705 0 0 0 11 19.298z",
    ),
    Shape::Path("M16 9a5 5 0 0 1 0 6"),
    Shape::Path("M19.364 18.364a9 9 0 0 0 0-12.728"),
];

/// Lucide `volume-x`.
const VOLUME_X: &[Shape] = &[
    Shape::Path(
        "M11 4.702a.7.7 0 0 0-1.203-.498L6.413 7.587A1.4 1.4 0 0 1 5.416 8H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2.416a1.4 1.4 0 0 1 .997.413l3.383 3.384A.7.7 0 0 0 11 19.298z",
    ),
    Shape::Path("m16.5 14.5 5-5"),
    Shape::Path("m16.5 9.5 5 5"),
];

/// Lucide `mic`.
const MIC: &[Shape] = &[
    Shape::Path("M12 19v3"),
    Shape::Path("M19 10v2a7 7 0 0 1-14 0v-2"),
    Shape::Rect {
        x: "9",
        y: "2",
        width: "6",
        height: "13",
        rx: "3",
    },
];

/// Lucide `mic-off`.
const MIC_OFF: &[Shape] = &[
    Shape::Path("M12 19v3"),
    Shape::Path("M15 9.34V5a3 3 0 0 0-5.68-1.33"),
    Shape::Path("M16.95 16.95A7 7 0 0 1 5 12v-2"),
    Shape::Path("M18.89 13.23A7 7 0 0 0 19 12v-2"),
    Shape::Path("m2 2 20 20"),
    Shape::Path("M9 9v3a3 3 0 0 0 5.12 2.12"),
];

/// Lucide `sun`.
const SUN: &[Shape] = &[
    Shape::Circle {
        cx: "12",
        cy: "12",
        r: "4",
    },
    Shape::Path("M12 2v2"),
    Shape::Path("M12 20v2"),
    Shape::Path("m4.93 4.93 1.41 1.41"),
    Shape::Path("m17.66 17.66 1.41 1.41"),
    Shape::Path("M2 12h2"),
    Shape::Path("M20 12h2"),
    Shape::Path("m6.34 17.66-1.41 1.41"),
    Shape::Path("m19.07 4.93-1.41 1.41"),
];

/// Lucide `moon`.
const MOON: &[Shape] = &[Shape::Path(
    "M20.985 12.486a9 9 0 1 1-9.473-9.472c.405-.022.617.46.402.803a6 6 0 0 0 8.268 8.268c.344-.215.825-.004.803.401",
)];

/// Lucide `sun-moon`.
const SUN_MOON: &[Shape] = &[
    Shape::Path("M12 2v2"),
    Shape::Path(
        "M14.837 16.385a6 6 0 1 1-7.223-7.222c.624-.147.97.66.715 1.248a4 4 0 0 0 5.26 5.259c.589-.255 1.396.09 1.248.715",
    ),
    Shape::Path("M16 12a4 4 0 0 0-4-4"),
    Shape::Path("m19 5-1.256 1.256"),
    Shape::Path("M20 12h2"),
];

/// Lucide `power`.
const POWER: &[Shape] = &[
    Shape::Path("M12 2v10"),
    Shape::Path("M18.4 6.6a9 9 0 1 1-12.77.04"),
];

/// Lucide `lock`.
const LOCK: &[Shape] = &[
    Shape::Rect {
        x: "3",
        y: "11",
        width: "18",
        height: "11",
        rx: "2",
    },
    Shape::Path("M7 11V7a5 5 0 0 1 10 0v4"),
];

/// Lucide `bell`.
const BELL: &[Shape] = &[
    Shape::Path("M10.268 21a2 2 0 0 0 3.464 0"),
    Shape::Path(
        "M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326",
    ),
];

/// Lucide `bell-off`.
const BELL_OFF: &[Shape] = &[
    Shape::Path("M10.268 21a2 2 0 0 0 3.464 0"),
    Shape::Path("M17 17H4a1 1 0 0 1-.74-1.673C4.59 13.956 6 12.499 6 8a6 6 0 0 1 .258-1.742"),
    Shape::Path("m2 2 20 20"),
    Shape::Path("M8.668 3.01A6 6 0 0 1 18 8c0 2.687.77 4.653 1.707 6.05"),
];

/// Lucide `layout-grid`.
const GRID: &[Shape] = &[
    Shape::Rect {
        x: "3",
        y: "3",
        width: "7",
        height: "7",
        rx: "1",
    },
    Shape::Rect {
        x: "14",
        y: "3",
        width: "7",
        height: "7",
        rx: "1",
    },
    Shape::Rect {
        x: "14",
        y: "14",
        width: "7",
        height: "7",
        rx: "1",
    },
    Shape::Rect {
        x: "3",
        y: "14",
        width: "7",
        height: "7",
        rx: "1",
    },
];

/// Lucide `app-window`.
const WINDOW: &[Shape] = &[
    Shape::Rect {
        x: "2",
        y: "4",
        width: "20",
        height: "16",
        rx: "2",
    },
    Shape::Path("M10 4v4"),
    Shape::Path("M2 8h20"),
    Shape::Path("M6 4v4"),
];

/// Lucide `monitor`.
const MONITOR: &[Shape] = &[
    Shape::Rect {
        x: "2",
        y: "3",
        width: "20",
        height: "14",
        rx: "2",
    },
    Shape::Path("M8 21L16 21"),
    Shape::Path("M12 17L12 21"),
];

/// Lucide `keyboard`.
const KEYBOARD: &[Shape] = &[
    Shape::Path("M10 8h.01"),
    Shape::Path("M12 12h.01"),
    Shape::Path("M14 8h.01"),
    Shape::Path("M16 12h.01"),
    Shape::Path("M18 8h.01"),
    Shape::Path("M6 8h.01"),
    Shape::Path("M7 16h10"),
    Shape::Path("M8 12h.01"),
    Shape::Rect {
        x: "2",
        y: "4",
        width: "20",
        height: "16",
        rx: "2",
    },
];

/// Lucide `chevron-left`.
const CHEVRON_LEFT: &[Shape] = &[Shape::Path("m15 18-6-6 6-6")];

/// Lucide `chevron-right`.
const CHEVRON_RIGHT: &[Shape] = &[Shape::Path("m9 18 6-6-6-6")];

/// Lucide `chevron-up`.
const CHEVRON_UP: &[Shape] = &[Shape::Path("m18 15-6-6-6 6")];

/// Lucide `chevron-down`.
const CHEVRON_DOWN: &[Shape] = &[Shape::Path("m6 9 6 6 6-6")];

/// Lucide `folder`.
const FOLDER: &[Shape] = &[Shape::Path(
    "M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z",
)];

/// Lucide `file`.
const FILE: &[Shape] = &[
    Shape::Path(
        "M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z",
    ),
    Shape::Path("M14 2v5a1 1 0 0 0 1 1h5"),
];

/// Lucide `image`.
const IMAGE: &[Shape] = &[
    Shape::Rect {
        x: "3",
        y: "3",
        width: "18",
        height: "18",
        rx: "2",
    },
    Shape::Circle {
        cx: "9",
        cy: "9",
        r: "2",
    },
    Shape::Path("m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"),
];

/// Lucide `square-terminal`.
const TERMINAL: &[Shape] = &[
    Shape::Path("m7 11 2-2-2-2"),
    Shape::Path("M11 13h4"),
    Shape::Rect {
        x: "3",
        y: "3",
        width: "18",
        height: "18",
        rx: "2",
    },
];

/// Lucide `sticky-note`.
const STICKY_NOTE: &[Shape] = &[
    Shape::Path(
        "M21 9a2.4 2.4 0 0 0-.706-1.706l-3.588-3.588A2.4 2.4 0 0 0 15 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2z",
    ),
    Shape::Path("M15 3v5a1 1 0 0 0 1 1h5"),
];

/// Lucide `camera`.
const CAMERA: &[Shape] = &[
    Shape::Path(
        "M13.997 4a2 2 0 0 1 1.76 1.05l.486.9A2 2 0 0 0 18.003 7H20a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V9a2 2 0 0 1 2-2h1.997a2 2 0 0 0 1.759-1.048l.489-.904A2 2 0 0 1 10.004 4z",
    ),
    Shape::Circle {
        cx: "12",
        cy: "13",
        r: "3",
    },
];

/// Lucide `download`.
const DOWNLOAD: &[Shape] = &[
    Shape::Path("M12 15V3"),
    Shape::Path("M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"),
    Shape::Path("m7 10 5 5 5-5"),
];

/// Lucide `upload`.
const UPLOAD: &[Shape] = &[
    Shape::Path("M12 3v12"),
    Shape::Path("m17 8-5-5-5 5"),
    Shape::Path("M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"),
];

/// Lucide `copy`.
const COPY: &[Shape] = &[
    Shape::Rect {
        x: "8",
        y: "8",
        width: "14",
        height: "14",
        rx: "2",
    },
    Shape::Path("M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"),
];

/// Lucide `link`.
const LINK: &[Shape] = &[
    Shape::Path("M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"),
    Shape::Path("M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"),
];

/// Lucide `sparkles`.
const SPARKLES: &[Shape] = &[
    Shape::Path(
        "M11.017 2.814a1 1 0 0 1 1.966 0l1.051 5.558a2 2 0 0 0 1.594 1.594l5.558 1.051a1 1 0 0 1 0 1.966l-5.558 1.051a2 2 0 0 0-1.594 1.594l-1.051 5.558a1 1 0 0 1-1.966 0l-1.051-5.558a2 2 0 0 0-1.594-1.594l-5.558-1.051a1 1 0 0 1 0-1.966l5.558-1.051a2 2 0 0 0 1.594-1.594z",
    ),
    Shape::Path("M20 2v4"),
    Shape::Path("M22 4h-4"),
    Shape::Circle {
        cx: "4",
        cy: "20",
        r: "2",
    },
];

/// Lucide `gauge`.
const GAUGE: &[Shape] = &[
    Shape::Path("m12 14 4-4"),
    Shape::Path("M3.34 19a10 10 0 1 1 17.32 0"),
];

/// Tabler `brightness-half`.
const BRIGHTNESS: &[Shape] = &[
    Shape::Path("M12 9a3 3 0 0 0 0 6v-6"),
    Shape::Path(
        "M6 6h3.5l2.5 -2.5l2.5 2.5h3.5v3.5l2.5 2.5l-2.5 2.5v3.5h-3.5l-2.5 2.5l-2.5 -2.5h-3.5v-3.5l-2.5 -2.5l2.5 -2.5l0 -3.5",
    ),
];
