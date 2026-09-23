//! Driving a quire app in a test the way a user would: pointer, keys, and time, against a real
//! Blitz document (no window). Timers run on the harness's clock, so a test advances 450 ms and
//! sees the hover card open, not before.
#![allow(unused_variables, dead_code)] // Freeze stubs: remove with the last todo!().

use crate::snapshot::Viewport;
use dioxus::prelude::*;
use ds::{Key, Point};
use std::time::Duration;

/// A headless document under test.
pub struct Harness {
    viewport: Viewport,
}

impl std::fmt::Debug for Harness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Harness")
            .field("viewport", &self.viewport)
            .finish_non_exhaustive()
    }
}

impl Harness {
    /// Build `app` at `viewport` and render its first frame.
    pub fn new(app: fn() -> Element, viewport: Viewport) -> Self {
        todo!()
    }

    /// Move the pointer to `at`.
    pub fn pointer_move(&mut self, at: Point) {
        todo!()
    }

    /// Press and release the primary button at `at`.
    pub fn click(&mut self, at: Point) {
        todo!()
    }

    /// Press and release `key` with the focus where it is.
    pub fn key(&mut self, key: Key) {
        todo!()
    }

    /// Let `time` pass: fire due timers and render.
    pub fn advance(&mut self, time: Duration) {
        todo!()
    }

    /// The document as HTML, for assertions.
    pub fn html(&self) -> String {
        todo!()
    }

    /// The border-box rect of the first element matching `selector`, if any.
    pub fn rect(&self, selector: &str) -> Option<ds::Rect> {
        todo!()
    }
}
