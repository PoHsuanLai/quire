//! A texture on the GPU shown inside the document: textures of known colours are made on the
//! harness's own device, a `TextureLayer` places them, and the hybrid painter's pixels are read
//! back. Every case skips, with a note, where no GPU adapter opens (CI without one).

use dioxus::prelude::*;
use ds_blitz::{
    PixelFormat, Pixels, Sampling, TexelRect, TextureFit, TextureHandle, TextureLayer, use_gpu,
};
use ds_harness::{Backdrop, Backend, Harness, HarnessConfig, Viewport};
use std::sync::{Arc, Mutex};

/// One layer's box is this many pixels square.
const BOX: u32 = 100;

const RED: [u8; 4] = [255, 0, 0, 255];
const GREEN: [u8; 4] = [0, 255, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const CLEAR: [u8; 4] = [0, 0, 0, 0];

/// What the app under test places: a row of boxes, one layer in each.
#[derive(Debug, Clone)]
struct Scene {
    layers: u32,
    fit: TextureFit,
    source: Option<TexelRect>,
    sampling: Sampling,
}

/// The handles the app made, for the test to put textures in.
#[derive(Debug, Clone, Default)]
struct Handles(Arc<Mutex<Vec<TextureHandle>>>);

/// The device the app saw on its first render.
#[derive(Debug, Clone, Default)]
struct SeenDevice(Arc<Mutex<Option<wgpu::Device>>>);

fn app() -> Element {
    let scene = use_context::<Scene>();
    let handles = use_context::<Handles>();
    let seen = use_context::<SeenDevice>();
    let gpu = use_gpu();
    let made = use_hook(|| {
        *seen.0.lock().expect("lock") = gpu.device();
        let made: Vec<TextureHandle> = (0..scene.layers).map(|_| gpu.handle()).collect();
        handles.0.lock().expect("lock").extend(made.iter().cloned());
        made
    });
    rsx! {
        style { "html, body {{ margin: 0 }}" }
        for (at, texture) in made.iter().enumerate() {
            div {
                key: "{at}",
                style: "position:absolute; top:0; left:{at as u32 * BOX}px; width:{BOX}px; height:{BOX}px",
                TextureLayer {
                    texture: texture.clone(),
                    fit: scene.fit,
                    source: scene.source,
                    sampling: scene.sampling,
                }
            }
        }
    }
}

/// A hybrid harness showing `scene`, with the handles its layers use, or `None` (with a note)
/// where no GPU opens.
fn gpu_harness(scene: Scene) -> Option<(Harness, Vec<TextureHandle>, SeenDevice)> {
    let handles = Handles::default();
    let seen = SeenDevice::default();
    let viewport = Viewport {
        width: BOX * scene.layers,
        height: BOX,
        scale_percent: 100,
    };
    let config = HarnessConfig::new(viewport)
        .with_backend(Backend::Hybrid)
        .with_context(scene)
        .with_context(handles.clone())
        .with_context(seen.clone());
    match Harness::try_new(app, config) {
        Ok(harness) => {
            let made = handles.0.lock().expect("lock").clone();
            Some((harness, made, seen))
        }
        Err(error) => {
            eprintln!("skipped: no GPU ({error})");
            None
        }
    }
}

fn one(fit: TextureFit) -> Scene {
    Scene {
        layers: 1,
        fit,
        source: None,
        sampling: Sampling::Nearest,
    }
}

/// `pixels` (each a four-byte colour in `format`) as a `width` by `height` picture.
fn picture(format: PixelFormat, width: u32, height: u32, texels: &[&[u8]]) -> Vec<u8> {
    let bytes: Vec<u8> = texels
        .iter()
        .flat_map(|texel| texel.iter().copied())
        .collect();
    assert_eq!(
        bytes.len(),
        (width * height) as usize * format.bytes_per_pixel()
    );
    bytes
}

fn upload(handle: &TextureHandle, format: PixelFormat, width: u32, height: u32, bytes: &[u8]) {
    let pixels = Pixels::new(format, width, height, bytes).expect("a valid picture");
    handle.update(&pixels).expect("the device is there");
}

/// A two-texel picture: `left`, then `right`, both opaque.
fn pair(handle: &TextureHandle, left: [u8; 4], right: [u8; 4]) {
    let bytes = picture(PixelFormat::Rgba8Premultiplied, 2, 1, &[&left, &right]);
    upload(handle, PixelFormat::Rgba8Premultiplied, 2, 1, &bytes);
}

/// The document over a transparent ground, so a pixel the layer leaves alone reads `CLEAR`.
fn shot(harness: &mut Harness) -> image::RgbaImage {
    harness
        .render_over(Backdrop::Clear)
        .expect("the hybrid painter draws")
}

fn at(image: &image::RgbaImage, x: u32, y: u32) -> [u8; 4] {
    image.get_pixel(x, y).0
}

#[test]
fn the_app_sees_the_device_the_renderer_paints_with() {
    let Some((harness, _, seen)) = gpu_harness(one(TextureFit::Fill)) else {
        return;
    };
    let device = harness.gpu().and_then(|gpu| gpu.device());
    assert!(device.is_some(), "the harness's GPU has a device");
    assert_eq!(
        *seen.0.lock().expect("lock"),
        device,
        "the first render of the app already had the renderer's device"
    );
}

#[test]
fn a_texture_of_known_colours_fills_the_box() {
    let Some((mut harness, handles, _)) = gpu_harness(one(TextureFit::Fill)) else {
        return;
    };
    let before = shot(&mut harness);
    assert_eq!(at(&before, 25, 50), CLEAR, "an empty layer draws nothing");
    pair(&handles[0], RED, BLUE);
    let after = shot(&mut harness);
    assert_eq!(
        at(&after, 25, 50),
        RED,
        "the left texel stretched over the left half"
    );
    assert_eq!(
        at(&after, 75, 50),
        BLUE,
        "the right texel over the right half"
    );
    assert_eq!(at(&after, 1, 1), RED, "to the corner");
    assert_eq!(at(&after, 98, 98), BLUE, "to the other corner");
}

/// (what is checked, a fit, a point inside the box, what is there) for a 2x1 picture, red then
/// blue, in a 100x100 box.
type Case = (&'static str, TextureFit, (u32, u32), [u8; 4]);

const CASES: &[Case] = &[
    (
        "contain leaves ground above the picture",
        TextureFit::Contain,
        (50, 10),
        CLEAR,
    ),
    (
        "contain draws the picture in the middle band",
        TextureFit::Contain,
        (25, 50),
        RED,
    ),
    (
        "contain draws the other texel",
        TextureFit::Contain,
        (75, 50),
        BLUE,
    ),
    (
        "contain leaves ground below the picture",
        TextureFit::Contain,
        (50, 90),
        CLEAR,
    ),
    (
        "cover reaches the top edge",
        TextureFit::Cover,
        (30, 2),
        RED,
    ),
    (
        "cover reaches the bottom edge",
        TextureFit::Cover,
        (70, 97),
        BLUE,
    ),
    (
        "actual is one texel to a pixel, centred: the left texel",
        TextureFit::Actual,
        (49, 49),
        RED,
    ),
    (
        "actual, the right texel",
        TextureFit::Actual,
        (50, 49),
        BLUE,
    ),
    (
        "actual leaves the pixel beside it alone",
        TextureFit::Actual,
        (48, 49),
        CLEAR,
    ),
    (
        "tile repeats from the corner, texel 0",
        TextureFit::Tile,
        (0, 0),
        RED,
    ),
    ("tile, texel 1", TextureFit::Tile, (1, 0), BLUE),
    ("tile, texel 0 again", TextureFit::Tile, (2, 0), RED),
    (
        "tile reaches the far column",
        TextureFit::Tile,
        (99, 40),
        BLUE,
    ),
    (
        "tile stops at the most tiles, 4096 of these 5000",
        TextureFit::Tile,
        (99, 99),
        CLEAR,
    ),
];

#[test]
fn each_fit_puts_the_picture_where_it_says() {
    for fit in [
        TextureFit::Contain,
        TextureFit::Cover,
        TextureFit::Actual,
        TextureFit::Tile,
    ] {
        let Some((mut harness, handles, _)) = gpu_harness(one(fit)) else {
            return;
        };
        pair(&handles[0], RED, BLUE);
        let image = shot(&mut harness);
        for (name, case_fit, (x, y), want) in CASES {
            if *case_fit == fit {
                assert_eq!(at(&image, *x, *y), *want, "{name}");
            }
        }
    }
}

#[test]
fn a_source_rectangle_crops_the_texture() {
    let scene = Scene {
        source: Some(TexelRect::new(1, 0, 1, 1)),
        ..one(TextureFit::Fill)
    };
    let Some((mut harness, handles, _)) = gpu_harness(scene) else {
        return;
    };
    pair(&handles[0], RED, BLUE);
    let image = shot(&mut harness);
    assert_eq!(
        at(&image, 5, 5),
        BLUE,
        "only the right texel is shown, over the whole box"
    );
    assert_eq!(at(&image, 95, 95), BLUE, "everywhere in the box");
}

#[test]
fn several_layers_each_draw_their_own_texture() {
    let scene = Scene {
        layers: 3,
        ..one(TextureFit::Fill)
    };
    let Some((mut harness, handles, _)) = gpu_harness(scene) else {
        return;
    };
    pair(&handles[0], RED, RED);
    pair(&handles[1], GREEN, GREEN);
    pair(&handles[2], BLUE, BLUE);
    let image = shot(&mut harness);
    assert_eq!(at(&image, 50, 50), RED);
    assert_eq!(at(&image, 150, 50), GREEN);
    assert_eq!(at(&image, 250, 50), BLUE);
}

#[test]
fn a_new_frame_replaces_the_last() {
    let Some((mut harness, handles, _)) = gpu_harness(one(TextureFit::Fill)) else {
        return;
    };
    pair(&handles[0], RED, RED);
    let first = at(&shot(&mut harness), 50, 50);
    // The same size: written into the texture the layer already has.
    pair(&handles[0], BLUE, BLUE);
    let second = at(&shot(&mut harness), 50, 50);
    // Another size: a new texture, registered again.
    let bytes = picture(PixelFormat::Rgb8, 1, 1, &[&[0, 255, 0]]);
    upload(&handles[0], PixelFormat::Rgb8, 1, 1, &bytes);
    let third = at(&shot(&mut harness), 50, 50);
    assert_eq!((first, second, third), (RED, BLUE, GREEN));
    handles[0].clear();
    assert_eq!(
        at(&shot(&mut harness), 50, 50),
        CLEAR,
        "cleared, it draws nothing"
    );
}

#[test]
fn a_texture_the_app_made_is_drawn_and_written_to_in_place() {
    let Some((mut harness, handles, _)) = gpu_harness(one(TextureFit::Fill)) else {
        return;
    };
    let gpu = harness.gpu().expect("a hybrid harness has a GPU");
    let (device, queue) = (gpu.device().expect("device"), gpu.queue().expect("queue"));
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("test frame"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let write = |colour: [u8; 4]| {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &colour,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        handles[0].redraw();
    };
    handles[0].replace(texture.clone());
    write(GREEN);
    let first = at(&shot(&mut harness), 50, 50);
    write(RED);
    let second = at(&shot(&mut harness), 50, 50);
    assert_eq!(
        (first, second),
        (GREEN, RED),
        "a write into the registered texture shows"
    );
}

#[test]
fn alpha_is_premultiplied_in_the_texture_and_composited_over_what_is_beneath() {
    let Some((mut harness, handles, _)) = gpu_harness(one(TextureFit::Fill)) else {
        return;
    };
    // Half-transparent red, both ways of saying it.
    let straight = picture(PixelFormat::Rgba8Straight, 1, 1, &[&[255, 0, 0, 128]]);
    upload(&handles[0], PixelFormat::Rgba8Straight, 1, 1, &straight);
    let from_straight = at(&shot(&mut harness), 50, 50);
    let premultiplied = picture(PixelFormat::Rgba8Premultiplied, 1, 1, &[&[128, 0, 0, 128]]);
    upload(
        &handles[0],
        PixelFormat::Rgba8Premultiplied,
        1,
        1,
        &premultiplied,
    );
    let from_premultiplied = at(&shot(&mut harness), 50, 50);
    assert_eq!(
        from_straight, from_premultiplied,
        "both uploads hold the same texel"
    );
    // Over nothing, the target holds the premultiplied texel as it is.
    assert_eq!(from_straight, [128, 0, 0, 128]);
    // Over the scheme's ground the red shows at half strength, whatever the ground is.
    let over = scheme_ground(&mut harness);
    handles[0].clear();
    let ground = scheme_ground(&mut harness);
    assert_eq!(over[3], 255, "the ground is opaque");
    let blend = |layer: u8, ground: u8| i32::from(layer) + i32::from(ground) * 127 / 255;
    for channel in 0..3 {
        let layer = [128, 0, 0][channel];
        let want = blend(layer, ground[channel]);
        assert!(
            (i32::from(over[channel]) - want).abs() <= 2,
            "channel {channel}: {over:?} over {ground:?}"
        );
    }
}

/// The scheme's ground and what is over it, at the middle of the box.
fn scheme_ground(harness: &mut Harness) -> [u8; 4] {
    let image = harness
        .render_over(Backdrop::Scheme)
        .expect("the hybrid painter draws");
    at(&image, 50, 50)
}

#[test]
fn a_layer_sizes_to_its_box_and_never_reads_the_cpu_painter() {
    // On vello_cpu there is no device: the layer is in the document and draws nothing.
    let handles = Handles::default();
    let seen = SeenDevice::default();
    let viewport = Viewport {
        width: BOX,
        height: BOX,
        scale_percent: 100,
    };
    let config = HarnessConfig::new(viewport)
        .with_context(one(TextureFit::Fill))
        .with_context(handles.clone())
        .with_context(seen.clone());
    let mut harness = Harness::new(app, config);
    assert!(harness.gpu().is_none(), "the CPU painter has no GPU");
    assert!(
        seen.0.lock().expect("lock").is_none(),
        "so the app saw no device"
    );
    let image = shot(&mut harness);
    assert_eq!(at(&image, 50, 50), CLEAR);
}
