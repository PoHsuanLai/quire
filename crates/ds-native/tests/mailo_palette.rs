//! mailo gaps 2, the command panel, on a real Blitz document: `PaletteEntrance::Opaque` paints
//! the card on its first frame where `cmdk-in` paints nothing yet, and a row's trailing action
//! fires without running, closing or selecting its row.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::{
    Appearance, CommandPalette, Ds, Grain, Icon, Material, MenuEntry, MenuRow, PaletteEntrance,
    Rect, RowAction, SpaceLook,
};
use ds_native::{Harness, Viewport};
use image::RgbaImage;
use probe::rect;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 480,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A flat ground, so a pixel differs only where something is painted over it.
fn flat() -> SpaceLook {
    SpaceLook {
        grain: Grain(0),
        ..SpaceLook::default()
    }
}

/// Three recent searches, each with a remove that logs its row.
fn rows(mut log: Signal<Vec<String>>) -> Vec<(String, Vec<MenuEntry<u8>>)> {
    let entries = (1..=3u8)
        .map(|value| {
            MenuEntry::Row(MenuRow {
                trailing: Some(RowAction {
                    icon: Icon::X,
                    label: "Remove from recent".to_string(),
                    on_press: EventHandler::new(move |_| {
                        log.with_mut(|log| log.push(format!("remove:{value}")))
                    }),
                }),
                ..MenuRow::new(value, format!("recent search {value}"))
            })
        })
        .collect();
    vec![("Recent".to_string(), entries)]
}

/// Which page: the palette with an entrance, or the page alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shows {
    Palette(PaletteEntrance),
    Nothing,
}

#[component]
fn Page(shows: Shows) -> Element {
    let mut log = use_signal(Vec::<String>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, look: flat(),
            // The root fills the view: the palette's layer is laid out inside it.
            p { class: "log", style: "height:440px; margin:0", {log().join(",")} }
            if let Shows::Palette(entrance) = shows {
                CommandPalette::<u8> {
                    label: "Search and commands".to_string(),
                    placeholder: "Search".to_string(),
                    query: String::new(),
                    tokens: Vec::new(),
                    groups: rows(log),
                    empty: "Nothing".to_string(),
                    oninput: move |_| {},
                    onpick: move |value: u8| log.with_mut(|log| log.push(format!("pick:{value}"))),
                    onclose: move |()| log.with_mut(|log| log.push("close".to_string())),
                    entrance,
                    on_select: move |index: usize| log.with_mut(|log| log.push(format!("select:{index}"))),
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn Opaque() -> Element {
    rsx! { Page { shows: Shows::Palette(PaletteEntrance::Opaque) } }
}

#[allow(non_snake_case)]
fn Cmdk() -> Element {
    rsx! { Page { shows: Shows::Palette(PaletteEntrance::CmdkIn) } }
}

#[allow(non_snake_case)]
fn Bare() -> Element {
    rsx! { Page { shows: Shows::Nothing } }
}

/// How many pixels inside `area` differ between two frames.
fn differing(a: &RgbaImage, b: &RgbaImage, area: Rect) -> usize {
    probe::pixels(a, area, 0.0)
        .into_iter()
        .zip(probe::pixels(b, area, 0.0))
        .filter(|(a, b)| a != b)
        .count()
}

/// The first frame of `app`, and where its card is laid out.
fn first_frame(app: fn() -> Element) -> (RgbaImage, Rect) {
    let mut harness = Harness::new(app, VIEW);
    let frame = harness.render().expect("a frame");
    (frame, rect(&harness, ".ds-palette"))
}

#[test]
fn an_opaque_entrance_paints_the_card_on_its_first_frame() {
    let ground = Harness::new(Bare, VIEW).render().expect("a frame");
    let (cmdk, card) = first_frame(Cmdk);
    let (opaque, same) = first_frame(Opaque);
    assert_eq!(card, same, "both cards are laid out alike");
    // The middle half of the card: inside it whatever the entrance's scale.
    let middle = Rect {
        origin: ds::Point {
            x: ds::Px(card.origin.x.0 + card.size.width.0 / 4.0),
            y: ds::Px(card.origin.y.0 + card.size.height.0 / 4.0),
        },
        size: ds::Size {
            width: ds::Px(card.size.width.0 / 2.0),
            height: ds::Px(card.size.height.0 / 2.0),
        },
    };
    let area = middle.size.width.0 * middle.size.height.0;
    probe::keep(&cmdk, "palette-cmdk-first");
    probe::keep(&opaque, "palette-opaque-first");
    assert!(
        (differing(&ground, &cmdk, middle) as f32) < area * 0.01,
        "cmdk-in's first frame paints nothing over the page yet"
    );
    assert!(
        differing(&ground, &opaque, middle) as f32 > area * 0.5,
        "the opaque entrance's first frame paints the card"
    );
}

#[test]
fn a_trailing_action_fires_without_picking_or_selecting_its_row() {
    let mut harness = Harness::new(Opaque, VIEW);
    harness.advance(ms(600));
    assert_eq!(harness.text_of(".log").as_deref(), Some("select:0"));
    // The second row's remove: the pointer crosses the row to reach it.
    let remove = ".ds-menu-item:nth-child(3) .ds-menu-action .ds-icon-button";
    let at = harness.centre(remove).expect("the second row's remove");
    harness.click(at);
    harness.advance(ms(100));
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some("select:0,remove:2"),
        "the remove fired; nothing was picked, closed or selected"
    );
    assert_eq!(harness.count(".ds-palette"), 1, "the palette is still open");
}
