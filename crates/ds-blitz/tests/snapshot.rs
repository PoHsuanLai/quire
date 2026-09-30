//! Headless pictures: the size a viewport asks for, and time moving between two moments. The
//! PNGs land in the test's scratch directory for review; nothing compares them to a golden (the
//! rasteriser changes with Blitz revisions).

use dioxus::prelude::*;
use ds::detail::{Operation, PendingToken};
use ds::{
    Anim, Answers, Appearance, Button, ControlSize, Ds, Material, Progress, ProgressIndicator,
    ProgressStyle, PulseKey,
};
use ds_harness::harness::settle_until;
use ds_harness::{Driver, Harness, Query, Viewport, snapshot, snapshot_at};
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
            Button { answers: Answers::Return, label: "Send", onclick: |_| {} }
            ProgressIndicator { style: ProgressStyle::Spinner, progress: Progress::Unknown(Operation::Idle), size: ControlSize::Small }
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

/// An ink square playing `fade`: a CSS motion that `snapshot_at` measures exactly.
#[allow(non_snake_case)]
fn FadeApp() -> Element {
    let (class, alias) = PulseKey::rest(Anim::Fade)
        .fired()
        .attrs()
        .expect("a fired pulse plays");
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div {
                class: "{class}",
                "data-pulse": alias,
                style: "width:60px; height:60px; margin:20px; background:var(--ink)",
            }
        }
    }
}

#[test]
fn a_fade_at_two_motion_moments() {
    let moments = [Duration::ZERO, Duration::from_millis(125)];
    let frames = snapshot_at(FadeApp, VIEW, &moments).expect("renders");
    assert_eq!(frames.len(), 2);
    for (frame, moment) in frames.iter().zip(moments) {
        keep(frame, &format!("fade-t{:03}", moment.as_millis()));
        assert_eq!(frame.dimensions(), (480, 240));
    }
    // The square fades in between the two moments, so time reached the document.
    assert_ne!(frames[0], frames[1], "nothing moved between 0 and 125 ms");
}

#[allow(non_snake_case)]
fn SpinApp() -> Element {
    let operation = use_hook(|| Operation::Running(PendingToken::start()));
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "position:relative; width:40px; height:40px; margin:20px",
                ProgressIndicator { style: ProgressStyle::Spinner, progress: Progress::Unknown(operation), size: ControlSize::Regular }
            }
        }
    }
}

#[test]
fn the_spinner_turns() {
    // Three steps apart the lit spoke has gone a further quarter round (a twelfth per
    // `--t-spin-step`), so the spokes' shades sit elsewhere. The step is written as it is, a step
    // at a time, so the loop has nothing to interpolate.
    let mut harness = Harness::new(SpinApp, VIEW);
    let step = |h: &Harness| {
        h.attr(".ds-progress", "style")
            .and_then(|style| style.strip_prefix("--step:")?.parse::<u32>().ok())
    };
    settle_until(&mut harness, |h| step(h).is_some());
    let first = harness.render().expect("renders");
    let at = step(&harness).unwrap_or(0);
    settle_until(&mut harness, |h| {
        step(h).is_some_and(|now| (now + 12 - at) % 12 >= 3)
    });
    let later = harness.render().expect("renders");
    for (frame, name) in [(&first, "spin-first"), (&later, "spin-later")] {
        keep(frame, name);
        assert!(colours(frame) > 2, "the {name} frame is blank");
    }
    assert!(first != later, "the spinner did not turn between two steps");
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
