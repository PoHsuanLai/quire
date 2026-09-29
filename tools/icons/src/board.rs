//! One icon in a look at one size, drawn natively. Pure: a spec in, an image out.

use image::Rgba32FImage;

use crate::{Look, Plane, Template, drop_shadow, emblem, grid_for};

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
