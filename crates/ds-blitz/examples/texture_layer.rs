//! A texture inside a quire window: a thread uploads a moving gradient every frame to a
//! `TextureLayer`, the way a player would hand over decoded frames. It reports the window's
//! adapter and the frames it showed, and closes itself after three seconds.
//!
//! `cargo run -p ds-blitz --example texture_layer`

use dioxus::prelude::*;
use ds::prelude::*;
use ds_blitz::{
    AppConfig, AppId, PixelFormat, Pixels, TextureFit, TextureHandle, TextureLayer, WindowSize,
    launch, use_gpu,
};
use std::thread;
use std::time::{Duration, Instant};

/// How long the window stays up on its own.
const LIFETIME: Duration = Duration::from_secs(3);
/// The picture's size in pixels.
const SIDE: u32 = 256;

fn main() -> Result<(), ds_blitz::LaunchError> {
    launch(
        App,
        AppConfig::new("quire: ds-blitz texture layer", WindowSize::new(480, 320))
            .with_app_id(AppId("dev.quire.TextureLayer".to_owned())),
    )
}

#[allow(non_snake_case)]
fn App() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, Player {} }
    }
}

/// The player: one handle, one thread writing frames into it.
#[allow(non_snake_case)]
fn Player() -> Element {
    let gpu = use_gpu();
    let frame = use_hook(|| {
        let frame = gpu.handle();
        let writer = frame.clone();
        thread::spawn(move || play(&writer));
        frame
    });
    rsx! {
        div { style: "width:100%; height:100%",
            TextureLayer { texture: frame, fit: TextureFit::Contain }
        }
    }
}

/// Write a frame about every 16 ms, and exit after `LIFETIME`.
fn play(frame: &TextureHandle) {
    let started = Instant::now();
    let mut frames = 0u32;
    let mut reported = false;
    while started.elapsed() < LIFETIME {
        let shift = (started.elapsed().as_millis() / 8) as u32;
        let bytes: Vec<u8> = (0..SIDE * SIDE)
            .flat_map(|at| {
                let (x, y) = (at % SIDE, at / SIDE);
                [((x + shift) % SIDE) as u8, y as u8, 128]
            })
            .collect();
        let Ok(pixels) = Pixels::new(PixelFormat::Rgb8, SIDE, SIDE, &bytes) else {
            return;
        };
        // Until the window's renderer has made its device there is nowhere to put the frame.
        if frame.update(&pixels).is_ok() {
            if !reported {
                reported = true;
                let adapter = frame.gpu().adapter().map(|adapter| adapter.get_info().name);
                println!("device arrived: {adapter:?}");
            }
            frames += 1;
        }
        thread::sleep(Duration::from_millis(16));
    }
    println!("{frames} frames written in {LIFETIME:?}");
    std::process::exit(0);
}
