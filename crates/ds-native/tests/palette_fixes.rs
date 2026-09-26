//! Three launcher fixes on a real Blitz document: the palette keeps its selected stop in view
//! whoever moved it (sill Q340), puts the caret after an initial query (Q341), and keeps a gap
//! between a shaped row's time and its shortcut (Q343).

use dioxus::prelude::*;
use ds::{
    Appearance, Availability, Caret, Claim, CommandPalette, CommandPaletteHost, Ds, FieldKey, Icon,
    InitialCaret, Key, Material, MenuEntry, MenuRow, PaletteGroup, PaletteGroups, Rect, RowShape,
    Shortcut, Tile, Trail,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 1000,
    height: 900,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn item(value: u8, title: &str) -> MenuEntry<u8> {
    MenuEntry::Item {
        value,
        title: title.to_string(),
        detail: None,
        tile: None,
        trail: Trail::None,
        check: None,
        availability: Availability::Enabled,
    }
}

/// Twenty rows in one group with a "Show More" after them (stop 20): far more than the list's
/// 360 px shows.
fn long_groups() -> PaletteGroups<u8> {
    let rows = (0..20).map(|n| item(n, &format!("Row {n}"))).collect();
    PaletteGroups(vec![
        PaletteGroup::list("Many", rows).with_action("Show More", EventHandler::new(|()| {})),
    ])
}

/// The stop a key moves the caller's selection to: `j` far down, `m` onto the action, `k` back
/// to the top, `n` one row below the action's group start (for a move inside the view).
fn jump(key: &dioxus::prelude::Key) -> Option<usize> {
    match key {
        dioxus::prelude::Key::Character(text) => match text.as_str() {
            "j" => Some(15),
            "m" => Some(20),
            "k" => Some(0),
            "n" => Some(1),
            _ => None,
        },
        _ => None,
    }
}

#[allow(non_snake_case)]
fn LongPalette() -> Element {
    let mut selected = use_signal(|| 0usize);
    let claim = move |key: FieldKey| -> Claim {
        match jump(&key.event.key()) {
            Some(stop) => {
                selected.set(stop);
                Claim::Take
            }
            None => Claim::Pass,
        }
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "width:600px; height:800px",
                CommandPalette::<u8> {
                    label: "Launch",
                    placeholder: "Search",
                    query: String::new(),
                    tokens: Vec::new(),
                    groups: long_groups(),
                    empty: "Nothing",
                    oninput: move |_| {},
                    onpick: move |_| {},
                    onclose: move |()| {},
                    host: CommandPaletteHost::Overlay,
                    id: "card".to_string(),
                    selected: selected(),
                    on_select: move |index: usize| selected.set(index),
                    claim,
                }
            }
        }
    }
}

/// The selected stop's rect: a row, or the header's action.
fn selected_rect(harness: &Harness) -> Rect {
    harness
        .rect("#card .ds-menu-item[*|aria-selected=true]")
        .or_else(|| harness.rect("#card .ds-section-header-action[*|data-selected=true]"))
        .expect("a stop is selected")
}

fn top(rect: Rect) -> f32 {
    rect.origin.y.0
}

fn bottom(rect: Rect) -> f32 {
    rect.origin.y.0 + rect.size.height.0
}

/// Whether `inner` lies inside `outer` vertically, to half a pixel.
fn inside(inner: Rect, outer: Rect) -> bool {
    top(inner) >= top(outer) - 0.5 && bottom(inner) <= bottom(outer) + 0.5
}

/// Q340: the caller moves `selected` (through a claimed key) to a row below the fold, back up to
/// a row above the view, to the "Show More" action in the group's header, then to the first row:
/// each time the list scrolls the least that shows it, the stop against the nearer edge, never
/// centred, and not at all when it is in view.
#[test]
fn the_list_keeps_the_callers_selection_in_view() {
    let mut harness = Harness::new(LongPalette, VIEW);
    harness.advance(ms(200));
    // At rest the list is not scrolled, so its rect now is its scrollport. (Blitz moves an
    // element's own client rect by its own scroll offset, so it is read before any scroll.)
    let view = harness.rect("#card .ds-menu").expect("the list is drawn");
    let row = |n: usize| format!("#card .ds-menu-item:nth-of-type({})", n + 2);
    let far = harness.rect(&row(15)).expect("row 15 is laid out");
    assert!(top(far) > bottom(view), "row 15 starts below the fold");

    harness.key(Key::Char('j'));
    harness.advance(ms(100));
    let shown = selected_rect(&harness);
    assert_eq!(
        harness
            .text_of("#card .ds-menu-item[*|aria-selected=true] .ds-menu-title")
            .as_deref(),
        Some("Row 15")
    );
    assert!(inside(shown, view), "row 15 in view: {shown:?} in {view:?}");
    assert!(
        (bottom(shown) - bottom(view)).abs() < 1.0,
        "aligned with the bottom edge, not centred: {shown:?} in {view:?}"
    );

    // Back up to row 1, above the view now: it lands against the top edge.
    harness.key(Key::Char('n'));
    harness.advance(ms(100));
    let up = selected_rect(&harness);
    assert!(inside(up, view), "row 1 in view: {up:?} in {view:?}");
    assert!(
        (top(up) - top(view)).abs() < 1.0,
        "aligned with the top edge: {up:?} in {view:?}"
    );

    // The group's "Show More" sits in its header, above the rows: from row 15 the list goes back
    // up to show it.
    harness.key(Key::Char('j'));
    harness.advance(ms(100));
    harness.key(Key::Char('m'));
    harness.advance(ms(100));
    assert_eq!(
        harness
            .text_of("#card .ds-section-header-action[*|data-selected=true]")
            .as_deref(),
        Some("Show More")
    );
    let action = selected_rect(&harness);
    assert!(
        inside(action, view),
        "the action in view: {action:?} in {view:?}"
    );

    // Row 0 is in view with the header's action: the list does not move for it.
    harness.key(Key::Char('k'));
    harness.advance(ms(100));
    let first = selected_rect(&harness);
    assert!(inside(first, view), "row 0 in view: {first:?} in {view:?}");
    assert!(
        top(first) - bottom(action) > 0.0,
        "still below the header, nothing scrolled: {first:?} under {action:?}"
    );
}

/// Twenty-four rows in one group, as sill's real-input run had them.
fn rows_24() -> PaletteGroups<u8> {
    let rows = (0..24).map(|n| item(n, &format!("Row {n}"))).collect();
    PaletteGroups(vec![PaletteGroup::list("Many", rows)])
}

/// Q340 under the palette's own keys (sill's real-input run): Down pressed eleven times in a
/// 24-row group, with no caller holding the selection, brings row 11 (below the fold at rest)
/// into view against the bottom edge; Up back to row 3 brings it in against the top edge. The
/// palette is on a surface 420 px tall, as sill's launcher panel bounds it.
#[test]
fn the_palettes_own_down_and_up_scroll_the_selection_into_view() {
    #[allow(non_snake_case)]
    fn Own() -> Element {
        rsx! {
            Ds { appearance: Appearance::default(), material: Material::Sheet,
                div { style: "width:600px; height:420px; display:flex; flex-direction:column",
                    CommandPalette::<u8> {
                        label: "Launch",
                        placeholder: "Search",
                        query: String::new(),
                        tokens: Vec::new(),
                        groups: rows_24(),
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
    let mut harness = Harness::new(Own, VIEW);
    harness.advance(ms(200));
    let view = harness.rect("#card .ds-menu").expect("the list is drawn");
    let row_11 = harness
        .rect("#card .ds-menu-item:nth-of-type(13)")
        .expect("row 11 is laid out");
    assert!(top(row_11) > bottom(view), "row 11 starts below the fold");
    for _ in 0..11 {
        harness.key(Key::Down);
        harness.advance(ms(20));
    }
    harness.advance(ms(100));
    assert_eq!(
        harness
            .text_of("#card .ds-menu-item[*|aria-selected=true] .ds-menu-title")
            .as_deref(),
        Some("Row 11")
    );
    let shown = selected_rect(&harness);
    assert!(inside(shown, view), "row 11 in view: {shown:?} in {view:?}");
    assert!(
        (bottom(shown) - bottom(view)).abs() < 1.0,
        "against the bottom edge: {shown:?} in {view:?}"
    );
    for _ in 0..8 {
        harness.key(Key::Up);
        harness.advance(ms(20));
    }
    harness.advance(ms(100));
    let up = selected_rect(&harness);
    assert!(inside(up, view), "row 3 in view: {up:?} in {view:?}");
    assert!(
        (top(up) - top(view)).abs() < 1.0,
        "against the top edge: {up:?} in {view:?}"
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

#[component]
fn Opened(initial_caret: Option<InitialCaret>) -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut query = use_signal(|| "abc".to_string());
    let claim = move |key: FieldKey| -> Claim {
        log.with_mut(|log| log.push(caret_word(key.caret).to_string()));
        Claim::Take
    };
    let rows = vec![item(1, "Files")];
    let palette = match initial_caret {
        None => rsx! {
            CommandPalette::<u8> {
                label: "Launch",
                placeholder: "Search",
                query: query(),
                tokens: Vec::new(),
                groups: vec![("Applications".to_string(), rows)],
                empty: "Nothing",
                oninput: move |text: String| query.set(text),
                onpick: move |_| {},
                onclose: move |()| {},
                host: CommandPaletteHost::Surface,
                id: "card".to_string(),
                claim,
            }
        },
        Some(initial_caret) => rsx! {
            CommandPalette::<u8> {
                label: "Launch",
                placeholder: "Search",
                query: query(),
                tokens: Vec::new(),
                groups: vec![("Applications".to_string(), rows)],
                empty: "Nothing",
                oninput: move |text: String| query.set(text),
                onpick: move |_| {},
                onclose: move |()| {},
                host: CommandPaletteHost::Surface,
                id: "card".to_string(),
                claim: move |key: FieldKey| -> Claim {
                    log.with_mut(|log| log.push(caret_word(key.caret).to_string()));
                    match key.event.key() {
                        dioxus::prelude::Key::Character(_) => Claim::Pass,
                        _ => Claim::Take,
                    }
                },
                initial_caret,
            }
        },
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "width:600px; height:600px", {palette} }
            p { class: "query", "[{query}]" }
            p { class: "log", {log().join(",")} }
        }
    }
}

#[allow(non_snake_case)]
fn DefaultCaret() -> Element {
    rsx! { Opened { initial_caret: None } }
}

#[allow(non_snake_case)]
fn StartCaret() -> Element {
    rsx! { Opened { initial_caret: Some(InitialCaret::Start) } }
}

#[allow(non_snake_case)]
fn SelectAllCaret() -> Element {
    rsx! { Opened { initial_caret: Some(InitialCaret::SelectAll) } }
}

/// The caret's place the first claimed key reports, after the palette opened on "abc".
fn first_caret(app: fn() -> Element) -> String {
    let mut harness = Harness::new(app, VIEW);
    harness.advance(ms(200));
    harness.key(Key::Right);
    harness.advance(ms(20));
    harness.text_of(".log").unwrap_or_default()
}

/// Q341: opened on a query, the caret is after it by default (Right reads `AtEnd` at once); with
/// `Start` it is before it; with `SelectAll` the query is selected and a typed key replaces it.
#[test]
fn a_palette_opened_on_a_query_puts_the_caret_where_it_is_asked() {
    assert_eq!(first_caret(DefaultCaret), "end", "the default is the end");
    assert_eq!(
        first_caret(StartCaret),
        "inside",
        "Start puts it before the query"
    );
    assert_eq!(
        first_caret(SelectAllCaret),
        "inside",
        "a selection is not a bare caret"
    );
    let mut harness = Harness::new(SelectAllCaret, VIEW);
    harness.advance(ms(200));
    harness.key(Key::Char('x'));
    harness.advance(ms(20));
    assert_eq!(harness.text_of(".query").as_deref(), Some("[x]"));
}

#[allow(non_snake_case)]
fn KeyedRow() -> Element {
    let row = MenuEntry::Row(MenuRow {
        tile: Some(Tile::Icon(Icon::File)),
        shape: RowShape::File {
            thumb: None,
            location: "~/Documents".to_string(),
            modified: "00:33".to_string(),
        },
        trail: Trail::Shortcut(Shortcut(vec![Key::Enter])),
        ..MenuRow::new(1, "Invoice.pdf")
    });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "width:600px; height:600px",
                CommandPalette::<u8> {
                    label: "Launch",
                    placeholder: "Search",
                    query: String::new(),
                    tokens: Vec::new(),
                    groups: PaletteGroups(vec![PaletteGroup::list("Documents", vec![row])]),
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

/// Q343: a file row's time and its shortcut sit apart by `--s-6` (6 px), not run together.
#[test]
fn a_rows_time_and_its_shortcut_keep_their_gap() {
    let mut harness = Harness::new(KeyedRow, VIEW);
    harness.advance(ms(200));
    let when = harness
        .rect("#card .ds-menu-when")
        .expect("the time is drawn");
    let keys = harness
        .rect("#card .ds-menu-keys")
        .expect("the shortcut is drawn");
    let gap = keys.origin.x.0 - (when.origin.x.0 + when.size.width.0);
    assert!(
        (gap - 6.0).abs() < 0.75,
        "gap {gap}: {when:?} then {keys:?}"
    );
}
