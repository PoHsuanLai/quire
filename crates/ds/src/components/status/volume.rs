//! VolumeGlyph: the bar's volume item on `LevelGlyph`'s layers (design/26-DETAILS.md 5.1.4;
//! G12): the waves cross-fade by thirds over `--t-quick` as the level moves, and muting fades the
//! waves out as the slash comes in. A held volume key moves nothing else (R12): no count, no bump.

use crate::components::level::glyph::{LevelGlyphView, waves};
use crate::components::level::vocab::{LevelGlyph, Muting};
use crate::components::vocab::Fraction;
use crate::detail::{Detailed, Moment};
use crate::icon::render::IconSize;
use dioxus::prelude::*;

/// How many waves a heard speaker shows: none at 0, then by thirds (as `LevelGlyph`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VolumeWaves {
    /// Silent: the body alone.
    Zero,
    /// A third or less.
    One,
    /// Two thirds or less.
    Two,
    /// Above.
    Three,
}

impl VolumeWaves {
    /// The waves a level (thousandths) shows (R2: 50 then 51 % is the same two waves).
    pub fn of(level: Fraction) -> VolumeWaves {
        match waves(level) {
            0 => VolumeWaves::Zero,
            1 => VolumeWaves::One,
            2 => VolumeWaves::Two,
            _ => VolumeWaves::Three,
        }
    }

    /// A level inside this band, for `LevelGlyph`'s own quantising.
    fn level(self) -> Fraction {
        match self {
            VolumeWaves::Zero => Fraction(0),
            VolumeWaves::One => Fraction(333),
            VolumeWaves::Two => Fraction(666),
            VolumeWaves::Three => Fraction(1000),
        }
    }
}

/// What the volume item shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VolumeState {
    /// Heard, with this many waves.
    Heard(VolumeWaves),
    /// Muted: no waves, the slash.
    Muted,
    /// No output device: drawn as muted, said in words.
    NoDevice,
}

impl VolumeState {
    /// The state in words (R8).
    pub fn words(self) -> &'static str {
        match self {
            VolumeState::Heard(VolumeWaves::Zero) => "Silent",
            VolumeState::Heard(_) => "Sound on",
            VolumeState::Muted => "Muted",
            VolumeState::NoDevice => "No output device",
        }
    }

    fn glyph(self) -> (LevelGlyph, Fraction) {
        match self {
            VolumeState::Heard(waves) => (LevelGlyph::Volume(Muting::Audible), waves.level()),
            VolumeState::Muted | VolumeState::NoDevice => {
                (LevelGlyph::Volume(Muting::Muted), Fraction(0))
            }
        }
    }
}

impl Detailed for VolumeState {
    fn moment(from: &Self, to: &Self) -> Moment {
        use VolumeState::{Heard, Muted, NoDevice};
        match (from, to) {
            (a, b) if a == b => Moment::Rest,
            (Heard(_) | Muted | NoDevice, NoDevice) => Moment::Unavailable,
            (Heard(_) | Muted | NoDevice, Heard(_) | Muted) => Moment::Change,
        }
    }

    fn first(state: &Self) -> Moment {
        match state {
            VolumeState::Heard(_) | VolumeState::Muted | VolumeState::NoDevice => Moment::Rest,
        }
    }
}

/// `span.ds-status-glyph[data-kind=volume]`: the speaker in `state` at `size`, in
/// `currentColor`. Decorative; put [`VolumeState::words`] beside it (R8). Its only motion is
/// `LevelGlyph`'s own `--t-quick` cross-fades, so it paints 0 frames once they end (R3).
#[component]
pub fn VolumeGlyph(
    state: VolumeState,
    #[props(default = IconSize::Bar)] size: IconSize,
) -> Element {
    let (glyph, value) = state.glyph();
    let px = size.px();
    rsx! {
        span { class: "ds-status-glyph", "data-kind": "volume", "aria-hidden": "true",
            style: "width:{px}px;height:{px}px",
            LevelGlyphView { glyph, value, size }
        }
    }
}
