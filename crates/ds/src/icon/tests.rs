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
    for icon in Icon::MAILO.iter().chain(Icon::ACTIONS) {
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
        Icon::MAILO.len() + Icon::SHELL.len() + Icon::ACTIONS.len(),
        "ALL is the two sets and the actions"
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

/// The "more" glyphs as Lucide 1.47.0 publishes them (mailo gaps 6): three unit circles on the
/// middle row for `ellipsis`, on the middle column for `ellipsis-vertical`, centre first.
#[test]
fn the_ellipses_are_lucides() {
    let dot = |cx, cy| Shape::Circle { cx, cy, r: "1" };
    assert_eq!(
        Icon::Ellipsis.shapes(),
        &[dot("12", "12"), dot("19", "12"), dot("5", "12")][..]
    );
    assert_eq!(
        Icon::EllipsisVertical.shapes(),
        &[dot("12", "12"), dot("12", "5"), dot("12", "19")][..]
    );
    for icon in [Icon::Ellipsis, Icon::EllipsisVertical] {
        assert!(
            Icon::ACTIONS.contains(&icon) && Icon::ALL.contains(&icon),
            "{icon:?}"
        );
    }
}
