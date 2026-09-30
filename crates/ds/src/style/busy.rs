//! Waiting out a renderer that holds the document.
//!
//! A host's focus write or rect read answers "busy" when dioxus polls the asking task inside
//! `render_immediate`, while the mutation writer holds the Blitz document: a task woken in the
//! same turn as a dirty scope is polled there, in scope-height order. Whether a given ask lands
//! there depends on what else was dirty that turn, so a fixed "try again a frame later" made the
//! same ask land at once in one run and a `FRAME_SLACK` later in the next, even on a virtual clock.
//!
//! The renderer holds the document only for the length of one `render_immediate`, and dioxus
//! runs effects only outside it (after the renderer has applied the mutations). So the first
//! retries wait for [`after_render`], an effect that wakes the task, and land in the same frame
//! at the same instant, every run; only a document still busy after those waits a frame.

use dioxus::core::queue_effect;
use ds_core::time::{FRAME_SLACK, clock::sleep};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// How many retries wait for the render to end before the rest wait a frame each.
const AFTER_RENDER_RETRIES: usize = 4;

/// Wait before retry `attempt` (from 0) of a write or read the renderer refused: until the
/// current render has ended for the first few, a frame for the rest.
pub async fn wait_out_busy(attempt: usize) {
    match attempt {
        0..AFTER_RENDER_RETRIES => after_render().await,
        _ => sleep(FRAME_SLACK).await,
    }
}

/// Ready once the render in progress has ended: an effect of the polling task's scope wakes it,
/// and dioxus runs effects after the renderer has let go of the document. Call it from a task.
pub fn after_render() -> AfterRender {
    AfterRender {
        stage: Stage::Unqueued,
    }
}

/// Whether the waking effect has been queued yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Unqueued,
    Queued,
}

/// The future [`after_render`] returns.
#[derive(Debug)]
pub(crate) struct AfterRender {
    stage: Stage,
}

impl Future for AfterRender {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let this = self.get_mut();
        match this.stage {
            Stage::Unqueued => {
                let waker = cx.waker().clone();
                queue_effect(move || waker.wake());
                this.stage = Stage::Queued;
                Poll::Pending
            }
            Stage::Queued => Poll::Ready(()),
        }
    }
}
