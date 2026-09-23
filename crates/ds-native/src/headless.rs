//! One quire document with no window: the pieces `Harness` and `snapshot` share. It gets the
//! shared font context, the `data:`/`file:` net provider, sequential styling (deterministic, and
//! no rayon pool per test), the host's input modality as root context, and a waker to sleep on.

use crate::error::NativeError;
use crate::fonts::font_context;
use crate::net::DsNet;
use crate::scheme;
use crate::snapshot::Viewport;
use crate::wake::Wakeup;
use anyrender::{PaintScene as _, render_to_buffer};
use anyrender_vello_cpu::VelloCpuImageRenderer;
use blitz_dom::{Document as _, DocumentConfig, StyleThreading};
use blitz_paint::paint_scene;
use blitz_traits::net::NetWaker;
use blitz_traits::shell::{ColorScheme, Viewport as BlitzViewport};
use dioxus::prelude::*;
use dioxus_native_dom::DioxusDocument;
use ds::{HostModality, InputModality};
use peniko::kurbo::{Affine, Rect};
use peniko::{Color, Fill};
use std::sync::Arc;
use std::task::{Context, Waker};
use std::time::Duration;

/// More rounds than this without the document settling is a render loop in the app.
const MAX_ROUNDS: usize = 64;

/// A headless document and what drives it.
pub(crate) struct Headless {
    pub(crate) doc: DioxusDocument,
    modality: Signal<InputModality>,
    wakeup: Arc<Wakeup>,
    viewport: Viewport,
}

impl Headless {
    /// Build `app` at `viewport` and run its first render (no layout yet).
    pub(crate) fn new(app: fn() -> Element, viewport: Viewport) -> Self {
        let wakeup = Arc::new(Wakeup::default());
        let fetches = Arc::clone(&wakeup);
        let net_waker: Arc<dyn NetWaker> = Arc::new(move |_doc: usize| fetches.note_fetch());
        let config = DocumentConfig {
            viewport: Some(blitz_viewport(viewport)),
            font_ctx: Some(font_context()),
            net_provider: Some(DsNet::shared(None, Some(net_waker))),
            style_threading: StyleThreading::Sequential,
            ..Default::default()
        };
        let vdom = VirtualDom::new(app);
        let modality =
            vdom.in_runtime(|| Signal::new_in_scope(InputModality::default(), ScopeId::ROOT));
        vdom.provide_root_context(HostModality(modality));
        let mut doc = DioxusDocument::new(vdom, config);
        doc.initial_build();
        Headless {
            doc,
            modality,
            wakeup,
            viewport,
        }
    }

    /// What wakes this document.
    pub(crate) fn wakeup(&self) -> &Arc<Wakeup> {
        &self.wakeup
    }

    /// Record the kind of input that arrived last, so `Ds` stamps `data-modality`.
    pub(crate) fn set_modality(&mut self, next: InputModality) {
        let mut modality = self.modality;
        self.doc.vdom.in_runtime(|| {
            if *modality.peek() != next {
                modality.set(next);
            }
        });
    }

    /// Run every render the document has queued; whether there was any.
    fn flush(&mut self) -> bool {
        let waker = Waker::from(Arc::clone(&self.wakeup));
        (0..MAX_ROUNDS)
            .take_while(|_| self.doc.poll(Some(Context::from_waker(&waker))))
            .count()
            > 0
    }

    /// Bring the document to animation time `at`: render what is queued, follow the root's
    /// scheme, style and lay out, and go round again while a round produced more work (an image
    /// that landed during styling is applied on the next resolve, spike S7).
    pub(crate) fn frame(&mut self, at: Duration) {
        for _ in 0..MAX_ROUNDS {
            let rendered = self.flush();
            let restyled = scheme::follow_root(&mut self.doc.inner.borrow_mut()).is_some();
            let fetched = self.wakeup.fetched();
            self.doc.inner.borrow_mut().resolve(at.as_secs_f64());
            let landed = self.wakeup.fetched() != fetched;
            if !(rendered || restyled || landed) {
                return;
            }
        }
    }

    /// Paint the document as it was last resolved.
    pub(crate) fn paint(&mut self) -> Result<image::RgbaImage, NativeError> {
        let (width, height) = physical(self.viewport);
        let scale = scale(self.viewport);
        let mut inner = self.doc.inner.borrow_mut();
        let ground = match inner.viewport().color_scheme {
            ColorScheme::Dark => Color::BLACK,
            ColorScheme::Light => Color::WHITE,
        };
        let canvas = Rect::new(0.0, 0.0, f64::from(width), f64::from(height));
        let pixels = render_to_buffer::<VelloCpuImageRenderer, _>(
            |scene| {
                scene.fill(Fill::NonZero, Affine::IDENTITY, ground, None, &canvas);
                paint_scene(scene, &mut inner, scale, width, height, 0, 0);
            },
            width,
            height,
        );
        let length = pixels.len();
        image::RgbaImage::from_raw(width, height, pixels).ok_or_else(|| {
            NativeError::Renderer(format!(
                "vello_cpu returned {length} bytes for {width}x{height}"
            ))
        })
    }
}

/// Device pixels per logical pixel.
fn scale(viewport: Viewport) -> f64 {
    f64::from(viewport.scale_percent) / 100.0
}

/// The viewport in device pixels.
fn physical(viewport: Viewport) -> (u32, u32) {
    let device = |logical: u32| (f64::from(logical) * scale(viewport)).round() as u32;
    (device(viewport.width), device(viewport.height))
}

/// Blitz's viewport for ours, light until the root says otherwise.
fn blitz_viewport(viewport: Viewport) -> BlitzViewport {
    let (width, height) = physical(viewport);
    BlitzViewport::new(width, height, scale(viewport) as f32, ColorScheme::Light)
}

#[cfg(test)]
mod tests {
    use super::physical;
    use crate::snapshot::Viewport;

    /// A logical size and scale, and the device pixels it renders at.
    type Case = (Viewport, (u32, u32));

    const fn at(width: u32, height: u32, scale_percent: u16) -> Viewport {
        Viewport {
            width,
            height,
            scale_percent,
        }
    }

    const CASES: &[Case] = &[
        (at(400, 300, 100), (400, 300)),
        (at(400, 300, 200), (800, 600)),
        (at(401, 301, 150), (602, 452)),
    ];

    #[test]
    fn device_pixels_follow_the_scale() {
        for &(viewport, expected) in CASES {
            assert_eq!(physical(viewport), expected, "{viewport:?}");
        }
    }
}
