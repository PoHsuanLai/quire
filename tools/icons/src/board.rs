//! Round four's sheets (design/08-ICONS.md 2.10): one icon per dialect with its small sizes,
//! and the Monochrome set tinted by a Space. Pure: specs and images in, cells out.

use ds::Scheme;
use ds::space::{Dot, derive};
use image::Rgba32FImage;

use crate::{
    CHROMA_CAP, Cell, Lch, Look, Plane, Shadow, Srgb8, Template, Tint, drop_shadow, emblem, export,
    grid_for, strip,
};

/// One size of an icon in a look, drawn natively; the baked shadow from 48 px (08 2.5).
pub fn render_icon(
    spec: &crate::Spec,
    look: Look,
    size: u32,
    t: &Template,
    tile: &Plane,
) -> Rgba32FImage {
    let flat = emblem(spec, look, size, t, tile);
    match size >= 48 {
        true => drop_shadow(&flat, grid_for(size, t), t),
        false => flat,
    }
}

/// A sheet cell: the icon at `tile_px` (drawn natively) over its 16/32/48 strip.
pub fn look_cell(
    spec: &crate::Spec,
    look: Look,
    tile_px: u32,
    caption: &str,
    t: &Template,
    tile: &Plane,
) -> Cell {
    Cell {
        image: render_icon(spec, look, tile_px, t, tile),
        below: Some(strip(&[16, 32, 48], |s| {
            render_icon(spec, look, s, t, tile)
        })),
        caption: caption.to_owned(),
    }
}

/// A sheet cell for a model-drawn icon: its master over the strip exported from its flat.
pub fn face_cell(master: Rgba32FImage, flat: &Rgba32FImage, caption: &str, t: &Template) -> Cell {
    Cell {
        image: master,
        below: Some(strip(&[16, 32, 48], |s| {
            export(flat, t, s, Shadow::Baked).image
        })),
        caption: caption.to_owned(),
    }
}

/// The tint a Space gives a Monochrome icon: the hue and chroma of the accent `ds` derives for
/// the Space (the Space's first dot at Postmark's weight, design/03-COLOR.md section 5), so the
/// icon follows the same colour the frame and the card accent do. Strength is that chroma over
/// the cap.
pub fn space_tint(dots: &[Dot]) -> Tint {
    let accent = derive(dots, Scheme::Light).accent;
    let rgb = u32::from_str_radix(accent.trim_start_matches('#'), 16).unwrap_or(0x808080);
    let lch = Lch::of(Srgb8::hex(rgb));
    Tint {
        hue: lch.h,
        strength: (lch.c / CHROMA_CAP).min(1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ds::space::PRESETS;

    /// Work (hue 268) and Home (hue 152) tint the icon near their own hues.
    #[test]
    fn space_tints_follow_the_preset_hue() {
        for (preset, hue) in [(0, 268.0_f32), (1, 152.0)] {
            let t = space_tint(PRESETS[preset].dots);
            let d = (t.hue - hue + 540.0).rem_euclid(360.0) - 180.0;
            assert!(d.abs() < 12.0, "preset {preset}: {} vs {hue}", t.hue);
            assert!(t.strength > 0.5 && t.strength <= 1.0, "{}", t.strength);
        }
    }
}
