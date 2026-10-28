//! The document one frame at a time, in the window's order, for a test that counts frames: what a
//! frame shows is what the document is when [`Headless::step`] returns.

use crate::headless::{Headless, RE_RENDERED};
use ds_blitz::seam::Layout as PhaseLayout;
use std::time::Duration;

/// Where in a frame the phase publishes what a moved caret needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PhaseOrder {
    /// Before the frame paints, as the window runs it: the caret is in the frame with its text.
    #[default]
    BeforePaint,
    /// Only after the frame paints: the caret follows its text by a frame, as it did before the
    /// early step.
    AfterPaint,
}

impl Headless {
    /// Before the paint, when a caret moved: lay out the renders' changes, publish the caret's
    /// box, and render what that changed, so this frame's layout has the caret in it.
    pub(crate) fn early_phase(&mut self, at: Duration) {
        if !self.phase.wants_early_run() {
            return;
        }
        let _ = self.resolve(at);
        if self.phase.run(PhaseLayout::Resolved).changed() {
            self.flush();
        }
    }

    /// Run one frame if one is owed; whether the document rendered, laid out and painted one
    /// (false: nothing asked for a frame and the document is at rest).
    pub(crate) fn step(&mut self, at: Duration) -> bool {
        if !self.owed {
            return false;
        }
        // As in the window, a frame whose renders ran is not owed another for them: the render
        // loop already ran them dry before the frame drew.
        self.owed = self.round(at).into_iter().any(|why| why != RE_RENDERED);
        true
    }
}
