//! Reaching into a document a renderer may be holding. On dioxus-native, a task woken in the
//! same turn as a dirty scope is polled inside `render_immediate`, while the mutation writer
//! holds the document; a rect read or a focus change made then borrows it again and panics
//! ("RefCell already borrowed", wave 2 integration; sill FINDINGS Q10 and Q43). `ds` cannot name
//! the Blitz node to `try_borrow` it, so both the call and the poll are guarded instead: a
//! panic there left nothing half-written (the borrow failed before anything changed) and reads
//! as "busy", and the caller tries again a frame later. Hosts provide `HostMeasure` and
//! `HostFocus` so the collision is a plain `Busy` and the default panic hook prints nothing.

use std::future::Future;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::pin::Pin;
use std::task::{Context, Poll};

/// A future whose poll may panic, polled so that a panic ends it with `None`.
pub(crate) struct Guarded<F: Future>(pub(crate) Pin<Box<F>>);

impl<F: Future> Future for Guarded<F> {
    type Output = Option<F::Output>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let inner = self.0.as_mut();
        match catch_unwind(AssertUnwindSafe(move || inner.poll(cx))) {
            Ok(Poll::Ready(value)) => Poll::Ready(Some(value)),
            Ok(Poll::Pending) => Poll::Pending,
            Err(_) => Poll::Ready(None),
        }
    }
}

/// `start()`, then its future, each guarded: `None` when either panicked. For a call that
/// borrows the document when it is made rather than when it is polled (dioxus-native-dom's
/// `set_focus`).
pub(crate) async fn guarded_call<F: Future>(start: impl FnOnce() -> F) -> Option<F::Output> {
    let future = catch_unwind(AssertUnwindSafe(start)).ok()?;
    Guarded(Box::pin(future)).await
}

#[cfg(test)]
mod tests {
    use super::{Guarded, guarded_call};
    use std::cell::RefCell;
    use std::future::Future;
    use std::pin::pin;
    use std::task::{Context, Poll, Waker};

    /// Poll `future` once with a waker that does nothing.
    fn poll_once<F: Future>(future: F) -> Poll<F::Output> {
        let mut context = Context::from_waker(Waker::noop());
        pin!(future).poll(&mut context)
    }

    /// The Blitz read's failure, reduced: a read that borrows a document the renderer holds.
    async fn read(document: &RefCell<u32>) -> u32 {
        *document.borrow()
    }

    /// The Blitz focus's failure, reduced: a call that borrows the document mutably when it is
    /// made, then returns a ready future.
    fn focus(document: &RefCell<u32>) -> std::future::Ready<u32> {
        let mut held = document.borrow_mut();
        *held += 1;
        std::future::ready(*held)
    }

    #[test]
    fn a_read_of_a_held_document_is_busy_not_a_panic() {
        let document = RefCell::new(7);
        assert_eq!(
            poll_once(Guarded(Box::pin(read(&document)))),
            Poll::Ready(Some(7)),
            "a free document reads"
        );
        let held = document.borrow_mut();
        assert_eq!(
            poll_once(Guarded(Box::pin(read(&document)))),
            Poll::Ready(None),
            "a held document is busy"
        );
        drop(held);
        assert_eq!(
            poll_once(Guarded(Box::pin(read(&document)))),
            Poll::Ready(Some(7)),
            "and reads again once it is free"
        );
    }

    #[test]
    fn a_call_into_a_held_document_is_busy_and_changes_nothing() {
        let document = RefCell::new(0);
        let held = document.borrow();
        assert_eq!(
            poll_once(guarded_call(|| focus(&document))),
            Poll::Ready(None),
            "a held document is busy"
        );
        drop(held);
        assert_eq!(*document.borrow(), 0, "the failed call changed nothing");
        assert_eq!(
            poll_once(guarded_call(|| focus(&document))),
            Poll::Ready(Some(1)),
            "a free document takes the call"
        );
    }
}
