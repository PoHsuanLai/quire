//! Solid forms drawn here: the Wi-Fi fan and the speaker with its waves (design/08-ICONS.md
//! section 1.2.1). Neither Tabler nor Phosphor draws these as separate parts, and the animated
//! glyphs (`PosedGlyph`, the status glyphs, the level glyph) light them one by one.
//!
//! The speaker body is Lucide `volume`'s path (ISC, `LICENSE-lucide.txt`) filled; the fan's arcs
//! are three concentric annular sectors on (12, 20) at radii 5, 10 and 15, 2.6 thick, and the
//! waves three on (10, 12) at radii 5, 8.25 and 11.5, 2.2 thick, each with round ends.

use super::Icon;
use super::shape::Shape;

/// The Wi-Fi dot, on (12, 20).
pub const WIFI_DOT: Shape = Shape::Solid("M10 20a2 2 0 1 0 4 0a2 2 0 1 0-4 0z");
/// The fan's inner arc.
pub const WIFI_SMALL: Shape = Shape::Solid(
    "M7.59 15.5A6.3 6.3 0 0 1 16.41 15.5A1.3 1.3 0 0 1 14.59 17.36A3.7 3.7 0 0 0 9.41 17.36A1.3 1.3 0 0 1 7.59 15.5Z",
);
/// The fan's middle arc.
pub const WIFI_MID: Shape = Shape::Solid(
    "M4.09 11.93A11.3 11.3 0 0 1 19.91 11.93A1.3 1.3 0 0 1 18.09 13.78A8.7 8.7 0 0 0 5.91 13.78A1.3 1.3 0 0 1 4.09 11.93Z",
);
/// The fan's outer arc.
pub const WIFI_LARGE: Shape = Shape::Solid(
    "M1.14 7.85A16.3 16.3 0 0 1 22.86 7.85A1.3 1.3 0 0 1 21.13 9.79A13.7 13.7 0 0 0 2.87 9.79A1.3 1.3 0 0 1 1.14 7.85Z",
);

/// The speaker's body: Lucide `volume`, filled.
pub const SPEAKER: Shape = Shape::Solid(
    "M11 4.702a.705.705 0 0 0-1.203-.498L6.413 7.587A1.4 1.4 0 0 1 5.416 8H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2.416a1.4 1.4 0 0 1 .997.413l3.383 3.384A.705.705 0 0 0 11 19.298z",
);
/// The first wave, nearest the speaker.
pub const WAVE_1: Shape = Shape::Solid(
    "M14.67 8.08A6.1 6.1 0 0 1 14.67 15.92A1.1 1.1 0 0 1 12.99 14.51A3.9 3.9 0 0 0 12.99 9.49A1.1 1.1 0 0 1 14.67 8.08Z",
);
/// The second wave.
pub const WAVE_2: Shape = Shape::Solid(
    "M17.16 5.99A9.35 9.35 0 0 1 17.16 18.01A1.1 1.1 0 0 1 15.48 16.6A7.15 7.15 0 0 0 15.48 7.4A1.1 1.1 0 0 1 17.16 5.99Z",
);
/// The third wave, the furthest.
pub const WAVE_3: Shape = Shape::Solid(
    "M19.65 3.9A12.6 12.6 0 0 1 19.65 20.1A1.1 1.1 0 0 1 17.97 18.68A10.4 10.4 0 0 0 17.97 5.32A1.1 1.1 0 0 1 19.65 3.9Z",
);

const FAN_LOW: &[Shape] = &[WIFI_DOT, WIFI_SMALL];
const FAN_HIGH: &[Shape] = &[WIFI_DOT, WIFI_SMALL, WIFI_MID];
const FAN: &[Shape] = &[WIFI_DOT, WIFI_LARGE, WIFI_MID, WIFI_SMALL];
const SPEAKER_NONE: &[Shape] = &[SPEAKER];
const SPEAKER_LOW: &[Shape] = &[SPEAKER, WAVE_1];
const SPEAKER_HIGH: &[Shape] = &[SPEAKER, WAVE_1, WAVE_2];

/// The solid children of a glyph this file holds; empty for any other. The fan lists the dot,
/// then the outer, middle and inner arc, in Lucide's order, so [`Icon::parts`] indexes both.
pub(super) fn shapes(icon: Icon) -> &'static [Shape] {
    match icon {
        Icon::Wifi => FAN,
        Icon::WifiLow => FAN_LOW,
        Icon::WifiHigh => FAN_HIGH,
        Icon::Volume => SPEAKER_NONE,
        Icon::Volume1 => SPEAKER_LOW,
        Icon::Volume2 => SPEAKER_HIGH,
        _ => &[],
    }
}
