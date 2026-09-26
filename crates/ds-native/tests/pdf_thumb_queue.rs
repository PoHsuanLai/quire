//! The `pdf-thumb` worker's latest-wins queue (sill M9: the launcher's preview changes on every
//! arrow key). One thumbnail asked for 20 PDFs in quick succession, as holding Down would, runs
//! at most a few rasters (the one already running when each request came, and the last), never
//! 20, and ends showing the last page asked for. Its own test binary, so the process-wide
//! raster count is this test's alone.

use dioxus::prelude::*;
use ds::{Appearance, Ds, Material, Px, Size};
use ds_native::harness::settle_until;
use ds_native::{Harness, PdfFileThumb, Viewport, pdf_thumb_rasters};
use pdfrum_common::Limits;
use pdfrum_edit::{EditDoc, SaveOptions, blank_document, save};
use peniko::Color;
use peniko::kurbo::{Rect, Size as Sheet};
use std::path::PathBuf;
use std::time::Duration;

const FILES: usize = 20;

/// At most this many rasters for 20 superseding requests: the first, whichever were running when
/// the worker came free during the burst, and the last. Measured 2 on this machine.
const MOST_RASTERS: u64 = 4;

const RED: [u8; 3] = [220, 30, 30];
const BLUE: [u8; 3] = [30, 60, 220];

/// A room large enough that a raster outlasts several renders of the harness.
const ROOM: Size = Size {
    width: Px(600.0),
    height: Px(800.0),
};

const VIEW: Viewport = Viewport {
    width: 640,
    height: 840,
    scale_percent: 100,
};

static PATH: GlobalSignal<PathBuf> = Signal::global(PathBuf::new);

fn letter(rgb: [u8; 3]) -> Vec<u8> {
    let base = blank_document(&[Sheet::new(612.0, 792.0)]).expect("a blank page");
    let mut edit = EditDoc::new(&base);
    edit.draw_pages(&Limits::default(), |canvas| {
        canvas.fill_rect(
            Rect::new(0.0, 0.0, 612.0, 792.0),
            Color::from_rgb8(rgb[0], rgb[1], rgb[2]),
        );
    })
    .expect("the page draws");
    let mut bytes = Vec::new();
    save(&edit, &SaveOptions::default(), &mut bytes).expect("the PDF saves");
    bytes
}

/// Twenty PDFs: red, but the last blue.
fn files() -> Vec<PathBuf> {
    let dir = std::env::temp_dir().join(format!("quire-pdf-queue-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    (0..FILES)
        .map(|i| {
            let path = dir.join(format!("{i}.pdf"));
            let colour = if i + 1 == FILES { BLUE } else { RED };
            std::fs::write(&path, letter(colour)).expect("written");
            path
        })
        .collect()
}

fn app() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            PdfFileThumb { path: PATH(), size: ROOM }
        }
    }
}

#[test]
fn twenty_quick_requests_run_a_few_rasters_and_show_the_last() {
    let files = files();
    let mut harness = Harness::new(app, VIEW);
    let before = pdf_thumb_rasters();
    for path in &files {
        harness.within(|| *PATH.write() = path.clone());
        harness.advance(Duration::ZERO);
    }
    let last = files.last().cloned().unwrap_or_default();
    settle_until(&mut harness, |h| {
        h.attr(".ds-pdf-thumb", "data-state").as_deref() == Some("ready")
    });
    // Let any raster still running for a superseded request finish, so it is counted.
    harness.advance(Duration::from_millis(500));
    let ran = pdf_thumb_rasters() - before;
    eprintln!("{ran} rasters for {FILES} requests");
    assert!(ran <= MOST_RASTERS, "{ran} rasters for {FILES} requests");
    assert_eq!(harness.within(|| PATH.read().clone()), last);
    harness.advance(Duration::from_millis(300));
    let sheet = harness.rect(".ds-pdf-thumb-sheet").expect("a sheet");
    let shot = harness.render().expect("renders");
    let centre = shot.get_pixel(
        (sheet.left().0 + sheet.size.width.0 / 2.0) as u32,
        (sheet.top().0 + sheet.size.height.0 / 2.0) as u32,
    );
    let blue = centre.0[..3]
        .iter()
        .zip(BLUE)
        .all(|(p, q)| p.abs_diff(q) <= 12);
    assert!(blue, "the last page (blue) is shown: {centre:?}");
}
