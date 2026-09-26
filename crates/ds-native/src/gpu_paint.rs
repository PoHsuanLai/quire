//! A hybrid harness's painter: the document painted through anyrender_vello_hybrid, the
//! renderer shell surfaces use, into an offscreen texture on one wgpu device kept for the
//! harness's life. The renderer, its resources (glyph caches, image atlas) and the uploaded
//! images live across frames, as in a window. Every frame is submitted and waited for, so a
//! frame's time includes the GPU finishing it.

use crate::error::NativeError;
use crate::gpu_adapter::{AdapterPref, open_device};
use crate::painter::{Canvas, PaintTime, draw};
use anyrender::ResourceId;
use anyrender_vello_hybrid::{ImageManager, VelloHybridScenePainter};
use blitz_dom::BaseDocument;
use rustc_hash::FxHashMap;
use std::time::{Duration, Instant};
use vello_common::paint::ImageId;
use vello_hybrid::{RenderSize, RenderTargetConfig, Renderer, Resources, Scene, TextureBindings};
use wgpu_context::DeviceHandle;

/// The target format: premultiplied RGBA bytes on readback, the layout vello_cpu writes.
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// vello_hybrid on one offscreen device.
pub(crate) struct GpuPainter {
    handle: DeviceHandle,
    adapter: String,
    renderer: Renderer,
    resources: Resources,
    scene: Scene,
    /// Uploaded images by content hash, kept across frames as shell-host keeps them.
    images: FxHashMap<u64, ImageId>,
    /// Textures the document registered (a canvas), by anyrender resource.
    bindings: FxHashMap<ResourceId, wgpu::TextureView>,
    target: Target,
}

impl std::fmt::Debug for GpuPainter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuPainter")
            .field("adapter", &self.adapter)
            .finish_non_exhaustive()
    }
}

/// The texture painted into and the buffer it is read back through.
struct Target {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    readback: wgpu::Buffer,
    width: u32,
    height: u32,
    /// A readback row, padded to wgpu's copy alignment.
    row_bytes: u32,
}

impl Target {
    fn new(device: &wgpu::Device, width: u32, height: u32) -> Target {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("harness target"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let row_bytes = (width * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("harness readback"),
            size: u64::from(row_bytes) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Target {
            texture,
            view,
            readback,
            width,
            height,
            row_bytes,
        }
    }
}

impl GpuPainter {
    /// A device on the preferred adapter and a renderer for `width` x `height` device pixels.
    pub(crate) fn open(pref: &AdapterPref, width: u32, height: u32) -> Result<Self, NativeError> {
        let (handle, adapter) = open_device(pref)?;
        let renderer = Renderer::new(
            &handle.device,
            &RenderTargetConfig {
                format: FORMAT,
                width,
                height,
            },
        );
        let target = Target::new(&handle.device, width, height);
        Ok(GpuPainter {
            renderer,
            resources: Resources::new(),
            scene: Scene::new(width as u16, height as u16),
            images: FxHashMap::default(),
            bindings: FxHashMap::default(),
            target,
            handle,
            adapter,
        })
    }

    /// The adapter painting: `name (backend)`.
    pub(crate) fn adapter(&self) -> &str {
        &self.adapter
    }

    /// Paint `doc` and wait for the GPU to finish, without reading it back.
    pub(crate) fn time(
        &mut self,
        doc: &mut BaseDocument,
        canvas: Canvas,
    ) -> Result<PaintTime, NativeError> {
        let started = Instant::now();
        let (encoder, encode) = self.encode(doc, canvas)?;
        self.finish(encoder)?;
        Ok(PaintTime::new(encode, started.elapsed()))
    }

    /// Paint `doc` and read the pixels back: premultiplied RGBA, row by row.
    pub(crate) fn picture(
        &mut self,
        doc: &mut BaseDocument,
        canvas: Canvas,
    ) -> Result<Vec<u8>, NativeError> {
        let (mut encoder, _) = self.encode(doc, canvas)?;
        let target = &self.target;
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &target.readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(target.row_bytes),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: target.width,
                height: target.height,
                depth_or_array_layers: 1,
            },
        );
        let slice = self.target.readback.slice(..);
        self.handle.queue.submit([encoder.finish()]);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.wait()?;
        let row = (self.target.width * 4) as usize;
        let pixels = slice
            .get_mapped_range()
            .chunks_exact(self.target.row_bytes as usize)
            .flat_map(|padded| padded[..row].iter().copied())
            .collect();
        self.target.readback.unmap();
        Ok(pixels)
    }

    /// Build this frame's scene from `doc` and record its render; the time the scene took.
    fn encode(
        &mut self,
        doc: &mut BaseDocument,
        canvas: Canvas,
    ) -> Result<(wgpu::CommandEncoder, Duration), NativeError> {
        let started = Instant::now();
        let device = &self.handle.device;
        if (self.target.width, self.target.height) != (canvas.width, canvas.height) {
            self.target = Target::new(device, canvas.width, canvas.height);
            self.scene = Scene::new(canvas.width as u16, canvas.height as u16);
        }
        self.scene.reset();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("harness frame"),
        });
        {
            let images = ImageManager::new(
                &mut self.renderer,
                &mut self.resources,
                device,
                &self.handle.queue,
                &mut encoder,
                &mut self.images,
            );
            let mut painter = VelloHybridScenePainter::new(
                &mut self.scene,
                images,
                &mut self.bindings,
                &self.handle,
            );
            draw(&mut painter, doc, canvas);
        }
        let encoded = started.elapsed();
        let mut textures = TextureBindings::new();
        for (resource, view) in &self.bindings {
            textures.insert(vello_hybrid::TextureId(resource.into_ffi()), view.clone());
        }
        self.renderer
            .render(
                &self.scene,
                &mut self.resources,
                device,
                &self.handle.queue,
                &mut encoder,
                &RenderSize {
                    width: canvas.width,
                    height: canvas.height,
                },
                &self.target.view,
                &textures,
            )
            .map_err(|error| NativeError::Renderer(format!("vello_hybrid: {error:?}")))?;
        Ok((encoder, encoded))
    }

    /// Submit `encoder` and wait until the GPU has run it.
    fn finish(&mut self, encoder: wgpu::CommandEncoder) -> Result<(), NativeError> {
        self.handle.queue.submit([encoder.finish()]);
        self.wait()
    }

    fn wait(&self) -> Result<(), NativeError> {
        self.handle
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map(|_| ())
            .map_err(|error| NativeError::Renderer(format!("wgpu poll: {error}")))
    }
}
