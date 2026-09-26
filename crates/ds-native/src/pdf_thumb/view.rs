//! `PdfFileThumb`: a path and a size in, `ds::PdfThumb` out.

use super::cache::pdf_thumb_cached;
use super::request::ThumbRequest;
use super::worker::{Slot, ask};
use dioxus::core::Task;
use dioxus::prelude::*;
use ds::{PdfPage, PdfThumb, Size, use_scale};
use std::path::PathBuf;

/// A page that came back from the worker, for the request it answers.
#[derive(Debug, Clone, PartialEq)]
struct Arrived {
    request: ThumbRequest,
    page: PdfPage,
}

/// The first page of the PDF at `path`, fitted into `size` on a paper sheet (`ds::PdfThumb`).
/// A page already cached for the file as it is now (same modification time, device size and
/// scale) draws at once; otherwise the request joins the one `pdf-thumb` worker's queue, the
/// thumbnail loading meanwhile (nothing for 400 ms, then the pending look). A new path or size
/// replaces this thumbnail's queued request that has not started; one already running
/// finishes into the cache but is not shown. `label` is read by a screen reader (the file's name, say).
#[component]
pub fn PdfFileThumb(path: PathBuf, size: Size, #[props(default)] label: Option<String>) -> Element {
    let request = ThumbRequest {
        path,
        size,
        scale: use_scale(),
    };
    let slot = use_hook(Slot::fresh);
    let arrived = use_signal(|| None::<Arrived>);
    let mut running = use_hook(|| CopyValue::new(None::<Task>));
    let mut seen = use_hook(|| CopyValue::new(None::<ThumbRequest>));
    let fresh = seen.peek().as_ref() != Some(&request);
    if fresh {
        seen.set(Some(request.clone()));
    }
    let landed = arrived
        .read()
        .as_ref()
        .filter(|arrived| arrived.request == request)
        .map(|arrived| arrived.page.clone());
    let page = match landed.or_else(|| pdf_thumb_cached(&request)) {
        Some(page) => page,
        None => {
            if fresh {
                if let Some(previous) = running.take() {
                    previous.cancel();
                }
                running.set(Some(spawn(read(slot, request, arrived))));
            }
            PdfPage::Loading
        }
    };
    rsx! {
        PdfThumb { page, size, label }
    }
}

/// Ask the worker for the page, again if a full queue pushed the job out, and hand it back.
async fn read(slot: Slot, request: ThumbRequest, mut arrived: Signal<Option<Arrived>>) {
    let page = loop {
        if let Ok(page) = ask(slot, request.clone()).await {
            break page;
        }
    };
    if let Ok(mut landed) = arrived.try_write() {
        *landed = Some(Arrived { request, page });
    }
}
