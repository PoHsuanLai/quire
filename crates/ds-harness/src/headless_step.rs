//! The document one frame at a time, in the window's order, for a test that counts frames: what a
//! frame shows is what the document is when [`Headless::step`] returns.

use crate::headless::{Headless, RE_RENDERED};
use ds_blitz::seam::{Early, Layout as PhaseLayout};
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

/// Whether the document is owed a frame: input arrived, or the last frame asked for another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Owed {
    Frame,
    Nothing,
}

/// What one [`Headless::step`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stepped {
    /// The document rendered, laid out and painted one frame.
    Painted,
    /// Nothing asked for a frame: the document is at rest.
    Idle,
}

impl Headless {
    /// Before the paint, when a caret moved: lay out the renders' changes, publish the caret's
    /// box, and render what that changed, so this frame's layout has the caret in it.
    pub(crate) fn early_phase(&mut self, at: Duration) {
        if self.phase.early() == Early::Idle {
            return;
        }
        let _ = self.resolve(at);
        if self.phase.run(PhaseLayout::Resolved).changed() {
            self.flush();
        }
    }

    /// Run one frame if one is owed.
    pub(crate) fn step(&mut self, at: Duration) -> Stepped {
        match self.owed {
            Owed::Nothing => Stepped::Idle,
            Owed::Frame => {
                // As in the window, a frame whose renders ran is not owed another for them: the
                // render loop already ran them dry before the frame drew.
                let more = self.round(at).into_iter().any(|why| why != RE_RENDERED);
                self.owed = match more {
                    true => Owed::Frame,
                    false => Owed::Nothing,
                };
                Stepped::Painted
            }
        }
    }
}
