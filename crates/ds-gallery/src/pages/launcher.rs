//! The Overlays page's embedded palette: `CommandPalette` hosted in a surface of its own, as
//! sill's launcher panel draws it (sill FINDINGS Q40-Q42): no scrim, the card filling its
//! container and carrying the id a blur region names, `cmdk-in`, app icons in the rows.

use super::app_icons::{APPS, app_icon};
use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::{
    Availability, CommandPalette, CommandPaletteHost, Corner, Icon, IconSize, Material, MenuEntry,
    PaletteEntrance, Radius, Shortcut, Surface, Tile, Trail, components::vocab::Key,
};

fn row(value: u8, title: &str, detail: Option<&str>, tile: Tile, trail: Trail) -> MenuEntry<u8> {
    MenuEntry::Item {
        value,
        title: title.to_string(),
        detail: detail.map(str::to_string),
        tile: Some(tile),
        trail,
        check: None,
        availability: Availability::Enabled,
    }
}

/// The apps whose names hold `typed` as rows, their own icons in the tiles; with something typed,
/// the first is the top hit and shows the Enter hint.
fn apps(typed: &str) -> Vec<MenuEntry<u8>> {
    APPS.iter()
        .filter(|(name, _)| name.to_lowercase().contains(typed))
        .zip(1u8..)
        .map(|(&(name, hue), value)| {
            let tile =
                app_icon(hue, IconSize::Tile48).map_or(Tile::Icon(Icon::Window), Tile::Source);
            let trail = match (typed.is_empty(), value) {
                (false, 1) => Trail::Shortcut(Shortcut(vec![Key::Enter])),
                _ => Trail::None,
            };
            row(value, name, None, tile, trail)
        })
        .collect()
}

/// What an empty query lists: system actions and recent apps.
fn actions() -> Vec<MenuEntry<u8>> {
    vec![
        row(10, "Lock", None, Tile::Icon(Icon::Lock), Trail::None),
        row(
            11,
            "Log Out",
            Some("Closes every app"),
            Tile::Icon(Icon::Power),
            Trail::None,
        ),
    ]
}

/// One embedded palette in a Sheet surface of the launcher's panel shape.
#[component]
fn Panel(
    query: String,
    groups: Vec<(String, Vec<MenuEntry<u8>>)>,
    entrance: PaletteEntrance,
) -> Element {
    rsx! {
        div { class: "g-launcher",
            Surface { material: Material::Sheet, radius: Some(Corner::Token(Radius::Panel)),
                CommandPalette::<u8> {
                    label: "Launch",
                    placeholder: "Search apps, windows, actions",
                    query,
                    tokens: Vec::new(),
                    groups,
                    empty: "Nothing matches.",
                    oninput: |_| {},
                    onpick: |_| {},
                    onclose: |_| {},
                    host: CommandPaletteHost::Surface,
                    entrance,
                    id: "gallery-launcher",
                }
            }
        }
    }
}

/// The embedded palette section.
#[component]
pub fn EmbeddedPalette() -> Element {
    rsx! {
        Section { title: "Palette in a surface", note: "CommandPaletteHost::Surface: no scrim, the card fills its container and carries the id a shell's blur region names, and paints the enclosing material. A row's tile takes an app's own icon (Tile::Source), drawn as it is, filling the tile.",
            div { class: "g-row g-row-top",
                Specimen { name: "Typed \"f\": app icons, cmdk-in",
                    Panel { query: "f", groups: vec![("Applications".to_string(), apps("f"))], entrance: PaletteEntrance::CmdkIn }
                }
                Specimen { name: "Empty query: actions and recent apps, peek-in",
                    Panel {
                        query: "",
                        groups: vec![("Actions".to_string(), actions()), ("Recent".to_string(), apps(""))],
                        entrance: PaletteEntrance::PeekIn,
                    }
                }
            }
        }
    }
}
