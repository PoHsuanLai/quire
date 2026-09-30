//! The control center parts 2 specimens: a `ModulePanel` holding a
//! level in each scheme, a bare one, a three-column `ModuleGrid`, and the compact picker.

use dioxus::prelude::*;
use ds::Check;
use ds::{
    Accent, Appearance, Arrangement, Choice, Ds, Fraction, Icon, Inject, LevelGlyph, Material,
    Muting, Px, RadioGroup, SectionHeader, SegmentedControl, Theme, Tracking, Word,
};
use ds::{Slider, SliderLook};
use ds_shell::{GridColumns, ModuleGrid, ModulePanel, ModuleTile, PanelPlate, TileSpan};

/// One specimen of this file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartCase {
    /// The Sound module on the tile's plate, in a scheme.
    Panel(Theme),
    /// A panel with no header and no plate (a bar item's dropdown).
    Bare,
    /// Three columns, gap 6, padding 10, with a Full tile.
    ThreeColumns,
    /// The compact picker on a panel.
    Picker,
}

#[derive(Props, Clone, PartialEq)]
pub struct PartProps {
    pub case: PartCase,
}

pub const CASES: [(PartCase, &str); 5] = [
    (PartCase::Panel(Theme::Light), "panel-level-light"),
    (PartCase::Panel(Theme::Dark), "panel-level-dark"),
    (PartCase::Bare, "panel-bare"),
    (PartCase::ThreeColumns, "grid-3-columns"),
    (PartCase::Picker, "picker-compact"),
];

fn theme_of(case: PartCase) -> Theme {
    match case {
        PartCase::Panel(theme) => theme,
        PartCase::Bare | PartCase::ThreeColumns | PartCase::Picker => Theme::Light,
    }
}

/// `case` in a Popover root of its scheme.
pub fn part(props: PartProps) -> Element {
    let body = match props.case {
        PartCase::Panel(_) => rsx! {
            ModuleGrid {
                ModulePanel { glyph: Icon::Volume2, title: "Speakers", trailing: rsx! { "40%" },
                    Slider { label: "Volume", value: Fraction(400), glyph: LevelGlyph::Volume(Muting::Audible), onchange: |_| {}, look: SliderLook::Capsule }
                }
            }
        },
        PartCase::Bare => rsx! {
            ModulePanel { plate: PanelPlate::Bare, span: TileSpan::Half,
                Slider { label: "Brightness", value: Fraction(300), glyph: LevelGlyph::Brightness, onchange: |_| {}, look: SliderLook::Capsule }
            }
        },
        PartCase::ThreeColumns => rsx! {
            ModuleGrid { columns: GridColumns(3), gap: Px(6.0), padding: Px(10.0),
                ModuleTile { glyph: Icon::Wifi, title: "Wi-Fi", value: Check::On, onclick: |_| {} }
                ModuleTile { glyph: Icon::Bluetooth, title: "Bluetooth", value: Check::Off, onclick: |_| {} }
                ModuleTile { glyph: Icon::Moon, title: "Focus", value: Check::Off, onclick: |_| {} }
                ModuleTile { glyph: Icon::Play, title: "Now Playing", value: Check::Off, span: TileSpan::Full, onclick: |_| {} }
            }
        },
        PartCase::Picker => rsx! {
                    ModuleGrid {
                        ModulePanel {
        div { style: "display:grid;gap:14px",
                            SectionHeader { title: "Theme" }
                            SegmentedControl::<Theme> {
                                label: "Theme",
                                choices: Choice::pairs(Theme::ALL.iter().map(|theme| (*theme, theme.label()))),
                                tracking: Tracking::SelectOne(Appearance::default().theme),
                                onchange: |_| {},
                            }
                            SectionHeader { title: "Accent" }
                            RadioGroup::<Accent> {
                                label: "Accent",
                                arrangement: Arrangement::Swatches,
                                choices: Accent::ALL.iter().map(|accent| Choice::accent(*accent)).collect(),
                                value: Appearance::default().accent,
                                onchange: |_| {},
                            }
                        }
                        }
                    }
                },
    };
    rsx! {
        Ds {
            appearance: Appearance { theme: theme_of(props.case), ..Appearance::default() },
            material: Material::Popover,
            stylesheet: Inject::Host,
            {body}
        }
    }
}
