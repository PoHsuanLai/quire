//! A row's leading glyph follows the row's ink on a real Blitz document: an inline `<svg>` that
//! paints with `currentColor` repaints in the colour it now has when only the row's `color`
//! changes (a selection), so it stays legible on the ground it now sits on: dark on the plain
//! row, light on the accent. Blitz used to keep the colour the glyph was built with (blitz-gaps,
//! "A colour-only restyle"), which quire hid by remounting the leading span on every selection
//! change.

use dioxus::prelude::*;
use ds::base::vocab::RowState;
use ds::components::lists::row::size::RowSize;
use ds::prelude::*;
use ds_harness::{Backdrop, Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 360,
    height: 120,
    scale_percent: 100,
};

static SELECTION: GlobalSignal<Selection> = Signal::global(|| Selection::Unselected);

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { id: "list", style: "width:340px",
                Row {
                    leading: RowLeading::Icon(Icon::Wifi),
                    title: "Café",
                    size: RowSize::Settings,
                    state: RowState { selection: SELECTION(), ..RowState::default() },
                }
            }
        }
    }
}

fn select(harness: &mut Harness, selection: Selection) {
    harness.within(|| *SELECTION.write() = selection);
    harness.advance(Duration::from_secs(2));
}

/// The luminance of a pixel, 0 to 255.
fn luma(pixel: &image::Rgba<u8>) -> i32 {
    let [r, g, b, _] = pixel.0;
    (299 * i32::from(r) + 587 * i32::from(g) + 114 * i32::from(b)) / 1000
}

/// The glyph's ink luminance minus its ground's: the leading span's corner is the ground, and
/// the pixel furthest from it is the glyph. Negative is dark ink, positive light.
fn ink_against_ground(harness: &mut Harness) -> i32 {
    let rect = harness
        .rect(".ds-row-leading svg")
        .expect("the glyph is laid out");
    let image = harness.render_over(Backdrop::Scheme).expect("paints");
    let (x0, y0) = (rect.origin.x.0 as u32, rect.origin.y.0 as u32);
    let (x1, y1) = (
        x0 + rect.size.width.0 as u32,
        y0 + rect.size.height.0 as u32,
    );
    let ground = luma(image.get_pixel(x0.saturating_sub(2), y0.saturating_sub(2)));
    let image = &image;
    (x0..x1)
        .flat_map(|x| (y0..y1).map(move |y| luma(image.get_pixel(x, y)) - ground))
        .max_by_key(|delta| delta.abs())
        .unwrap_or(0)
}

#[test]
fn a_selection_repaints_the_leading_glyph_in_the_rows_new_ink() {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(100));
    let plain = ink_against_ground(&mut harness);
    assert!(plain < -60, "dark ink on the plain row: {plain}");

    select(&mut harness, Selection::Selected);
    let selected = ink_against_ground(&mut harness);
    assert!(selected > 60, "light ink on the accent row: {selected}");

    select(&mut harness, Selection::Unselected);
    let again = ink_against_ground(&mut harness);
    assert!(again < -60, "dark ink on the plain row again: {again}");
}
