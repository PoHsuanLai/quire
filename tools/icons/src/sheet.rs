use image::{Rgba, Rgba32FImage, imageops::FilterType, imageops::resize};

use crate::{Srgb8, Template, compose::Shadow, compose::over, draw_text, export, text_width};

/// One tile on a contact sheet: an image, an optional strip under it, a caption.
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub image: Rgba32FImage,
    pub below: Option<Rgba32FImage>,
    pub caption: String,
}

/// A contact sheet: rows of cells, each row named on the left (design/08-ICONS.md 3.4 item 4).
#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    pub title: String,
    pub rows: Vec<(String, Vec<Cell>)>,
}

/// Sizes and colours of a sheet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SheetStyle {
    pub tile: u32,
    pub gap: u32,
    pub ground: Srgb8,
    pub ink: Srgb8,
    pub text_scale: u32,
}

impl Default for SheetStyle {
    fn default() -> Self {
        Self {
            tile: 256,
            gap: 24,
            ground: Srgb8::hex(0xF1F3EE),
            ink: Srgb8::hex(0x1A1E1A),
            text_scale: 3,
        }
    }
}

const DOT_ROWS: u32 = 7;

fn rgba(c: Srgb8) -> [f32; 4] {
    let [r, g, b] = c.unit();
    [r, g, b, 1.0]
}

/// Paints `img` over `canvas` with its top-left at (x, y).
fn paste(canvas: &mut Rgba32FImage, img: &Rgba32FImage, (x, y): (u32, u32)) {
    for (ix, iy, p) in img.enumerate_pixels() {
        let (px, py) = (x + ix, y + iy);
        if px < canvas.width() && py < canvas.height() {
            let under = canvas.get_pixel(px, py).0;
            canvas.put_pixel(px, py, Rgba(over(p.0, under)));
        }
    }
}

/// Lays the sheet out: title, then one row per subject with its name on the left and its
/// cells to the right, each captioned underneath.
pub fn build_sheet(sheet: &Sheet, style: &SheetStyle) -> Rgba32FImage {
    let text_h = DOT_ROWS * style.text_scale;
    let label_w = sheet
        .rows
        .iter()
        .map(|(name, _)| text_width(name, style.text_scale))
        .max()
        .unwrap_or(0)
        + style.gap;
    let cols = sheet
        .rows
        .iter()
        .map(|(_, cells)| cells.len() as u32)
        .max()
        .unwrap_or(0);
    let below_h = sheet
        .rows
        .iter()
        .flat_map(|(_, cells)| cells)
        .filter_map(|c| c.below.as_ref())
        .map(|b| b.height() + style.gap / 2)
        .max()
        .unwrap_or(0);
    let row_h = style.tile + below_h + text_h + style.gap * 2;
    let width = (style.gap + label_w + cols * (style.tile + style.gap))
        .max(text_width(&sheet.title, style.text_scale) + 2 * style.gap);
    let top = style.gap * 2 + text_h;
    let height = top + sheet.rows.len() as u32 * row_h;
    let mut img = Rgba32FImage::from_pixel(width, height, Rgba(rgba(style.ground)));
    draw_text(
        &mut img,
        &sheet.title,
        (style.gap, style.gap),
        style.text_scale,
        rgba(style.ink),
    );
    for (r, (name, cells)) in sheet.rows.iter().enumerate() {
        let y = top + r as u32 * row_h;
        draw_text(
            &mut img,
            name,
            (style.gap, y + style.tile / 2 - text_h / 2),
            style.text_scale,
            rgba(style.ink),
        );
        for (c, cell) in cells.iter().enumerate() {
            let x = style.gap + label_w + c as u32 * (style.tile + style.gap);
            paste(
                &mut img,
                &resize(&cell.image, style.tile, style.tile, FilterType::Lanczos3),
                (x, y),
            );
            let mut next = y + style.tile + style.gap / 2;
            if let Some(below) = &cell.below {
                paste(&mut img, below, (x, next));
                next += below.height() + style.gap / 2;
            }
            draw_text(
                &mut img,
                &cell.caption,
                (x, next),
                style.text_scale,
                rgba(style.ink),
            );
        }
    }
    img
}

/// The 16, 32 and 48 px exports at 1:1 on a light and on a dark ground (08 3.4 item 4), from
/// a 1024 `flat` master; 48 carries its baked shadow like the hicolor file does.
pub fn size_strip(flat: &Rgba32FImage, t: &Template) -> Rgba32FImage {
    const SIZES: [u32; 3] = [16, 32, 48];
    const PAD: u32 = 8;
    let grounds = [Srgb8::hex(0xF1F3EE), Srgb8::hex(0x1D211B)];
    let panel_w = PAD + SIZES.iter().map(|s| s + PAD).sum::<u32>();
    let h = 48 + 2 * PAD;
    let mut img = Rgba32FImage::new(panel_w * 2, h);
    for (g, ground) in grounds.into_iter().enumerate() {
        let x0 = g as u32 * panel_w;
        paste(
            &mut img,
            &Rgba32FImage::from_pixel(panel_w, h, Rgba(rgba(ground))),
            (x0, 0),
        );
        let mut x = x0 + PAD;
        for size in SIZES {
            let e = export(flat, t, size, Shadow::Baked);
            paste(&mut img, &e.image, (x, PAD + 48 - size));
            x += size + PAD;
        }
    }
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sheet_grows_with_rows_and_columns() {
        let cell = || Cell {
            image: Rgba32FImage::from_pixel(64, 64, Rgba([1.0, 0.0, 0.0, 1.0])),
            below: None,
            caption: "S1".into(),
        };
        let style = SheetStyle {
            tile: 32,
            gap: 8,
            text_scale: 1,
            ..SheetStyle::default()
        };
        let one = build_sheet(
            &Sheet {
                title: "T".into(),
                rows: vec![("A".into(), vec![cell()])],
            },
            &style,
        );
        let two = build_sheet(
            &Sheet {
                title: "T".into(),
                rows: vec![
                    ("A".into(), vec![cell(), cell()]),
                    ("B".into(), vec![cell(), cell()]),
                ],
            },
            &style,
        );
        assert_eq!(two.width() - one.width(), style.tile + style.gap);
        assert_eq!(
            two.height() - one.height(),
            one.height() - (style.gap * 2 + DOT_ROWS)
        );
        let red = two
            .pixels()
            .filter(|p| p.0[0] > 0.9 && p.0[1] < 0.1)
            .count() as u32;
        assert_eq!(red, 4 * style.tile * style.tile, "four red tiles");
    }
}
