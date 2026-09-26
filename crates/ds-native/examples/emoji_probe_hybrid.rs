//! The colour-emoji probe, per backend (FINDINGS "Colour emoji"): the same glyph runs, 😀 and 🎉
//! at 48 px from the installed Noto Color Emoji COLRv1 file and (when `EMOJI_CBDT` names one)
//! from the CBDT build, painted through anyrender once by vello_cpu (the headless path) and once
//! by vello_hybrid on the GPU (the window path), then measured as `emoji_probe` measures: inked
//! pixels and coloured ones. Blitz hands every text run to anyrender's `draw_glyphs` exactly as
//! this does, so what differs here is only the backend. Glyph ids come from the fonts' cmaps;
//! ZWJ sequences and flags need shaping, which `emoji_probe` covers on the CPU.
//!
//! `cargo run -p ds-native --example emoji_probe_hybrid -- out-dir`

use anyrender::{Glyph, PaintScene, render_to_buffer};
use anyrender_vello_cpu::VelloCpuImageRenderer;
use anyrender_vello_hybrid::{ImageManager, VelloHybridScenePainter};
use peniko::kurbo::{Affine, Rect, Vec2};
use peniko::{Blob, Color, Fill, FontData};
use rustc_hash::FxHashMap;
use skrifa::MetadataProvider;
use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

const COLRV1: &str = "/usr/share/fonts/google-noto-color-emoji-fonts/Noto-COLRv1.ttf";
const EMOJI: [char; 2] = ['😀', '🎉'];
const WIDTH: u32 = 240;
const HEIGHT: u32 = 160;
const ROW: f64 = 72.0;

/// A font to probe: its row label, its data, and its glyph for each emoji.
struct Face {
    label: &'static str,
    font: FontData,
    glyphs: Vec<u32>,
}

fn face(label: &'static str, path: &str) -> Option<Face> {
    let bytes = std::fs::read(path).ok()?;
    let glyphs = {
        let font = skrifa::FontRef::new(&bytes).ok()?;
        let charmap = font.charmap();
        EMOJI
            .iter()
            .map(|c| charmap.map(*c).map_or(0, |g| g.to_u32()))
            .collect()
    };
    Some(Face {
        label,
        font: FontData::new(Blob::new(Arc::new(bytes)), 0),
        glyphs,
    })
}

fn draw(scene: &mut impl PaintScene, faces: &[Face]) {
    let canvas = Rect::new(0.0, 0.0, f64::from(WIDTH), f64::from(HEIGHT));
    scene.fill(Fill::NonZero, Affine::IDENTITY, Color::WHITE, None, &canvas);
    for (row, face) in faces.iter().enumerate() {
        let baseline = ROW * row as f64 + 56.0;
        let glyphs = face.glyphs.iter().enumerate().map(|(i, id)| Glyph {
            id: *id,
            x: 16.0 + 88.0 * i as f32,
            y: 0.0,
        });
        scene.draw_glyphs(
            &face.font,
            48.0,
            false,
            &[],
            Vec2::ZERO,
            Fill::NonZero,
            Color::BLACK,
            1.0,
            Affine::translate((0.0, baseline)),
            None,
            glyphs,
        );
    }
}

/// Inked and coloured pixels in each 88 x 72 cell of straight RGBA `pixels`.
fn measure(pixels: &[u8], row: usize, cell: usize) -> (u32, u32) {
    let (x0, y0) = (cell as u32 * 88, row as u32 * ROW as u32);
    let mut inked = 0;
    let mut coloured = 0;
    for y in y0..(y0 + ROW as u32).min(HEIGHT) {
        for x in x0..(x0 + 88).min(WIDTH) {
            let at = ((y * WIDTH + x) * 4) as usize;
            let (r, g, b) = (pixels[at], pixels[at + 1], pixels[at + 2]);
            if r.min(g).min(b) < 235 {
                inked += 1;
                if r.max(g).max(b) - r.min(g).min(b) > 60 {
                    coloured += 1;
                }
            }
        }
    }
    (inked, coloured)
}

struct Unpark(std::thread::Thread);

impl Wake for Unpark {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

fn block_on<T>(future: impl Future<Output = T>) -> T {
    let waker = Waker::from(Arc::new(Unpark(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut future = pin!(future);
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
            return value;
        }
        std::thread::park();
    }
}

fn hybrid(faces: &[Face]) -> Result<Vec<u8>, String> {
    let context = wgpu_context::WGPUContext::new();
    let handle = block_on(context.create_device_handle(None)).map_err(|e| format!("{e:?}"))?;
    let (device, queue) = (&handle.device, &handle.queue);
    println!("hybrid adapter: {:?}", handle.adapter.get_info().name);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("probe"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut renderer = vello_hybrid::Renderer::new(
        device,
        &vello_hybrid::RenderTargetConfig {
            format: texture.format(),
            width: WIDTH,
            height: HEIGHT,
        },
    );
    let mut resources = vello_hybrid::Resources::new();
    let mut scene = vello_hybrid::Scene::new(WIDTH as u16, HEIGHT as u16);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    let mut images = FxHashMap::default();
    let mut bindings = FxHashMap::default();
    {
        let manager = ImageManager::new(
            &mut renderer,
            &mut resources,
            device,
            queue,
            &mut encoder,
            &mut images,
        );
        let mut painter = VelloHybridScenePainter::new(&mut scene, manager, &mut bindings, &handle);
        draw(&mut painter, faces);
    }
    renderer
        .render(
            &scene,
            &mut resources,
            device,
            queue,
            &mut encoder,
            &vello_hybrid::RenderSize {
                width: WIDTH,
                height: HEIGHT,
            },
            &view,
            &vello_hybrid::TextureBindings::new(),
        )
        .map_err(|e| format!("{e:?}"))?;
    let row_bytes = (WIDTH * 4).next_multiple_of(256);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: u64::from(row_bytes) * u64::from(HEIGHT),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row_bytes),
                rows_per_image: None,
            },
        },
        wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    buffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|e| format!("{e:?}"))?;
    let mapped = buffer.slice(..).get_mapped_range();
    let pixels = mapped
        .chunks_exact(row_bytes as usize)
        .flat_map(|row| row[..(WIDTH * 4) as usize].to_vec())
        .collect();
    Ok(pixels)
}

fn report(backend: &str, pixels: &[u8], faces: &[Face], out: &str) {
    for (row, face) in faces.iter().enumerate() {
        let cells: Vec<String> = (0..EMOJI.len())
            .map(|cell| {
                let (inked, coloured) = measure(pixels, row, cell);
                format!("{inked:>7}/{coloured:<6}")
            })
            .collect();
        println!("{backend:<7} {:<7} {}", face.label, cells.join(""));
    }
    let path = format!("{out}/emoji_{backend}.png");
    if let Some(image) = image::RgbaImage::from_raw(WIDTH, HEIGHT, pixels.to_vec()) {
        let _ = image.save(&path);
    }
}

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    let mut faces: Vec<Face> = face("colrv1", COLRV1).into_iter().collect();
    if let Some(cbdt) = std::env::var("EMOJI_CBDT")
        .ok()
        .and_then(|p| face("cbdt", &p))
    {
        faces.push(cbdt);
    }
    for face in &faces {
        println!("{} glyph ids {:?}", face.label, face.glyphs);
    }
    let cpu = render_to_buffer::<VelloCpuImageRenderer, _>(|s| draw(s, &faces), WIDTH, HEIGHT);
    report("cpu", &cpu, &faces, &out);
    match hybrid(&faces) {
        Ok(gpu) => report("hybrid", &gpu, &faces, &out),
        Err(error) => println!("hybrid: no GPU device ({error})"),
    }
}
