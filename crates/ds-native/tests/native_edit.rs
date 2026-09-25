//! The edit surface (mailo Phase B, G2; FINDINGS "Edit surface"): typing, an IME composition,
//! a paste with HTML, and the geometry an app draws its own caret and selection from, all
//! through a real Blitz document.

use dioxus::prelude::*;
use ds::{
    Appearance, Composition, Ds, EditFocus, EditHandle, EditInput, EditPointer, EditSurface,
    ImeSwitch, Key, KeyInput, Material, Pasted, Point, PointerPhase, Probe, Px, Rect, Size,
    TextPosition, TextRange, use_edit_handle,
};
use ds_native::{Harness, Viewport};
use std::cell::RefCell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 300,
    scale_percent: 100,
};

thread_local! {
    static INPUT: RefCell<Vec<EditInput>> = const { RefCell::new(Vec::new()) };
    static POINTER: RefCell<Vec<EditPointer>> = const { RefCell::new(Vec::new()) };
    static FOCUS: RefCell<Vec<EditFocus>> = const { RefCell::new(Vec::new()) };
    static HANDLE: RefCell<Option<EditHandle>> = const { RefCell::new(None) };
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn heard() -> Vec<EditInput> {
    INPUT.with(|log| log.borrow_mut().drain(..).collect())
}

fn pointed() -> Vec<EditPointer> {
    POINTER.with(|log| log.borrow_mut().drain(..).collect())
}

/// Where the first paragraph's text starts: the surface's padding inside the page's.
const LEFT: f32 = 20.0 + 10.0;
const TOP: f32 = 20.0 + 10.0;
/// Every line is this tall (`line-height`).
const LINE: f32 = 20.0;

/// A surface over two paragraphs, the second with a hard line break and two text nodes, then a
/// paragraph holding an inline chip (an atom).
#[allow(non_snake_case)]
fn Editor() -> Element {
    let handle = use_edit_handle();
    HANDLE.with(|slot| *slot.borrow_mut() = Some(handle));
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "padding:20px; width:300px; font-size:16px; line-height:20px",
                EditSurface {
                    id: "editor",
                    label: "Message",
                    handle,
                    on_input: |input| INPUT.with(|log| log.borrow_mut().push(input)),
                    on_pointer: |pointer| POINTER.with(|log| log.borrow_mut().push(pointer)),
                    on_focus: |focus| FOCUS.with(|log| log.borrow_mut().push(focus)),
                    div { style: "padding:10px",
                        p { id: "one", "data-edit-node": "p0", style: "margin:0", "Hello world" }
                        p { id: "two", "data-edit-node": "p1", style: "margin:0",
                            span { "first line\n" }
                            span { "second line" }
                        }
                        p { id: "three", "data-edit-node": "p2", style: "margin:0",
                            "to "
                            span { id: "chip", "data-edit-node": "chip", "data-edit-kind": "atom",
                                style: "display:inline-block; width:40px; height:16px",
                                "Ada"
                            }
                            " now"
                        }
                    }
                }
            }
        }
    }
}

fn handle() -> EditHandle {
    HANDLE.with(|slot| slot.borrow().expect("the editor rendered"))
}

/// A harness with the surface focused by a click and every log empty.
fn focused() -> Harness {
    focused_at(VIEW)
}

/// As [`focused`], at `view`.
fn focused_at(view: Viewport) -> Harness {
    INPUT.with(|log| log.borrow_mut().clear());
    FOCUS.with(|log| log.borrow_mut().clear());
    let mut harness = Harness::new(Editor, view);
    harness.advance(ms(50));
    let into = harness.centre("#one").expect("the first paragraph");
    harness.click(into);
    harness.advance(ms(50));
    pointed();
    harness
}

fn caret(harness: &mut Harness, node: &str, offset: usize) -> Rect {
    let position = TextPosition::new(node, offset);
    harness.within(|| read_now(handle().caret_rect(&position)))
}

fn read_now<T: std::fmt::Debug>(probe: Probe<T>) -> T {
    match probe {
        Probe::Found(found) => found,
        other => panic!("no answer: {other:?}"),
    }
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.5
}

#[test]
fn a_click_focuses_the_surface_and_switches_the_ime_on() {
    let harness = focused();
    assert!(harness.is_focused("#editor"));
    assert_eq!(harness.ime_switch(), ImeSwitch::On);
    assert_eq!(FOCUS.with(|log| log.borrow().clone()), vec![EditFocus::In]);
}

#[test]
fn typed_keys_arrive_as_text_and_keys_in_order() {
    let mut harness = focused();
    harness.key(Key::Char('h'));
    harness.key(Key::Char('i'));
    harness.key(Key::Enter);
    harness.chord(&[Key::Ctrl], Key::Char('b'));
    assert_eq!(
        heard(),
        vec![
            EditInput::Text("h".to_owned()),
            EditInput::Text("i".to_owned()),
            EditInput::Key(KeyInput {
                key: dioxus::prelude::Key::Enter,
                modifiers: Modifiers::empty(),
            }),
            EditInput::Key(KeyInput {
                key: dioxus::prelude::Key::Character("b".to_owned()),
                modifiers: Modifiers::CONTROL,
            }),
        ]
    );
}

#[test]
fn a_composition_arrives_as_start_updates_and_end_in_order() {
    let mut harness = focused();
    harness.ime_start();
    harness.ime_update("ㄓ", 3);
    harness.ime_update("ㄓㄨ", 6);
    harness.ime_commit("注");
    let update = |text: &str, cursor: Option<usize>| {
        EditInput::Composition(Composition::Update {
            text: text.to_owned(),
            cursor: cursor.map(|at| ds::PreeditCursor { start: at, end: at }),
        })
    };
    assert_eq!(
        heard(),
        vec![
            EditInput::Composition(Composition::Start),
            update("ㄓ", Some(3)),
            update("ㄓㄨ", Some(6)),
            update("", None),
            EditInput::Composition(Composition::End {
                text: "注".to_owned()
            }),
        ]
    );
    harness.key(Key::Char('x'));
    assert_eq!(
        heard(),
        vec![EditInput::Text("x".to_owned())],
        "typing after it is text again"
    );
}

#[test]
fn keys_while_composing_belong_to_the_ime() {
    let mut harness = focused();
    harness.ime_update("ka", 2);
    heard();
    harness.key(Key::Char('n'));
    assert_eq!(heard(), Vec::new());
}

#[test]
fn a_paste_carries_the_clipboards_html_and_its_text() {
    let mut harness = focused();
    harness.paste_html("<b>bold</b> move", "bold move");
    assert_eq!(
        heard(),
        vec![EditInput::Paste(Pasted::Html {
            html: "<b>bold</b> move".to_owned(),
            text: "bold move".to_owned(),
        })]
    );
    harness.set_clipboard_text("plain");
    harness.chord(&[Key::Ctrl], Key::Char('v'));
    assert_eq!(
        heard(),
        vec![EditInput::Paste(Pasted::Text("plain".to_owned()))]
    );
    harness.chord(&[Key::Ctrl], Key::Char('c'));
    harness.chord(&[Key::Ctrl], Key::Char('x'));
    assert_eq!(heard(), vec![EditInput::Copy, EditInput::Cut]);
}

#[test]
fn caret_rects_sit_at_the_start_middle_and_end_of_a_line() {
    for scale_percent in [100, 150] {
        caret_rects_at(Viewport {
            scale_percent,
            ..VIEW
        });
    }
}

/// The caret cases at one display scale: parley measures in device units, the rects are logical.
fn caret_rects_at(view: Viewport) {
    let mut harness = focused_at(view);
    let top = harness
        .rect("#two")
        .expect("the second paragraph")
        .origin
        .y
        .0;
    let start = caret(&mut harness, "p1", 0);
    let middle = caret(&mut harness, "p1", 5);
    let end = caret(&mut harness, "p1", 10);
    let next = caret(&mut harness, "p1", 11);
    assert!(near(start.origin.x.0, LEFT), "start {start:?}");
    assert!(near(start.origin.y.0, top), "start {start:?} top {top}");
    assert!(near(start.size.height.0, LINE), "start {start:?}");
    assert_eq!(start.size.width, Px(0.0));
    assert!(
        middle.origin.x.0 > start.origin.x.0 + 20.0,
        "middle {middle:?}"
    );
    assert!(end.origin.x.0 > middle.origin.x.0 + 20.0, "end {end:?}");
    for rect in [middle, end] {
        assert!(near(rect.origin.y.0, top), "{rect:?} on the first line");
    }
    assert!(
        near(next.origin.x.0, LEFT),
        "the second line starts at the left: {next:?}"
    );
    assert!(near(next.origin.y.0, top + LINE), "one line down: {next:?}");
}

#[test]
fn a_caret_at_an_atom_sits_at_its_edge() {
    let mut harness = focused();
    let chip = harness.rect("#chip").expect("the chip");
    let before = caret(&mut harness, "chip", 0);
    let after = caret(&mut harness, "chip", 1);
    assert!(
        near(before.origin.x.0, chip.origin.x.0),
        "{before:?} {chip:?}"
    );
    assert!(
        near(after.origin.x.0, chip.origin.x.0 + chip.size.width.0),
        "{after:?} {chip:?}"
    );
}

#[test]
fn hit_testing_a_two_line_paragraph_finds_the_node_and_offset() {
    let mut harness = focused();
    let top = harness
        .rect("#two")
        .expect("the second paragraph")
        .origin
        .y
        .0;
    let at = |x: f32, y: f32| Point { x: Px(x), y: Px(y) };
    let second_line = top + LINE * 1.5;
    let cases = [
        // The very start of the second line.
        (at(LEFT + 0.5, second_line), TextPosition::new("p1", 11)),
        // Past the end of the first line: its end, before the break.
        (
            at(LEFT + 280.0, top + LINE / 2.0),
            TextPosition::new("p1", 10),
        ),
        // Past the end of the second line: the paragraph's end, across both text nodes.
        (at(LEFT + 280.0, second_line), TextPosition::new("p1", 22)),
        // The first paragraph.
        (at(LEFT + 0.5, TOP + LINE / 2.0), TextPosition::new("p0", 0)),
    ];
    for (point, expected) in cases {
        assert_eq!(
            harness.hit_test("#editor", point),
            Some(expected.clone()),
            "{point:?}"
        );
    }
    // Just right of the boundary before `n` in "second", as the caret rect puts it.
    let boundary = caret(&mut harness, "p1", 14);
    let point = at(boundary.origin.x.0 + 1.0, second_line);
    assert_eq!(
        harness.hit_test("#editor", point),
        Some(TextPosition::new("p1", 14))
    );
    // Either half of the chip.
    let chip = harness.rect("#chip").expect("the chip");
    let left = at(chip.origin.x.0 + 2.0, chip.origin.y.0 + 4.0);
    let right = at(
        chip.origin.x.0 + chip.size.width.0 - 2.0,
        chip.origin.y.0 + 4.0,
    );
    assert_eq!(
        harness.hit_test("#editor", left),
        Some(TextPosition::new("chip", 0))
    );
    assert_eq!(
        harness.hit_test("#editor", right),
        Some(TextPosition::new("chip", 1))
    );
}

#[test]
fn a_press_reports_the_position_under_it() {
    let mut harness = focused();
    let top = harness
        .rect("#two")
        .expect("the second paragraph")
        .origin
        .y
        .0;
    let point = Point {
        x: Px(LEFT + 0.5),
        y: Px(top + LINE * 1.5),
    };
    harness.click(point);
    let log = pointed();
    let phases: Vec<PointerPhase> = log.iter().map(|pointer| pointer.phase).collect();
    assert_eq!(phases, vec![PointerPhase::Press, PointerPhase::Release]);
    assert_eq!(log[0].position, Some(TextPosition::new("p1", 11)));
    assert_eq!(log[0].at, point);
}

#[test]
fn selection_rects_across_a_line_break_cover_one_box_per_line() {
    let mut harness = focused();
    let top = harness
        .rect("#two")
        .expect("the second paragraph")
        .origin
        .y
        .0;
    let from = caret(&mut harness, "p1", 6);
    let to = caret(&mut harness, "p1", 14);
    let range = TextRange {
        anchor: TextPosition::new("p1", 6),
        focus: TextPosition::new("p1", 14),
    };
    let rects = harness.within(|| read_now(handle().selection_rects(&range)));
    assert_eq!(rects.len(), 2, "{rects:?}");
    assert!(
        near(rects[0].origin.x.0, from.origin.x.0),
        "{rects:?} from {from:?}"
    );
    assert!(near(rects[0].origin.y.0, top), "{rects:?}");
    assert!(near(rects[1].origin.x.0, LEFT), "{rects:?}");
    assert!(near(rects[1].origin.y.0, top + LINE), "{rects:?}");
    assert!(
        near(rects[1].origin.x.0 + rects[1].size.width.0, to.origin.x.0),
        "{rects:?} to {to:?}"
    );
    let backwards = TextRange {
        anchor: range.focus.clone(),
        focus: range.anchor.clone(),
    };
    let reversed = harness.within(|| read_now(handle().selection_rects(&backwards)));
    assert_eq!(reversed, rects, "either end may come first");
}

#[test]
fn a_selection_over_an_atom_covers_its_box() {
    let mut harness = focused();
    let chip = harness.rect("#chip").expect("the chip");
    let range = TextRange {
        anchor: TextPosition::new("p2", 0),
        focus: TextPosition::new("p2", 7),
    };
    let rects = harness.within(|| read_now(handle().selection_rects(&range)));
    assert!(rects.contains(&chip), "{rects:?} {chip:?}");
}

/// A surface whose IME area follows what the app sets: the caret's rect, moved by a key.
#[allow(non_snake_case)]
fn Area() -> Element {
    let mut area = use_signal(|| rect(10.0, 12.0));
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            EditSurface { id: "editor", ime_area: Some(area()),
                on_input: move |_| area.set(rect(40.0, 32.0)),
                p { "data-edit-node": "0", "Hello" }
            }
        }
    }
}

fn rect(x: f32, y: f32) -> Rect {
    Rect {
        origin: Point { x: Px(x), y: Px(y) },
        size: Size {
            width: Px(0.0),
            height: Px(18.0),
        },
    }
}

#[test]
fn the_ime_cursor_area_follows_what_the_app_sets() {
    let mut harness = Harness::new(Area, VIEW);
    harness.advance(ms(50));
    assert_eq!(
        harness.ime_cursor_area(),
        None,
        "not before the surface has the keyboard"
    );
    let into = harness.centre("#editor").expect("the surface");
    harness.click(into);
    harness.advance(ms(50));
    assert_eq!(harness.ime_cursor_area(), Some(rect(10.0, 12.0)));
    harness.key(Key::Char('a'));
    harness.advance(ms(50));
    assert_eq!(harness.ime_cursor_area(), Some(rect(40.0, 32.0)));
}
