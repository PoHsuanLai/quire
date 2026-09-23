//! Headless pictures: the size a viewport asks for, and time moving between two moments. The
//! PNGs land in the test's scratch directory for review; nothing compares them to a golden (the
//! rasteriser changes with Blitz revisions).

use dioxus::prelude::*;
use ds::{Appearance, Button, ButtonVariant, Ds, Material, Spinner, SpinnerKind};
use ds_native::{Viewport, snapshot, snapshot_at};
use image::RgbaImage;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 240,
    height: 120,
    scale_percent: 200,
};

#[allow(non_snake_case)]
fn ButtonApp() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            Button { variant: ButtonVariant::Primary, label: "Send", onclick: |_| {} }
            Spinner { kind: SpinnerKind::Breathe }
        }
    }
}

/// How many distinct colours a frame holds: one means nothing was painted.
fn colours(frame: &RgbaImage) -> usize {
    let mut seen = frame.pixels().map(|pixel| pixel.0).collect::<Vec<_>>();
    seen.sort_unstable();
    seen.dedup();
    seen.len()
}

fn keep(frame: &RgbaImage, name: &str) {
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}.png"));
    frame.save(&path).ok();
}

#[test]
fn a_snapshot_has_the_viewport_size_in_device_pixels() {
    let frame = snapshot(ButtonApp, VIEW).expect("renders");
    keep(&frame, "button-at-rest");
    assert_eq!(frame.dimensions(), (480, 240));
    assert!(colours(&frame) > 16, "the frame is blank");
}

#[test]
fn a_button_root_at_two_motion_moments() {
    let moments = [Duration::ZERO, Duration::from_millis(250)];
    let frames = snapshot_at(ButtonApp, VIEW, &moments).expect("renders");
    assert_eq!(frames.len(), 2);
    for (frame, moment) in frames.iter().zip(moments) {
        keep(frame, &format!("button-t{:03}", moment.as_millis()));
        assert_eq!(frame.dimensions(), (480, 240));
        assert!(colours(frame) > 16, "the frame at {moment:?} is blank");
    }
    // The breathing ring fades in between the two moments, so time reached the document.
    assert_ne!(frames[0], frames[1], "nothing moved between 0 and 250 ms");
}

#[allow(non_snake_case)]
fn SpinApp() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "position:relative; width:40px; height:40px; margin:20px",
                Spinner { kind: SpinnerKind::Spin }
            }
        }
    }
}

#[test]
fn the_spinner_turns() {
    // A quarter of `--t-spin` apart: the dashed ring has turned 90 degrees, so its dashes sit
    // elsewhere. With a base `transform:scale(1)` the keyframe interpolated between two identity
    // matrices and the ring never moved (the wave 2 ds-native finding).
    let moments = [Duration::from_millis(100), Duration::from_millis(375)];
    let frames = snapshot_at(SpinApp, VIEW, &moments).expect("renders");
    for (frame, moment) in frames.iter().zip(moments) {
        keep(frame, &format!("spin-t{:03}", moment.as_millis()));
        assert!(colours(frame) > 2, "the frame at {moment:?} is blank");
    }
    assert!(
        frames[0] != frames[1],
        "the spinner did not turn between the two moments"
    );
}

/// A green square as an SVG, the shape quire's icon masks and grain arrive in.
const GREEN_SVG: &str = "<svg xmlns='http://www.w3.org/2000/svg' width='10' height='10'><rect width='10' height='10' fill='rgb(0,160,0)'/></svg>";

/// Where the `file:` copy of [`GREEN_SVG`] lives.
fn green_file() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("green.svg")
}

#[allow(non_snake_case)]
fn ImageApp() -> Element {
    let encoded = GREEN_SVG
        .replace('<', "%3C")
        .replace('>', "%3E")
        .replace(' ', "%20");
    let sheet = format!(
        ".from-data {{ width: 40px; height: 40px; background-image: url(\"data:image/svg+xml,{encoded}\") }}
         .from-file {{ width: 40px; height: 40px; background-image: url(\"file://{}\") }}",
        green_file().display()
    );
    rsx! {
        style { {sheet} }
        div { class: "from-data" }
        div { class: "from-file" }
    }
}

#[test]
fn data_and_file_images_land_before_the_snapshot() {
    std::fs::write(green_file(), GREEN_SVG).expect("write the svg");
    let view = Viewport {
        width: 60,
        height: 100,
        scale_percent: 100,
    };
    let frame = snapshot(ImageApp, view).expect("renders");
    keep(&frame, "images");
    // Inside the first tile of each 40 px box, stacked below the body's 8 px margin. (Blitz
    // paints an SVG background once at its intrinsic 10 px rather than tiling it; PNGs tile,
    // spike S8.)
    for (name, (x, y)) in [("data:", (12, 12)), ("file:", (12, 52))] {
        let [r, g, b, _] = frame.get_pixel(x, y).0;
        assert!(
            g > 120 && r < 60 && b < 60,
            "the {name} image did not land: rgb({r},{g},{b})"
        );
    }
}
