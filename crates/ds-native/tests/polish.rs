//! The polish pass (FINDINGS "Polish pass"), proved on a real Blitz document: the reset's
//! `:where(.ds)` scope loses to a lone class, the Space editor's dots stay round at any width,
//! a section header's action ends its row, Quiet spaces its glyph from its label, and Danger
//! turns red on hover where Mini does not.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::{
    Appearance, Button, ButtonVariant, DotIndex, Ds, Grain, HeaderKind, Icon, Material, PRESETS,
    Point, Px, Rect, Scheme, SectionHeader, SpaceEditor, SpaceLook, Theme,
};
use ds_native::{Harness, Viewport};
use image::RgbaImage;
use probe::{distance, keep, modal, pixels, rect};
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

/// How many of `seen` are near `want`: every channel within 60.
fn near(seen: &[[u8; 4]], want: [u8; 3]) -> usize {
    seen.iter()
        .filter(|pixel| (0..3).all(|i| pixel[i].abs_diff(want[i]) <= 60))
        .count()
}

// ---- The reset's element rules lose to a lone class ----------------------------------------

/// A probe rule with one class, as a component sheet writes one. Its colour and size differ
/// from the column's, so each shows which rule won.
const LONE_CLASS: &str = ".ds-where-probe{color:rgb(230,20,20);font-size:24px}";

#[allow(non_snake_case)]
fn WhereApp() -> Element {
    rsx! {
        Root {
            style { {LONE_CLASS} }
            div { style: "color:rgb(20,20,230); font-size:30px; line-height:1.2",
                button { class: "probe-plain", style: "border:0; background:none; padding:0", "Plain" }
                button { class: "ds-where-probe", style: "border:0; background:none; padding:0", "Class" }
            }
        }
    }
}

#[test]
fn the_resets_where_scope_applies_and_a_lone_class_outranks_it() {
    let mut harness = Harness::new(WhereApp, VIEW);
    let frame = harness.render().expect("renders");
    keep(&frame, "where-probe");
    const BLUE: [u8; 3] = [20, 20, 230];
    const RED: [u8; 3] = [230, 20, 20];
    // The reset applies through `:where(.ds) button`: the plain button inherits the column's
    // blue and 30 px (a user-agent button is black at about 13 px).
    let plain = rect(&harness, ".probe-plain");
    let plain_ink = pixels(&frame, plain, 0.0);
    assert!(
        plain.size.height.0 >= 30.0,
        "the plain button is {} px tall: it did not inherit the 30 px font",
        plain.size.height.0
    );
    assert!(near(&plain_ink, BLUE) > 40, "the plain label is not blue");
    // The lone class outranks the reset: red and 24 px, none of the column's blue.
    let class = rect(&harness, ".ds-where-probe");
    let class_ink = pixels(&frame, class, 0.0);
    assert!(
        near(&class_ink, RED) > 40,
        "the class's red lost to the reset"
    );
    assert_eq!(near(&class_ink, BLUE), 0, "the reset's inherited blue won");
    assert!(
        class.size.height.0 < plain.size.height.0 - 4.0,
        "the class's 24 px lost to the reset's inherited 30: {} vs {}",
        class.size.height.0,
        plain.size.height.0
    );
}

// ---- The Space editor's dots stay round at any width ---------------------------------------

fn look() -> SpaceLook {
    SpaceLook {
        dots: PRESETS[0].dots.to_vec(),
        grain: Grain(35),
        theme: Theme::System,
        card_accent: ds::CardAccent::SpaceHue,
    }
}

#[allow(non_snake_case)]
fn NarrowEditor() -> Element {
    rsx! {
        Root {
            div { style: "width:280px",
                SpaceEditor { look: look(), scheme: Scheme::Light, active_dot: DotIndex(0), onchange: |_| {} }
            }
        }
    }
}

#[allow(non_snake_case)]
fn WideEditor() -> Element {
    rsx! {
        Root {
            div { style: "width:560px",
                SpaceEditor { look: look(), scheme: Scheme::Light, active_dot: DotIndex(0), onchange: |_| {} }
            }
        }
    }
}

/// The plane's light ground (`S:1399`).
const GROUND: [u8; 4] = [0xf3, 0xf4, 0xf1, 0xff];

/// How many pixels from `(x, y)` stepping by `(dx, dy)` stand out from the ground, stopping
/// at the first that does not.
fn run(frame: &RgbaImage, (x, y): (i64, i64), (dx, dy): (i64, i64), by: u8) -> i64 {
    (1..)
        .map(|n| (x + dx * n, y + dy * n))
        .take_while(|&(px, py)| {
            px >= 0 && py >= 0 && distance(frame.get_pixel(px as u32, py as u32).0, GROUND) > by
        })
        .count() as i64
}

/// A dot's width and height in device pixels, measured through its centre: the dot in column
/// `column` and row `row` of the grid, which starts 4.5 px inside the field's border and steps
/// every 9 px (`space_editor.css`), at a scale of `scale`.
fn dot_size(frame: &RgbaImage, field: Rect, column: u32, row: u32, scale: f32) -> (i64, i64) {
    let at = |origin: f32, index: u32| ((origin + 1.0 + 4.5 + 9.0 * index as f32) * scale) as i64;
    let centre = (at(field.origin.x.0, column), at(field.origin.y.0, row));
    let ink = distance(frame.get_pixel(centre.0 as u32, centre.1 as u32).0, GROUND);
    assert!(ink > 20, "no dot at {centre:?}: the centre is the ground");
    // Half the centre's contrast: the anti-aliased rim counts where it is mostly dot.
    let by = ink / 2;
    let width = 1 + run(frame, centre, (1, 0), by) + run(frame, centre, (-1, 0), by);
    let height = 1 + run(frame, centre, (0, 1), by) + run(frame, centre, (0, -1), by);
    (width, height)
}

fn dots_are_round(app: fn() -> Element, name: &str) {
    let view = Viewport {
        width: 600,
        height: 900,
        scale_percent: 200,
    };
    let mut harness = Harness::new(app, view);
    let frame = harness.render().expect("renders");
    keep(&frame, name);
    let field = rect(&harness, ".ds-field");
    // Three dots away from the handles: the first column, a middle one, and near the end.
    let last = ((field.size.width.0 - 2.0 - 4.5) / 9.0) as u32 - 1;
    for (column, row) in [(0, 2), (last / 2, 15), (last, 8)] {
        let (width, height) = dot_size(&frame, field, column, row, 2.0);
        assert!(
            (width - height).abs() <= 1,
            "{name}: the dot at column {column}, row {row} is {width} x {height} device px"
        );
        // Radius 5.2 of an 18 px cell drawn at 9 px: about 5.2 px across, 10.4 at 2x.
        assert!(
            (8..=13).contains(&width),
            "{name}: the dot is {width} device px wide"
        );
    }
}

#[test]
fn the_space_editor_dots_stay_round_in_a_narrow_and_a_wide_panel() {
    dots_are_round(NarrowEditor, "dots-narrow");
    dots_are_round(WideEditor, "dots-wide");
}

// ---- A section header's action ends its row ------------------------------------------------

#[allow(non_snake_case)]
fn ActionHeadersApp() -> Element {
    let action = || Some(("Clear".to_string(), EventHandler::new(|()| {})));
    rsx! {
        Root {
            div { class: "probe-headers", style: "width:400px",
                SectionHeader { kind: HeaderKind::Frame, text: "Frame", action: action() }
                SectionHeader { kind: HeaderKind::Group, text: "Group", value: Some("12".to_string()), action: action() }
                SectionHeader { kind: HeaderKind::Field, text: "Field", value: Some("Light".to_string()), action: action() }
                SectionHeader { kind: HeaderKind::Menu, text: "Menu", action: action() }
            }
        }
    }
}

#[test]
fn a_section_headers_action_sits_at_its_right_edge_in_every_kind() {
    let harness = Harness::new(ActionHeadersApp, VIEW);
    // The inline end padding of each kind (design/04-COMPONENTS.md section 13): Frame 14px 6px
    // 5px, Group 12px 6px 6px, Field none, Menu 7px 8px 3px.
    for (n, kind, padding) in [
        (1, "frame", 6.0),
        (2, "group", 6.0),
        (3, "field", 0.0),
        (4, "menu", 8.0),
    ] {
        let header = rect(&harness, &format!(".probe-headers > :nth-child({n})"));
        let action = rect(
            &harness,
            &format!(".probe-headers > :nth-child({n}) .ds-section-header-action"),
        );
        let edge = header.origin.x.0 + header.size.width.0 - padding;
        let right = action.origin.x.0 + action.size.width.0;
        assert!(
            (right - edge).abs() <= 1.0,
            "{kind}: the action ends at {right}, the header's content at {edge}"
        );
    }
}

// ---- Quiet spaces its glyph; Danger is Mini at rest and red on hover -----------------------

#[allow(non_snake_case)]
fn ButtonsApp() -> Element {
    rsx! {
        Root {
            div { style: "display:flex; flex-direction:column; align-items:flex-start; gap:20px; padding:20px",
                div { class: "probe-quiet",
                    Button { variant: ButtonVariant::Quiet, label: "With icon", icon: Some(Icon::Star), onclick: |_| {} }
                }
                div { class: "probe-mini",
                    Button { variant: ButtonVariant::Mini, label: "Delete", onclick: |_| {} }
                }
                div { class: "probe-danger",
                    Button { variant: ButtonVariant::Danger, label: "Delete", onclick: |_| {} }
                }
            }
        }
    }
}

#[test]
fn a_quiet_buttons_glyph_is_six_px_from_its_label() {
    let harness = Harness::new(ButtonsApp, VIEW);
    let glyph = rect(&harness, ".probe-quiet .ds-ic");
    let label = rect(&harness, ".probe-quiet .ds-button > span");
    let gap = label.origin.x.0 - (glyph.origin.x.0 + glyph.size.width.0);
    assert!(
        (gap - 6.0).abs() <= 0.5,
        "the glyph is {gap} px from the label"
    );
    // One row: the glyph is centred on the label, not above it.
    let middle = |r: Rect| r.origin.y.0 + r.size.height.0 / 2.0;
    assert!((middle(glyph) - middle(label)).abs() <= 1.5);
}

/// The fill of the button in `probe`, hovered or not: the modal colour inside its border.
fn fill(harness: &mut Harness, probe: &str, hover: Hover) -> [u8; 4] {
    let selector = format!("{probe} .ds-button");
    let point = match hover {
        Hover::On => harness.centre(&selector).expect("on screen"),
        Hover::Off => Point {
            x: Px(470.0),
            y: Px(350.0),
        },
    };
    harness.pointer_move(point);
    // The background transition is `--t-quick`.
    harness.advance(Duration::from_millis(400));
    let frame = harness.render().expect("renders");
    modal(&pixels(&frame, rect(harness, &selector), 2.0))
}

#[derive(Clone, Copy)]
enum Hover {
    On,
    Off,
}

#[test]
fn danger_is_mini_at_rest_and_red_only_on_hover() {
    let mut harness = Harness::new(ButtonsApp, VIEW);
    // design/04-COMPONENTS.md section 1: "Danger (derived): at rest as Mini; red only on hover".
    let mini = fill(&mut harness, ".probe-mini", Hover::Off);
    let danger = fill(&mut harness, ".probe-danger", Hover::Off);
    assert_eq!(mini, danger, "at rest Danger is Mini");
    let mini_hover = fill(&mut harness, ".probe-mini", Hover::On);
    let danger_hover = fill(&mut harness, ".probe-danger", Hover::On);
    assert!(
        distance(mini_hover, danger_hover) > 8,
        "hovered, Danger {danger_hover:?} looks like Mini {mini_hover:?}"
    );
    // `--danger-wash` leans red.
    assert!(
        danger_hover[0] > danger_hover[1] && danger_hover[0] > danger_hover[2],
        "the hovered Danger fill {danger_hover:?} is not red"
    );
}
