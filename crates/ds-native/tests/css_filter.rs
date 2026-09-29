//! CSS `filter` on a real Blitz document, one swatch per filter function, read back from the
//! pixels of each backend: vello_cpu (the harness default) and vello_hybrid (skipped, with a
//! note, where no GPU adapter opens). Every case compares the filtered swatch with a control of
//! the same paint and no filter, so a function that paints nothing reads as the control.
//!
//! The cases at the bottom say what each backend paints today; when a fork makes another
//! function render, its case fails and flips (FINDINGS "CSS filter").

use dioxus::prelude::*;
use ds::Px;
use ds_native::{Backend, Harness, HarnessConfig, Viewport};
use image::RgbaImage;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 120,
    scale_percent: 100,
};

/// One swatch: a 60 px box at `slot`, the `filter` on it, and the pixel that shows the effect.
struct Swatch {
    /// Class suffix and test name.
    name: &'static str,
    /// The `filter` value under test.
    filter: &'static str,
    /// The swatch's paint: a full-box colour, or the small shape the effect works on.
    paint: &'static str,
    /// The probed pixel, from the swatch's top-left corner.
    probe: (u32, u32),
    /// What the probe reads when the filter has run, within `TOLERANCE` per channel.
    filtered: [u8; 3],
    /// What the probe reads when the filter is ignored (the control's own pixel).
    plain: [u8; 3],
}

const TOLERANCE: u8 = 10;

const WHITE: [u8; 3] = [255, 255, 255];

const SWATCHES: &[Swatch] = &[
    Swatch {
        name: "blur",
        filter: "blur(4px)",
        paint: "background:none",
        probe: (17, 30),
        filtered: [195, 195, 195],
        plain: WHITE,
    },
    Swatch {
        name: "brightness",
        filter: "brightness(0.5)",
        paint: "background:rgb(200,100,50)",
        probe: (30, 30),
        filtered: [100, 50, 25],
        plain: [200, 100, 50],
    },
    Swatch {
        name: "contrast",
        filter: "contrast(2)",
        paint: "background:rgb(153,153,153)",
        probe: (30, 30),
        filtered: [179, 179, 179],
        plain: [153, 153, 153],
    },
    Swatch {
        name: "drop-shadow",
        filter: "drop-shadow(10px 10px 0 #000)",
        paint: "background:none",
        probe: (35, 35),
        filtered: [0, 0, 0],
        plain: WHITE,
    },
    Swatch {
        name: "grayscale",
        filter: "grayscale(1)",
        paint: "background:rgb(255,0,0)",
        probe: (30, 30),
        filtered: [54, 54, 54],
        plain: [255, 0, 0],
    },
    Swatch {
        name: "hue-rotate",
        filter: "hue-rotate(180deg)",
        paint: "background:rgb(255,0,0)",
        probe: (30, 30),
        filtered: [0, 108, 108],
        plain: [255, 0, 0],
    },
    Swatch {
        name: "invert",
        filter: "invert(1)",
        paint: "background:rgb(255,0,0)",
        probe: (30, 30),
        filtered: [0, 255, 255],
        plain: [255, 0, 0],
    },
    Swatch {
        name: "opacity",
        filter: "opacity(0.5)",
        paint: "background:#000",
        probe: (30, 30),
        filtered: [128, 128, 128],
        plain: [0, 0, 0],
    },
    Swatch {
        name: "saturate",
        filter: "saturate(0)",
        paint: "background:rgb(255,0,0)",
        probe: (30, 30),
        filtered: [54, 54, 54],
        plain: [255, 0, 0],
    },
    Swatch {
        name: "sepia",
        filter: "sepia(1)",
        paint: "background:rgb(128,128,128)",
        probe: (30, 30),
        filtered: [173, 154, 120],
        plain: [128, 128, 128],
    },
];

/// The pixel a hard-edged black square gives the two shape swatches (blur, drop-shadow): a
/// 20 px square at (20, 20), and for the shadow (10, 10), inside the swatch.
fn shape(name: &str) -> &'static str {
    match name {
        "blur" => "left:20px; top:20px; width:20px; height:20px; background:#000",
        _ => "left:10px; top:10px; width:20px; height:20px; background:#000",
    }
}

/// The filtered box (row 0) and its control (row 1) of `SWATCHES[index]`, alone in the
/// document: a filter that panics the renderer must not take the other cases down.
fn page(index: usize) -> Element {
    let swatch = &SWATCHES[index];
    rsx! {
        div { style: "position:absolute; left:0; top:0; width:720px; height:120px; background:#fff;",
            for (row, filter) in [(0u32, swatch.filter), (1, "none")] {
                div {
                    class: "{swatch.name}-{row}",
                    style: "position:absolute; left:5px; top:{row * 60 + 5}px; width:60px; height:55px; {swatch.paint}; filter:{filter};",
                    if matches!(swatch.name, "blur" | "drop-shadow") {
                        div { style: "position:absolute; {shape(swatch.name)};" }
                    }
                }
            }
        }
    }
}

macro_rules! pages {
    ($($name:ident = $index:expr),* $(,)?) => {
        $(fn $name() -> Element { page($index) })*
    };
}

pages!(
    blur_page = 0,
    brightness_page = 1,
    contrast_page = 2,
    drop_shadow_page = 3,
    grayscale_page = 4,
    hue_rotate_page = 5,
    invert_page = 6,
    opacity_page = 7,
    saturate_page = 8,
    sepia_page = 9,
);

const PAGES: [fn() -> Element; 10] = [
    blur_page,
    brightness_page,
    contrast_page,
    drop_shadow_page,
    grayscale_page,
    hue_rotate_page,
    invert_page,
    opacity_page,
    saturate_page,
    sepia_page,
];

/// The pixel of `swatch`'s filtered box or its control (`row` 0 or 1).
fn probe(frame: &RgbaImage, harness: &Harness, swatch: &Swatch, row: u32) -> [u8; 3] {
    let at = harness
        .rect(&format!(".{}-{row}", swatch.name))
        .expect("the swatch is laid out");
    let Px(x0) = at.origin.x;
    let Px(y0) = at.origin.y;
    let p = frame.get_pixel(x0 as u32 + swatch.probe.0, y0 as u32 + swatch.probe.1);
    [p[0], p[1], p[2]]
}

fn near(got: [u8; 3], want: [u8; 3]) -> bool {
    got.iter()
        .zip(want)
        .all(|(g, w)| g.abs_diff(w) <= TOLERANCE)
}

/// What a backend did with one filter function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    /// The probe reads the filter's result.
    Rendered,
    /// The probe reads the control: the filter was dropped.
    Ignored,
    /// Neither (a real bug in the swatch or the backend).
    Other([u8; 3]),
}

/// The verdict for `SWATCHES[index]` on `backend`, or `None` where no GPU opens.
fn verdict(backend: Backend, index: usize) -> Option<Verdict> {
    let config = HarnessConfig::new(VIEW).with_backend(backend);
    let mut harness = match Harness::try_with_config(PAGES[index], config) {
        Ok(harness) => harness,
        Err(error) => {
            eprintln!("skipped: no GPU ({error})");
            return None;
        }
    };
    let frame = harness.render().expect("paints");
    let swatch = &SWATCHES[index];
    let control = probe(&frame, &harness, swatch, 1);
    assert!(
        near(control, swatch.plain),
        "{}: the control reads {control:?}, not {:?}",
        swatch.name,
        swatch.plain
    );
    let got = probe(&frame, &harness, swatch, 0);
    Some(if near(got, swatch.filtered) {
        Verdict::Rendered
    } else if near(got, swatch.plain) {
        Verdict::Ignored
    } else {
        Verdict::Other(got)
    })
}

/// One case: `backend` gives `SWATCHES[index]` the verdict `expected`. A hybrid case skips,
/// with a note, where no GPU adapter opens.
macro_rules! filter_case {
    ($($test:ident: $backend:ident, $index:expr => $expected:ident;)*) => {
        $(
            #[test]
            fn $test() {
                let Some(got) = verdict(Backend::$backend, $index) else {
                    return;
                };
                assert_eq!(got, Verdict::$expected, "{}", SWATCHES[$index].filter);
            }
        )*
    };
}

// vello_cpu as pinned (`multithreading`): the multi-threaded dispatcher has no filter support,
// so anyrender_vello_cpu hands it none and every filter is dropped.
filter_case! {
    vello_cpu_drops_blur: Cpu, 0 => Ignored;
    vello_cpu_drops_brightness: Cpu, 1 => Ignored;
    vello_cpu_drops_contrast: Cpu, 2 => Ignored;
    vello_cpu_drops_drop_shadow: Cpu, 3 => Ignored;
    vello_cpu_drops_grayscale: Cpu, 4 => Ignored;
    vello_cpu_drops_hue_rotate: Cpu, 5 => Ignored;
    vello_cpu_drops_invert: Cpu, 6 => Ignored;
    vello_cpu_drops_opacity: Cpu, 7 => Ignored;
    vello_cpu_drops_saturate: Cpu, 8 => Ignored;
    vello_cpu_drops_sepia: Cpu, 9 => Ignored;
}

// vello_hybrid: blur and drop-shadow are GPU passes; every colour function (a colour matrix or a
// component transfer) converts to nothing in anyrender_vello_hybrid.
filter_case! {
    vello_hybrid_blurs: Hybrid, 0 => Rendered;
    vello_hybrid_drops_brightness: Hybrid, 1 => Ignored;
    vello_hybrid_drops_contrast: Hybrid, 2 => Ignored;
    vello_hybrid_casts_a_drop_shadow: Hybrid, 3 => Rendered;
    vello_hybrid_drops_grayscale: Hybrid, 4 => Ignored;
    vello_hybrid_drops_hue_rotate: Hybrid, 5 => Ignored;
    vello_hybrid_drops_invert: Hybrid, 6 => Ignored;
    vello_hybrid_drops_opacity: Hybrid, 7 => Ignored;
    vello_hybrid_drops_saturate: Hybrid, 8 => Ignored;
    vello_hybrid_drops_sepia: Hybrid, 9 => Ignored;
}
