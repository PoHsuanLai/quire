//! Pixel snapping at fractional scales, measured in the picture (FINDINGS "Pixel snapping"): at
//! 1.25, 1.5 and 1.75 a hairline, a card's 1 px border, a menu's separator and a 16 px glyph's
//! stroke each cover whole device rows at full ink, with the ground on both sides; at 2 the same
//! lines are two rows. Every probe sits at an odd logical coordinate, where an unsnapped line
//! at 1.5 starts half-way through a device pixel.
//!
//! Before `ds_native::snap` and the pixel tokens, the same fixture measured a 1 px line at 1.5
//! as one full row and one half row (255 then 128 of ink), at 1.25 as 128 then 191, and at 1.75
//! as 128, 255, 64: Blitz rounds boxes to whole logical pixels, never device ones.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::icon::stroke::stroke_device_pixels;
use ds::{Appearance, Ds, Glyph, Icon, IconSize, Material, MenuEntry, Point, Px, Scale, Trail};
use ds_native::{Harness, Viewport};
use image::RgbaImage;
use probe::{keep, rect};
use std::ops::Range;
use std::time::Duration;

/// Test-only paint: black lines on a white ground, at odd logical coordinates. `.ds` is
/// re-declared (doubled, to outrank the sheet's own `.ds`) only to ink the menu's separator black; a consumer never does this.
const FIXTURE_CSS: &str = ".ds.ds{--line-soft:#000000}\
    .ground{position:absolute;left:0;top:0;width:480px;height:240px;background:#ffffff}\
    .rule{position:absolute;left:11px;top:11px;width:60px;height:var(--hair);background:#000000}\
    .card{position:absolute;left:21px;top:31px;width:61px;height:31px;box-sizing:border-box;\
      border:var(--hair) solid #000000}\
    .raw{position:absolute;left:101px;top:31px;width:61px;height:31px;box-sizing:border-box;\
      border:1px solid #000000}\
    .icon{position:absolute;left:41px;top:81px;display:flex;color:#000000}";

#[allow(non_snake_case)]
fn Fixture() -> Element {
    let item = |value: i32, title: &str| MenuEntry::Item {
        value,
        title: title.to_owned(),
        detail: None,
        tile: None,
        trail: Trail::None,
        check: None,
        availability: ds::Availability::Enabled,
    };
    let entries = vec![
        item(1, "New Window"),
        MenuEntry::Separator,
        item(2, "Settings"),
        MenuEntry::Separator,
        item(3, "Quit"),
    ];
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            style { {FIXTURE_CSS} }
            div { class: "ground",
                div { class: "rule" }
                div { class: "card" }
                div { class: "raw" }
                span { class: "icon", Glyph { icon: Icon::Plus, size: IconSize::Base } }
            }
            ds::Menu::<i32> {
                kind: ds::MenuKind::Slim,
                anchor: ds::Anchor::Point(Point { x: Px(191.0), y: Px(81.0) }),
                entries,
                onpick: move |_| {},
                onclose: move |_| {},
            }
        }
    }
}

/// The scales measured: three fractional, one whole.
const SCALES: [u16; 4] = [125, 150, 175, 200];

/// Ink: how far a pixel is from white, 0 (white) to 255 (black).
fn ink(frame: &RgbaImage, x: u32, y: u32) -> u8 {
    let [r, g, b, _] = frame.get_pixel(x, y).0;
    255 - r.min(g).min(b)
}

/// Which way a probe walks across a line.
#[derive(Debug, Clone, Copy)]
enum Across {
    /// Down a column at `x`: a horizontal line.
    Rows { x: f32 },
    /// Along a row at `y`: a vertical line.
    Columns { y: f32 },
}

/// A line to find: where to walk (logical), and what it is called in a failure.
struct Line {
    name: &'static str,
    across: Across,
    span: Range<f32>,
}

/// Anything above this is part of the line; the ground on either side must stay below it.
const FAINT: u8 = 48;
/// A device row at full ink.
const FULL: u8 = 240;

/// The inked device pixels along `line`, each as (device coordinate, ink).
fn inked(frame: &RgbaImage, line: &Line, scale: f32) -> Vec<(u32, u8)> {
    let device = |logical: f32| (logical * scale).floor() as u32;
    (device(line.span.start)..device(line.span.end))
        .map(|at| match line.across {
            Across::Rows { x } => (at, ink(frame, device(x), at)),
            Across::Columns { y } => (at, ink(frame, at, device(y))),
        })
        .filter(|&(_, value)| value > FAINT)
        .collect()
}

/// `line` covers exactly `rows` contiguous device pixels, each at full ink.
fn assert_crisp(frame: &RgbaImage, line: &Line, scale: u16, rows: usize) {
    let found = inked(frame, line, f32::from(scale) / 100.0);
    let contiguous = found.windows(2).all(|pair| pair[1].0 == pair[0].0 + 1);
    let full = found.iter().all(|&(_, value)| value >= FULL);
    assert!(
        found.len() == rows && contiguous && full,
        "{} at {scale}%: want {rows} full rows, found {found:?}",
        line.name
    );
}

fn render(scale: u16) -> (Harness, RgbaImage) {
    let viewport = Viewport {
        width: 480,
        height: 240,
        scale_percent: scale,
    };
    let mut harness = Harness::new(Fixture, viewport);
    harness.advance(Duration::from_millis(400));
    let frame = harness.render().expect("a frame");
    keep(&frame, &format!("pixel-snap-{scale}"));
    (harness, frame)
}

/// The fixed lines: the hairline rule, the token card's top and left edges, and the literal
/// `1px` card's (Blitz's style engine floors a border width to device pixels; the snap keeps it).
fn fixed_lines() -> [Line; 5] {
    [
        Line {
            name: "rule",
            across: Across::Rows { x: 41.0 },
            span: 8.0..16.0,
        },
        Line {
            name: "card top",
            across: Across::Rows { x: 51.0 },
            span: 28.0..36.0,
        },
        Line {
            name: "card left",
            across: Across::Columns { y: 46.0 },
            span: 18.0..26.0,
        },
        Line {
            name: "1px card top",
            across: Across::Rows { x: 131.0 },
            span: 28.0..36.0,
        },
        Line {
            name: "1px card left",
            across: Across::Columns { y: 46.0 },
            span: 98.0..106.0,
        },
    ]
}

/// Device rows a `--hair` line covers: one at a fractional scale, two at 2x.
fn hair_rows(scale: u16) -> usize {
    ds::PixelToken::Hair
        .device_pixels(Scale::from_percent(scale))
        .map_or(0, |rows| rows as usize)
}

#[test]
fn hairlines_and_borders_are_whole_device_rows_at_every_scale() {
    for scale in SCALES {
        let (_, frame) = render(scale);
        for line in fixed_lines() {
            assert_crisp(&frame, &line, scale, hair_rows(scale));
        }
    }
}

#[test]
fn menu_separators_are_whole_device_rows_at_every_scale() {
    for scale in SCALES {
        let (harness, frame) = render(scale);
        assert_eq!(harness.count(".ds-menu-separator"), 2, "{}", harness.html());
        let separator = rect(&harness, ".ds-menu-separator");
        let line = Line {
            name: "menu separator",
            across: Across::Rows {
                x: separator.origin.x.0 + separator.size.width.0 / 2.0,
            },
            span: separator.origin.y.0 - 4.0..separator.origin.y.0 + 5.0,
        };
        assert_crisp(&frame, &line, scale, hair_rows(scale));
    }
}

/// The plus glyph's horizontal bar (grid y 12, 8 px down a 16 px glyph), walked down a column
/// between its left end and the vertical bar: an even number of device rows at full ink.
#[test]
fn a_16_px_glyph_stroke_is_whole_device_rows_at_fractional_scales() {
    for scale in [125, 150, 175] {
        let (_, frame) = render(scale);
        let line = Line {
            name: "plus bar",
            across: Across::Rows {
                x: 41.0 + 16.0 * 8.5 / 24.0,
            },
            span: 84.0..94.0,
        };
        let rows = stroke_device_pixels(IconSize::Base, Scale::from_percent(scale));
        assert_crisp(&frame, &line, scale, rows as usize);
    }
}
