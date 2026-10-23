use super::Icon;
use super::native::Native;
use super::render::{Glyph, GlyphProps, IconSize};
use super::style::GlyphStyle;
use crate::icon::shape::Shape;
use dioxus::prelude::*;

/// An icon's outline markup: the geometry these tests pin is Lucide's.
fn markup(icon: Icon) -> String {
    markup_in(icon, GlyphStyle::Outline)
}

fn markup_in(icon: Icon, style: GlyphStyle) -> String {
    let mut dom = VirtualDom::new_with_props(
        Glyph,
        GlyphProps {
            icon,
            size: IconSize::Base,
            style,
            cut: Default::default(),
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
        Shape::Path(_) | Shape::Solid(_) => "path",
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

/// A picture's rotate buttons are a pair: `RotateLeft` is `rotate-ccw`, `RotateRight` is its
/// mirror (`rotate-cw`), and each has a solid form, which is a mirror too.
#[test]
fn the_rotate_pair_is_mirrored_in_both_styles() {
    assert_eq!(Icon::RotateLeft.shapes(), Icon::Restart.shapes());
    assert_eq!(
        Icon::RotateRight.shapes(),
        &[
            Shape::Path("M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"),
            Shape::Path("M21 3v5h-5"),
        ][..]
    );
    assert_eq!(
        Icon::RotateLeft.solid_shapes(),
        Icon::Restart.solid_shapes()
    );
    assert_eq!(
        Icon::RotateRight.solid_shapes().len(),
        Icon::RotateLeft.solid_shapes().len()
    );
    assert_ne!(
        Icon::RotateRight.solid_shapes(),
        Icon::RotateLeft.solid_shapes()
    );
}

/// The control center's own `Switches`: two 20 x 8 pill tracks 4 px apart
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
    // The shell set ends with quire's own glyphs: Switches, then the filled moon (design/26).
    assert_eq!(
        &Icon::SHELL[Icon::SHELL.len() - 2..],
        &[Icon::Switches, Icon::MoonFilled][..]
    );
}

#[test]
fn every_icon_draws_its_native_pick_by_default() {
    let mut failures = Vec::new();
    for &icon in Icon::ALL {
        let page = markup_in(icon, GlyphStyle::default());
        let (shapes, style) = match icon.native() {
            Native::Solid => (icon.solid_shapes(), "solid"),
            Native::Drawn(shapes) => (shapes, "solid"),
            Native::Outline | Native::Pair => (icon.shapes(), "outline"),
        };
        let expect: Vec<&str> = std::iter::once("svg")
            .chain(shapes.iter().map(shape_tag))
            .collect();
        if element_names(&page) != expect || !page.contains(&format!("data-style=\"{style}\"")) {
            failures.push(format!("{icon:?}: {page}"));
        }
        let filled = style == "solid";
        if filled != (page.contains("stroke=\"none\"") && !page.contains("stroke-width")) {
            failures.push(format!("{icon:?} paint: {page}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn an_explicit_style_beats_the_pick() {
    let page = markup_in(Icon::Bluetooth, GlyphStyle::Solid);
    assert!(page.contains("data-style=\"solid\""), "{page}");
    let page = markup_in(Icon::Play, GlyphStyle::Outline);
    assert!(page.contains("data-style=\"outline\"") && page.contains("stroke-width"));
}

#[test]
fn a_replacement_is_filled_paths_on_the_24_grid() {
    for icon in [Icon::Download, Icon::Upload, Icon::Sparkles] {
        let Native::Drawn(shapes) = icon.native() else {
            panic!("{icon:?}: not drawn");
        };
        for shape in shapes {
            let Shape::Solid(d) = shape else {
                panic!("{icon:?}: {shape:?}");
            };
            for number in d.split(|c: char| c.is_ascii_alphabetic() || c.is_whitespace()) {
                if let Ok(value) = number.parse::<f32>() {
                    assert!(value.abs() <= 24.0, "{icon:?}: {value} in {d}");
                }
            }
        }
    }
}

#[test]
fn an_outline_is_the_stroke_and_names_its_style() {
    let page = markup(Icon::Star);
    assert!(page.contains("data-style=\"outline\""), "{page}");
    assert!(page.contains("stroke-linecap=\"round\""), "{page}");
}

/// Glyphs that sit side by side in a folder listing share Lucide `file`'s page outline, so the
/// rows line up; the audio, archive and app glyphs cut the page where their badge overlaps it.
#[test]
fn the_page_glyphs_share_the_file_outline() {
    const PAGES: &[Icon] = &[
        Icon::FileText,
        Icon::FileImage,
        Icon::FileVideo,
        Icon::FileCode,
        Icon::FilePdf,
    ];
    let page = Icon::File.shapes().first();
    assert!(page.is_some());
    for icon in PAGES {
        assert_eq!(icon.shapes().first(), page, "{icon:?}");
        assert_eq!(icon.native(), Native::Outline, "{icon:?}");
    }
}

/// The file manager's places and drives are in the actions set, which the gallery draws.
#[test]
fn the_file_manager_glyphs_are_in_the_actions_set() {
    const GLYPHS: &[Icon] = &[
        Icon::Home,
        Icon::Eject,
        Icon::DriveInternal,
        Icon::DriveExternal,
        Icon::Disc,
        Icon::FileApp,
        Icon::FileArchive,
        Icon::FileAudio,
        Icon::Puzzle,
        Icon::Cloud,
        Icon::CloudDownload,
        Icon::CloudUpload,
    ];
    for icon in GLYPHS {
        assert!(Icon::ACTIONS.contains(icon), "{icon:?}");
        assert!(Icon::ALL.contains(icon), "{icon:?}");
    }
    // The eject mark is a triangle over a bar: one path, then the bar.
    assert!(matches!(
        Icon::Eject.shapes(),
        [Shape::Path(_), Shape::Rect { .. }]
    ));
}

/// The markup and document-view glyphs are in the actions set, drawn outline by default, and
/// carry both forms.
#[test]
fn the_markup_glyphs_are_outline_actions_with_both_forms() {
    const GLYPHS: &[Icon] = &[
        Icon::Hand,
        Icon::Crop,
        Icon::MousePointer,
        Icon::Highlighter,
        Icon::Type,
        Icon::Circle,
        Icon::Line,
        Icon::ArrowUpRight,
        Icon::PenLine,
        Icon::ListTree,
        Icon::Table,
    ];
    for icon in GLYPHS {
        assert!(Icon::ACTIONS.contains(icon), "{icon:?}");
        assert!(Icon::ALL.contains(icon), "{icon:?}");
        assert_eq!(icon.native(), Native::Outline, "{icon:?}");
        assert!(!icon.shapes().is_empty(), "{icon:?}");
        assert!(!icon.solid_shapes().is_empty(), "{icon:?}");
    }
    assert_eq!(Icon::Line.shapes(), &[Shape::Path("M22 2 2 22")]);
    assert!(matches!(
        Icon::Table.shapes(),
        [
            Shape::Path(_),
            Shape::Rect { .. },
            Shape::Path(_),
            Shape::Path(_)
        ]
    ));
}
