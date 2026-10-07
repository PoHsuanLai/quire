//! The screen a window is on, as an app can know it: the size of the output in logical pixels,
//! the area a window may use, and the output's scale. What a client can learn is limited (see
//! [`ScreenArea::work`]); this type says what it learned and how. Pure.

use crate::window_size::Extent;
use ds::prelude::Scale;

/// Whose output the area is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScreenOf {
    /// The output the window is on: its size and scale are the ones it draws at.
    Window,
    /// The output a window is likeliest to open on, before it is open: the primary monitor, or
    /// the first one listed, since the compositor chooses and never says in advance. Its scale is
    /// the output's whole-number scale, which on a fractionally scaled output is larger than
    /// the one the window will draw at, so the logical sizes are smaller than the window will
    /// see.
    Primary,
}

/// What `work` is made from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkBasis {
    /// The whole output. A client cannot see the panel, dock or menu bar a compositor reserves
    /// (a layer-shell exclusive zone is applied by the compositor, never reported to clients), so
    /// the work area is the output; a window is capped to 85% of it, which leaves room for them.
    WholeOutput,
    /// The output less a reserve the app knows of and gave (see [`ScreenArea::less`]).
    LessReserve,
}

/// Logical pixels kept free along each edge of an output: a panel's height, a dock's width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Reserve {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

/// One output: its size, the part of it a window may use, and its scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenArea {
    /// The whole output, in logical pixels.
    pub output: Extent,
    /// The part of it a window may use, in logical pixels: the whole output unless the app gave
    /// a reserve ([`WorkBasis`]). The size to fit a window to (`WindowSize::fitting`).
    pub work: Extent,
    /// Device pixels per logical pixel on this output. Fractional where the compositor scales
    /// by a fraction and the window has said so (`ScreenOf::Window`).
    pub scale: Scale,
    /// Whose output this is.
    pub of: ScreenOf,
    /// What `work` is made from.
    pub basis: WorkBasis,
}

impl ScreenArea {
    /// An output of `physical` pixels at `scale`: its logical size is the pixels over the
    /// scale, and with no reserve known its work area is all of it. `None` for a scale that is
    /// not a positive number.
    pub fn new(physical: Extent, scale: Scale, of: ScreenOf) -> Option<ScreenArea> {
        let output = physical.logical_at(scale.as_f64())?;
        Some(ScreenArea {
            output,
            work: output,
            scale,
            of,
            basis: WorkBasis::WholeOutput,
        })
    }

    /// The same output with `reserve` kept free: the work area is what is left, never below
    /// zero.
    pub fn less(self, reserve: Reserve) -> ScreenArea {
        let left = |all: u32, a: u32, b: u32| all.saturating_sub(a.saturating_add(b));
        ScreenArea {
            work: Extent::new(
                left(self.output.width, reserve.left, reserve.right),
                left(self.output.height, reserve.top, reserve.bottom),
            ),
            basis: WorkBasis::LessReserve,
            ..self
        }
    }

    /// The logical size of a window that shows `pixels` device pixels one to one on this
    /// output: an image of 1200 x 800 pixels on a 2x output is a 600 x 400 window.
    pub fn logical_for_pixels(&self, pixels: Extent) -> Extent {
        pixels.logical_at(self.scale.as_f64()).unwrap_or(pixels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn e(width: u32, height: u32) -> Extent {
        Extent::new(width, height)
    }

    #[test]
    fn an_output_is_its_pixels_over_its_scale_and_its_work_area_is_all_of_it() {
        // name, physical, scale, logical output
        const CASES: &[(&str, Extent, Scale, Extent)] = &[
            ("1x", e(1920, 1080), Scale(120), e(1920, 1080)),
            ("2x", e(3840, 2160), Scale(240), e(1920, 1080)),
            ("1.5x", e(3840, 2160), Scale(180), e(2560, 1440)),
            ("1.25x rounds", e(2560, 1440), Scale(150), e(2048, 1152)),
        ];
        for (name, physical, scale, logical) in CASES {
            let area = ScreenArea::new(*physical, *scale, ScreenOf::Window).expect(name);
            assert_eq!((area.output, area.work), (*logical, *logical), "{name}");
            assert_eq!(area.basis, WorkBasis::WholeOutput, "{name}");
        }
    }

    #[test]
    fn a_known_reserve_comes_off_the_work_area_and_never_below_zero() {
        let area = ScreenArea::new(e(1920, 1080), Scale(120), ScreenOf::Primary).expect("scale");
        let panel = Reserve {
            top: 32,
            bottom: 72,
            ..Reserve::default()
        };
        let less = area.less(panel);
        assert_eq!((less.work, less.output), (e(1920, 976), e(1920, 1080)));
        assert_eq!(less.basis, WorkBasis::LessReserve);
        let all = area.less(Reserve {
            left: 4000,
            top: 4000,
            ..Reserve::default()
        });
        assert_eq!(all.work, e(0, 0));
    }

    #[test]
    fn pixels_map_to_the_logical_size_that_shows_them_one_to_one() {
        // name, scale, image pixels, window logical size
        const CASES: &[(&str, Scale, Extent, Extent)] = &[
            ("1x", Scale(120), e(1200, 800), e(1200, 800)),
            ("2x", Scale(240), e(1200, 800), e(600, 400)),
            ("1.5x", Scale(180), e(1200, 800), e(800, 533)),
        ];
        for (name, scale, pixels, window) in CASES {
            let area = ScreenArea::new(e(3840, 2160), *scale, ScreenOf::Window).expect(name);
            assert_eq!(area.logical_for_pixels(*pixels), *window, "{name}");
        }
    }
}
