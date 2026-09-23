//! The eight preset Spaces and the default look for a workspace that has none
//! (design/03-COLOR.md section 7, design/21-SPACES.md section 4).

use super::look::SpaceLook;
use super::palette::Dot;
use crate::appearance::Theme;

/// One preset: its dots, and the grain it ships with when the prototype names one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Preset {
    /// One to three dots.
    pub dots: &'static [Dot],
    /// The grain the prototype gives it (Work 35, Home 55), or `None` for the settings default
    /// `spaces.default_grain`.
    pub grain: Option<u8>,
}

const fn dot(hue: f32, chroma: f32) -> Dot {
    Dot { hue, chroma }
}

/// The eight presets, in the editor's order. Presets 0 and 1 are the sample Spaces Work and
/// Home (settled, `S:1118-1119`).
pub const PRESETS: [Preset; 8] = [
    Preset {
        dots: &[dot(268.0, 0.72), dot(318.0, 0.55)],
        grain: Some(35),
    },
    Preset {
        dots: &[dot(152.0, 0.62), dot(62.0, 0.55), dot(28.0, 0.5)],
        grain: Some(55),
    },
    Preset {
        dots: &[dot(220.0, 0.7)],
        grain: None,
    },
    Preset {
        dots: &[dot(20.0, 0.66), dot(55.0, 0.6)],
        grain: None,
    },
    Preset {
        dots: &[dot(190.0, 0.6), dot(240.0, 0.55)],
        grain: None,
    },
    Preset {
        dots: &[dot(340.0, 0.6), dot(290.0, 0.5)],
        grain: None,
    },
    Preset {
        dots: &[dot(95.0, 0.5)],
        grain: None,
    },
    Preset {
        dots: &[dot(250.0, 0.06)],
        grain: None,
    },
];

/// The look a workspace with no stored one takes: `PRESETS[index % 8]`, the preset's own grain
/// or `default_grain`, theme System, and `default_accent` (design/21-SPACES.md section 4,
/// keys `spaces.default_grain` and `spaces.default_card_accent`).
pub fn default_look(
    index: usize,
    default_grain: super::look::Grain,
    default_accent: super::look::CardAccent,
) -> SpaceLook {
    let preset = PRESETS[index % PRESETS.len()];
    SpaceLook {
        dots: preset.dots.to_vec(),
        grain: preset.grain.map_or(default_grain, super::look::Grain),
        theme: Theme::System,
        card_accent: default_accent,
    }
}

#[cfg(test)]
mod tests {
    use super::{PRESETS, default_look};
    use crate::appearance::Theme;
    use crate::space::look::{CardAccent, Grain};

    #[test]
    fn a_workspace_takes_its_preset_by_index() {
        const CASES: &[(usize, usize, u8)] = &[
            (0, 0, 35),
            (1, 1, 55),
            (2, 2, 40),
            (7, 7, 40),
            (8, 0, 35),
            (10, 2, 40),
        ];
        for &(index, preset, grain) in CASES {
            let look = default_look(index, Grain(40), CardAccent::SpaceHue);
            assert_eq!(look.dots, PRESETS[preset].dots, "workspace {index}");
            assert_eq!(look.grain, Grain(grain), "workspace {index}");
            assert_eq!(look.theme, Theme::System, "workspace {index}");
            assert_eq!(look.card_accent, CardAccent::SpaceHue, "workspace {index}");
        }
    }
}
