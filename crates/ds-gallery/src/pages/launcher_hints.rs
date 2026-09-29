//! The Overlays page's Spotlight hints (launcher chords): file rows whose action chord shows on
//! the selected row only (`RowChord::on_selected`), beside a preview pane whose actions end in
//! a plain chord (`Chord`) rather than key caps.

use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::{
    CommandPalette, CommandPaletteHost, Corner, Icon, ImageSize, ImageSource, Material, MenuEntry,
    MenuRow, MenuTile, PaletteGroup, PaletteGroups, PaneAction, PaneContent, PreviewPane, Radius,
    RowChord, RowShape, Shortcut, ShortcutKey, Surface,
};

fn reveal() -> Shortcut {
    Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('r')])
}

fn file(value: u8, title: &str, modified: &str) -> MenuEntry<u8> {
    MenuEntry::Row(MenuRow {
        tile: Some(MenuTile::Icon(Icon::File)),
        shape: RowShape::File {
            thumb: None,
            location: "~/Documents".to_owned(),
            modified: modified.to_owned(),
        },
        chord: RowChord::on_selected(reveal()),
        ..MenuRow::new(value, title)
    })
}

fn groups() -> PaletteGroups<u8> {
    PaletteGroups(vec![PaletteGroup::list(
        "Documents",
        vec![
            file(1, "Invoice.pdf", "13:00"),
            file(2, "Receipt.pdf", "Yesterday"),
            file(3, "Lease.pdf", "Monday"),
        ],
    )])
}

fn actions() -> Vec<PaneAction> {
    vec![
        PaneAction {
            label: "Reveal in Files".to_owned(),
            shortcut: reveal(),
        },
        PaneAction {
            label: "Copy Path".to_owned(),
            shortcut: Shortcut(vec![
                ShortcutKey::Super,
                ShortcutKey::Shift,
                ShortcutKey::Char('c'),
            ]),
        },
    ]
}

/// A page-sized grey picture standing in for a PDF's first page.
fn page() -> ImageSource {
    ImageSource(
        "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='612' height='792'><rect width='612' height='792' fill='%23f4f4f4'/><rect x='60' y='80' width='400' height='24' fill='%23c8c8c8'/><rect x='60' y='130' width='492' height='12' fill='%23dcdcdc'/><rect x='60' y='156' width='492' height='12' fill='%23dcdcdc'/></svg>"
            .to_owned(),
    )
}

/// The Spotlight hints section.
#[component]
pub fn SpotlightHints() -> Element {
    let aside = rsx! {
        PreviewPane {
            content: PaneContent::Pdf {
                page: ds::PdfPage::Ready {
                    image: page(),
                    sheet: ImageSize { width: 612, height: 792 },
                },
                name: "Invoice.pdf".to_owned(),
            },
            actions: actions(),
        }
    };
    rsx! {
        Section { title: "Spotlight hints", note: "A row's action chord (MenuRow.chord, RowChord::on_selected) shows only on the selected row, after its time, as plain text in --ink-soft; the other rows show their time alone. The preview pane's actions end in a plain Chord, not key caps.",
            div { class: "g-row g-row-top",
                Specimen { name: "Selected row shows ⌘R; pane actions as plain chords", code: "chord: RowChord::on_selected(⌘R); PaneAction { label, shortcut }".to_string(),
                    div { class: "g-launcher-wide",
                        Surface { material: Material::Sheet, radius: Some(Corner::Token(Radius::Panel)),
                            CommandPalette::<u8> {
                                label: "Launch",
                                placeholder: "Search",
                                query: "".to_string(),
                                tokens: Vec::new(),
                                groups: groups(),
                                empty: "Nothing matches.",
                                oninput: |_| {},
                                onpick: |_| {},
                                onclose: |_| {},
                                host: CommandPaletteHost::Surface,
                                id: "gallery-hints",
                                aside,
                            }
                        }
                    }
                }
            }
        }
    }
}
