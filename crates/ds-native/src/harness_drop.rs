//! Files dragged into the harness's document from outside, as the window's hook feeds a drag
//! from the platform (`crate::window_drop`): the same `ds::HostFileDrop`, the same hit test.

use crate::harness::Harness;
use dioxus::core::consume_context_from_scope;
use dioxus::prelude::ScopeId;
use ds::{DropAcceptance, FileDragInput, HostFileDrop};

impl Harness {
    /// One step of a drag from outside the window (`FileDragInput::Entered`, `Offered`, `Moved`,
    /// `Dropped`, `Left`), then a render. Answers what the window would tell the platform: a copy
    /// over a drop target, a refusal elsewhere.
    pub fn file_drag(&mut self, input: FileDragInput) -> DropAcceptance {
        let answer = self.within(|| {
            consume_context_from_scope::<HostFileDrop>(ScopeId::ROOT)
                .map_or(DropAcceptance::Refuse, |host| host.feed(input))
        });
        self.settle_now();
        answer
    }
}
