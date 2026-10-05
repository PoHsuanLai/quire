//! A window's size, in logical pixels: what it opens at, and the least a person may resize it
//! to (`NSWindow.contentMinSize`). One value, so a window's size is said in one place.

use dioxus_native::{LogicalSize, WindowAttributes};

/// A width and a height in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Extent {
    /// Across.
    pub width: u32,
    /// Down.
    pub height: u32,
}

impl Extent {
    /// `width` by `height`.
    pub const fn new(width: u32, height: u32) -> Self {
        Extent { width, height }
    }

    fn logical(self) -> LogicalSize<u32> {
        LogicalSize::new(self.width, self.height)
    }
}

/// What a window opens at, and the least it may be resized to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSize {
    start: Extent,
    least: Option<Extent>,
}

impl WindowSize {
    /// A window that opens `width` by `height` and may be resized to anything.
    pub const fn new(width: u32, height: u32) -> Self {
        WindowSize {
            start: Extent::new(width, height),
            least: None,
        }
    }

    /// The same window, never resized below `width` by `height`: the least its layout fits in,
    /// so panes with least widths of their own are never squeezed past them.
    pub const fn with_least(self, width: u32, height: u32) -> Self {
        WindowSize {
            least: Some(Extent::new(width, height)),
            ..self
        }
    }

    /// What the window opens at.
    pub fn start(&self) -> Extent {
        self.start
    }

    /// The least the window may be resized to, if it has one.
    pub fn least(&self) -> Option<Extent> {
        self.least
    }

    /// `attributes` given this size.
    pub(crate) fn apply(self, attributes: WindowAttributes) -> WindowAttributes {
        let attributes = attributes.with_surface_size(self.start.logical());
        match self.least {
            Some(least) => attributes.with_min_surface_size(least.logical()),
            None => attributes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Extent, WindowSize};

    #[test]
    fn a_size_opens_where_it_says_and_has_a_least_only_when_given_one() {
        const CASES: &[(&str, WindowSize, Extent, Option<Extent>)] = &[
            (
                "no least",
                WindowSize::new(1200, 800),
                Extent::new(1200, 800),
                None,
            ),
            (
                "a least",
                WindowSize::new(1200, 800).with_least(760, 480),
                Extent::new(1200, 800),
                Some(Extent::new(760, 480)),
            ),
        ];
        for (name, size, start, least) in CASES {
            assert_eq!(size.start(), *start, "{name}: start");
            assert_eq!(size.least(), *least, "{name}: least");
        }
    }
}
