//! The edit surface's second round (FINDINGS "Edit surface 2"): a programmatic focus and blur
//! that do what a press and a blur do, Shift+click and a drag driven through the harness, the
//! pointer captured past the surface's box until the release, the navigation keys, the caret's
//! width, and the app's own class and data on the surface.

use dioxus::prelude::*;
use ds::{
    Appearance, Composition, DataAttr, DataName, Ds, EditFocus, EditHandle, EditInput, EditPointer,
    EditSurface, Extend, ExtraClass, ImeSwitch, Key, KeyInput, Material, Point, PointerPhase,
    Probe, Px, TextPosition, use_edit_handle,
};
use ds_native::{Harness, Viewport};
use std::cell::RefCell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
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

fn drain<T>(log: &'static std::thread::LocalKey<RefCell<Vec<T>>>) -> Vec<T> {
    log.with(|log| log.borrow_mut().drain(..).collect())
}

fn at(x: f32, y: f32) -> Point {
    Point { x: Px(x), y: Px(y) }
}

/// Two paragraphs in a surface that carries the app's own class and data, above a button
/// outside it.
#[allow(non_snake_case)]
fn Editor() -> Element {
    let handle = use_edit_handle();
    HANDLE.with(|slot| *slot.borrow_mut() = Some(handle));
    let data = DataName::parse("draft")
        .map(|name| vec![DataAttr::new(name, "42")])
        .unwrap_or_default();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "padding:20px; width:300px; font-size:16px; line-height:20px",
                EditSurface {
                    id: "editor",
                    handle,
                    extra_class: ExtraClass::parse("c-body").ok(),
                    data,
                    on_input: |input| INPUT.with(|log| log.borrow_mut().push(input)),
                    on_pointer: |pointer| POINTER.with(|log| log.borrow_mut().push(pointer)),
                    on_focus: |focus| FOCUS.with(|log| log.borrow_mut().push(focus)),
                    p { id: "one", "data-edit-node": "p0", style: "margin:0", "Hello world" }
                    p { id: "two", "data-edit-node": "p1", style: "margin:0", "Second line here" }
                }
                button { id: "away", style: "margin-top:80px", "Away" }
            }
        }
    }
}

fn handle() -> EditHandle {
    HANDLE.with(|slot| slot.borrow().expect("the editor rendered"))
}

fn fresh() -> Harness {
    drain(&INPUT);
    drain(&POINTER);
    drain(&FOCUS);
    let mut harness = Harness::new(Editor, VIEW);
    harness.advance(ms(50));
    harness
}

fn centre(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} missing"))
}

#[test]
fn the_surface_carries_the_apps_class_and_data() {
    let harness = fresh();
    assert!(harness.has_class("#editor", "ds-edit"));
    assert!(harness.has_class("#editor", "c-body"));
    assert_eq!(harness.attr("#editor", "data-draft").as_deref(), Some("42"));
}

#[test]
fn a_programmatic_focus_does_what_a_press_does() {
    let mut harness = fresh();
    assert_eq!(harness.ime_switch(), ImeSwitch::Off);
    harness.within(|| handle().focus());
    harness.advance(ms(50));
    assert!(harness.is_focused("#editor"));
    assert_eq!(drain(&FOCUS), vec![EditFocus::In]);
    assert_eq!(harness.ime_switch(), ImeSwitch::On);
    harness.ime_update("か", 3);
    harness.ime_commit("火");
    let heard = drain(&INPUT);
    assert_eq!(
        heard.first(),
        Some(&EditInput::Composition(Composition::Start))
    );
    assert_eq!(
        heard.last(),
        Some(&EditInput::Composition(Composition::End {
            text: "火".to_owned()
        })),
        "the surface is the IME's target"
    );
    harness.within(|| handle().blur());
    harness.advance(ms(50));
    assert!(!harness.is_focused("#editor"));
    assert_eq!(drain(&FOCUS), vec![EditFocus::Out]);
    assert_eq!(harness.ime_switch(), ImeSwitch::Off);
    harness.ime_commit("x");
    assert_eq!(drain(&INPUT), Vec::new(), "no longer the IME's target");
}

#[test]
fn shift_click_extends_from_the_anchor() {
    let mut harness = fresh();
    harness.click(centre(&harness, "#one"));
    harness.advance(ms(600));
    drain(&POINTER);
    let two = harness.rect("#two").expect("the second paragraph");
    let point = at(two.origin.x.0 + 1.0, two.origin.y.0 + 10.0);
    harness.click_with(point, Modifiers::SHIFT);
    let log = drain(&POINTER);
    assert_eq!(log[0].phase, PointerPhase::Press);
    assert_eq!(log[0].extend, Extend::FromAnchor);
    assert_eq!(log[0].position, Some(TextPosition::new("p1", 0)));
}

#[test]
fn a_drag_across_two_nodes_reports_each_with_the_button_held() {
    let mut harness = fresh();
    let one = harness.rect("#one").expect("the first paragraph");
    let two = harness.rect("#two").expect("the second paragraph");
    let from = at(one.origin.x.0 + 1.0, one.origin.y.0 + 10.0);
    let to = at(two.origin.x.0 + 400.0 - 150.0, two.origin.y.0 + 10.0);
    harness.pointer_move(from);
    harness.pointer_down(from);
    assert!(harness.held_buttons().contains(ds::PointerButton::Primary));
    harness.pointer_move(to);
    harness.pointer_up(to);
    assert!(harness.held_buttons().is_empty());
    let log = drain(&POINTER);
    let phases: Vec<PointerPhase> = log.iter().map(|pointer| pointer.phase).collect();
    assert_eq!(
        phases,
        vec![
            PointerPhase::Press,
            PointerPhase::Drag,
            PointerPhase::Release
        ]
    );
    assert_eq!(log[0].position, Some(TextPosition::new("p0", 0)));
    let nodes: Vec<Option<String>> = log[1..]
        .iter()
        .map(|pointer| pointer.position.as_ref().map(|p| p.node.0.clone()))
        .collect();
    assert_eq!(nodes, vec![Some("p1".to_owned()), Some("p1".to_owned())]);
}

#[test]
fn a_drag_keeps_reporting_outside_the_surface_until_the_release() {
    let mut harness = fresh();
    let one = harness.rect("#one").expect("the first paragraph");
    let surface = harness.rect("#editor").expect("the surface");
    let from = at(one.origin.x.0 + 1.0, one.origin.y.0 + 10.0);
    // Below the surface, over the button: not the surface's box at all.
    let outside = centre(&harness, "#away");
    assert!(outside.y.0 > surface.origin.y.0 + surface.size.height.0);
    harness.drag(from, outside, 3);
    let log = drain(&POINTER);
    let phases: Vec<PointerPhase> = log.iter().map(|pointer| pointer.phase).collect();
    assert_eq!(phases.first(), Some(&PointerPhase::Press));
    assert_eq!(phases.last(), Some(&PointerPhase::Release));
    let release = log.last().expect("a release");
    assert_eq!(release.at, outside);
    assert_eq!(
        release.position.as_ref().map(|p| p.node.0.as_str()),
        Some("p1"),
        "the nearest text below the point"
    );
    assert!(
        phases
            .iter()
            .filter(|&&phase| phase == PointerPhase::Drag)
            .count()
            >= 2,
        "{phases:?}"
    );
    harness.pointer_move(centre(&harness, "#one"));
    assert_eq!(
        drain(&POINTER),
        Vec::new(),
        "the capture ends at the release"
    );
}

#[test]
fn home_end_and_delete_reach_the_app() {
    let mut harness = fresh();
    harness.click(centre(&harness, "#one"));
    drain(&INPUT);
    for key in [Key::Home, Key::End, Key::Delete, Key::PageDown] {
        harness.key(key);
    }
    let keys: Vec<dioxus::prelude::Key> = drain(&INPUT)
        .into_iter()
        .filter_map(|input| match input {
            EditInput::Key(KeyInput { key, .. }) => Some(key),
            _ => None,
        })
        .collect();
    use dioxus::prelude::Key as K;
    assert_eq!(keys, vec![K::Home, K::End, K::Delete, K::PageDown]);
    harness.chord(&[Key::Shift], Key::Insert);
    assert!(matches!(
        drain(&INPUT).as_slice(),
        [] | [EditInput::Paste(_)]
    ));
}

#[test]
fn the_caret_is_one_device_pixel_wide_at_a_fractional_scale() {
    for (scale_percent, width) in [(100, 1.0), (150, 2.0 / 3.0), (200, 1.0)] {
        drain(&INPUT);
        let mut harness = Harness::new(
            Editor,
            Viewport {
                scale_percent,
                ..VIEW
            },
        );
        harness.advance(ms(50));
        let position = TextPosition::new("p0", 2);
        let caret = harness.within(|| handle().caret_rect(&position));
        match caret {
            Probe::Found(rect) => assert!(
                (rect.size.width.0 - width).abs() < 1e-3,
                "{scale_percent}: {rect:?}"
            ),
            other => panic!("{scale_percent}: {other:?}"),
        }
    }
}
