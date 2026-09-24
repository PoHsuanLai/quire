//! Round five's palette board (design/08-ICONS.md 2.10): the eight hues at each chroma level,
//! as the Solid dialect's plate colour on a small squircle, on the light and the dark ground,
//! each labelled with its name, its OKLCh values and its hex, so a pick can be named.

use image::{Rgba, Rgba32FImage};

use crate::{
    ChromaCap, Dialect, PALETTE, PlateGrid, Srgb8, Template, Tint, compose::over, draw_text,
    in_srgb, plate::superellipse, roles, srgb,
};

const SWATCH: u32 = 150;
const CELL_W: u32 = 210;
const PAD: u32 = 24;
const SCALE: u32 = 2;
const LINE: u32 = 7 * SCALE + 6;

fn rgba(c: Srgb8) -> [f32; 4] {
    let [r, g, b] = c.unit();
    [r, g, b, 1.0]
}

/// One row: the eight swatches at one level on one ground.
fn row(cap: ChromaCap, ground: Srgb8, ink: Srgb8, t: &Template) -> Rgba32FImage {
    let h = PAD + SWATCH + 4 * LINE + PAD;
    let w = PAD + PALETTE.len() as u32 * CELL_W;
    let mut img = Rgba32FImage::from_pixel(w, h, Rgba(rgba(ground)));
    let grid = PlateGrid {
        canvas: SWATCH,
        side: SWATCH,
        origin: 0,
    };
    let mask = superellipse(
        SWATCH,
        (grid.centre(), grid.centre()),
        grid.half(),
        t.exponent,
    );
    for (i, (name, hue)) in PALETTE.iter().enumerate() {
        let x0 = PAD + i as u32 * CELL_W;
        let tint = Tint {
            hue: *hue,
            strength: 1.0,
        };
        let plate = roles(Dialect::Solid, tint, cap).plate;
        let [r, g, b] = srgb(plate);
        for (x, y, _) in Rgba32FImage::new(SWATCH, SWATCH).enumerate_pixels() {
            let a = mask.at(i64::from(x), i64::from(y));
            let under = img.get_pixel(x0 + x, PAD + y).0;
            img.put_pixel(x0 + x, PAD + y, Rgba(over([r, g, b, a], under)));
        }
        let hex = [r, g, b].map(|v| (v * 255.0).round() as u32);
        let chroma = plate.a.hypot(plate.b);
        let lines = [
            name.to_uppercase(),
            format!("L{:.2} C{:.3}", plate.l, chroma),
            format!("H{hue:.0}"),
            match in_srgb(plate) {
                true => format!("#{:02X}{:02X}{:02X}", hex[0], hex[1], hex[2]),
                false => format!("#{:02X}{:02X}{:02X} CLIP", hex[0], hex[1], hex[2]),
            },
        ];
        for (k, text) in lines.iter().enumerate() {
            draw_text(
                &mut img,
                text,
                (x0, PAD + SWATCH + 6 + k as u32 * LINE),
                SCALE,
                rgba(ink),
            );
        }
    }
    img
}

/// The board's rows, named: the three levels on the light ground, then on the dark.
pub fn palette_board(t: &Template) -> Vec<(String, Rgba32FImage)> {
    let grounds = [
        ("LIGHT", Srgb8::hex(0xF1F3EE), Srgb8::hex(0x1A1E1A)),
        ("DARK", Srgb8::hex(0x1D211B), Srgb8::hex(0xE7EBE3)),
    ];
    grounds
        .iter()
        .flat_map(|(scheme, ground, ink)| {
            ChromaCap::LEVELS.iter().map(move |cap| {
                (
                    format!("{scheme} C {:.2}", cap.0),
                    row(*cap, *ground, *ink, t),
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_rows_of_eight() {
        let rows = palette_board(&Template::default());
        assert_eq!(rows.len(), 6);
        let w = rows[0].1.width();
        assert!(rows.iter().all(|(_, r)| r.width() == w));
        assert!(w >= PALETTE.len() as u32 * CELL_W);
    }
}
