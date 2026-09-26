//! PDF thumbnails from a path (sill M9's launcher preview): the first page of a PDF written here
//! is rasterised at the page's aspect and painted red where the page is red; a second request for
//! the file as it is is answered from the cache (proved by rewriting the file's bytes under its
//! old modification time and still getting the old page), and a new modification time misses;
//! a file that is not a PDF, a missing one, and one locked with a password fail to a plate; an
//! empty file and a document with no pages are a blank sheet; and on a Blitz document the page
//! read on the worker lands and paints (the loading and pending states are `ds`'s SSR goldens).

use dioxus::prelude::*;
use ds::{Appearance, Ds, Material, PdfPage, PdfTrouble, Px, Scale, Size};
use ds_native::harness::settle_until;
use ds_native::{DeviceBox, Harness, PdfFileThumb, ThumbRequest, Viewport};
use ds_native::{pdf_thumb_blocking, pdf_thumb_bytes, pdf_thumb_cached};
use pdfrum_common::Limits;
use pdfrum_edit::{EditDoc, Encryption, SaveOptions, blank_document, save};
use peniko::Color;
use peniko::kurbo::{Rect, Size as Sheet};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

const RED: [u8; 3] = [220, 30, 30];
const BLUE: [u8; 3] = [30, 60, 220];

const ROOM: Size = Size {
    width: Px(120.0),
    height: Px(160.0),
};

const FIT: DeviceBox = DeviceBox {
    width: 120,
    height: 160,
};

const VIEW: Viewport = Viewport {
    width: 200,
    height: 200,
    scale_percent: 100,
};

/// A one-page PDF of `width` x `height` points, filled with `rgb`, saved with `options`.
fn pdf(width: f64, height: f64, rgb: [u8; 3], options: &SaveOptions) -> Vec<u8> {
    let base = blank_document(&[Sheet::new(width, height)]).expect("a blank page");
    let mut edit = EditDoc::new(&base);
    edit.draw_pages(&Limits::default(), |canvas| {
        canvas.fill_rect(
            Rect::new(0.0, 0.0, width, height),
            Color::from_rgb8(rgb[0], rgb[1], rgb[2]),
        );
    })
    .expect("the page draws");
    let mut bytes = Vec::new();
    save(&edit, options, &mut bytes).expect("the PDF saves");
    bytes
}

fn letter(rgb: [u8; 3]) -> Vec<u8> {
    pdf(612.0, 792.0, rgb, &SaveOptions::default())
}

/// A catalog whose page tree has no pages, written by hand (a writer will not make one).
fn no_pages() -> Vec<u8> {
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>",
        "<< /Type /Pages /Count 0 /Kids [] >>",
    ];
    let mut out = String::from("%PDF-1.7\n");
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.push_str(&format!("{} 0 obj\n{body}\nendobj\n", i + 1));
    }
    let xref = out.len();
    out.push_str(&format!(
        "xref\n0 {}\n0000000000 65535 f \n",
        objects.len() + 1
    ));
    for offset in offsets {
        out.push_str(&format!("{offset:010} 00000 n \n"));
    }
    out.push_str(&format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
        objects.len() + 1
    ));
    out.into_bytes()
}

/// A fresh file under the temp directory, private to this process and `name`.
fn file(name: &str, bytes: &[u8]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("quire-pdf-thumb-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join(name);
    std::fs::write(&path, bytes).expect("written");
    path
}

fn request(path: PathBuf) -> ThumbRequest {
    ThumbRequest {
        path,
        size: ROOM,
        scale: Scale::ONE,
    }
}

/// The PNG inside a `data:image/png;base64,` source, decoded.
fn pixels(page: &PdfPage) -> image::RgbaImage {
    let PdfPage::Ready { image, .. } = page else {
        panic!("not ready: {page:?}");
    };
    let encoded = image
        .0
        .strip_prefix("data:image/png;base64,")
        .expect("a PNG URI");
    let bytes = base64(encoded);
    image::load_from_memory(&bytes).expect("a PNG").to_rgba8()
}

/// Standard base64, no dependency for one test.
fn base64(text: &str) -> Vec<u8> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let values: Vec<u8> = text
        .bytes()
        .filter(|b| *b != b'=')
        .filter_map(|b| ALPHABET.iter().position(|a| *a == b).map(|v| v as u8))
        .collect();
    values
        .chunks(4)
        .flat_map(|chunk| {
            let word = chunk
                .iter()
                .enumerate()
                .fold(0u32, |word, (i, v)| word | (u32::from(*v) << (18 - 6 * i)));
            let bytes = word.to_be_bytes();
            bytes[1..chunk.len()].to_vec()
        })
        .collect()
}

fn near(pixel: image::Rgba<u8>, rgb: [u8; 3]) -> bool {
    pixel.0[..3]
        .iter()
        .zip(rgb)
        .all(|(p, q)| p.abs_diff(q) <= 12)
}

#[test]
fn a_page_rasterises_at_its_aspect_in_its_colour() {
    let page = pdf_thumb_bytes(letter(RED), FIT);
    let PdfPage::Ready { sheet, .. } = &page else {
        panic!("not ready: {page:?}");
    };
    assert_eq!((sheet.width, sheet.height), (612, 792));
    let picture = pixels(&page);
    // Letter fitted into 120 x 160: as wide as the room, 155 high.
    assert_eq!(picture.width(), 120);
    assert!(
        (154..=156).contains(&picture.height()),
        "{}",
        picture.height()
    );
    let centre = *picture.get_pixel(60, picture.height() / 2);
    assert!(near(centre, RED), "{centre:?}");
    let landscape = pixels(&pdf_thumb_bytes(
        pdf(842.0, 595.0, BLUE, &SaveOptions::default()),
        FIT,
    ));
    assert_eq!(landscape.width(), 120);
    assert!(landscape.height() < 90, "{}", landscape.height());
    assert!(near(*landscape.get_pixel(60, 40), BLUE));
}

#[test]
fn the_same_file_unchanged_is_a_cache_hit_and_a_new_mtime_misses() {
    let path = file("cached.pdf", &letter(RED));
    let request = request(path.clone());
    assert_eq!(pdf_thumb_cached(&request), None, "nothing cached yet");
    let first = pdf_thumb_blocking(&request);
    assert!(near(*pixels(&first).get_pixel(60, 77), RED));
    // Rewrite the bytes, then put the old modification time back: only a cache answers red now.
    let modified = std::fs::metadata(&path)
        .and_then(|m| m.modified())
        .expect("mtime");
    std::fs::write(&path, letter(BLUE)).expect("rewritten");
    std::fs::File::options()
        .write(true)
        .open(&path)
        .and_then(|f| f.set_modified(modified))
        .expect("mtime restored");
    assert_eq!(pdf_thumb_cached(&request), Some(first.clone()), "a hit");
    assert_eq!(pdf_thumb_blocking(&request), first, "still the hit");
    // Another size is its own entry, and rasterises the file as it is now.
    let wider = ThumbRequest {
        size: Size {
            width: Px(60.0),
            height: Px(80.0),
        },
        ..request.clone()
    };
    assert!(near(
        *pixels(&pdf_thumb_blocking(&wider)).get_pixel(30, 38),
        BLUE
    ));
    // A new modification time misses.
    std::fs::File::options()
        .write(true)
        .open(&path)
        .and_then(|f| f.set_modified(modified + Duration::from_secs(5)))
        .expect("mtime bumped");
    assert_eq!(pdf_thumb_cached(&request), None, "a miss");
    assert!(near(
        *pixels(&pdf_thumb_blocking(&request)).get_pixel(60, 77),
        BLUE
    ));
}

#[test]
fn what_cannot_be_drawn_fails_and_what_has_no_page_is_blank() {
    let locked = SaveOptions::builder()
        .encrypt(
            Encryption::builder()
                .user_password(b"open-me".to_vec())
                .owner_password(b"owner".to_vec())
                .build(),
        )
        .build();
    let cases: &[(&str, Vec<u8>, PdfPage)] = &[
        (
            "not a PDF",
            b"%PDF-1.7 and then nothing".to_vec(),
            PdfPage::Failed(PdfTrouble::Unreadable),
        ),
        (
            "a PNG",
            b"\x89PNG\r\n\x1a\n".to_vec(),
            PdfPage::Failed(PdfTrouble::Unreadable),
        ),
        (
            "locked",
            pdf(612.0, 792.0, RED, &locked),
            PdfPage::Failed(PdfTrouble::Locked),
        ),
        ("empty file", Vec::new(), PdfPage::Empty),
        ("no pages", no_pages(), PdfPage::Empty),
    ];
    for (name, bytes, want) in cases {
        assert_eq!(&pdf_thumb_bytes(bytes.clone(), FIT), want, "{name}");
    }
    let missing = request(std::env::temp_dir().join("quire-pdf-thumb-no-such-file.pdf"));
    assert_eq!(
        pdf_thumb_blocking(&missing),
        PdfPage::Failed(PdfTrouble::Unreadable)
    );
    // An owner-only lock (empty user password) opens and draws.
    let owner_only = SaveOptions::builder()
        .encrypt(
            Encryption::builder()
                .owner_password(b"owner".to_vec())
                .build(),
        )
        .build();
    let opened = pdf_thumb_bytes(pdf(612.0, 792.0, RED, &owner_only), FIT);
    assert!(near(*pixels(&opened).get_pixel(60, 77), RED));
}

static SHOWN_PATH: OnceLock<PathBuf> = OnceLock::new();
static BROKEN_PATH: OnceLock<PathBuf> = OnceLock::new();

#[allow(non_snake_case)]
fn Shown() -> Element {
    let path = SHOWN_PATH.get().cloned().unwrap_or_default();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            PdfFileThumb { path, size: ROOM, label: "Red.pdf" }
        }
    }
}

#[allow(non_snake_case)]
fn Broken() -> Element {
    let path = BROKEN_PATH.get().cloned().unwrap_or_default();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            PdfFileThumb { path, size: ROOM }
        }
    }
}

#[test]
fn on_a_document_it_loads_then_paints_the_page() {
    SHOWN_PATH.get_or_init(|| file("shown.pdf", &letter(RED)));
    let mut harness = Harness::new(Shown, VIEW);
    settle_until(&mut harness, |h| {
        h.attr(".ds-pdf-thumb", "data-state").as_deref() == Some("ready")
    });
    assert_eq!(
        harness.attr(".ds-pdf-thumb", "aria-label").as_deref(),
        Some("Red.pdf")
    );
    // The sheet was transparent while loading; it fades in over `--t-quick`.
    harness.advance(Duration::from_millis(400));
    let sheet = harness.rect(".ds-pdf-thumb-sheet").expect("a sheet");
    let shot = harness.render().expect("renders");
    if let Ok(dir) = std::env::var("PDF_THUMB_OUT") {
        shot.save(format!("{dir}/thumb.png")).expect("saved");
    }
    let centre = *shot.get_pixel(
        (sheet.left().0 + sheet.size.width.0 / 2.0) as u32,
        (sheet.top().0 + sheet.size.height.0 / 2.0) as u32,
    );
    assert!(near(centre, RED), "the page paints red: {centre:?}");
}

#[test]
fn a_broken_file_lands_on_the_plate() {
    BROKEN_PATH.get_or_init(|| file("broken.pdf", b"not a pdf at all"));
    let mut harness = Harness::new(Broken, VIEW);
    settle_until(&mut harness, |h| {
        h.attr(".ds-pdf-thumb", "data-state").as_deref() == Some("failed")
    });
    assert_eq!(
        harness
            .attr(".ds-pdf-thumb-plate", "data-trouble")
            .as_deref(),
        Some("unreadable")
    );
    assert_eq!(harness.count(".ds-pdf-thumb-sheet"), 0);
}
