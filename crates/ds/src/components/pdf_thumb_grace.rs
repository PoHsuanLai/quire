//! The pending look's grace (design/26 R4): a thumbnail whose page is still being read shows
//! nothing until [`PDF_THUMB_GRACE`](super::pdf_thumb::PDF_THUMB_GRACE) has passed, so a read
//! that lands quickly never flashes a placeholder. A task owned by the component's scope waits
//! the grace once per read and is dropped with the scope, or cancelled when the page arrives.

use super::pdf_thumb::PDF_THUMB_GRACE;
use crate::task::{spawn_in, try_get, try_set};
use crate::time::sleep;
use dioxus::core::{Task, current_scope_id, queue_effect};
use dioxus::prelude::*;

/// Whether the page is being read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reading {
    /// It is: the grace runs.
    Yes,
    /// It arrived, or failed: no grace.
    No,
}

/// Where a read stands against its grace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Grace {
    /// Younger than the grace, or not reading: show the state as it is.
    Within,
    /// Reading for longer than the grace: the pending look.
    Over,
}

/// The grace of the current read, restarted each time `reading` turns to `Yes`.
pub(crate) fn use_grace(reading: Reading) -> Grace {
    let grace = use_signal(|| Grace::Within);
    let task = use_signal(|| None::<Task>);
    let scope = use_hook(current_scope_id);
    let mut seen = use_hook(|| CopyValue::new(None::<Reading>));
    if *seen.peek() != Some(reading) {
        seen.set(Some(reading));
        queue_effect(move || restart(grace, task, scope, reading));
    }
    *grace.read()
}

fn restart(grace: Signal<Grace>, task: Signal<Option<Task>>, scope: ScopeId, reading: Reading) {
    if let Ok(Some(running)) = try_get(task) {
        running.cancel();
    }
    if try_set(grace, Grace::Within).is_err() {
        return;
    }
    let next = match reading {
        Reading::Yes => Some(spawn_in(scope, async move {
            sleep(PDF_THUMB_GRACE).await;
            let _ = try_set(grace, Grace::Over);
        })),
        Reading::No => None,
    };
    let _ = try_set(task, next);
}
