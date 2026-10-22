//! `VirtualTable { outline }` draws the first column as an outline (a triangle on a branch, a gap on
//! a leaf, an indent by depth, `aria-level` and `aria-expanded` on the row), reports a press on the
//! triangle with the state asked for and Option widening it to the subtree, and answers Right and
//! Left on the cursor's row. `NameCell` writes a glyph and a name cut from the middle to the room
//! it was given.

use dioxus::prelude::*;
use ds::base::press::PointerButton;
use ds::base::vocab::Shown;
use ds::components::content::icon_source::IconSource;
use ds::components::controls::scroller::handle::use_scroller;
use ds::components::lists::name_cell::NameCell;
use ds::components::lists::table::look::TableLook;
use ds::components::lists::table::model::TableColumn;
use ds::components::lists::virtual_table::{OutlineRow, OutlineToggle, VirtualTable};
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Input, PointerAction, PointerInput, Viewport};

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};
const NAME: &str = "holiday-picture-2026-final-edit.png";
/// The room the name cell is given: 14 characters beside the icon.
const ROOM: Px = Px(120.0);

#[allow(non_snake_case)]
fn Page() -> Element {
    let scroller = use_scroller();
    let mut log = use_signal(String::new);
    let columns = vec![
        TableColumn::new(0u8, "Name", Px(240.0)),
        TableColumn::new(1u8, "Kind", Px(80.0)),
    ];
    let cell = Callback::new(|(_, column): (usize, usize)| match column {
        0 => rsx! {
            NameCell { icon: IconSource::Glyph(Icon::File), name: NAME, room: ROOM }
        },
        _ => rsx! { "Picture" },
    });
    let outline = Callback::new(|row: usize| match row {
        0 => OutlineRow::branch(0, Shown::Hidden),
        1 => OutlineRow::branch(1, Shown::Visible),
        _ => OutlineRow::leaf(2),
    });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:200px; width:360px; display:flex; flex-direction:column",
                VirtualTable::<u8> {
                    label: "Outline",
                    columns,
                    rows: 5usize,
                    cell,
                    scroller,
                    on_sort: |_| {},
                    look: TableLook::Striped,
                    cursor: Some(0),
                    onselect: |_| {},
                    outline,
                    ontoggle: move |toggle: OutlineToggle| {
                        log.with_mut(|log| {
                            log.push_str(&format!(
                                "{}:{:?}:{:?},",
                                toggle.row, toggle.shown, toggle.reach
                            ));
                        });
                    },
                }
            }
            p { class: "log", "{log()}" }
        }
    }
}

fn harness() -> Harness {
    Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

fn row(index: usize) -> String {
    format!(".ds-vtable-item[data-row=\"{index}\"]")
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

fn click_twisty(harness: &mut Harness, row_index: usize, held: Modifiers) {
    let at = harness
        .centre(&format!("{} .ds-vtable-twisty", row(row_index)))
        .expect("a triangle");
    let click = PointerInput::new(at, PointerAction::Click(PointerButton::Primary));
    harness.send(Input::Pointer(click.with_mods(held)));
}

#[test]
fn a_branch_has_a_triangle_a_leaf_a_gap_and_each_level_is_indented() {
    let harness = harness();
    assert_eq!(harness.count(".ds-vtable-twisty"), 2);
    assert_eq!(harness.count(".ds-vtable-twisty-gap"), 3);
    let indent =
        |index: usize| harness.attr(&format!("{} .ds-vtable-outline", row(index)), "style");
    assert_eq!(indent(0).as_deref(), Some("padding-left:0px"));
    assert_eq!(indent(1).as_deref(), Some("padding-left:16px"));
    assert_eq!(indent(2).as_deref(), Some("padding-left:32px"));
    let levels: Vec<_> = (0..3)
        .map(|index| harness.attr(&row(index), "aria-level"))
        .collect();
    assert_eq!(
        levels,
        [
            Some("1".to_owned()),
            Some("2".to_owned()),
            Some("3".to_owned())
        ]
    );
    assert_eq!(
        harness.attr(&row(0), "aria-expanded").as_deref(),
        Some("false")
    );
    assert_eq!(
        harness.attr(&row(1), "aria-expanded").as_deref(),
        Some("true")
    );
    assert_eq!(harness.attr(&row(2), "aria-expanded"), None);
}

#[test]
fn a_press_on_the_triangle_asks_for_the_other_state_and_option_asks_for_the_subtree() {
    let mut harness = harness();
    click_twisty(&mut harness, 0, Modifiers::empty());
    click_twisty(&mut harness, 1, Modifiers::empty());
    click_twisty(&mut harness, 0, Modifiers::ALT);
    assert_eq!(
        log(&harness),
        "0:Visible:Row,1:Hidden:Row,0:Visible:Subtree,"
    );
}

#[test]
fn right_opens_the_cursor_row_and_left_does_nothing_on_a_row_that_is_closed() {
    let mut harness = harness();
    // A header takes the focus, so the keys reach the table.
    let header = harness.centre(".ds-table-column").expect("a header");
    harness.send(Input::click(header));
    harness.send(Input::key(ShortcutKey::Left));
    assert_eq!(log(&harness), "", "row 0 is closed already");
    harness.send(Input::key(ShortcutKey::Right));
    assert_eq!(log(&harness), "0:Visible:Row,");
}

#[test]
fn a_name_is_cut_from_the_middle_to_its_room_and_keeps_its_extension() {
    let harness = harness();
    let shown = harness.text_of(".ds-name-text").expect("a name");
    assert_eq!(shown.chars().count(), 14, "{shown}");
    assert!(shown.contains('…') && shown.ends_with(".png"), "{shown}");
    assert_eq!(
        harness.attr(".ds-name-text", "aria-label").as_deref(),
        Some(NAME)
    );
}
