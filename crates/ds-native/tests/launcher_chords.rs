//! Spotlight's hints on a real Blitz document (launcher chords): a row's action chord shows on
//! the selected row only and moves with the selection under the palette's own keys, and a
//! preview pane's action ends in a plain chord at its label's size, not in key caps.

use dioxus::prelude::*;
use ds::{
    Appearance, CommandPalette, CommandPaletteHost, Ds, Icon, Material, MenuEntry, MenuRow,
    MenuTile, PaletteGroup, PaletteGroups, PaneAction, PaneContent, PreviewPane, RowChord,
    RowShape, Shortcut, ShortcutKey,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 1000,
    height: 700,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn reveal() -> Shortcut {
    Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('r')])
}

fn file(value: u8, title: &str) -> MenuEntry<u8> {
    MenuEntry::Row(MenuRow {
        tile: Some(MenuTile::Icon(Icon::File)),
        shape: RowShape::File {
            thumb: None,
            location: "~/Documents".to_string(),
            modified: "13:00".to_string(),
        },
        chord: RowChord::on_selected(reveal()),
        ..MenuRow::new(value, title)
    })
}

#[allow(non_snake_case)]
fn Files() -> Element {
    let rows = vec![file(1, "One.pdf"), file(2, "Two.pdf"), file(3, "Three.pdf")];
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "width:600px; height:500px",
                CommandPalette::<u8> {
                    label: "Launch",
                    placeholder: "Search",
                    query: String::new(),
                    tokens: Vec::new(),
                    groups: PaletteGroups(vec![PaletteGroup::list("Documents", rows)]),
                    empty: "Nothing",
                    oninput: move |_| {},
                    onpick: move |_| {},
                    onclose: move |()| {},
                    host: CommandPaletteHost::Surface,
                    id: "card".to_string(),
                }
            }
        }
    }
}

const SELECTED: &str = "#card .ds-menu-item[*|aria-selected=true]";

/// The selected row's title, and the title of the one row that draws a chord.
fn where_the_chord_is(harness: &Harness) -> (Option<String>, Option<String>) {
    let selected = harness.text_of(&format!("{SELECTED} .ds-menu-title"));
    let chorded = ["One.pdf", "Two.pdf", "Three.pdf"]
        .into_iter()
        .enumerate()
        .find(|(at, _)| {
            let row = format!("#card .ds-menu-item:nth-of-type({})", at + 2);
            harness.count(&format!("{row} .ds-key-equivalent")) == 1
        })
        .map(|(_, title)| title.to_string());
    (selected, chorded)
}

#[test]
fn moving_the_selection_moves_the_chord() {
    let mut harness = Harness::new(Files, VIEW);
    harness.advance(ms(200));
    assert_eq!(
        harness.count("#card .ds-key-equivalent"),
        1,
        "one chord at rest"
    );
    assert_eq!(
        where_the_chord_is(&harness),
        (Some("One.pdf".into()), Some("One.pdf".into()))
    );
    assert_eq!(
        harness
            .text_of(&format!("{SELECTED} .ds-key-equivalent"))
            .as_deref(),
        Some("⌘R")
    );
    assert_eq!(
        harness.count("#card .ds-menu-when"),
        3,
        "every row keeps its time"
    );
    harness.key(ShortcutKey::Down);
    harness.advance(ms(100));
    assert_eq!(
        harness.count("#card .ds-key-equivalent"),
        1,
        "still one chord"
    );
    assert_eq!(
        where_the_chord_is(&harness),
        (Some("Two.pdf".into()), Some("Two.pdf".into()))
    );
    harness.key(ShortcutKey::Down);
    harness.advance(ms(100));
    assert_eq!(
        where_the_chord_is(&harness),
        (Some("Three.pdf".into()), Some("Three.pdf".into()))
    );
    let when = harness
        .rect(&format!("{SELECTED} .ds-menu-when"))
        .expect("the time is drawn");
    let chord = harness
        .rect(&format!("{SELECTED} .ds-key-equivalent"))
        .expect("the chord is drawn");
    let gap = chord.origin.x.0 - (when.origin.x.0 + when.size.width.0);
    assert!(
        (gap - 6.0).abs() < 0.75,
        "gap {gap}: {when:?} then {chord:?}"
    );
}

#[allow(non_snake_case)]
fn Pane() -> Element {
    let actions = vec![PaneAction {
        label: "Reveal in Files".to_string(),
        shortcut: reveal(),
    }];
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "width:360px; height:420px",
                PreviewPane {
                    content: PaneContent::Emoji { glyph: "🎉".to_string(), name: "party popper".to_string() },
                    actions,
                }
            }
        }
    }
}

/// The chord shares the label's line: the same line box, so the same size, and no cap boxes.
#[test]
fn a_pane_action_draws_its_chord_as_plain_text_beside_its_label() {
    let mut harness = Harness::new(Pane, VIEW);
    harness.advance(ms(400));
    assert_eq!(
        harness.count(".ds-preview-action .ds-key-equivalent-key"),
        0
    );
    assert_eq!(
        harness
            .text_of(".ds-preview-action .ds-key-equivalent")
            .as_deref(),
        Some("⌘R")
    );
    let label = harness
        .rect(".ds-preview-action-label")
        .expect("the label is drawn");
    let chord = harness
        .rect(".ds-preview-action .ds-key-equivalent")
        .expect("the chord is drawn");
    assert!(
        (label.size.height.0 - chord.size.height.0).abs() < 0.5,
        "one line height: {label:?} and {chord:?}"
    );
    assert!(
        chord.origin.x.0 > label.origin.x.0 + label.size.width.0,
        "after the label"
    );
}
