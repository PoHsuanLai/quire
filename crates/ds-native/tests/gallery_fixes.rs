//! The component bugs the gallery's contact sheets exposed (FINDINGS "Gallery fixes A"), each
//! proved on a real Blitz document: a rendered height, a laid-out rect, or a pixel probe. The
//! markup side of each fix is in ds's goldens; these are the pictures the goldens cannot show.

use dioxus::prelude::*;
use ds::{
    Appearance, DotIndex, Ds, Fraction, Grain, HeaderKind, Here, Icon, InputVariant, ItemKind,
    Material, PRESETS, Presence, PulseKey, Rect, Scheme, SearchField, SectionHeader, SendPhase,
    SendPill, SidebarItem, SpaceEditor, SpaceLook, TextInput, Theme, use_toasts,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 360,
    scale_percent: 100,
};

/// The root every quire surface draws inside.
#[component]
fn Root(children: Element) -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, {children} }
    }
}

#[path = "support/probe.rs"]
mod probe;

use probe::{centred, distance, ink, keep, pixels, rect};

// ---- 1. TextInput is one line tall ----------------------------------------------------------

#[allow(non_snake_case)]
fn FieldsApp() -> Element {
    rsx! {
        Root {
            div { style: "width:240px",
                div { class: "probe-boxed",
                    TextInput { variant: InputVariant::Boxed, label: "To", value: "", placeholder: "Add a person", oninput: |_| {} }
                }
                div { class: "probe-inline",
                    TextInput { variant: InputVariant::Inline, label: "To", value: "dana@example.org", oninput: |_| {} }
                }
                div { class: "probe-search",
                    SearchField { label: "Search", value: "", placeholder: "Search mail", tokens: Vec::new(), oninput: |_| {}, onkey: |_| {} }
                }
            }
        }
    }
}

#[test]
fn a_text_input_is_one_line_tall_and_fills_its_wrapper() {
    let harness = Harness::new(FieldsApp, VIEW);
    // design/04-COMPONENTS.md section 6: one 13.5 px line at the base line height 1.55, plus the
    // padding (7 + 7 boxed, 3 + 3 inline) and the 1 px border; the search row's is 16 px with no
    // padding or border (section 7).
    const CASES: &[(&str, f32)] = &[
        (".probe-boxed .ds-input", 13.5 * 1.55 + 16.0),
        (".probe-inline .ds-input", 13.5 * 1.55 + 8.0),
        (".probe-search .ds-input", 16.0 * 1.55),
    ];
    for &(selector, want) in CASES {
        let got = rect(&harness, selector).size.height.0;
        assert!(
            (got - want).abs() <= 2.0,
            "{selector} is {got} px tall, want {want} (a replaced-element default is 150)"
        );
    }
    // Width 100% inside the wrapper: the boxed field is as wide as the 240 px column.
    let boxed = rect(&harness, ".probe-boxed .ds-input");
    assert!(
        (boxed.size.width.0 - 240.0).abs() <= 1.0,
        "the boxed field is {} px wide, want the column's 240",
        boxed.size.width.0
    );
}

// ---- 2 and 3. The toast: nothing when empty, and a legible tab ----------------------------

#[allow(non_snake_case)]
fn EmptyApp() -> Element {
    rsx! {
        Root {
            div { style: "height:340px" }
        }
    }
}

#[test]
fn a_root_with_an_empty_hub_lays_out_no_toast() {
    let mut harness = Harness::new(EmptyApp, VIEW);
    harness.advance(Duration::from_millis(100));
    assert_eq!(harness.count(".ds-toast"), 0, "{}", harness.html());
    // No pill in the picture either: the root's bottom edge is one flat ground.
    let frame = harness.render().expect("renders");
    let bottom = Rect {
        origin: ds::Point {
            x: ds::Px(120.0),
            y: ds::Px(280.0),
        },
        size: ds::Size {
            width: ds::Px(240.0),
            height: ds::Px(58.0),
        },
    };
    keep(&frame, "empty-root");
    let region = pixels(&frame, bottom, 0.0);
    assert_eq!(
        ink(&region, 6),
        0,
        "something is drawn at the bottom centre"
    );
}

#[component]
fn Push() -> Element {
    let toasts = use_toasts();
    use_hook(move || toasts.push("Archived".to_string(), Some(ds::UndoToken(1))));
    rsx! {}
}

#[allow(non_snake_case)]
fn ToastApp() -> Element {
    rsx! {
        Root {
            div { style: "position:relative; height:340px", Push {} }
        }
    }
}

#[test]
fn the_toast_tab_shows_its_label() {
    let mut harness = Harness::new(ToastApp, VIEW);
    harness.advance(Duration::from_millis(700));
    assert_eq!(
        harness.attr(".ds-toast", "data-shown").as_deref(),
        Some("shown")
    );
    assert_eq!(harness.text_of(".ds-toast-tab").as_deref(), Some("Undo"));
    let frame = harness.render().expect("renders");
    // The tab is `--paper` with `--ink` text: its label and icon are ink on the tab's ground.
    // With the label in the toast's own `--paper` there was nothing to see.
    keep(&frame, "toast-tab");
    let tab = pixels(&frame, centred(&harness, ".ds-toast", ".ds-toast-tab"), 3.0);
    let label = ink(&tab, 60);
    assert!(label > 40, "the tab's label painted {label} pixels");
}

#[allow(non_snake_case)]
fn SendApp() -> Element {
    rsx! {
        Root {
            div { style: "position:relative; height:340px",
                SendPill { text: "Sending in 3 s", progress: Fraction(400), phase: SendPhase::Counting, onundo: |_| {} }
            }
        }
    }
}

#[test]
fn the_send_pill_undo_shows_its_label() {
    let mut harness = Harness::new(SendApp, VIEW);
    harness.advance(Duration::from_millis(700));
    assert_eq!(
        harness.text_of(".ds-send-pill-undo").as_deref(),
        Some("Undo")
    );
    let frame = harness.render().expect("renders");
    keep(&frame, "send-undo");
    let undo = pixels(
        &frame,
        centred(&harness, ".ds-send-pill", ".ds-send-pill-undo"),
        3.0,
    );
    let label = ink(&undo, 60);
    assert!(label > 30, "Undo's label painted {label} pixels");
}

// ---- 5. SectionHeader: one full-width row each ----------------------------------------------

#[allow(non_snake_case)]
fn HeadersApp() -> Element {
    rsx! {
        Root {
            div { class: "probe-grid", style: "display:grid; grid-template-columns:repeat(4, minmax(0,1fr)); width:400px",
                SectionHeader { kind: HeaderKind::Frame, text: "Frame" }
                SectionHeader { kind: HeaderKind::Group, text: "Group", value: Some("12".to_string()) }
                SectionHeader { kind: HeaderKind::Field, text: "Field", value: Some("Light".to_string()) }
                SectionHeader { kind: HeaderKind::Menu, text: "Menu" }
            }
        }
    }
}

#[test]
fn each_section_header_is_a_full_width_row_of_its_own() {
    let harness = Harness::new(HeadersApp, VIEW);
    let mut bottom = None::<f32>;
    for n in 1..=4 {
        let header = rect(
            &harness,
            &format!(".probe-grid > .ds-section-header:nth-child({n})"),
        );
        assert!(
            (header.size.width.0 - 400.0).abs() <= 1.0,
            "header {n} is {} px wide, want the grid's 400",
            header.size.width.0
        );
        if let Some(above) = bottom {
            assert!(
                header.origin.y.0 >= above - 0.5,
                "header {n} starts at {} inside the one above (ends {above})",
                header.origin.y.0
            );
        }
        bottom = Some(header.origin.y.0 + header.size.height.0);
    }
}

// ---- 6. SidebarItem without a count starts at the left -------------------------------------

#[allow(non_snake_case)]
fn ItemsApp() -> Element {
    rsx! {
        Root {
            div { style: "width:220px",
                div { class: "probe-counted",
                    SidebarItem { kind: ItemKind::Place { icon: Icon::Inbox }, label: "Inbox", here: Here::Elsewhere, count: Some(12), presence: Presence::Present, preview: None, pulse: PulseKey::rest(ds::Anim::Gulp), onclick: |_| {}, onclose: None }
                }
                div { class: "probe-bare",
                    SidebarItem { kind: ItemKind::Place { icon: Icon::Star }, label: "Starred", here: Here::Elsewhere, count: None, presence: Presence::Present, preview: None, pulse: PulseKey::rest(ds::Anim::Gulp), onclick: |_| {}, onclose: None }
                }
            }
        }
    }
}

#[test]
fn a_sidebar_item_without_a_count_keeps_its_label_left() {
    let harness = Harness::new(ItemsApp, VIEW);
    let counted = rect(&harness, ".probe-counted .ds-sidebar-item-text");
    let bare = rect(&harness, ".probe-bare .ds-sidebar-item-text");
    let item = rect(&harness, ".probe-bare .ds-sidebar-item");
    // Padding 8, icon 16, gap 9: the label starts 33 px in, with or without a count.
    let start = bare.origin.x.0 - item.origin.x.0;
    assert!(
        (start - 33.0).abs() <= 1.0,
        "the label starts {start} px in, want 33"
    );
    assert!(
        (bare.origin.x.0 - counted.origin.x.0).abs() <= 0.5,
        "the labels start at {} and {}",
        bare.origin.x.0,
        counted.origin.x.0
    );
}

// ---- 8. SpaceEditor: the field paints and the presets are 22 px ---------------------------

#[allow(non_snake_case)]
fn EditorApp() -> Element {
    let look = SpaceLook {
        dots: PRESETS[0].dots.to_vec(),
        grain: Grain(35),
        theme: Theme::System,
        card_accent: ds::CardAccent::SpaceHue,
    };
    rsx! {
        Root {
            div { style: "width:460px",
                SpaceEditor { look, scheme: Scheme::Light, active_dot: DotIndex(0), onchange: |_| {} }
            }
        }
    }
}

#[test]
fn the_space_editor_field_paints_and_its_presets_are_22_px() {
    let view = Viewport {
        width: 480,
        height: 900,
        scale_percent: 100,
    };
    let mut harness = Harness::new(EditorApp, view);
    let frame = harness.render().expect("renders");
    keep(&frame, "space-editor");
    let field = rect(&harness, ".ds-field");
    assert!(
        field.size.width.0 > 400.0 && (field.size.height.0 - 176.0).abs() <= 1.0,
        "the field is {:?}",
        field.size
    );
    // The plane is a grid of coloured dots on its ground: a real picture, not the panel's own
    // surface. Blank, the field's inside was one colour.
    let inside = pixels(&frame, field, 4.0);
    let dots = ink(&inside, 12);
    assert!(
        dots > inside.len() / 20,
        "the field painted {dots} of {} pixels",
        inside.len()
    );
    let centre = frame
        .get_pixel(
            (field.origin.x.0 + field.size.width.0 / 2.0) as u32,
            (field.origin.y.0 + field.size.height.0 / 2.0) as u32,
        )
        .0;
    // Light `--paper` is #e9ece6 (design/03-COLOR.md): the field's centre is the plane, not the
    // ground showing through an empty box.
    const PAPER: [u8; 4] = [0xe9, 0xec, 0xe6, 0xff];
    assert!(
        distance(centre, PAPER) > 2,
        "the field's centre is the paper colour: {centre:?}"
    );
    // A handle sits on the field, inside it.
    let handle = rect(&harness, ".ds-handle");
    assert!(
        handle.origin.x.0 > field.origin.x.0 && handle.origin.y.0 >= field.origin.y.0 - 11.0,
        "the handle is at {:?}, the field at {:?}",
        handle.origin,
        field.origin
    );
    let preset = rect(&harness, ".ds-preset");
    assert!(
        (preset.size.width.0 - 22.0).abs() <= 0.5 && (preset.size.height.0 - 22.0).abs() <= 0.5,
        "a preset is {:?}",
        preset.size
    );
}
