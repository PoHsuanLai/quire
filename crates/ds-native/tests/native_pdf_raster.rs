//! The visual check: a printed page, rasterised by pdfrum at 96 dpi, against the headless
//! snapshot of the same document. The page's margins are whole CSS pixels so the two grids
//! line up; what may differ is anti-aliasing and glyph rasterisation (vello's and pdfrum's),
//! which the tolerance allows for, not layout.

use dioxus::prelude::*;
use ds_native::{Harness, Margins, PageSize, PageSpec, Pt, Viewport};
use pdfrum::{Document, RenderOptions, VelloCpuBackend};

/// The content box: 480 x 360 CSS px, 64 px (48 pt) of margin all round.
const CONTENT: Viewport = Viewport {
    width: 480,
    height: 360,
    scale_percent: 100,
};
const MARGIN_PX: u32 = 64;

fn spec() -> PageSpec {
    PageSpec {
        size: PageSize::Custom {
            width: Pt(480.0 * 0.75 + 96.0),
            height: Pt(360.0 * 0.75 + 96.0),
        },
        margins: Margins::uniform(Pt(48.0)),
    }
}

fn app() -> Element {
    rsx! {
        div { style: "font: 18px/1.4 'Karla', sans-serif; color: #1b1b1b; background: #fff; height: 360px; margin: 0;",
            h1 { style: "font-size: 30px; margin: 0; padding: 16px; background: #2b59c3; color: #fff;",
                "Printed page"
            }
            p { style: "margin: 16px;", "Regular text, then " b { "bold text" } ", then a box:" }
            div { style: "margin: 16px; height: 80px; border: 4px solid #d33; border-radius: 12px; background: linear-gradient(90deg, #ffd166, #06d6a0);" }
            p { style: "margin: 16px; font-family: 'Noto Serif';", "Serif line with a field of flowers." }
        }
    }
}

/// Mean absolute channel difference, and the share of pixels differing by more than a quarter
/// of the range in any channel.
fn difference(a: &[u8], b: &[u8]) -> (f64, f64) {
    let pixels = a.len() / 4;
    let total: u64 = a
        .iter()
        .zip(b)
        .map(|(x, y)| u64::from(x.abs_diff(*y)))
        .sum();
    let far = a
        .chunks(4)
        .zip(b.chunks(4))
        .filter(|(x, y)| x.iter().zip(y.iter()).any(|(p, q)| p.abs_diff(*q) > 64))
        .count();
    (total as f64 / a.len() as f64, far as f64 / pixels as f64)
}

#[test]
fn a_printed_page_looks_like_the_snapshot() {
    let mut harness = Harness::new(app, CONTENT);
    let snapshot = harness.render().expect("the snapshot renders");
    let bytes = harness.pdf(spec()).expect("the page prints");
    let doc = Document::from_bytes(bytes).expect("opens");
    let options = RenderOptions::builder()
        .scale(96.0 / 72.0)
        .background(peniko::Color::WHITE)
        .build();
    let page = doc
        .page(0)
        .expect("page 1")
        .render_with(VelloCpuBackend, &options)
        .expect("the page rasterises");
    let content = page.cropped(MARGIN_PX, MARGIN_PX, CONTENT.width, CONTENT.height);
    let printed = content.to_straight_rgba();
    assert_eq!(printed.len(), snapshot.as_raw().len(), "same size");
    if let Ok(dir) = std::env::var("PDF_RASTER_OUT") {
        let save = |name: &str, pixels: &[u8]| {
            image::RgbaImage::from_raw(CONTENT.width, CONTENT.height, pixels.to_vec())
                .expect("sized")
                .save(format!("{dir}/{name}.png"))
                .expect("saved");
        };
        save("printed", &printed);
        save("snapshot", snapshot.as_raw());
    }
    let (mean, far) = difference(&printed, snapshot.as_raw());
    eprintln!(
        "mean channel difference {mean:.2}, pixels far apart {:.2}%",
        far * 100.0
    );
    assert!(mean < 3.0, "mean channel difference {mean:.2}");
    assert!(
        far < 0.02,
        "{:.2}% of pixels differ by more than 64",
        far * 100.0
    );
}
