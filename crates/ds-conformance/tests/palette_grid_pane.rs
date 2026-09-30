//! The launcher v2 parts on a real Blitz document: the palette's
//! cursor entering, crossing and leaving an emoji grid; a group's "Show More" reached with Down
//! and run with Enter; a key claimed before the field types it, with the caret's place; and a
//! pane beside the results widening the card.

use dioxus::prelude::*;
use ds::components::lists::emoji_grid::grid::{EMOJI_CELL, EmojiCell, EmojiCells};
use ds::components::lists::preview::content::PaneContent;
use ds::components::menus::palette::palette_claim::{Claim, FieldKey};
use ds::components::menus::palette::palette_group::PaletteRow;
use ds::components::menus::palette::palette_group::{PaletteGroup, PaletteGroups};
use ds::host::caret::Caret;
use ds::prelude::*;
use ds_harness::{
    ClassPresence, Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport,
};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 1200,
    height: 640,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn item(value: u8, title: &str) -> PaletteRow<u8> {
    PaletteRow::new(value, title.to_string())
}

/// Ten cells, four to a row: stops 2..=11 after the two rows above.
fn grid() -> EmojiCells<u8> {
    EmojiCells {
        cells: (0..10)
            .map(|n| EmojiCell {
                value: 20 + n,
                glyph: "🙂".to_string(),
                name: format!("cell {n}"),
            })
            .collect(),
        columns: 4,
        cell: EMOJI_CELL,
    }
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

#[allow(non_snake_case)]
fn GridPalette() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut note = move |line: String| log.with_mut(|log| log.push(line));
    let groups = vec![
        PaletteGroup::list("Top", vec![item(1, "One"), item(2, "Two")]),
        PaletteGroup::grid("Emoji", grid()),
        PaletteGroup::list("After", vec![item(50, "Fifty")]),
    ];
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "width:600px; height:600px",
                CommandPalette::<u8> {
                    label: "Launch",
                    placeholder: "Search",
                    query: String::new(),
                    tokens: Vec::new(),
                    groups: PaletteGroups(groups),
                    empty: "Nothing",
                    oninput: move |_| {},
                    onpick: move |value: u8| note(format!("pick:{value}")),
                    onclose: move |()| note("close".to_string()),
                    host: CommandPaletteHost::Surface,
                    id: "card".to_string(),
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

/// The selected stop, as the markup shows it: a row's title or a cell's name.
fn selected(harness: &Harness) -> String {
    harness
        .attr(".ds-emoji-cell[*|aria-selected=true]", "aria-label")
        .or_else(|| harness.text_of(".ds-row[*|aria-selected=true] .ds-row-title"))
        .unwrap_or_default()
}

/// Down enters the grid at its first cell; Left and Right walk cells in reading order,
/// wrapping rows; Down moves a row in the same column, onto the short last row's last cell, then
/// out to the next group; Up comes back in; Enter on a cell picks it and closes.
#[test]
fn the_cursor_moves_through_the_grid_in_two_dimensions_and_leaves_it() {
    let mut harness = Harness::new(GridPalette, VIEW);
    harness.advance(ms(200));
    assert_eq!(
        harness.focus_of("#card .ds-input"),
        FocusState::Focused,
        "the field has the keyboard"
    );
    assert_eq!(selected(&harness), "One");
    // (key, the stop it lands on)
    let walk: &[(ShortcutKey, &str)] = &[
        (ShortcutKey::Down, "Two"),
        (ShortcutKey::Down, "cell 0"),
        (ShortcutKey::Right, "cell 1"),
        (ShortcutKey::Right, "cell 2"),
        (ShortcutKey::Right, "cell 3"),
        (ShortcutKey::Right, "cell 4"),
        (ShortcutKey::Left, "cell 3"),
        (ShortcutKey::Down, "cell 7"),
        (ShortcutKey::Down, "cell 9"),
        (ShortcutKey::Down, "Fifty"),
        (ShortcutKey::Up, "cell 9"),
        (ShortcutKey::Up, "cell 5"),
        (ShortcutKey::Up, "cell 1"),
        (ShortcutKey::Up, "Two"),
        (ShortcutKey::Down, "cell 0"),
    ];
    for (step, (key, want)) in walk.iter().enumerate() {
        harness.send(Input::key(*key));
        harness.advance(ms(20));
        assert_eq!(selected(&harness), *want, "step {step}: {key:?}");
    }
    assert_eq!(harness.count(".ds-emoji-cell[*|aria-selected=true]"), 1);
    assert_eq!(
        harness.has_class("#card .ds-emoji-grid", "ds-emoji-text"),
        ClassPresence::Present,
        "the grid paints in the colour face"
    );
    harness.send(Input::key(ShortcutKey::Enter));
    harness.advance(ms(20));
    assert_eq!(log(&harness), "close,pick:20");
}

#[allow(non_snake_case)]
fn MorePalette() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut note = move |line: String| log.with_mut(|log| log.push(line));
    let mut more = use_signal(|| false);
    let apps: Vec<PaletteRow<u8>> = match more() {
        false => vec![item(1, "Files"), item(2, "Firefox")],
        true => vec![item(1, "Files"), item(2, "Firefox"), item(3, "Terminal")],
    };
    let label = if more() { "Show Less" } else { "Show More" };
    let groups = vec![
        PaletteGroup::list("Applications", apps).with_action(
            label,
            EventHandler::new(move |()| {
                note("more".to_string());
                more.toggle();
            }),
        ),
        PaletteGroup::list("Settings", vec![item(9, "Displays")]),
    ];
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "width:600px; height:600px",
                CommandPalette::<u8> {
                    label: "Launch",
                    placeholder: "Search",
                    query: String::new(),
                    tokens: Vec::new(),
                    groups: PaletteGroups(groups),
                    empty: "Nothing",
                    oninput: move |_| {},
                    onpick: move |value: u8| note(format!("pick:{value}")),
                    onclose: move |()| note("close".to_string()),
                    host: CommandPaletteHost::Surface,
                    id: "card".to_string(),
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

/// Down past a group's last row rests on its header's "Show More" (drawn selected); Enter
/// runs it and the palette stays open, the group now longer; Down goes on to the next group.
#[test]
fn show_more_is_a_stop_after_the_groups_last_row_and_enter_runs_it() {
    let mut harness = Harness::new(MorePalette, VIEW);
    harness.advance(ms(200));
    assert_eq!(
        harness.count(".ds-section-header-action[*|data-selected=true]"),
        0
    );
    harness.send(Input::key(ShortcutKey::Down));
    harness.send(Input::key(ShortcutKey::Down));
    harness.advance(ms(20));
    assert_eq!(harness.count(".ds-row[*|aria-selected=true]"), 0);
    assert_eq!(
        harness
            .text_of(".ds-section-header-action[*|data-selected=true]")
            .as_deref(),
        Some("Show More"),
        "the cursor rests on the header's action"
    );
    let rows_before = harness.count("#card .ds-row");
    harness.send(Input::key(ShortcutKey::Enter));
    harness.advance(ms(20));
    assert_eq!(log(&harness), "more", "the action ran and nothing closed");
    assert_eq!(harness.count("#card .ds-row"), rows_before + 1);
    assert_eq!(
        harness.text_of(".ds-section-header-action").as_deref(),
        Some("Show Less")
    );
    // The action is now after the third row: stop 3; the cursor, still at stop 2, is on Terminal.
    assert_eq!(
        harness
            .text_of(".ds-row[*|aria-selected=true] .ds-row-title")
            .as_deref(),
        Some("Terminal")
    );
    harness.send(Input::key(ShortcutKey::Down));
    harness.send(Input::key(ShortcutKey::Down));
    harness.advance(ms(20));
    assert_eq!(
        harness
            .text_of(".ds-row[*|aria-selected=true] .ds-row-title")
            .as_deref(),
        Some("Displays")
    );
}

/// The words a claim's log writes for a caret.
fn caret_word(caret: Caret) -> &'static str {
    match caret {
        Caret::AtEnd => "end",
        Caret::Inside => "inside",
        Caret::Unknown => "unknown",
    }
}

#[allow(non_snake_case)]
fn ClaimPalette() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut query = use_signal(String::new);
    // Browsing: the selection last moved by Up or Down and nothing typed since.
    let mut browsing = use_signal(|| false);
    let claim = move |key: FieldKey| -> Claim {
        let name = key.event.key();
        let taken = match name {
            dioxus::prelude::Key::ArrowDown | dioxus::prelude::Key::ArrowUp => {
                browsing.set(true);
                None
            }
            dioxus::prelude::Key::Character(ref text) if text == " " && browsing() => Some("pane"),
            dioxus::prelude::Key::Character(_) => {
                browsing.set(false);
                None
            }
            dioxus::prelude::Key::ArrowRight if key.caret == Caret::AtEnd => Some("show"),
            _ => None,
        };
        log.with_mut(|log| {
            log.push(format!("{name}@{}", caret_word(key.caret)));
            if let Some(what) = taken {
                log.push(what.to_string());
            }
        });
        match taken {
            Some(_) => Claim::Take,
            None => Claim::Pass,
        }
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "width:600px; height:600px",
                CommandPalette::<u8> {
                    label: "Launch",
                    placeholder: "Search",
                    query: query(),
                    tokens: Vec::new(),
                    groups: vec![ds::components::menus::palette::palette_group::PaletteGroup::list("Applications", vec![item(1, "Files"), item(2, "Firefox")])],
                    empty: "Nothing",
                    oninput: move |text: String| query.set(text),
                    onpick: move |_| {},
                    onclose: move |()| {},
                    host: CommandPaletteHost::Surface,
                    id: "card".to_string(),
                    claim,
                }
            }
            p { class: "query", "[{query}]" }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn query(harness: &Harness) -> String {
    harness.text_of(".query").unwrap_or_default()
}

/// A claim hears each key first with the caret's place. Space while typing is passed and
/// the field types it; after Down (browsing) Space is taken and the field types nothing. Right
/// with the caret inside the text is passed (the caret moves); with the caret at the end it is
/// taken.
#[test]
fn a_claim_takes_space_only_while_browsing_and_right_only_at_the_end() {
    let mut harness = Harness::new(ClaimPalette, VIEW);
    harness.advance(ms(200));
    for key in [
        ShortcutKey::Char('a'),
        ShortcutKey::Char('b'),
        ShortcutKey::Space,
    ] {
        harness.send(Input::key(key));
        harness.advance(ms(20));
    }
    assert_eq!(query(&harness), "[ab ]", "typing, Space is typed");
    harness.send(Input::key(ShortcutKey::Down));
    harness.advance(ms(20));
    harness.send(Input::key(ShortcutKey::Space));
    harness.advance(ms(20));
    assert_eq!(query(&harness), "[ab ]", "browsing, Space is the caller's");
    assert!(log(&harness).ends_with(" @end,pane"), "{}", log(&harness));
    harness.send(Input::key(ShortcutKey::Left));
    harness.advance(ms(20));
    harness.send(Input::key(ShortcutKey::Right));
    harness.advance(ms(20));
    assert!(
        log(&harness).ends_with("ArrowLeft@end,ArrowRight@inside"),
        "Right inside the text is the field's: {}",
        log(&harness)
    );
    harness.send(Input::key(ShortcutKey::Right));
    harness.advance(ms(20));
    assert!(
        log(&harness).ends_with("ArrowRight@end,show"),
        "Right at the end is taken: {}",
        log(&harness)
    );
    assert_eq!(query(&harness), "[ab ]", "no claimed key changed the text");
}

#[component]
fn AsidePalette(host: CommandPaletteHost) -> Element {
    let mut pane = use_signal(|| false);
    // Tab toggles the pane: over a window the palette's scrim takes every click outside it.
    let claim = move |key: FieldKey| -> Claim {
        match key.event.key() {
            dioxus::prelude::Key::Tab => {
                pane.toggle();
                Claim::Take
            }
            _ => Claim::Pass,
        }
    };
    let aside = pane().then(|| {
        rsx! {
            PreviewPane {
                content: PaneContent::Emoji { glyph: "🎉".to_string(), name: "party popper".to_string() },
            }
        }
    });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { class: "room", style: "width:900px; height:560px",
                CommandPalette::<u8> {
                    label: "Launch",
                    placeholder: "Search",
                    query: String::new(),
                    tokens: Vec::new(),
                    groups: vec![ds::components::menus::palette::palette_group::PaletteGroup::list("Applications", vec![item(1, "Files")])],
                    empty: "Nothing",
                    oninput: move |_| {},
                    onpick: move |_| {},
                    onclose: move |()| {},
                    host,
                    id: "card".to_string(),
                    aside,
                    claim,
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn OverlayAside() -> Element {
    rsx! { AsidePalette { host: CommandPaletteHost::Overlay } }
}

#[allow(non_snake_case)]
fn SurfaceAside() -> Element {
    rsx! { AsidePalette { host: CommandPaletteHost::Surface } }
}

fn width(harness: &Harness, selector: &str) -> f32 {
    harness
        .rect(selector)
        .unwrap_or_else(|| panic!("{selector} is drawn"))
        .size
        .width
        .0
}

fn toggle(harness: &mut Harness) {
    harness.send(Input::key(ShortcutKey::Tab));
    harness.advance(ms(60));
}

/// Over a window the card widens by exactly the pane's width, the results keeping theirs,
/// and the pane sits right of them under the field; without it the card is as it was.
#[test]
fn a_pane_beside_the_results_widens_the_card_by_its_width() {
    let mut harness = Harness::new(OverlayAside, VIEW);
    harness.advance(ms(200));
    let card = width(&harness, "#card");
    let list = width(&harness, "#card > .ds-palette-list");
    toggle(&mut harness);
    assert_eq!(harness.count("#card .ds-preview"), 1);
    assert!(
        (width(&harness, "#card") - (card + 360.0)).abs() < 1.0,
        "{card}"
    );
    assert!((width(&harness, "#card .ds-palette-aside") - 360.0).abs() < 1.0);
    assert!((width(&harness, "#card > .ds-palette-list") - list).abs() < 1.0);
    let field = harness.rect("#card .ds-text-field").expect("field");
    let pane = harness.rect("#card .ds-palette-aside").expect("pane");
    let results = harness.rect("#card > .ds-palette-list").expect("results");
    assert!(
        pane.origin.y.0 >= field.origin.y.0 + field.size.height.0 - 1.0,
        "under the field"
    );
    assert!(
        pane.origin.x.0 >= results.origin.x.0 + results.size.width.0 - 1.0,
        "right of the results"
    );
    toggle(&mut harness);
    assert!(
        (width(&harness, "#card") - card).abs() < 1.0,
        "and narrow again"
    );
}

/// In a surface the card still fills its container, and the results give the pane its
/// width.
#[test]
fn in_a_surface_the_results_narrow_by_the_pane() {
    let mut harness = Harness::new(SurfaceAside, VIEW);
    harness.advance(ms(200));
    let card = width(&harness, "#card");
    toggle(&mut harness);
    assert!((width(&harness, "#card") - card).abs() < 1.0);
    let list = width(&harness, "#card > .ds-palette-list");
    let pane = width(&harness, "#card .ds-palette-aside");
    assert!((pane - 360.0).abs() < 1.0);
    // The card's hairline border is inside its width, one on each side.
    assert!(
        (list + pane + 2.0 - card).abs() < 1.0,
        "{list} + {pane} vs {card}"
    );
}

#[allow(non_snake_case)]
fn LoneGrid() -> Element {
    let mut at = use_signal(|| None::<usize>);
    let mut picked = use_signal(|| None::<u8>);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            EmojiGrid::<u8> {
                cells: grid().cells,
                columns: 4,
                selected: at(),
                on_select: move |index: usize| at.set(Some(index)),
                onpick: move |value: u8| picked.set(Some(value)),
            }
            p { class: "picked", "{picked():?}" }
        }
    }
}

/// A grid on its own takes the keyboard when clicked into; the arrows move in two
/// dimensions and stop at its edges (nowhere to leave to), and Enter picks.
#[test]
fn a_grid_on_its_own_moves_with_the_arrows_and_stops_at_its_edges() {
    let mut harness = Harness::new(LoneGrid, VIEW);
    harness.advance(ms(100));
    let first = harness.centre(".ds-emoji-cell").expect("a cell");
    harness.send(Input::click(first));
    ds_harness::harness::settle_until(&mut harness, |harness| {
        harness.focus_of(".ds-emoji-grid") == FocusState::Focused
    });
    assert_eq!(
        harness.focus_of(".ds-emoji-grid"),
        FocusState::Focused,
        "the grid has the keyboard"
    );
    let at = |harness: &Harness| {
        harness
            .attr(".ds-emoji-cell[*|aria-selected=true]", "aria-label")
            .unwrap_or_default()
    };
    let walk: &[(ShortcutKey, &str)] = &[
        (ShortcutKey::Up, "cell 0"),
        (ShortcutKey::Right, "cell 1"),
        (ShortcutKey::Down, "cell 5"),
        (ShortcutKey::Down, "cell 9"),
        (ShortcutKey::Down, "cell 9"),
        (ShortcutKey::Left, "cell 8"),
    ];
    for (step, (key, want)) in walk.iter().enumerate() {
        harness.send(Input::key(*key));
        harness.advance(ms(20));
        assert_eq!(at(&harness), *want, "step {step}: {key:?}");
    }
    harness.send(Input::key(ShortcutKey::Enter));
    harness.advance(ms(20));
    assert_eq!(harness.text_of(".picked").as_deref(), Some("Some(28)"));
}

#[allow(non_snake_case)]
fn LeavingPane() -> Element {
    let mut shown = use_signal(|| Shown::Visible);
    let mut gone = use_signal(|| false);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            button { class: "hide", onclick: move |_| shown.set(Shown::Hidden), "Hide" }
            if !gone() {
                PreviewPane {
                    content: PaneContent::Web { host: "duckduckgo.com".to_string(), url: "https://duckduckgo.com".to_string() },
                    shown: shown(),
                    on_hidden: move |()| gone.set(true),
                }
            }
        }
    }
}

/// A pane turned hidden plays its exit (`pane-out-r`) and calls `on_hidden` only once
/// that has settled (`settle(PaneOutR)`, 284 ms at Standard), so the caller drops it then.
#[test]
fn a_hidden_pane_leaves_and_says_so_when_its_exit_settles() {
    let mut harness = Harness::new(
        LeavingPane,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    harness.advance(ms(400));
    assert_eq!(
        harness.attr(".ds-preview", "data-presence").as_deref(),
        Some("present")
    );
    let at = harness.centre(".hide").expect("the button");
    harness.send(Input::click(at));
    let asked = harness.now();
    harness.advance(ms(20));
    assert_eq!(
        harness.attr(".ds-preview", "data-presence").as_deref(),
        Some("leaving")
    );
    let dropped = ds_harness::harness::settle_until(&mut harness, |harness| {
        harness.count(".ds-preview") == 0
    });
    assert_eq!(harness.count(".ds-preview"), 0, "dropped at on_hidden");
    assert!(
        dropped.duration_since(asked) >= ms(250),
        "not before the exit settled"
    );
}
