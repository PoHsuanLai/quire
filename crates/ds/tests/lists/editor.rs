//! The Space editor: its field plane decoded and probed, its handles placed from the dots, and
//! its contrast pills compared with `space::readout`.

use super::*;

#[derive(Props, Clone, PartialEq)]
struct Editing {
    look: SpaceLook,
    scheme: Scheme,
    active: u8,
}

fn editing(props: Editing) -> Element {
    editor(props.look, props.scheme, props.active)
}

/// The editor for `look` in `scheme`, rendered.
fn render_editor(look: SpaceLook, scheme: Scheme, active: u8) -> String {
    let mut dom = VirtualDom::new_with_props(
        editing,
        Editing {
            look,
            scheme,
            active,
        },
    );
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// The field's images for `scheme`, decoded from the editor's own markup, in order: the
/// colour plane, then the dot tile.
fn plane(scheme: Scheme) -> (png::Image, png::Image) {
    let html = render_editor(preset_look(0, Grain(35)), scheme, 0);
    let mut images = html.match_indices(PLANE).map(|(at, _)| {
        let payload: String = html[at + PLANE.len()..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '+' || *c == '/' || *c == '=')
            .collect();
        png::decode(&png::unbase64(&payload))
    });
    let colours = images
        .next()
        .unwrap_or_else(|| panic!("no plane in {html}"));
    let dots = images
        .next()
        .unwrap_or_else(|| panic!("no dot tile in {html}"));
    (colours, dots)
}

#[test]
fn the_field_colours_are_the_prototypes_hue_by_chroma_plane() {
    // `S:1399-1405`: hue across and chroma down, one sample per 18 px cell of S's 540 x 352.
    for scheme in Scheme::ALL {
        let (image, _) = plane(scheme);
        assert_eq!((image.width, image.height), (60, 39), "{scheme:?}");
        assert_eq!((image.depth, image.colour_type), (8, 2), "8-bit RGB");
        for (x, y) in [(0, 0), (30, 19), (59, 38)] {
            let dot = Dot {
                hue: ((x as f64 + 0.5) / 60.0 * 360.0) as f32,
                chroma: (1.0 - (y as f64 + 0.5) / 39.0) as f32,
            };
            assert_eq!(
                image.hex(x, y),
                swatch(dot, scheme),
                "{scheme:?} at {x},{y}"
            );
        }
    }
    assert_ne!(
        plane(Scheme::Light).0.hex(30, 19),
        plane(Scheme::Dark).0.hex(30, 19)
    );
}

#[test]
fn the_dot_tile_is_the_ground_with_a_round_hole() {
    // One 18 px cell: the ground, opaque, with the dot of radius 5.2 at its centre (`S:1404`).
    for scheme in Scheme::ALL {
        let (_, tile) = plane(scheme);
        assert_eq!((tile.width, tile.height), (18, 18), "{scheme:?}");
        assert_eq!((tile.depth, tile.colour_type), (8, 6), "8-bit RGBA");
        let ground = match scheme {
            Scheme::Light => "#f3f4f1",
            Scheme::Dark => "#1b1d1a",
        };
        for (x, y) in [(0, 0), (17, 17), (0, 9), (9, 0)] {
            assert_eq!(tile.hex(x, y), ground, "{scheme:?} ground at {x},{y}");
            assert_eq!(tile.alpha(x, y), 255, "{scheme:?} opaque at {x},{y}");
        }
        for (x, y) in [(9, 9), (8, 8), (11, 11)] {
            assert_eq!(tile.alpha(x, y), 0, "{scheme:?} hole at {x},{y}");
        }
        // The rim is partial, and the hole is round: as wide as it is tall.
        let rim = tile.alpha(14, 9);
        assert!(rim > 0 && rim < 255, "{scheme:?} rim alpha {rim}");
        assert_eq!(tile.alpha(14, 9), tile.alpha(9, 14), "{scheme:?} round");
    }
}

/// The pills an editor renders, in order: `(value text, data-status, pill text)`.
fn pills(html: &str) -> Vec<(String, String, String)> {
    html.split("class=\"ds-check\"")
        .skip(1)
        .map(|check| {
            let value = check
                .split("class=\"ds-check-value\">")
                .nth(1)
                .and_then(|rest| rest.split('<').next())
                .unwrap_or_default()
                .to_owned();
            let status = check
                .split("data-status=\"")
                .nth(1)
                .and_then(|rest| rest.split('"').next())
                .unwrap_or_default()
                .to_owned();
            let text = check
                .split("data-status=\"")
                .nth(1)
                .and_then(|rest| rest.split('>').nth(1))
                .and_then(|rest| rest.split('<').next())
                .unwrap_or_default()
                .to_owned();
            (value, status, text)
        })
        .collect()
}

#[test]
fn the_contrast_pills_are_the_readout() {
    const CASES: &[(&str, usize, u8, Scheme, u8)] = &[
        ("work light", 0, 35, Scheme::Light, 0),
        ("home light", 1, 55, Scheme::Light, 2),
        ("work dark", 0, 35, Scheme::Dark, 1),
    ];
    for &(name, index, grain, scheme, active) in CASES {
        let look = preset_look(index, Grain(grain));
        let got = pills(&render_editor(look.clone(), scheme, active));
        let want: Vec<(String, String, String)> = readout(&look, scheme)
            .iter()
            .map(|check| {
                let (status, sign) = match check.verdict() {
                    Verdict::Pass => ("ok", "≥"),
                    Verdict::Fail => ("bad", "&lt;"),
                };
                (
                    format!("{:.2}", check.measured),
                    status.to_owned(),
                    format!("{sign} {}", check.need),
                )
            })
            .collect();
        assert_eq!(got.len(), 4, "{name}: four checks");
        assert_eq!(got, want, "{name}");
    }
}

/// The note is the derivation's `capped`. No dot the field can place caps a stop (a sweep of
/// every 10 degrees at chroma up to 8, both schemes, never does), so only the uncapped text is
/// reachable from a real look.
#[test]
fn the_cap_note_follows_the_derivation() {
    for (index, scheme) in [(0, Scheme::Light), (1, Scheme::Dark)] {
        let look = preset_look(index, Grain(35));
        assert_eq!(derive(&look.dots, scheme).capped, Capping::Uncapped);
    }
    let uncapped = render(|| editor(preset_look(0, Grain(35)), Scheme::Light, 0));
    assert!(uncapped.contains("No capping needed"), "{uncapped}");
}

#[test]
fn a_handle_sits_where_its_dot_is() {
    // Work's first dot {268, .72}: left 268 / 360, top 1 - .72.
    let html = render(|| editor(preset_look(0, Grain(35)), Scheme::Light, 0));
    assert!(html.contains("left:74.44%;top:28.00%"), "{html}");
    let palette = derive(&preset_look(0, Grain(35)).dots, Scheme::Light);
    for picked in &palette.picked {
        assert!(
            html.contains(&format!("background:{picked}")),
            "no handle in {picked}"
        );
    }
    assert_eq!(html.matches("class=\"ds-handle\"").count(), 2);
    assert_eq!(
        html.matches("aria-pressed=\"true\" style=\"left").count(),
        1,
        "one active handle"
    );
}
