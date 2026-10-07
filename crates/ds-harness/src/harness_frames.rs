//! Driving a document one frame at a time, for a test that counts frames: a key arrives and no
//! frame runs until the test asks for it, so which frame shows what can be read.

use crate::harness::Harness;
use crate::headless_step::{PhaseOrder, Stepped};
use crate::input::Input;
use ds::prelude::ShortcutKey;

/// Whether the document is brought up to date as input is delivered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Delivery {
    /// At once: every frame it takes runs before the call returns.
    Settled,
    /// Not at all: the frames the input owes are for the test to step.
    Pending,
}

impl Harness {
    /// Press and release `key` and run no frame: the next [`Harness::step_frame`] is the first
    /// that shows it.
    pub fn key_pending(&mut self, key: ShortcutKey) {
        self.chord_pending(&[], key);
    }

    /// Press and release `key` with `held` modifiers down (as [`Input::chord`]) and run no frame.
    pub fn chord_pending(&mut self, held: &[ShortcutKey], key: ShortcutKey) {
        if let Input::Key(input) = Input::chord(held, key) {
            self.press_key(input, Delivery::Pending);
        }
    }

    /// Run the one frame the document is owed, in the window's order (render what is queued,
    /// lay out, paint, then run the frame phase), and read the document as that frame left it.
    /// [`Stepped::Idle`] when nothing asked for a frame, as a window then draws none.
    pub fn step_frame(&mut self) -> Stepped {
        let at = self.clock;
        self.doc.step(at)
    }

    /// How many frames the document has laid out and painted since it was built.
    pub fn frames(&self) -> u64 {
        self.doc.frames
    }

    /// Where in a frame the phase's early step runs from now on: [`PhaseOrder::AfterPaint`] is
    /// the order before it existed, kept so a test can show what it fixed.
    pub fn set_phase_order(&mut self, order: PhaseOrder) {
        self.doc.order = order;
    }
}
