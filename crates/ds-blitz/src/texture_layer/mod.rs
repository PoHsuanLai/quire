//! Show a GPU texture inside the document (`TextureLayer`), and give the app the window's device
//! to make it on (`use_gpu`). This is how a viewer shows a decoded image, a PDF page tile a CPU
//! rasteriser produced, or a video frame a player wrote into a texture.
//!
//! ```ignore
//! let gpu = use_gpu();
//! let frame = use_hook(|| gpu.handle());            // empty; a layer shows it once it has a texture
//! // on any thread, once per decoded frame:
//! frame.update(&Pixels::new(PixelFormat::Rgba8Premultiplied, w, h, &bytes)?)?;   // or
//! frame.replace(texture);  /* or write into the one it holds on gpu.queue() */  frame.redraw();
//! rsx! { TextureLayer { texture: frame.clone(), fit: TextureFit::Contain } }
//! ```
//!
//! # Design
//!
//! **Mechanism: a Blitz custom widget and a registered texture.** `TextureLayer` renders an
//! `<object data=...>` whose attribute is a `blitz_dom::Widget`. When the renderer paints, Blitz
//! asks the widget for a scene; the widget registers the app's `wgpu::Texture` with the renderer
//! (`RenderContext::try_register_custom_resource`, which anyrender_vello_hybrid keeps as a
//! `ResourceId -> TextureView` binding) and records a fill whose paint is that resource. The
//! renderer binds the view and samples it in the same pass as the rest of the document, so the
//! texture takes part in layers, clips, opacity and paint order, and nothing is copied. No change
//! to the Blitz fork or to anyrender was needed. Two limits of the renderer's texture paint are
//! worked around here instead: it always draws the whole texture at its own size (`fit` and the
//! source rectangle are an affine transform plus a clip, `fit`), and it ignores extend modes
//! (tiling is repeated draws, capped).
//!
//! **The device.** The window's renderer makes its wgpu device privately. A widget is handed the
//! renderer's context when the renderer first paints it (`can_create_surfaces`), and that context
//! is the renderer's own `DeviceHandle`; the widget passes it to the window's [`Gpu`]. `Host`
//! mounts a one-pixel transparent layer (`GpuProbe`) so this happens on the first frame even
//! before an app has a layer; `use_gpu` returns the `Gpu` and re-renders its caller when the
//! device arrives. Only ds-blitz names wgpu: `ds` components never see a device, they take
//! whatever element the app renders around a `TextureLayer`. One `Gpu` per window: each window's
//! renderer has its own device, and a texture made on one is not drawn in another.
//!
//! **Headless.** The harness's hybrid backend (`ds_harness::Backend::Hybrid`) paints through
//! the same widget path on one offscreen device, and installs a `Gpu` already attached to it at
//! the root, so a test makes textures on `Harness::gpu()` before the first frame and reads the
//! pixels back. The CPU painter has no device: `Gpu::device` stays `None` and a layer draws
//! nothing.
//!
//! **Alpha.** The GPU holds RGBA8 `Rgba8Unorm`, premultiplied, and the renderer composites it as
//! premultiplied over what is beneath. `Pixels` states its layout: `Rgba8Premultiplied` (pdfrum's
//! `Pixmap`) is uploaded as is; `Rgb8` (a video frame without alpha, what a player gives) is made
//! opaque; `Rgba8Straight` is multiplied by its alpha on the CPU, rounding to nearest. A texture
//! the app registers itself must already be premultiplied, in a non-sRGB colour format
//! (`Rgba8Unorm`, `Bgra8Unorm`, `Rgba16Float`: an sRGB format would be decoded to linear before
//! the renderer's blending, which works on encoded values) and carry `TEXTURE_BINDING`; a video
//! player's output is registered, a decoder's pixels are uploaded.
//!
//! **Frame pacing and redraws.** Painting is the window's: it paints when something asks. A new
//! frame is "write the texture, then `TextureHandle::redraw`", which asks each window showing
//! the handle (`ShellProvider::request_redraw`, thread-safe) for a paint; winit coalesces
//! requests to one paint per display refresh, and the paint reads the texture as it is then, so
//! a player faster than the display never queues frames. `replace` and `update` redraw by
//! themselves. A layer's own props changing repaint too. `Pace::EveryFrame` makes the layer
//! report that it needs a frame always, for a texture something writes without calling `redraw`,
//! at the cost of a window that never idles. GPU work that writes the texture is submitted
//! before `redraw`, so the window's own submission orders after it.
//!
//! **Lifetimes and resizing.** A `TextureHandle` owns the texture it was given; the renderer's
//! registration lives as long as the layer shows it, is replaced when the handle's texture
//! changes (an in-place write keeps it), and is dropped when the layer's node is removed (Blitz
//! unregisters a removed widget's resources) or the surface is destroyed. A layer is sized by
//! its CSS box and re-fits at every paint, so resizing the window or the box needs nothing from
//! the app; a texture of another size (`update` with another picture size, `replace`) re-fits
//! the same way. If the renderer ever gives the window a new device (`Gpu::generation` changes)
//! textures made on the old one stop drawing and the app makes them again.
//!
//! The layer is one device pixel per texel at `Actual` and `Tile`, in device pixels (the box's
//! size times the window's scale), and the picture is sampled at the box's device size.

mod convert;
mod fit;
mod gpu;
mod model;
mod view;
mod watch;
mod widget;

pub use gpu::{Gpu, GpuError, TextureHandle, attached_gpu};
pub use model::{Pace, PixelFormat, Pixels, PixelsError, Sampling, TexelRect, Texels, TextureFit};
pub(crate) use view::GpuProbe;
pub use view::{TextureLayer, use_gpu};
