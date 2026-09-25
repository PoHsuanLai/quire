//! The printout fixture shared by the PDF tests and the `pdf` example: Latin and CJK text, a
//! JPEG and a PNG as `data:` URLs, a keep-together block placed to straddle page 1's end, and
//! a forced page break.

#![allow(dead_code, reason = "each includer uses its own part of the fixture")]

use image::codecs::jpeg::JpegEncoder;
use image::{ImageFormat, Rgb, RgbImage, Rgba, RgbaImage};
use std::io::Cursor;

/// A Latin word only the first paragraph has.
pub const LATIN_WORD: &str = "kestrel";
/// A CJK word only the CJK paragraph has.
pub const CJK_WORD: &str = "會議紀錄";
/// The keep-together block's heading.
pub const KEEP_HEADING: &str = "Keep-together block";
/// The forced-break message's heading.
pub const BREAK_HEADING: &str = "Second message";
/// Text that only the screen sees (`@media print` hides it).
pub const SCREEN_ONLY: &str = "Shown on screen only";

/// A 320 x 200 photo-like JPEG (quality 80).
pub fn jpeg() -> Vec<u8> {
    let photo = RgbImage::from_fn(320, 200, |x, y| {
        let wave = ((x as f32 / 9.0).sin() * 40.0 + (y as f32 / 7.0).cos() * 40.0) as i32;
        Rgb([
            (x * 255 / 320) as u8,
            (128 + wave).clamp(0, 255) as u8,
            (y * 255 / 200) as u8,
        ])
    });
    let mut bytes = Vec::new();
    JpegEncoder::new_with_quality(&mut bytes, 80)
        .encode_image(&photo)
        .expect("a JPEG encodes");
    bytes
}

/// A 120 x 40 PNG with an alpha ramp.
pub fn png() -> Vec<u8> {
    let chart = RgbaImage::from_fn(120, 40, |x, y| {
        Rgba([30, (y * 6) as u8, 200, (55 + x * 200 / 120) as u8])
    });
    let mut bytes = Vec::new();
    chart
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .expect("a PNG encodes");
    bytes
}

/// Standard base64 with padding, for `data:` URLs.
pub fn base64(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    bytes
        .chunks(3)
        .flat_map(|chunk| {
            let word = chunk.iter().enumerate().fold(0u32, |word, (n, &byte)| {
                word | (u32::from(byte) << (16 - 8 * n))
            });
            (0..4).map(move |n| {
                if n <= chunk.len() {
                    char::from(DIGITS[((word >> (18 - 6 * n)) & 63) as usize])
                } else {
                    '='
                }
            })
        })
        .collect()
}

/// The printout, with `jpeg` and `png` inlined.
pub fn html(jpeg: &[u8], png: &[u8]) -> String {
    let jpeg = base64(jpeg);
    let png = base64(png);
    format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>Print fixture</title><style>
html {{ background: #fff; color: #1a1a1a; }}
body {{ margin: 0; font: 15px/1.5 "Noto Serif", serif; }}
h1 {{ font: 700 24px/1.3 "Karla", sans-serif; margin: 0 0 12px; }}
h2 {{ font: 700 18px/1.3 "Karla", sans-serif; margin: 0 0 8px; }}
p {{ margin: 0 0 10px; }}
.cjk {{ font-family: "Noto Sans CJK TC", sans-serif; }}
.figures img {{ vertical-align: top; margin-right: 12px; }}
.spacer {{ height: 440px; border-left: 3px solid #ccc; }}
.keep {{ border: 1px solid #999; padding: 8px 12px; background: linear-gradient(#eef3ff, #ffffff); }}
.screen-only {{ color: #c00; }}
@media print {{ .screen-only {{ display: none; }} }}
</style></head><body>
<h1>Quarterly {LATIN_WORD} report {CJK_WORD}</h1>
<p>The quick brown fox jumps over the lazy dog while the {LATIN_WORD} hovers above the field, watching for anything that moves in the long grass below the hedge.</p>
<p class="cjk">這是一封測試郵件，用來檢查中文字在 PDF 裡能否被正確嵌入、子集化，並且可以被選取與複製。{CJK_WORD}一式兩份。</p>
<p class="figures"><img src="data:image/jpeg;base64,{jpeg}" width="320" height="200" alt="photo"><img src="data:image/png;base64,{png}" width="120" height="40" alt="chart"></p>
<p class="screen-only">{SCREEN_ONLY}</p>
<div class="spacer"></div>
<div class="keep" data-break-inside="avoid">
<h2>{KEEP_HEADING}</h2>
<p>These lines belong together: the block is marked to avoid a break inside, so when it would straddle the end of the first page it moves whole to the second.</p>
<p>A second paragraph makes the block tall enough that it cannot fit in the space left on the first page.</p>
<p>A third paragraph, for good measure, so the block is well over two hundred pixels tall.</p>
</div>
<article data-break-before="page">
<h2>{BREAK_HEADING}</h2>
<p>This message starts a page of its own, whatever room was left on the page before it.</p>
</article>
</body></html>"#
    )
}
