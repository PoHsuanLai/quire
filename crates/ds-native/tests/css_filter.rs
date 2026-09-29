//! CSS `filter` on a real Blitz document, one swatch per filter function and per filter list,
//! read back from the pixels of each backend: vello_cpu (the harness default, with the pinned
//! `multithreading`) and vello_hybrid (skipped, with a note, where no GPU adapter opens). Every
//! case compares the filtered swatch with a control of the same paint and no filter, so a
//! function that paints nothing reads as the control.
//!
//! Both backends must paint every function and every list (FINDINGS "CSS filter"): the pinned
//! renderers are the local vello and anyrender forks that add the colour matrix pass and
//! multi-threaded filters, so a case that reads `Ignored` means a fork regressed.

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
    // Filter lists: one filter layer per function, the first function innermost.
    Swatch {
        name: "list-order",
        // (200,100,50) -> brightness(0.5) (100,50,25) -> invert (155,205,230); the other order
        // would read (28,78,103).
        filter: "brightness(0.5) invert(1)",
        paint: "background:rgb(200,100,50)",
        probe: (30, 30),
        filtered: [155, 205, 230],
        plain: [200, 100, 50],
    },
    Swatch {
        name: "list-blur",
        // Far from the edge a blur leaves the colour alone; the brightness before it still shows.
        filter: "brightness(0.5) blur(4px)",
        paint: "background:rgb(200,100,50)",
        probe: (30, 30),
        filtered: [100, 50, 25],
        plain: [200, 100, 50],
    },
    Swatch {
        name: "list-colour-after-blur",
        filter: "blur(4px) contrast(2)",
        paint: "background:rgb(153,153,153)",
        probe: (30, 30),
        filtered: [179, 179, 179],
        plain: [153, 153, 153],
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
    list_order_page = 10,
    list_blur_page = 11,
    list_colour_after_blur_page = 12,
);

const PAGES: [fn() -> Element; 13] = [
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
    list_order_page,
    list_blur_page,
    list_colour_after_blur_page,
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

filter_case! {
    vello_cpu_blurs: Cpu, 0 => Rendered;
    vello_cpu_brightens: Cpu, 1 => Rendered;
    vello_cpu_contrasts: Cpu, 2 => Rendered;
    vello_cpu_casts_a_drop_shadow: Cpu, 3 => Rendered;
    vello_cpu_grays: Cpu, 4 => Rendered;
    vello_cpu_rotates_the_hue: Cpu, 5 => Rendered;
    vello_cpu_inverts: Cpu, 6 => Rendered;
    vello_cpu_fades: Cpu, 7 => Rendered;
    vello_cpu_saturates: Cpu, 8 => Rendered;
    vello_cpu_sepias: Cpu, 9 => Rendered;
    vello_cpu_lists_in_order: Cpu, 10 => Rendered;
    vello_cpu_lists_a_colour_and_a_blur: Cpu, 11 => Rendered;
    vello_cpu_lists_a_blur_and_a_colour: Cpu, 12 => Rendered;
}

filter_case! {
    vello_hybrid_blurs: Hybrid, 0 => Rendered;
    vello_hybrid_brightens: Hybrid, 1 => Rendered;
    vello_hybrid_contrasts: Hybrid, 2 => Rendered;
    vello_hybrid_casts_a_drop_shadow: Hybrid, 3 => Rendered;
    vello_hybrid_grays: Hybrid, 4 => Rendered;
    vello_hybrid_rotates_the_hue: Hybrid, 5 => Rendered;
    vello_hybrid_inverts: Hybrid, 6 => Rendered;
    vello_hybrid_fades: Hybrid, 7 => Rendered;
    vello_hybrid_saturates: Hybrid, 8 => Rendered;
    vello_hybrid_sepias: Hybrid, 9 => Rendered;
    vello_hybrid_lists_in_order: Hybrid, 10 => Rendered;
    vello_hybrid_lists_a_colour_and_a_blur: Hybrid, 11 => Rendered;
    vello_hybrid_lists_a_blur_and_a_colour: Hybrid, 12 => Rendered;
}
