use super::render::{Glyph, GlyphProps, IconSize};
use super::{Icon, Shape};
use dioxus::prelude::*;

fn markup(icon: Icon) -> String {
    let mut dom = VirtualDom::new_with_props(
        Glyph,
        GlyphProps {
            icon,
            size: IconSize::Base,
        },
    );
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// Opening-tag names, in order. A close tag is not an element.
fn element_names(html: &str) -> Vec<&str> {
    let bytes = html.as_bytes();
    let mut names = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'<' {
            index += 1;
            continue;
        }
        index += 1;
        if index >= bytes.len() || matches!(bytes[index], b'/' | b'!' | b'?') {
            continue;
        }
        let start = index;
        while index < bytes.len()
            && !matches!(bytes[index], b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/')
        {
            index += 1;
        }
        names.push(&html[start..index]);
    }
    names
}

fn shape_tag(shape: &Shape) -> &'static str {
    match shape {
        Shape::Path(_) => "path",
        Shape::Circle { .. } => "circle",
        Shape::Rect { .. } => "rect",
    }
}

#[test]
fn every_icon_is_an_svg_of_its_shapes() {
    let mut failures = Vec::new();
    for icon in Icon::MAILO.iter().chain(Icon::ACTIONS).chain(Icon::CONTROL) {
        let page = markup(*icon);
        let shapes = icon.shapes();
        let names = element_names(&page);
        let expect: Vec<&str> = std::iter::once("svg")
            .chain(shapes.iter().map(shape_tag))
            .collect();
        if shapes.is_empty()
            || names != expect
            || !page.contains("<svg")
            || !page.contains("class=\"ds-ic\"")
            || !page.contains("stroke=\"currentColor\"")
            || !page.contains("stroke-width=\"2\"")
            || !page.contains("fill=\"none\"")
            || !page.contains("viewBox=\"0 0 24 24\"")
            || !page.contains("aria-hidden=\"true\"")
        {
            failures.push(format!(
                "{icon:?}: {} shapes, names {names:?}, page {page}",
                shapes.len()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_set_matches_icons_js() {
    // The first 23 are the keys of `ICON` in mailo-design/icons.js (inbox through key).
    // The rest are the glyphs the frame's places, rows and foot draw, fetched from Lucide.
    // Plus is the Spaces mockup's `ICON.plus`: the foot's "New Space".
    const KEYS_IN_ICONS_JS: usize = 23;
    const FRAME_GLYPHS: usize = 10;
    assert_eq!(
        Icon::MAILO.len(),
        KEYS_IN_ICONS_JS + FRAME_GLYPHS,
        "{:?}",
        Icon::MAILO
    );
    assert_eq!(
        Icon::ALL.len(),
        Icon::MAILO.len() + Icon::SHELL.len() + Icon::ACTIONS.len() + Icon::CONTROL.len(),
        "ALL is the two sets, the actions and the control set"
    );
    let mut duplicates = Vec::new();
    for (index, icon) in Icon::ALL.iter().enumerate() {
        if Icon::ALL[index + 1..].contains(icon) {
            duplicates.push(format!("{icon:?}"));
        }
    }
    assert!(
        duplicates.is_empty(),
        "duplicates: {}",
        duplicates.join(", ")
    );
}

/// First path `d` of star, check and undo, copied from `icons.js`.
const FIRST_PATH: &[(Icon, &str)] = &[
    (
        Icon::Star,
        "M12 2.5l2.9 5.88 6.49.94-4.7 4.58 1.11 6.46L12 17.31l-5.8 3.05 1.1-6.46-4.69-4.58 6.48-.94z",
    ),
    (Icon::Check, "M20 6 9 17l-5-5"),
    (Icon::Undo, "M9 14 4 9l5-5"),
];

#[test]
fn star_check_and_undo_keep_their_first_path() {
    let mut failures = Vec::new();
    for (icon, expect) in FIRST_PATH {
        let got = match icon.shapes().first() {
            Some(Shape::Path(d)) => Some(*d),
            _ => None,
        };
        if got != Some(*expect) {
            failures.push(format!("{icon:?}: first path is {got:?}"));
        }
        let page = markup(*icon);
        let needle = format!("d=\"{expect}\"");
        if !page.contains(&needle) {
            failures.push(format!("{icon:?} rendered without {needle}: {page}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Printer and folder-input as Lucide 1.47.0 publishes them: two paths and a rect, three paths.
#[test]
fn printer_and_folder_input_are_lucides() {
    assert_eq!(
        Icon::Printer.shapes().last(),
        Some(&Shape::Rect {
            x: "6",
            y: "14",
            width: "12",
            height: "8",
            rx: "1"
        })
    );
    assert_eq!(Icon::Printer.shapes().len(), 3);
    assert_eq!(
        Icon::FolderInput.shapes().get(1..),
        Some(&[Shape::Path("M2 13h10"), Shape::Path("m9 16 3-3-3-3")][..])
    );
}

/// The control set as Lucide 1.47.0 publishes it (sill FINDINGS Q81): eleven glyphs, Restart
/// is `rotate-ccw`, Phone is `smartphone`, and `Power` stays in the shell set.
#[test]
fn the_control_set_is_lucides() {
    assert_eq!(Icon::CONTROL.len(), 11);
    assert!(Icon::SHELL.contains(&Icon::Power), "Power is not repeated");
    assert_eq!(
        Icon::Restart.shapes(),
        &[
            Shape::Path("M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"),
            Shape::Path("M3 3v5h5"),
        ][..]
    );
    assert_eq!(
        Icon::Phone.shapes().last(),
        Some(&Shape::Path("M12 18h.01"))
    );
    assert_eq!(Icon::Gamepad.shapes().len(), 5);
}

/// The control center's own `Switches` (sill FINDINGS Q103): two 20 x 8 pill tracks 4 px apart
/// with a dot knob each at opposite ends, inside Lucide's reach, in the shell set.
#[test]
fn switches_is_two_tracks_with_knobs_at_opposite_ends() {
    let shapes = Icon::Switches.shapes();
    let tracks: Vec<&Shape> = shapes
        .iter()
        .filter(|shape| matches!(shape, Shape::Rect { .. }))
        .collect();
    let knobs: Vec<(&str, &str)> = shapes
        .iter()
        .filter_map(|shape| match shape {
            Shape::Circle { cx, cy, .. } => Some((*cx, *cy)),
            _ => None,
        })
        .collect();
    assert_eq!(tracks.len(), 2, "{shapes:?}");
    assert_eq!(
        knobs,
        [("6", "6"), ("18", "18")],
        "left on top, right below"
    );
    assert!(Icon::SHELL.contains(&Icon::Switches));
    assert_eq!(Icon::SHELL.last(), Some(&Icon::Switches));
}
