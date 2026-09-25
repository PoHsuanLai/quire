//! Where a tiling placement puts a window on its output, in physical pixels: pure, so the
//! arithmetic is tested without a window. winit reports no work area (the output minus the
//! panels), so a half covers the whole output's height, panels included (FINDINGS "Window frame").

use ds::WindowTile;

/// A rectangle on the desktop, in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Area {
    /// The left edge.
    pub(crate) x: i32,
    /// The top edge.
    pub(crate) y: i32,
    /// The width.
    pub(crate) width: u32,
    /// The height.
    pub(crate) height: u32,
}

/// Where `tile` puts a window of `size` (width, height) on `output`. `Fill` is the compositor's
/// maximize, not a placement, so it has none.
pub(crate) fn placement(output: Area, size: (u32, u32), tile: WindowTile) -> Option<Area> {
    let half = output.width / 2;
    match tile {
        WindowTile::Fill => None,
        WindowTile::LeftHalf => Some(Area {
            width: half,
            ..output
        }),
        WindowTile::RightHalf => Some(Area {
            x: output.x + offset(half),
            width: output.width - half,
            ..output
        }),
        WindowTile::Centre => {
            let (width, height) = (size.0.min(output.width), size.1.min(output.height));
            Some(Area {
                x: output.x + offset((output.width - width) / 2),
                y: output.y + offset((output.height - height) / 2),
                width,
                height,
            })
        }
    }
}

/// A length as an offset; no output is wider than `i32::MAX` pixels.
fn offset(length: u32) -> i32 {
    i32::try_from(length).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::{Area, placement};
    use ds::WindowTile;

    const OUTPUT: Area = Area {
        x: 1920,
        y: 0,
        width: 2561,
        height: 1440,
    };

    #[test]
    fn each_placement_lands_on_its_output() {
        #[rustfmt::skip]
        const CASES: &[(WindowTile, Option<(i32, i32, u32, u32)>)] = &[
            (WindowTile::Fill, None),
            (WindowTile::LeftHalf, Some((1920, 0, 1280, 1440))),
            (WindowTile::RightHalf, Some((3200, 0, 1281, 1440))),
            (WindowTile::Centre, Some((2560, 320, 1200, 800))),
        ];
        for &(tile, expected) in CASES {
            let placed = placement(OUTPUT, (1200, 800), tile)
                .map(|area| (area.x, area.y, area.width, area.height));
            assert_eq!(placed, expected, "{tile:?}");
        }
    }

    #[test]
    fn a_window_larger_than_its_output_is_centred_at_the_output_size() {
        let placed = placement(OUTPUT, (4000, 2000), WindowTile::Centre);
        assert_eq!(placed, Some(OUTPUT));
    }
}
