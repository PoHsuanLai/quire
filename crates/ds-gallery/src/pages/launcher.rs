//! The Overlays page's embedded palette: `CommandPalette` hosted in a surface of its own, as
//! sill's launcher panel draws it (sill FINDINGS Q40-Q42): no scrim, the card filling its
//! container and carrying the id a blur region names, `cmdk-in`, app icons in the rows; and a
//! warm palette kept mounted while hidden and shown by a button (sill FINDINGS Q63).

use super::app_icons::{APPS, app_icon};
use super::{Section, Specimen};
use crate::axes::{Axes, Showcase};
use dioxus::prelude::*;
use ds::{
    Availability, Button, ButtonVariant, CommandPalette, CommandPaletteHost, Corner, Icon,
    IconSize, Material, MenuEntry, PaletteEntrance, Radius, Retain, Shortcut, Shown, Surface,
    Switch, Tile, Trail, components::vocab::Key,
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
        Section { title: "Palette in a surface", note: "CommandPaletteHost::Surface: no scrim, the card spans its container's width, is as tall as its content (up to the container), carries the id a shell's blur region names, and paints the enclosing material. A row's tile takes an app's own icon (Tile::Source), drawn as it is, filling the tile.",
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
            div { class: "g-row g-row-top",
                Specimen { name: "Warm: kept mounted, shown by the button", code: "shown: Some(Shown::Visible | Shown::Hidden), retain: Retain::Nothing".to_string(),
                    WarmPalette {}
                }
            }
        }
    }
}

/// A launcher's warm palette: mounted once, hidden and shown by the button, replaying `cmdk-in`
/// and starting from an empty query at each showing. Posed, it is shown.
#[component]
fn WarmPalette() -> Element {
    let showcase = use_context::<Signal<Axes>>().peek().showcase;
    let mut shown = use_signal(|| match showcase {
        Showcase::Posed => Shown::Visible,
        Showcase::Live => Shown::Hidden,
    });
    let mut query = use_signal(String::new);
    let typed = query().to_lowercase();
    let groups = if typed.is_empty() {
        vec![
            ("Actions".to_string(), actions()),
            ("Recent".to_string(), apps("")),
        ]
    } else {
        vec![("Applications".to_string(), apps(&typed))]
    };
    let flipped = match shown() {
        Shown::Visible => Shown::Hidden,
        Shown::Hidden => Shown::Visible,
    };
    rsx! {
        div { class: "g-col",
            Button {
                variant: ButtonVariant::Secondary,
                label: "Launcher",
                pressed: Some(if shown() == Shown::Visible { Switch::On } else { Switch::Off }),
                onclick: move |_| shown.set(flipped),
            }
            div { class: "g-launcher",
                Surface { material: Material::Sheet, radius: Some(Corner::Token(Radius::Panel)),
                    CommandPalette::<u8> {
                        label: "Launch",
                        placeholder: "Search apps, windows, actions",
                        query: query(),
                        tokens: Vec::new(),
                        groups,
                        empty: "Nothing matches.",
                        oninput: move |text: String| query.set(text),
                        onpick: move |_| shown.set(Shown::Hidden),
                        onclose: move |_| shown.set(Shown::Hidden),
                        host: CommandPaletteHost::Surface,
                        entrance: PaletteEntrance::CmdkIn,
                        id: "gallery-warm-launcher",
                        shown: shown(),
                        retain: Retain::Nothing,
                    }
                }
            }
        }
    }
}
