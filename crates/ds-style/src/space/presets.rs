//! The eight preset Spaces and the default look for a workspace that has none
//! (design/03-COLOR.md section 7, design/21-SPACES.md section 4).

use super::look::SpaceLook;
use super::palette::Dot;
use crate::appearance::theme::Theme;

/// One preset: its name, and its dots.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Preset {
    /// What the editor calls it, its presets' accessible names: mailo's names, in
    /// design/21-SPACES.md section 4's order. (S's sample Spaces Work and Home are Spaces made
    /// from the first two, not the presets' names.)
    pub name: &'static str,
    /// One to three dots.
    pub dots: &'static [Dot],
}

const fn dot(hue: f32, chroma: f32) -> Dot {
    Dot { hue, chroma }
}

/// The eight presets, in the editor's order. Presets 0 and 1 are the sample Spaces Work and
/// Home (settled, `S:1118-1119`).
pub const PRESETS: [Preset; 8] = [
    Preset {
        name: "Dusk",
        dots: &[dot(268.0, 0.72), dot(318.0, 0.55)],
    },
    Preset {
        name: "Orchard",
        dots: &[dot(152.0, 0.62), dot(62.0, 0.55), dot(28.0, 0.5)],
    },
    Preset {
        name: "Harbour",
        dots: &[dot(220.0, 0.7)],
    },
    Preset {
        name: "Ember",
        dots: &[dot(20.0, 0.66), dot(55.0, 0.6)],
    },
    Preset {
        name: "Lagoon",
        dots: &[dot(190.0, 0.6), dot(240.0, 0.55)],
    },
    Preset {
        name: "Heather",
        dots: &[dot(340.0, 0.6), dot(290.0, 0.5)],
    },
    Preset {
        name: "Moss",
        dots: &[dot(95.0, 0.5)],
    },
    Preset {
        name: "Stone",
        dots: &[dot(250.0, 0.06)],
    },
];

/// The look a workspace with no stored one takes: `PRESETS[index % 8]`, theme System, and
/// `default_accent` (design/21-SPACES.md section 4, key `spaces.default_card_accent`).
pub fn default_look(index: usize, default_accent: super::look::CardAccent) -> SpaceLook {
    let preset = PRESETS[index % PRESETS.len()];
    SpaceLook {
        dots: preset.dots.to_vec(),
        theme: Theme::System,
        card_accent: default_accent,
    }
}

#[cfg(test)]
mod tests {
    use super::{PRESETS, default_look};
    use crate::appearance::theme::Theme;
    use crate::space::look::CardAccent;

    #[test]
    fn the_presets_are_named_in_the_design_order() {
        let names: Vec<&str> = PRESETS.iter().map(|preset| preset.name).collect();
        assert_eq!(
            names,
            [
                "Dusk", "Orchard", "Harbour", "Ember", "Lagoon", "Heather", "Moss", "Stone"
            ]
        );
    }

    #[test]
    fn a_workspace_takes_its_preset_by_index() {
        const CASES: &[(usize, usize)] = &[(0, 0), (1, 1), (2, 2), (7, 7), (8, 0), (10, 2)];
        for &(index, preset) in CASES {
            let look = default_look(index, CardAccent::SpaceHue);
            assert_eq!(look.dots, PRESETS[preset].dots, "workspace {index}");
            assert_eq!(look.theme, Theme::System, "workspace {index}");
            assert_eq!(look.card_accent, CardAccent::SpaceHue, "workspace {index}");
        }
    }
}
