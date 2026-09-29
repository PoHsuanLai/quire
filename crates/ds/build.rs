//! One generated input, written to `$OUT_DIR`: `grain.uri`, the frame's 128 px grain tile as a
//! `data:image/png;base64,…` URI, which `ds::css::GRAIN_PNG` includes (see [`grain`]).
//!
//! `concat!` can include a file but cannot base64-encode one, so the encoding happens here. The
//! encoder is fifteen lines, so it is not worth a build dependency.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap_or_default());
    let uri = format!("data:image/png;base64,{}", base64(&grain::png()));
    if let Err(why) = fs::write(out.join("grain.uri"), uri) {
        panic!("writing grain.uri: {why}");
    }
}

/// Standard base64 with padding, RFC 4648 section 4.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, b)| n | u32::from(*b) << (16 - 8 * i));
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(char::from(ALPHABET[(n >> (18 - 6 * i) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// The grain tile (design/03-COLOR.md section 8): the prototype's Park-Miller generator, seed 7,
/// one draw per pixel over 128 x 128, as a PNG.
///
/// The prototype draws an opaque grey `v = floor(rnd x 255)` and blends it with `overlay`,
/// which Blitz cannot do. This tile is the "alpha noise" the plan names instead (open decision
/// 2, proposed mapping): a grey below the middle becomes black at alpha `255 - 2v`, one above
/// becomes white at alpha `2v - 255`. Painted normally, that is exactly `overlay` of the same grey
/// wherever the frame is darker than the middle in the black half, and wherever it is lighter in
/// the white half; the element's opacity (`--f-grain`) then sets the strength as before.
mod grain {
    const SIZE: u32 = 128;
    const MODULUS: u64 = 2_147_483_647;
    const MULTIPLIER: u64 = 16_807;
    const SEED: u64 = 7;

    /// The prototype's greys, in raster order, computed the way its JavaScript does.
    fn greys() -> Vec<u8> {
        let mut seed = SEED;
        (0..SIZE * SIZE)
            .map(|_| {
                seed = seed * MULTIPLIER % MODULUS;
                let rnd = seed as f64 / MODULUS as f64;
                // `Math.floor(rnd * 255)`: rnd is below 1, so this is 0..=254.
                (rnd * 255.0).floor() as u8
            })
            .collect()
    }

    /// Grey-and-alpha pixels, two bytes each.
    fn pixel(grey: u8) -> [u8; 2] {
        let twice = i16::from(grey) * 2;
        if twice < 255 {
            [0, (255 - twice) as u8]
        } else {
            [255, (twice - 255) as u8]
        }
    }

    /// The tile as a PNG: 8-bit grey with alpha, no filtering, stored (uncompressed) deflate.
    pub fn png() -> Vec<u8> {
        let greys = greys();
        let mut raw = Vec::with_capacity((SIZE * (SIZE * 2 + 1)) as usize);
        for row in greys.chunks(SIZE as usize) {
            raw.push(0); // filter type None
            raw.extend(row.iter().flat_map(|&grey| pixel(grey)));
        }
        let mut header = Vec::with_capacity(13);
        header.extend(SIZE.to_be_bytes());
        header.extend(SIZE.to_be_bytes());
        header.extend([8, 4, 0, 0, 0]); // depth 8, grey + alpha, deflate, no filter, no interlace
        let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
        chunk(&mut png, b"IHDR", &header);
        chunk(&mut png, b"IDAT", &zlib_stored(&raw));
        chunk(&mut png, b"IEND", &[]);
        png
    }

    fn chunk(png: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
        png.extend(u32::try_from(data.len()).unwrap_or(u32::MAX).to_be_bytes());
        let start = png.len();
        png.extend(kind);
        png.extend(data);
        let crc = crc32(&png[start..]);
        png.extend(crc.to_be_bytes());
    }

    /// A zlib stream of stored deflate blocks (RFC 1950, RFC 1951 section 3.2.4).
    fn zlib_stored(data: &[u8]) -> Vec<u8> {
        let mut out = vec![0x78, 0x01];
        let blocks = data.chunks(65_535).collect::<Vec<_>>();
        for (index, block) in blocks.iter().enumerate() {
            let last = u8::from(index + 1 == blocks.len());
            let len = u16::try_from(block.len()).unwrap_or(u16::MAX);
            out.push(last);
            out.extend(len.to_le_bytes());
            out.extend((!len).to_le_bytes());
            out.extend(*block);
        }
        out.extend(adler32(data).to_be_bytes());
        out
    }

    fn adler32(data: &[u8]) -> u32 {
        let (a, b) = data.iter().fold((1u32, 0u32), |(a, b), &byte| {
            let a = (a + u32::from(byte)) % 65_521;
            (a, (b + a) % 65_521)
        });
        (b << 16) | a
    }

    /// CRC-32 as PNG uses it (ISO 3309, reflected, polynomial 0xEDB88320).
    fn crc32(data: &[u8]) -> u32 {
        !data.iter().fold(!0u32, |crc, &byte| {
            (0..8).fold(crc ^ u32::from(byte), |crc, _| {
                if crc & 1 == 1 {
                    (crc >> 1) ^ 0xEDB8_8320
                } else {
                    crc >> 1
                }
            })
        })
    }
}
