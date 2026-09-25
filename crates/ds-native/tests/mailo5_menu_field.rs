//! mailo gaps 5, a menu whose filter is a drawn field, on a real Blitz document:
//! `filter: Filter::Field { placeholder }` shows the placeholder in a row at the top of the
//! menu, then each typed character there as it filters the rows and `onquery` hears it; the
//! cursor stays on the rows below the field, and Enter picks the highlighted match.

use dioxus::prelude::*;
use ds::{
    Anchor, Appearance, Ds, Filter, Key, Material, Menu, MenuEntry, MenuKind, MenuRow, Point, Px,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 480,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

const LABELS: [&str; 4] = ["Invoices", "Travel", "Family", "Receipts"];

/// A label picker with a query field; the page logs each query and the pick.
#[allow(non_snake_case)]
fn Page() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut open = use_signal(|| true);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            if open() {
                Menu::<usize> {
                    kind: MenuKind::Rich,
                    anchor: Anchor::Point(Point { x: Px(40.0), y: Px(40.0) }),
                    entries: LABELS
                        .into_iter()
                        .enumerate()
                        .map(|(value, name)| MenuEntry::Row(MenuRow::new(value, name)))
                        .collect::<Vec<_>>(),
                    filter: Filter::Field { placeholder: "Filter labels…".to_string() },
                    onquery: move |query: String| log.with_mut(|log| log.push(format!("query:{query}"))),
                    onpick: move |value: usize| log.with_mut(|log| log.push(format!("pick:{}", LABELS[value]))),
                    onclose: move |_| open.set(false),
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

/// The rows' text, the field's excluded.
fn rows(harness: &Harness) -> String {
    let menu = harness.text_of(".ds-menu").unwrap_or_default();
    let field = harness.text_of(".ds-menu-filter").unwrap_or_default();
    menu.replacen(&field, "", 1)
}

#[test]
fn the_typed_query_shows_in_the_field_and_filters_the_rows() {
    let mut harness = Harness::new(Page, VIEW);
    harness.advance(ms(300));
    assert!(
        harness.is_focused(".ds-menu"),
        "the menu holds the keyboard"
    );
    assert_eq!(
        harness.text_of(".ds-menu-filter-placeholder").as_deref(),
        Some("Filter labels…")
    );
    assert_eq!(harness.count(".ds-menu-item"), 4);
    let field = harness.rect(".ds-menu-filter").expect("the field row");
    let first = harness.rect(".ds-menu-item").expect("the first row");
    assert!(
        field.origin.y + field.size.height <= first.origin.y,
        "the field sits above the rows: {field:?} {first:?}"
    );

    harness.key(Key::Char('r'));
    harness.key(Key::Char('e'));
    harness.advance(ms(50));
    assert_eq!(
        harness.text_of(".ds-menu-filter-text").as_deref(),
        Some("re")
    );
    assert_eq!(harness.count(".ds-menu-filter-placeholder"), 0);
    assert_eq!(harness.count(".ds-menu-item"), 2);
    let shown = rows(&harness);
    assert!(
        shown.contains("Receipts") && shown.contains("Travel") && !shown.contains("Family"),
        "the fuzzy ranker kept r..e: {shown:?}"
    );
    assert_eq!(
        harness
            .attr(
                ".ds-menu-item[*|aria-selected=true] .ds-menu-title",
                "class"
            )
            .is_some(),
        true,
        "the cursor is on a row, below the field"
    );

    harness.key(Key::Backspace);
    harness.key(Key::Char('c'));
    harness.advance(ms(50));
    assert_eq!(
        harness.text_of(".ds-menu-filter-text").as_deref(),
        Some("rc")
    );
    assert_eq!(harness.count(".ds-menu-item"), 1);
    assert_eq!(
        harness.text_of(".ds-menu-item .ds-menu-title").as_deref(),
        Some("Receipts")
    );
    harness.key(Key::Enter);
    harness.advance(ms(300));
    assert_eq!(
        log(&harness),
        "query:r,query:re,query:r,query:rc,pick:Receipts"
    );
}
