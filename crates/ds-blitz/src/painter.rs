//! What paints a headless document: vello_cpu (the default, a fresh renderer per picture) or
//! vello_hybrid on a GPU (`crate::gpu_paint`), chosen by
//! [`HarnessConfig::with_backend`](crate::HarnessConfig::with_backend). Both paint the same
//! scene, [`draw`], and hand back the same premultiplied RGBA bytes.

use crate::error::NativeError;
use crate::gpu_paint::GpuPainter;
use anyrender::{PaintScene, render_to_buffer};
use anyrender_vello_cpu::VelloCpuImageRenderer;
use blitz_dom::BaseDocument;
use blitz_paint::paint_scene;
use peniko::kurbo::{Affine, Rect};
use peniko::{Color, Fill};
use std::time::{Duration, Instant};

/// A frame's size in device pixels, its scale, and the ground under the document.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Canvas {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) scale: f64,
    pub(crate) ground: Color,
}

/// The ground, then the document as it was last resolved.
pub(crate) fn draw(scene: &mut impl PaintScene, doc: &mut BaseDocument, canvas: Canvas) {
    let area = Rect::new(0.0, 0.0, f64::from(canvas.width), f64::from(canvas.height));
    scene.fill(Fill::NonZero, Affine::IDENTITY, canvas.ground, None, &area);
    paint_scene(scene, doc, canvas.scale, canvas.width, canvas.height, 0, 0);
}

/// How long one painted frame took, from [`Harness::paint_timed`](crate::Harness::paint_timed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PaintTime {
    /// Building the scene: Blitz walking the document into the renderer's painter (on
    /// vello_hybrid this includes flattening paths and glyphs into strips on the CPU).
    pub scene: Duration,
    /// The whole frame: the scene, then rasterising it (vello_cpu) or recording, submitting
    /// and waiting for the GPU to finish it (vello_hybrid).
    pub total: Duration,
}

impl PaintTime {
    pub(crate) fn new(scene: Duration, total: Duration) -> Self {
        PaintTime { scene, total }
    }

    /// The part after the scene: rasterising, or the GPU's work and the wait for it.
    pub fn render(&self) -> Duration {
        self.total.saturating_sub(self.scene)
    }
}

/// The renderer a headless document paints with.
#[derive(Debug, Default)]
pub(crate) enum Painter {
    /// vello_cpu, a renderer per picture.
    #[default]
    Cpu,
    /// vello_hybrid on one offscreen device.
    Gpu(Box<GpuPainter>),
    /// A hybrid backend was asked for and no GPU device could be opened: why.
    Unavailable(String),
}

impl Painter {
    /// Paint `doc` into premultiplied RGBA bytes.
    pub(crate) fn picture(
        &mut self,
        doc: &mut BaseDocument,
        canvas: Canvas,
    ) -> Result<Vec<u8>, NativeError> {
        match self {
            Painter::Cpu => Ok(render_to_buffer::<VelloCpuImageRenderer, _>(
                |scene| draw(scene, doc, canvas),
                canvas.width,
                canvas.height,
            )),
            Painter::Gpu(gpu) => gpu.picture(doc, canvas),
            Painter::Unavailable(why) => Err(NativeError::Renderer(why.clone())),
        }
    }

    /// Paint `doc` to the end (the pixels in memory, or the GPU idle) and say how long it took.
    pub(crate) fn time(
        &mut self,
        doc: &mut BaseDocument,
        canvas: Canvas,
    ) -> Result<PaintTime, NativeError> {
        match self {
            Painter::Cpu => {
                let started = Instant::now();
                let mut scene_done = None;
                let pixels = render_to_buffer::<VelloCpuImageRenderer, _>(
                    |scene| {
                        draw(scene, doc, canvas);
                        scene_done = Some(started.elapsed());
                    },
                    canvas.width,
                    canvas.height,
                );
                let total = started.elapsed();
                drop(pixels);
                Ok(PaintTime::new(scene_done.unwrap_or(total), total))
            }
            Painter::Gpu(gpu) => gpu.time(doc, canvas),
            Painter::Unavailable(why) => Err(NativeError::Renderer(why.clone())),
        }
    }

    /// The adapter a GPU painter runs on, if this is one.
    pub(crate) fn adapter(&self) -> Option<&str> {
        match self {
            Painter::Gpu(gpu) => Some(gpu.adapter()),
            Painter::Cpu | Painter::Unavailable(_) => None,
        }
    }
}
