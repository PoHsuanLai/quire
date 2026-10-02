//! The window's GPU device and the textures an app shows on it (effects: it creates textures and
//! writes to the queue, and asks windows to repaint).
//!
//! [`Gpu`] is the one device the window's renderer draws with, found where the renderer made it:
//! a mounted layer hands it over when the renderer first paints it, so it is `None` until the
//! window's first frame, and `use_gpu` re-renders its caller when it arrives. [`TextureHandle`]
//! is a cell the app writes a texture into and the layer reads from at each paint, so a new
//! frame needs no change to the document: write, then repaint.

use super::convert::premultiplied;
use super::model::{Pixels, Texels};
use blitz_traits::shell::ShellProvider;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use wgpu_context::DeviceHandle;

/// The texture format every upload uses and every layer expects: RGBA8, premultiplied, not
/// sRGB-tagged, because the renderer blends raw encoded values and reads a texel as premultiplied.
pub(crate) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// Why a texture could not be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuError {
    /// The window's renderer has not made its device yet (before the first frame), or its device
    /// went away. The caller of `use_gpu` re-renders when one arrives.
    NotReady,
    /// The picture is wider or taller than the device's largest texture.
    TooLarge {
        /// The most pixels along one side.
        max: Texels,
    },
}

impl fmt::Display for GpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuError::NotReady => f.write_str("the window has no GPU device yet"),
            GpuError::TooLarge { max } => write!(f, "a texture is at most {} pixels a side", max.0),
        }
    }
}

impl std::error::Error for GpuError {}

/// The parts of the renderer's device.
#[derive(Clone)]
struct Attached {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

/// Something to run when the device arrives or changes.
type Wake = Arc<dyn Fn() + Send + Sync>;

#[derive(Default)]
struct GpuState {
    attached: Option<Attached>,
    /// Counts devices: a texture made for an earlier one is not drawn on a later one.
    generation: u64,
    subscribers: Vec<(u64, Wake)>,
    next_subscriber: u64,
}

/// A window's GPU: the device and queue its renderer draws with, and the way to put textures on
/// them. Cheap to clone and `Send + Sync`: hand it to the thread that decodes or plays.
#[derive(Clone)]
pub struct Gpu {
    state: Arc<Mutex<GpuState>>,
}

impl fmt::Debug for Gpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = lock(&self.state);
        f.debug_struct("Gpu")
            .field("attached", &state.attached.is_some())
            .field("generation", &state.generation)
            .finish()
    }
}

impl PartialEq for Gpu {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.state, &other.state)
    }
}

/// A subscription made with [`Gpu::subscribe`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Subscription(u64);

impl Gpu {
    /// A GPU with no device yet.
    pub(crate) fn empty() -> Self {
        Gpu {
            state: Arc::default(),
        }
    }

    /// The device the window's renderer draws with, once it has made one.
    pub fn device(&self) -> Option<wgpu::Device> {
        lock(&self.state)
            .attached
            .as_ref()
            .map(|gpu| gpu.device.clone())
    }

    /// The queue of that device.
    pub fn queue(&self) -> Option<wgpu::Queue> {
        lock(&self.state)
            .attached
            .as_ref()
            .map(|gpu| gpu.queue.clone())
    }

    /// The adapter that device came from.
    pub fn adapter(&self) -> Option<wgpu::Adapter> {
        lock(&self.state)
            .attached
            .as_ref()
            .map(|gpu| gpu.adapter.clone())
    }

    /// The instance that adapter came from.
    pub fn instance(&self) -> Option<wgpu::Instance> {
        lock(&self.state)
            .attached
            .as_ref()
            .map(|gpu| gpu.instance.clone())
    }

    /// How many devices this window has had. A texture made on an earlier device is not drawn;
    /// an app that holds GPU resources made on `device()` rebuilds them when this changes.
    pub fn generation(&self) -> u64 {
        lock(&self.state).generation
    }

    /// A handle with no texture in it yet: a layer given it paints nothing until a texture is
    /// put in.
    pub fn handle(&self) -> TextureHandle {
        TextureHandle {
            inner: Arc::new(HandleInner {
                gpu: self.clone(),
                state: Mutex::default(),
            }),
        }
    }

    /// A handle showing `texture`, made on [`Gpu::device`]: a colour format the renderer samples
    /// as float and does not decode (`Rgba8Unorm`, `Bgra8Unorm`, `Rgba16Float`; never an sRGB
    /// format, which would be decoded to linear), premultiplied alpha, one mip level and
    /// `TEXTURE_BINDING` usage.
    pub fn register(&self, texture: wgpu::Texture) -> TextureHandle {
        let handle = self.handle();
        handle.replace(texture);
        handle
    }

    /// A handle showing the whole texture `view` looks at, under the same rules as
    /// [`Gpu::register`].
    pub fn register_view(&self, view: wgpu::TextureView) -> TextureHandle {
        let handle = self.handle();
        handle.replace_view(view);
        handle
    }

    /// A handle showing `pixels` uploaded to a new texture.
    pub fn upload(&self, pixels: &Pixels<'_>) -> Result<TextureHandle, GpuError> {
        let handle = self.handle();
        handle.update(pixels)?;
        Ok(handle)
    }

    /// Make `handle`'s device the window's. The same device again changes nothing; a different
    /// one starts a new generation and wakes every subscriber.
    pub(crate) fn attach(&self, handle: &DeviceHandle) {
        let wakes: Vec<Wake> = {
            let mut state = lock(&self.state);
            if state
                .attached
                .as_ref()
                .is_some_and(|gpu| gpu.device == handle.device)
            {
                return;
            }
            state.attached = Some(Attached {
                instance: handle.instance.clone(),
                adapter: handle.adapter.clone(),
                device: handle.device.clone(),
                queue: handle.queue.clone(),
            });
            state.generation += 1;
            state
                .subscribers
                .iter()
                .map(|(_, wake)| wake.clone())
                .collect()
        };
        for wake in wakes {
            wake();
        }
    }

    /// Run `wake` whenever the device arrives, changes or goes.
    pub(crate) fn subscribe(&self, wake: Wake) -> Subscription {
        let mut state = lock(&self.state);
        let id = state.next_subscriber;
        state.next_subscriber += 1;
        state.subscribers.push((id, wake));
        Subscription(id)
    }

    /// Stop running the wake `subscription` made.
    pub(crate) fn unsubscribe(&self, subscription: Subscription) {
        lock(&self.state)
            .subscribers
            .retain(|(id, _)| *id != subscription.0);
    }
}

/// A GPU on a device its owner already has: the headless harness's, which opens one before the
/// first frame.
pub fn attached_gpu(
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
) -> Gpu {
    let gpu = Gpu::empty();
    gpu.attach(&DeviceHandle {
        instance,
        adapter,
        device,
        queue,
    });
    gpu
}

/// Where a texture came from, which decides whether `update` may write into it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Origin {
    /// Made here by an upload: RGBA8 with `COPY_DST`, so a same-size update writes into it.
    Upload,
    /// Registered by the app: never written to from here.
    Registered,
}

/// What the renderer is given to draw.
#[derive(Debug, Clone)]
pub(crate) enum Source {
    Texture(wgpu::Texture),
    View(wgpu::TextureView),
}

#[derive(Debug, Clone)]
struct Content {
    source: Source,
    width: Texels,
    height: Texels,
    generation: u64,
    origin: Origin,
}

/// The texture at one moment, as a layer reads it at paint.
#[derive(Debug, Clone)]
pub(crate) struct Frame {
    /// Changes whenever the texture object does, so a layer registers it with the renderer
    /// again; writes into the same texture keep it.
    pub(crate) version: u64,
    pub(crate) source: Source,
    pub(crate) width: Texels,
    pub(crate) height: Texels,
}

#[derive(Default)]
struct HandleState {
    content: Option<Content>,
    version: u64,
    /// The windows showing this texture, and how many layers each has, to repaint when it changes.
    shells: Vec<(Arc<dyn ShellProvider>, usize)>,
}

struct HandleInner {
    gpu: Gpu,
    state: Mutex<HandleState>,
}

/// A texture an app shows, replaced or updated as often as it likes from any thread. Give it to
/// a [`TextureLayer`](super::TextureLayer); every layer showing it, in any window the
/// handle's `Gpu` belongs to, draws what is in it at its next paint.
///
/// Writing a new frame is: put the texture in (`replace`, `update`), or write into the one
/// already there on the queue, then `redraw`. The GPU commands that make the frame are
/// submitted before `redraw` is called, so the window's own submission orders after them.
#[derive(Clone)]
pub struct TextureHandle {
    inner: Arc<HandleInner>,
}

impl fmt::Debug for TextureHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = lock(&self.inner.state);
        f.debug_struct("TextureHandle")
            .field("version", &state.version)
            .field(
                "size",
                &state.content.as_ref().map(|c| (c.width.0, c.height.0)),
            )
            .finish()
    }
}

impl PartialEq for TextureHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl TextureHandle {
    /// The GPU this handle's textures are on.
    pub fn gpu(&self) -> &Gpu {
        &self.inner.gpu
    }

    /// The size of the texture in it, or `None` while it is empty (or made on a device that has
    /// gone).
    pub fn size(&self) -> Option<(Texels, Texels)> {
        self.frame().map(|frame| (frame.width, frame.height))
    }

    /// Show `texture` from now on, under the rules of [`Gpu::register`], and repaint.
    pub fn replace(&self, texture: wgpu::Texture) {
        let (width, height) = (Texels(texture.width()), Texels(texture.height()));
        self.put(Source::Texture(texture), width, height, Origin::Registered);
    }

    /// Show the whole texture `view` looks at from now on, and repaint.
    pub fn replace_view(&self, view: wgpu::TextureView) {
        let texture = view.texture();
        let (width, height) = (Texels(texture.width()), Texels(texture.height()));
        self.put(Source::View(view), width, height, Origin::Registered);
    }

    /// Show `pixels` from now on, and repaint. A picture of the size already shown is written
    /// into the same texture; another size makes a new one. Straight alpha is multiplied and RGB
    /// made opaque here (`PixelFormat`), so what the GPU holds is always premultiplied.
    pub fn update(&self, pixels: &Pixels<'_>) -> Result<(), GpuError> {
        let (device, queue, generation) = {
            let state = lock(&self.inner.gpu.state);
            let attached = state.attached.clone().ok_or(GpuError::NotReady)?;
            (attached.device, attached.queue, state.generation)
        };
        let max = Texels(device.limits().max_texture_dimension_2d);
        if pixels.width() > max || pixels.height() > max {
            return Err(GpuError::TooLarge { max });
        }
        let bytes = premultiplied(pixels);
        let reusable = {
            let state = lock(&self.inner.state);
            state
                .content
                .as_ref()
                .and_then(|content| match &content.source {
                    Source::Texture(texture)
                        if content.origin == Origin::Upload
                            && content.generation == generation
                            && content.width == pixels.width()
                            && content.height == pixels.height() =>
                    {
                        Some(texture.clone())
                    }
                    Source::Texture(_) | Source::View(_) => None,
                })
        };
        let (width, height) = (pixels.width().0, pixels.height().0);
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let (texture, fresh) = match reusable {
            Some(texture) => (texture, false),
            None => (
                device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("ds texture layer"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: FORMAT,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                }),
                true,
            ),
        };
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            size,
        );
        match fresh {
            true => self.put(
                Source::Texture(texture),
                pixels.width(),
                pixels.height(),
                Origin::Upload,
            ),
            false => self.redraw(),
        }
        Ok(())
    }

    /// Show nothing from now on, and repaint.
    pub fn clear(&self) {
        {
            let mut state = lock(&self.inner.state);
            state.content = None;
            state.version += 1;
        }
        self.redraw();
    }

    /// Ask every window showing this texture to repaint it. Many calls between two display
    /// refreshes make one paint, of whatever is in the texture then.
    pub fn redraw(&self) {
        let shells: Vec<Arc<dyn ShellProvider>> = lock(&self.inner.state)
            .shells
            .iter()
            .map(|(shell, _)| Arc::clone(shell))
            .collect();
        for shell in shells {
            shell.request_redraw();
        }
    }

    fn put(&self, source: Source, width: Texels, height: Texels, origin: Origin) {
        let generation = self.inner.gpu.generation();
        {
            let mut state = lock(&self.inner.state);
            state.content = Some(Content {
                source,
                width,
                height,
                generation,
                origin,
            });
            state.version += 1;
        }
        self.redraw();
    }

    /// The texture as a layer paints it, unless there is none or its device has gone.
    pub(crate) fn frame(&self) -> Option<Frame> {
        let generation = self.inner.gpu.generation();
        let state = lock(&self.inner.state);
        let content = state.content.as_ref()?;
        (content.generation == generation).then(|| Frame {
            version: state.version,
            source: content.source.clone(),
            width: content.width,
            height: content.height,
        })
    }

    /// A layer in `shell`'s window shows this texture: repaint it when the texture changes.
    pub(crate) fn watch(&self, shell: &Arc<dyn ShellProvider>) {
        let mut state = lock(&self.inner.state);
        match state
            .shells
            .iter_mut()
            .find(|(known, _)| Arc::ptr_eq(known, shell))
        {
            Some((_, layers)) => *layers += 1,
            None => state.shells.push((Arc::clone(shell), 1)),
        }
    }

    /// A layer that called [`TextureHandle::watch`] with `shell` is gone.
    pub(crate) fn unwatch(&self, shell: &Arc<dyn ShellProvider>) {
        let mut state = lock(&self.inner.state);
        if let Some(at) = state
            .shells
            .iter()
            .position(|(known, _)| Arc::ptr_eq(known, shell))
        {
            state.shells[at].1 -= 1;
            if state.shells[at].1 == 0 {
                state.shells.remove(at);
            }
        }
    }
}

/// A poisoned lock holds a plain value no panic could leave half-written.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
