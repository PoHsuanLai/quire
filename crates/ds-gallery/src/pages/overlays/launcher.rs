//! The Overlays page's embedded palette: `CommandPalette` hosted in a surface of its own, as
//! sill's launcher panel draws it: no scrim, the card filling its
//! container and carrying the id a blur region names, `cmdk-in`, app icons in the rows; and a
//! warm palette kept mounted while hidden and shown by a button.

use crate::axes::{Axes, Showcase};
use crate::pages::content::app_icons::{APPS, app_icon};
use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::menus::palette::palette_group::PaletteGroup;
use ds::components::menus::palette::palette_group::PaletteRow;
use ds::prelude::*;
use ds::style::tokens::shape::{Corner, Radius};

fn row(
    value: u8,
    title: &str,
    detail: Option<&str>,
    leading: RowLeading,
    accessory: Accessory,
) -> PaletteRow<u8> {
    PaletteRow {
        detail: detail.map(|detail| detail.into()),
        leading,
        accessory,
        ..PaletteRow::new(value, title)
    }
}

/// The apps whose names hold `typed` as rows, their own icons in the tiles; with something typed,
/// the first is the top hit and shows the Enter hint.
fn apps(typed: &str) -> Vec<PaletteRow<u8>> {
    APPS.iter()
        .filter(|(name, _)| name.to_lowercase().contains(typed))
        .zip(1u8..)
        .map(|(&(name, hue), value)| {
            let leading = app_icon(hue, IconSize::Tile48)
                .map_or(RowLeading::Icon(Icon::Window), RowLeading::Source);
            let accessory = match (typed.is_empty(), value) {
                (false, 1) => Accessory::Text(Shortcut(vec![ShortcutKey::Enter]).glyphs()),
                _ => Accessory::None,
            };
            row(value, name, None, leading, accessory)
        })
        .collect()
}

/// What an empty query lists: system actions and recent apps.
fn actions() -> Vec<PaletteRow<u8>> {
    vec![
        row(
            10,
            "Lock",
            None,
            RowLeading::Icon(Icon::Lock),
            Accessory::None,
        ),
        row(
            11,
            "Log Out",
            Some("Closes every app"),
            RowLeading::Icon(Icon::Power),
            Accessory::None,
        ),
    ]
}

/// One embedded palette in a Sheet surface of the launcher's panel shape.
#[component]
fn Panel(query: String, groups: Vec<PaletteGroup<u8>>) -> Element {
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
        Section { title: "Palette in a surface", note: "CommandPaletteHost::Surface: no scrim, the card spans its container's width, is as tall as its content (up to the container), carries the id a shell's blur region names, and paints the enclosing material. A row's tile takes an app's own icon (RowLeading::Source), drawn as it is, filling the tile.",
            div { class: "g-row g-row-top",
                Specimen { name: "Typed \"f\": app icons, cmdk-in",
                    Panel { query: "f", groups: vec![PaletteGroup::list("Applications", apps("f"))] }
                }
                Specimen { name: "Empty query: actions and recent apps, peek-in",
                    Panel {
                        query: "",
                        groups: vec![PaletteGroup::list("Actions", actions()), PaletteGroup::list("Recent", apps(""))],
                    }
                }
            }
            div { class: "g-row g-row-top",
                Specimen { name: "Warm: kept mounted, shown by the button", code: "shown: Some(Shown::Visible | Shown::Hidden)".to_string(),
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
            PaletteGroup::list("Actions", actions()),
            PaletteGroup::list("Recent", apps("")),
        ]
    } else {
        vec![PaletteGroup::list("Applications", apps(&typed))]
    };
    let flipped = match shown() {
        Shown::Visible => Shown::Hidden,
        Shown::Hidden => Shown::Visible,
    };
    rsx! {
        div { class: "g-col",
            Button {
                label: "Launcher",
                value: Some(if shown() == Shown::Visible { Check::On } else { Check::Off }),
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
                        id: "gallery-warm-launcher",
                        shown: shown(),
                    }
                }
            }
        }
    }
}
