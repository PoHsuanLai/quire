//! `PdfFileThumb`: a path and a size in, `ds::PdfThumb` out.

use super::cache::pdf_thumb_cached;
use super::request::{ThumbRequest, pdf_thumb_blocking};
use dioxus::core::Task;
use dioxus::prelude::*;
use ds::{PdfPage, PdfThumb, PdfTrouble, Size, use_scale};
use std::path::PathBuf;

/// A page that came back from the worker, for the request it answers.
#[derive(Debug, Clone, PartialEq)]
struct Arrived {
    request: ThumbRequest,
    page: PdfPage,
}

/// The first page of the PDF at `path`, fitted into `size` on a paper sheet (`ds::PdfThumb`).
/// A page already cached for the file as it is now (same modification time, device size and
/// scale) draws at once; otherwise the file is read and rasterised on a worker thread, the
/// thumbnail loading meanwhile (nothing for 400 ms, then the pending look). A new path or size
/// starts a new read; one still running for the old request finishes into the cache but is not
/// shown. `label` is read by a screen reader (the file's name, say).
#[component]
pub fn PdfFileThumb(path: PathBuf, size: Size, #[props(default)] label: Option<String>) -> Element {
    let request = ThumbRequest {
        path,
        size,
        scale: use_scale(),
    };
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
                running.set(Some(spawn(read(request, arrived))));
            }
            PdfPage::Loading
        }
    };
    rsx! {
        PdfThumb { page, size, label }
    }
}

/// Rasterise on a worker thread and hand the page back to the scope.
async fn read(request: ThumbRequest, mut arrived: Signal<Option<Arrived>>) {
    let (send, receive) = tokio::sync::oneshot::channel();
    let job = request.clone();
    let started = std::thread::Builder::new()
        .name("pdf-thumb".to_owned())
        .spawn(move || {
            let _ = send.send(pdf_thumb_blocking(&job));
        });
    let page = match started {
        Ok(_) => receive
            .await
            .unwrap_or(PdfPage::Failed(PdfTrouble::Unreadable)),
        Err(_) => PdfPage::Failed(PdfTrouble::Unreadable),
    };
    if let Ok(mut slot) = arrived.try_write() {
        *slot = Some(Arrived { request, page });
    }
}
