//! The control center parts 2 specimens (sill FINDINGS Q100-Q102): a `ModulePanel` holding a
//! level in each scheme, a bare one, and a three-column `ModuleGrid`.

use dioxus::prelude::*;
use ds::{
    Appearance, Ds, Fraction, GridColumns, Icon, Inject, LevelControl, LevelGlyph, Material,
    ModuleGrid, ModulePanel, ModuleState, ModuleTile, Muting, PanelPlate, Px, Theme, TileSpan,
};

/// One specimen of this file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartCase {
    /// The Sound module on the tile's plate, in a scheme.
    Panel(Theme),
    /// A panel with no header and no plate (a bar item's dropdown).
    Bare,
    /// Three columns, gap 6, padding 10, with a Full tile.
    ThreeColumns,
}

#[derive(Props, Clone, PartialEq)]
pub struct PartProps {
    pub case: PartCase,
}

pub const CASES: [(PartCase, &str); 4] = [
    (PartCase::Panel(Theme::Light), "panel-level-light"),
    (PartCase::Panel(Theme::Dark), "panel-level-dark"),
    (PartCase::Bare, "panel-bare"),
    (PartCase::ThreeColumns, "grid-3-columns"),
];

fn theme_of(case: PartCase) -> Theme {
    match case {
        PartCase::Panel(theme) => theme,
        PartCase::Bare | PartCase::ThreeColumns => Theme::Light,
    }
}

/// `case` in a Popover root of its scheme.
pub fn part(props: PartProps) -> Element {
    let body = match props.case {
        PartCase::Panel(_) => rsx! {
            ModuleGrid {
                ModulePanel { glyph: Icon::Volume2, title: "Speakers", trailing: rsx! { "40%" },
                    LevelControl { label: "Volume", value: Fraction(400), glyph: LevelGlyph::Volume(Muting::Audible), onchange: |_| {} }
                }
            }
        },
        PartCase::Bare => rsx! {
            ModulePanel { plate: PanelPlate::Bare, span: TileSpan::Half,
                LevelControl { label: "Brightness", value: Fraction(300), glyph: LevelGlyph::Brightness, onchange: |_| {} }
            }
        },
        PartCase::ThreeColumns => rsx! {
            ModuleGrid { columns: GridColumns(3), gap: Px(6.0), padding: Px(10.0),
                ModuleTile { glyph: Icon::Wifi, title: "Wi-Fi", state: ModuleState::On, onclick: |_| {} }
                ModuleTile { glyph: Icon::Bluetooth, title: "Bluetooth", state: ModuleState::Off, onclick: |_| {} }
                ModuleTile { glyph: Icon::Moon, title: "Focus", state: ModuleState::Off, onclick: |_| {} }
                ModuleTile { glyph: Icon::Play, title: "Now Playing", state: ModuleState::Off, span: TileSpan::Full, onclick: |_| {} }
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
