//! Headless driving of a DioxusDocument: build, resolve at a time, render to PNG, synthesize
//! input. Everything a probe needs and nothing more.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Wake, Waker};

use anyrender::{PaintScene as _, render_to_buffer};
use anyrender_vello_cpu::VelloCpuImageRenderer;
use atomic_refcell::AtomicRefCell;
use blitz_dom::{Document as _, DocumentConfig, FontContext, StyleThreading};
use blitz_paint::paint_scene;
use blitz_traits::events::{
    BlitzKeyEvent, BlitzPointerEvent, BlitzPointerId, KeyState, MouseEventButton,
    MouseEventButtons, PointerCoords, UiEvent,
};
use blitz_traits::net::{Bytes, NetHandler, NetProvider, Request};
use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus::prelude::*;
use dioxus_native_dom::DioxusDocument;
use keyboard_types::{Code, Key, Location, Modifiers};
use peniko::{Color, Fill};

pub const W: u32 = 400;
pub const H: u32 = 300;

pub fn out_dir() -> PathBuf {
    let dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../out"));
    std::fs::create_dir_all(&dir).expect("create spike/out");
    dir
}

/// Serves `data:` URLs and nothing else, the way blitz-shell's DataUriNetProvider does. The
/// default provider (DummyNetProvider) drops every request, data: included.
pub struct DataUriNet;

impl NetProvider for DataUriNet {
    fn fetch(&self, _doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        if request.url.scheme() != "data" {
            return;
        }
        let Ok(url) = data_url::DataUrl::process(request.url.as_str()) else {
            eprintln!("  data-url: unparseable {}", &request.url.as_str()[..40]);
            return;
        };
        let Ok((body, _)) = url.decode_to_vec() else {
            eprintln!("  data-url: undecodable");
            return;
        };
        handler.bytes(request.url.to_string(), Bytes::from(body));
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Net {
    Dummy,
    DataUri,
}

pub fn build(app: fn() -> Element, fonts: Option<FontContext>, net: Net) -> DioxusDocument {
    let config = DocumentConfig {
        viewport: Some(Viewport::new(W, H, 1.0, ColorScheme::Light)),
        font_ctx: fonts,
        style_threading: StyleThreading::Sequential,
        net_provider: match net {
            Net::Dummy => None,
            Net::DataUri => Some(Arc::new(DataUriNet) as Arc<dyn NetProvider>),
        },
        ..Default::default()
    };
    let mut doc = DioxusDocument::new(VirtualDom::new(app), config);
    doc.initial_build();
    doc
}

/// An RGBA8 frame, already written to `spike/out/<name>.png`.
pub struct Shot {
    pub name: String,
    pub buf: Vec<u8>,
}

impl Shot {
    pub fn px(&self, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * W + x) * 4) as usize;
        [
            self.buf[i],
            self.buf[i + 1],
            self.buf[i + 2],
            self.buf[i + 3],
        ]
    }
    pub fn path(&self) -> String {
        format!("spike/out/{}.png", self.name)
    }
}

pub fn resolve(doc: &mut DioxusDocument, t: f64) {
    doc.inner.borrow_mut().resolve(t);
}

/// Resolve at `t` seconds, paint with anyrender_vello_cpu, save.
pub fn shot(doc: &mut DioxusDocument, t: f64, name: &str) -> Shot {
    resolve(doc, t);
    let buf = render_to_buffer::<VelloCpuImageRenderer, _>(
        |scene| {
            scene.fill(
                Fill::NonZero,
                Default::default(),
                Color::WHITE,
                Default::default(),
                &kurbo::Rect::new(0.0, 0.0, W as f64, H as f64),
            );
            paint_scene(scene, &mut doc.inner.borrow_mut(), 1.0, W, H, 0, 0);
        },
        W,
        H,
    );
    save(name, buf)
}

pub fn save(name: &str, buf: Vec<u8>) -> Shot {
    let path = out_dir().join(format!("{name}.png"));
    image::save_buffer(&path, &buf, W, H, image::ColorType::Rgba8).expect("write png");
    Shot {
        name: name.to_string(),
        buf,
    }
}

fn pointer(x: f32, y: f32, buttons: MouseEventButtons) -> BlitzPointerEvent {
    BlitzPointerEvent {
        id: BlitzPointerId::Mouse,
        is_primary: true,
        coords: PointerCoords {
            page_x: x,
            page_y: y,
            screen_x: x,
            screen_y: y,
            client_x: x,
            client_y: y,
        },
        button: MouseEventButton::Main,
        buttons,
        mods: Modifiers::empty(),
        details: Default::default(),
        element: Default::default(),
        active_pointers: Arc::new(AtomicRefCell::new(Vec::new())),
    }
}

/// Move, press, release at (x, y) in logical px, the order a host must synthesise.
pub fn click(doc: &mut DioxusDocument, x: f32, y: f32) {
    doc.handle_ui_event(UiEvent::PointerMove(pointer(x, y, MouseEventButtons::None)));
    doc.handle_ui_event(UiEvent::PointerDown(pointer(
        x,
        y,
        MouseEventButtons::Primary,
    )));
    doc.handle_ui_event(UiEvent::PointerUp(pointer(x, y, MouseEventButtons::None)));
}

pub fn key(doc: &mut DioxusDocument, key: Key, code: Code) {
    for state in [KeyState::Pressed, KeyState::Released] {
        let ev = BlitzKeyEvent {
            key: key.clone(),
            code,
            modifiers: Modifiers::empty(),
            location: Location::Standard,
            is_auto_repeating: false,
            is_composing: false,
            state,
            text: None,
        };
        doc.handle_ui_event(match state {
            KeyState::Pressed => UiEvent::KeyDown(ev),
            KeyState::Released => UiEvent::KeyUp(ev),
        });
    }
}

/// A waker that counts how often it was woken, standing in for the host's calloop ping.
#[derive(Default)]
pub struct CountingWaker(AtomicUsize);

impl Wake for CountingWaker {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

impl CountingWaker {
    pub fn count(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }
}

pub fn poll_with(doc: &mut DioxusDocument, waker: &Arc<CountingWaker>) -> bool {
    let waker = Waker::from(Arc::clone(waker));
    let cx = Context::from_waker(&waker);
    doc.poll(Some(cx))
}

/// Mark the root scope dirty and flush it, for probes whose state lives in a static.
pub fn rerender(doc: &mut DioxusDocument) {
    doc.vdom.mark_dirty(ScopeId::APP);
    doc.poll(None);
}

/// Classify a pixel into a coarse colour name so the table can show what is visible.
pub fn hue(p: [u8; 4]) -> &'static str {
    let [r, g, b, _] = p.map(i32::from);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    if max - min < 30 {
        return match max {
            0..=60 => "black",
            61..=200 => "grey",
            _ => "white",
        };
    }
    if r == max && g > 150 && b < 100 {
        "yellow"
    } else if r == max && b > 150 && g < 120 {
        "magenta"
    } else if r == max {
        "red"
    } else if g == max {
        "green"
    } else if r > 100 && b == max {
        "purple"
    } else {
        "blue"
    }
}

/// Offscreen render through anyrender_vello_hybrid on the first wgpu adapter.
pub fn shot_hybrid(doc: &mut DioxusDocument, t: f64, name: &str) -> Result<Shot, String> {
    use anyrender_vello_hybrid::{ImageManager, VelloHybridScenePainter};
    use wgpu_context::{BufferRenderer, BufferRendererConfig, DeviceHandle, WGPUContext};

    resolve(doc, t);
    let ctx = WGPUContext::new();
    let dh = pollster::block_on(DeviceHandle::new_from_compatible_surface(
        ctx.instance.clone(),
        None,
        None,
        None,
    ))
    .map_err(|e| format!("no wgpu device: {e}"))?;
    eprintln!("  hybrid adapter: {:?}", dh.adapter.get_info().name);
    let target = BufferRenderer::new(
        BufferRendererConfig {
            width: W,
            height: H,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        },
        dh.clone(),
        0,
    );
    let mut renderer = vello_hybrid::Renderer::new(
        &dh.device,
        &vello_hybrid::RenderTargetConfig {
            format: wgpu::TextureFormat::Rgba8Unorm,
            width: W,
            height: H,
        },
    );
    let mut resources = vello_hybrid::Resources::new();
    let mut scene = vello_hybrid::Scene::new(W as u16, H as u16);
    let mut cache = rustc_hash::FxHashMap::default();
    let mut bindings = rustc_hash::FxHashMap::default();
    let mut encoder = dh
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let images = ImageManager::new(
            &mut renderer,
            &mut resources,
            &dh.device,
            &dh.queue,
            &mut encoder,
            &mut cache,
        );
        let mut painter = VelloHybridScenePainter::new(&mut scene, images, &mut bindings, &dh);
        painter.fill(
            Fill::NonZero,
            Default::default(),
            Color::WHITE,
            Default::default(),
            &kurbo::Rect::new(0.0, 0.0, W as f64, H as f64),
        );
        paint_scene(&mut painter, &mut doc.inner.borrow_mut(), 1.0, W, H, 0, 0);
    }
    renderer
        .render(
            &scene,
            &mut resources,
            &dh.device,
            &dh.queue,
            &mut encoder,
            &vello_hybrid::RenderSize {
                width: W,
                height: H,
            },
            &target.target_texture_view(),
            &vello_hybrid::TextureBindings::new(),
        )
        .map_err(|e| format!("hybrid render: {e:?}"))?;
    dh.queue.submit([encoder.finish()]);
    let mut buf = vec![0u8; (W * H * 4) as usize];
    target.copy_texture_to_buffer(&mut buf);
    Ok(save(name, buf))
}
