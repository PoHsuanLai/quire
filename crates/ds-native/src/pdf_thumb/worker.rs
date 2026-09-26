//! The one `pdf-thumb` worker thread and its latest-wins queue.
//!
//! A launcher's preview asks for a new page on every arrow key; holding Down through fifty PDFs
//! must not start fifty rasters. So there is one long-lived worker, started on first use, fed
//! by a short queue in which each [`Slot`] (one `PdfFileThumb` instance) holds at most one job:
//! a new request from a slot replaces its job that has not started yet. A job already running
//! finishes into the cache. A job whose asker has gone (its task was cancelled or its component
//! unmounted) is skipped when it comes up. At most [`QUEUE_DEPTH`] jobs wait; past that the
//! oldest is dropped, and its asker hears so and asks again at the back.

use super::request::{ThumbRequest, pdf_thumb_blocking};
use ds::PdfPage;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Condvar, LazyLock, Mutex, PoisonError};
use tokio::sync::oneshot;

/// How many jobs may wait for the worker at once.
pub const QUEUE_DEPTH: usize = 8;

/// One asker: a `PdfFileThumb` instance, whose newer request supersedes its older one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Slot(u64);

impl Slot {
    /// A slot no other asker has.
    pub(crate) fn fresh() -> Slot {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        Slot(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

/// Why no page came back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Dropped;

struct Job {
    slot: Slot,
    request: ThumbRequest,
    reply: oneshot::Sender<PdfPage>,
}

struct Queue {
    jobs: Mutex<VecDeque<Job>>,
    ready: Condvar,
}

static QUEUE: LazyLock<Queue> = LazyLock::new(|| {
    // The worker lives as long as the process; a failure to start it leaves every job queued,
    // and each asker waits in its loading state rather than the app failing.
    let _ = std::thread::Builder::new()
        .name("pdf-thumb".to_owned())
        .spawn(work);
    Queue {
        jobs: Mutex::new(VecDeque::new()),
        ready: Condvar::new(),
    }
});

/// Queue `request` for `slot`, replacing the slot's job that has not started, and wait for its
/// page. [`Dropped`] when the job was pushed out of a full queue.
pub(crate) async fn ask(slot: Slot, request: ThumbRequest) -> Result<PdfPage, Dropped> {
    let (reply, answer) = oneshot::channel();
    {
        let mut jobs = QUEUE.jobs.lock().unwrap_or_else(PoisonError::into_inner);
        jobs.retain(|job| job.slot != slot);
        jobs.push_back(Job {
            slot,
            request,
            reply,
        });
        while jobs.len() > QUEUE_DEPTH {
            jobs.pop_front();
        }
    }
    QUEUE.ready.notify_one();
    answer.await.map_err(|_| Dropped)
}

/// The worker: take the oldest job whose asker still waits, raster it, answer.
fn work() {
    loop {
        let job = {
            let queue = &*QUEUE;
            let mut jobs = queue.jobs.lock().unwrap_or_else(PoisonError::into_inner);
            loop {
                match jobs.pop_front() {
                    Some(job) if job.reply.is_closed() => continue,
                    Some(job) => break job,
                    None => {
                        jobs = queue
                            .ready
                            .wait(jobs)
                            .unwrap_or_else(PoisonError::into_inner)
                    }
                }
            }
        };
        let _ = job.reply.send(pdf_thumb_blocking(&job.request));
    }
}
