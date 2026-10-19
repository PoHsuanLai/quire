//! A window as tall as what it shows: [`use_content_fit`] measures one element after every layout
//! and asks the window for its height, at the width the caller names, capped to the screen. A
//! dialog-like window (a sheet of steps, an alert) never guesses its own height, so a sentence that
//! wraps, a longer translation or a larger text size is never cut off.
//!
//! The measuring is quire's frame phase (`ds::host::measure::use_rect`); the asking is the
//! window's [`WindowSizer`](crate::WindowSizer), so a window the person has resized themselves
//! ([`SizeOrigin::Person`]) is left as they made it.

use crate::size_ledger::SizeOrigin;
use crate::window_size::Extent;
use crate::window_sizer::use_window_sizer;
use dioxus::prelude::*;
use ds::host::measure::{RectProbe, use_rect};
use std::cell::Cell;
use std::rc::Rc;

/// The element a window is sized to: hand [`on_mounted`](ContentFit::on_mounted) its `onmounted`.
///
/// The element must be as tall as its content, never as tall as the window (no `height: 100%`,
/// no `flex: 1` in a column the window's height), or the window would only ever be asked for the
/// height it already has. Its top is taken as the window's top: whatever sits above it inside the
/// window is counted, from where layout put it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContentFit {
    probe: RectProbe,
}

impl ContentFit {
    /// Hand this the measured element's `onmounted` event.
    pub fn on_mounted(&self, event: Event<MountedData>) {
        self.probe.on_mounted(event);
    }
}

/// Size the window the calling component renders in to one element's height: `width` across
/// (the content's own width is the window's, so it cannot be measured), and as tall as the
/// element's bottom edge, rounded up. The size is capped to 85% of the screen's work area
/// ([`Extent::fit`]) and never below `least`. Asked again whenever layout moves the edge or
/// `width` changes, and not at all once the person has resized the window. A window opens at its
/// [`WindowSpec`](crate::WindowSpec) size, so give that a fair estimate: the fit follows the
/// first layout.
///
/// Outside a window that can be resized (a server render, a snapshot) this measures nothing.
pub fn use_content_fit(width: u32, least: Extent) -> ContentFit {
    let probe = use_rect();
    let sizer = use_window_sizer();
    let asked = use_hook(|| Rc::new(Cell::new(None::<Extent>)));
    use_effect(use_reactive!(|width, least| {
        let Some(sizer) = &sizer else {
            return;
        };
        let Some(rect) = probe.rect() else {
            return;
        };
        if sizer.origin() == Some(SizeOrigin::Person) {
            return;
        }
        let bottom = rect.origin.y.0.max(0.0) + rect.size.height.0;
        let screen = sizer.screen().map(|screen| screen.work);
        let wanted = content_extent(width, bottom, least, screen);
        if asked.get() == Some(wanted) {
            return;
        }
        asked.set(Some(wanted));
        sizer.request_size(wanted);
    }));
    ContentFit { probe }
}

/// The window for content whose bottom edge is at `bottom` logical px: `width` across, the edge
/// rounded up, fitted to `screen` when it is known, and never below `least`.
fn content_extent(width: u32, bottom: f32, least: Extent, screen: Option<Extent>) -> Extent {
    let natural = Extent::new(width, whole_px(bottom));
    match screen {
        Some(screen) => natural.fit(screen, Some(least)),
        None => Extent::new(
            natural.width.max(least.width),
            natural.height.max(least.height),
        ),
    }
}

/// `px` rounded up to a whole pixel, so a fractional edge is never cut; nothing for a length that
/// is not a positive number.
fn whole_px(px: f32) -> u32 {
    if px.is_finite() && px > 0.0 {
        // The cast saturates, and a window's height fits a u32 many times over.
        px.ceil() as u32
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::{content_extent, whole_px};
    use crate::window_size::Extent;

    #[test]
    fn the_window_is_the_content_s_height_at_the_width_asked() {
        let least = Extent::new(480, 135);
        let screen = Some(Extent::new(1440, 900));
        // (bottom edge, screen, the window asked for)
        let cases = [
            // A fractional edge rounds up: the last line is never cut.
            (141.66, screen, Extent::new(480, 142)),
            (300.0, screen, Extent::new(480, 300)),
            // Shorter than the least is the least.
            (60.0, screen, Extent::new(480, 135)),
            // Taller than the screen allows is 85% of it; the content scrolls.
            (2000.0, screen, Extent::new(480, 765)),
            // No screen known: the content, never below the least.
            (2000.0, None, Extent::new(480, 2000)),
            (0.0, None, Extent::new(480, 135)),
        ];
        for (bottom, screen, want) in cases {
            assert_eq!(
                content_extent(480, bottom, least, screen),
                want,
                "{bottom} {screen:?}"
            );
        }
    }

    #[test]
    fn a_length_that_is_not_a_positive_number_is_nothing() {
        for (px, want) in [
            (f32::NAN, 0),
            (f32::INFINITY, 0),
            (-4.0, 0),
            (0.2, 1),
            (12.0, 12),
        ] {
            assert_eq!(whole_px(px), want, "{px}");
        }
    }
}
