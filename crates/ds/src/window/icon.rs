//! A window's icon as pixels: the small picture a window shows in its titlebar, task switcher
//! or dock, where the platform takes one from the app. X11, Windows and macOS read it from the
//! window; Wayland takes the icon from the app's desktop entry (matched by its application id),
//! and a compositor that implements `xdg_toplevel_icon` may also use this one.

/// Bytes in one pixel: red, green, blue, alpha.
const PIXEL: usize = 4;

/// Why pixels are not an icon.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum IconError {
    /// The icon has no pixels.
    #[error("an icon needs at least one pixel")]
    Empty,
    /// The bytes are not `width * height` pixels of four bytes.
    #[error("{width}x{height} pixels need {expected} bytes, not {found}")]
    Size {
        /// The width asked for.
        width: u32,
        /// The height asked for.
        height: u32,
        /// The bytes that many pixels take.
        expected: usize,
        /// The bytes given.
        found: usize,
    },
}

/// An icon: `width` by `height` pixels of straight (not premultiplied) RGBA, row by row from the
/// top left.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WindowIcon {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

impl WindowIcon {
    /// The icon of `rgba` at `width` by `height`, or why the bytes do not fit.
    pub fn new(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, IconError> {
        if width == 0 || height == 0 {
            return Err(IconError::Empty);
        }
        let expected = usize::try_from(u64::from(width) * u64::from(height))
            .ok()
            .and_then(|pixels| pixels.checked_mul(PIXEL))
            .unwrap_or(usize::MAX);
        if rgba.len() != expected {
            return Err(IconError::Size {
                width,
                height,
                expected,
                found: rgba.len(),
            });
        }
        Ok(WindowIcon {
            rgba,
            width,
            height,
        })
    }

    /// The pixels, four bytes each.
    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    /// The width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::{IconError, WindowIcon};

    #[test]
    fn only_whole_rgba_pixels_make_an_icon() {
        let cases = [
            ("two by one", vec![0; 8], 2, 1, None),
            ("empty", vec![], 0, 0, Some(IconError::Empty)),
            (
                "short",
                vec![0; 7],
                2,
                1,
                Some(IconError::Size {
                    width: 2,
                    height: 1,
                    expected: 8,
                    found: 7,
                }),
            ),
        ];
        for (name, bytes, width, height, want) in cases {
            assert_eq!(WindowIcon::new(bytes, width, height).err(), want, "{name}");
        }
    }
}
