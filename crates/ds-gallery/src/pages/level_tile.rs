//! One level specimen on its ground, shared by the Level page and the `--level-sheet` contact
//! sheet: an OSD card (title and a read-only `LevelControl`) in a transparent Osd root over either
//! the Work Space's frame or a light, wallpaper-like ground.

use dioxus::prelude::*;
use ds::{
    Appearance, BlurState, Ds, Fraction, Inject, Level, LevelGlyph, LevelLook, Material, Muting,
    Osd, PRESETS, RootChrome, Scheme, Shown, SpaceLook, Theme,
};

/// What the card sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ground {
    /// The Work Space's frame, in the card's scheme; the card takes the same Space's tint.
    Work,
    /// A light ground like a pale wallpaper, whatever the card's scheme: the Home Space's frame
    /// in the light scheme.
    Light,
}

impl Ground {
    /// Both grounds, in the sheet's order.
    pub const ALL: [Ground; 2] = [Ground::Work, Ground::Light];

    /// Its caption word.
    pub fn label(self) -> &'static str {
        match self {
            Ground::Work => "Work tint",
            Ground::Light => "light ground (Home)",
        }
    }
}

/// One level state a specimen shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    /// Its title line.
    pub title: &'static str,
    /// The level.
    pub value: Fraction,
    /// The glyph.
    pub glyph: LevelGlyph,
}

/// The states the brief asks for: volume at 0, 40 and 100 %, muted, and brightness at 30 %.
pub const STATES: [State; 5] = [
    State {
        title: "Sound 0 %",
        value: Fraction(0),
        glyph: LevelGlyph::Volume(Muting::Audible),
    },
    State {
        title: "Sound 40 %",
        value: Fraction(400),
        glyph: LevelGlyph::Volume(Muting::Audible),
    },
    State {
        title: "Sound 100 %",
        value: Fraction(1000),
        glyph: LevelGlyph::Volume(Muting::Audible),
    },
    State {
        title: "Sound, muted",
        value: Fraction(400),
        glyph: LevelGlyph::Volume(Muting::Muted),
    },
    State {
        title: "Display 30 %",
        value: Fraction(300),
        glyph: LevelGlyph::Brightness,
    },
];

/// The Work Space: the first preset (design/21 section 4).
pub fn work(theme: Theme) -> SpaceLook {
    SpaceLook {
        dots: PRESETS[0].dots.to_vec(),
        theme,
        ..SpaceLook::default()
    }
}

/// The Home Space: the second preset, in the light scheme.
fn home() -> SpaceLook {
    SpaceLook {
        dots: PRESETS[1].dots.to_vec(),
        theme: Theme::Light,
        ..SpaceLook::default()
    }
}

/// The theme for `scheme`.
pub fn theme(scheme: Scheme) -> Theme {
    match scheme {
        Scheme::Light => Theme::Light,
        Scheme::Dark => Theme::Dark,
    }
}

/// `state` in `look`, on an OSD card in `scheme` over `ground`.
#[component]
pub fn LevelTile(look: LevelLook, scheme: Scheme, ground: Ground, state: State) -> Element {
    let theme = theme(scheme);
    let (ground_look, ground_theme) = match ground {
        Ground::Work => (work(theme), theme),
        Ground::Light => (home(), Theme::Light),
    };
    rsx! {
        div { class: "g-level-tile",
            Ds {
                appearance: Appearance { theme: ground_theme, ..Appearance::default() },
                look: SpaceLook { theme: ground_theme, ..ground_look },
                material: Material::Window,
                stylesheet: Inject::Host,
                div { class: "g-level-ground",
                    Ds {
                        appearance: Appearance { theme, ..Appearance::default() },
                        look: work(theme),
                        material: Material::Osd,
                        chrome: Some(RootChrome::Transparent),
                        blur: BlurState::Available,
                        stylesheet: Inject::Host,
                        Osd {
                            shown: Shown::Visible,
                            label: state.title,
                            look,
                            level: Level { value: state.value, glyph: state.glyph },
                        }
                    }
                }
            }
        }
    }
}
