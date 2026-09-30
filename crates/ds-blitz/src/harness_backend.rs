//! The harness's renderer: vello_cpu by default, or vello_hybrid on a GPU, the renderer shell
//! surfaces use, so a test can see and time what a surface would paint. Pixel assertions and
//! PNG captures read the same premultiplied RGBA from either.

use crate::error::NativeError;
use crate::gpu_paint::GpuPainter;
use crate::harness::Harness;
use crate::harness_config::HarnessConfig;
use crate::headless::{Backdrop, physical};
use crate::painter::{PaintTime, Painter};
use dioxus::prelude::Element;

/// Which renderer a [`Harness`] paints with ([`HarnessConfig::with_backend`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Backend {
    /// vello_cpu, as the headless path always has: no GPU needed.
    #[default]
    Cpu,
    /// anyrender_vello_hybrid on an offscreen wgpu device, opened once when the harness is
    /// built, on the adapter [`HarnessConfig::with_adapter`] and `WGPU_ADAPTER_NAME` pick.
    /// Where no device opens (CI), [`Harness::try_with_config`] returns the error and a harness
    /// built with [`Harness::with_config`] returns it from every picture instead: never a
    /// panic.
    Hybrid,
}

/// The painter `config` asks for; a hybrid one that cannot open says why when it is used.
pub(crate) fn painter(config: &HarnessConfig) -> Painter {
    match config.backend() {
        Backend::Cpu => Painter::Cpu,
        Backend::Hybrid => {
            let (width, height) = physical(config.viewport());
            match GpuPainter::open(config.adapter(), width, height) {
                Ok(gpu) => Painter::Gpu(Box::new(gpu)),
                Err(error) => Painter::Unavailable(error.to_string()),
            }
        }
    }
}

impl Harness {
    /// Build `app` as `config` says and render its first frame, or the error that stops its
    /// renderer: a [`Backend::Hybrid`] harness with no GPU device to open, which a test skips
    /// on.
    pub fn try_with_config(
        app: fn() -> Element,
        config: HarnessConfig,
    ) -> Result<Self, NativeError> {
        let harness = Harness::with_config(app, config);
        match &harness.doc.painter {
            Painter::Unavailable(why) => Err(NativeError::Renderer(why.clone())),
            Painter::Cpu | Painter::Gpu(_) => Ok(harness),
        }
    }

    /// Paint the document as it is now over the scheme's ground, to the end, and say how long
    /// it took. Nothing is read back. On vello_cpu the frame ends with the pixels in memory; on
    /// vello_hybrid the frame is submitted and the device polled until the GPU has finished
    /// it, so the time is the whole frame's, GPU included, never just the recording. The
    /// document is not resolved here: style and layout (a scroll, a click) happen in the input
    /// call before, so time that separately.
    pub fn paint_timed(&mut self) -> Result<PaintTime, NativeError> {
        self.doc.paint_timed(Backdrop::Scheme)
    }

    /// The GPU adapter painting, `name (backend)`, or `None` on vello_cpu.
    pub fn adapter(&self) -> Option<&str> {
        self.doc.painter.adapter()
    }
}
