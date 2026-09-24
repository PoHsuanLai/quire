//! A small PNG writer for the editor's field: 8-bit RGB or RGBA, one IDAT, compressed with
//! fixed-Huffman DEFLATE (RFC 1951 section 3.2.6) inside zlib (RFC 1950). Both images are
//! small, so a greedy matcher that only tries "one pixel back" and "one scanline back" is
//! enough, and needs no dependency.

/// The PNG of `width` x `height` RGB `pixels`, rows top to bottom.
pub(super) fn rgb(width: usize, height: usize, pixels: &[[u8; 3]]) -> Vec<u8> {
    encode(width, height, Channels::Rgb, pixels.as_flattened())
}

/// The PNG of `width` x `height` RGBA `pixels` (straight alpha), rows top to bottom.
pub(super) fn rgba(width: usize, height: usize, pixels: &[[u8; 4]]) -> Vec<u8> {
    encode(width, height, Channels::Rgba, pixels.as_flattened())
}

/// A pixel's layout: PNG colour type 2 or 6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Channels {
    Rgb,
    Rgba,
}

impl Channels {
    fn bytes(self) -> usize {
        match self {
            Channels::Rgb => 3,
            Channels::Rgba => 4,
        }
    }

    fn colour_type(self) -> u8 {
        match self {
            Channels::Rgb => 2,
            Channels::Rgba => 6,
        }
    }
}

fn encode(width: usize, height: usize, channels: Channels, bytes: &[u8]) -> Vec<u8> {
    let row = width * channels.bytes();
    let stride = 1 + row;
    let raw: Vec<u8> = bytes
        .chunks(row)
        .take(height)
        .flat_map(|line| std::iter::once(0u8).chain(line.iter().copied()))
        .collect();
    let mut idat = vec![0x78, 0x01];
    idat.extend(deflate(&raw, &[channels.bytes(), stride]));
    idat.extend(adler32(&raw).to_be_bytes());

    let mut header = Vec::with_capacity(13);
    header.extend(u32::try_from(width).unwrap_or(u32::MAX).to_be_bytes());
    header.extend(u32::try_from(height).unwrap_or(u32::MAX).to_be_bytes());
    // Bit depth 8, the colour type, deflate, adaptive filtering, no interlace.
    header.extend([8, channels.colour_type(), 0, 0, 0]);

    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    chunk(&mut png, *b"IHDR", &header);
    chunk(&mut png, *b"IDAT", &idat);
    chunk(&mut png, *b"IEND", &[]);
    png
}

/// Standard base64 with padding (RFC 4648 section 4).
pub(super) fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for group in bytes.chunks(3) {
        let n = group
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, byte)| n | u32::from(*byte) << (16 - 8 * i));
        for i in 0..4 {
            if i <= group.len() {
                out.push(char::from(ALPHABET[(n >> (18 - 6 * i) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

fn chunk(png: &mut Vec<u8>, kind: [u8; 4], data: &[u8]) {
    png.extend(u32::try_from(data.len()).unwrap_or(u32::MAX).to_be_bytes());
    let start = png.len();
    png.extend(kind);
    png.extend(data);
    let crc = crc32(&png[start..]);
    png.extend(crc.to_be_bytes());
}

/// CRC-32 (ISO-HDLC), the PNG chunk checksum.
fn crc32(bytes: &[u8]) -> u32 {
    !bytes.iter().fold(u32::MAX, |crc, byte| {
        (0..8).fold(crc ^ u32::from(*byte), |crc, _| {
            if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            }
        })
    })
}

/// Adler-32, the zlib trailer.
fn adler32(bytes: &[u8]) -> u32 {
    let (a, b) = bytes.iter().fold((1u32, 0u32), |(a, b), byte| {
        let a = (a + u32::from(*byte)) % 65521;
        (a, (b + a) % 65521)
    });
    b << 16 | a
}

/// The longest DEFLATE match and the shortest.
const MAX_MATCH: usize = 258;
const MIN_MATCH: usize = 3;

/// `data` as one final fixed-Huffman block, matching greedily at the given back distances.
fn deflate(data: &[u8], distances: &[usize]) -> Vec<u8> {
    let mut bits = Bits::default();
    bits.push(1, 1); // BFINAL
    bits.push(1, 2); // BTYPE = 01, fixed Huffman
    let mut at = 0;
    while at < data.len() {
        let best = distances
            .iter()
            .filter(|&&distance| distance <= at)
            .map(|&distance| (matching(data, at, distance), distance))
            .max_by_key(|&(length, _)| length);
        match best {
            Some((length, distance)) if length >= MIN_MATCH => {
                bits.length(length);
                bits.distance(distance);
                at += length;
            }
            _ => {
                bits.literal(u16::from(data[at]));
                at += 1;
            }
        }
    }
    bits.literal(256);
    bits.finish()
}

/// How many bytes from `at` repeat the bytes `distance` back, up to [`MAX_MATCH`].
fn matching(data: &[u8], at: usize, distance: usize) -> usize {
    data[at..]
        .iter()
        .zip(&data[at - distance..])
        .take(MAX_MATCH)
        .take_while(|(a, b)| a == b)
        .count()
}

const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DISTANCE_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DISTANCE_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

/// A little-endian bit stream, as DEFLATE writes everything but Huffman codes.
#[derive(Default)]
struct Bits {
    out: Vec<u8>,
    acc: u32,
    used: u32,
}

impl Bits {
    fn push(&mut self, value: u32, count: u32) {
        self.acc |= value << self.used;
        self.used += count;
        while self.used >= 8 {
            self.out.push((self.acc & 0xff) as u8);
            self.acc >>= 8;
            self.used -= 8;
        }
    }

    /// A Huffman code, which DEFLATE writes most significant bit first.
    fn code(&mut self, code: u32, length: u32) {
        let reversed = (0..length).fold(0, |r, i| r << 1 | (code >> i & 1));
        self.push(reversed, length);
    }

    /// A literal/length symbol in the fixed code (RFC 1951 section 3.2.6).
    fn literal(&mut self, symbol: u16) {
        let symbol = u32::from(symbol);
        match symbol {
            0..=143 => self.code(0x30 + symbol, 8),
            144..=255 => self.code(0x190 + symbol - 144, 9),
            256..=279 => self.code(symbol - 256, 7),
            _ => self.code(0xC0 + symbol - 280, 8),
        }
    }

    fn length(&mut self, length: usize) {
        let length = u16::try_from(length).unwrap_or(258);
        let index = LENGTH_BASE
            .iter()
            .rposition(|&base| base <= length)
            .unwrap_or(0);
        self.literal(257 + index as u16);
        self.push(
            u32::from(length - LENGTH_BASE[index]),
            u32::from(LENGTH_EXTRA[index]),
        );
    }

    fn distance(&mut self, distance: usize) {
        let distance = u16::try_from(distance).unwrap_or(1);
        let index = DISTANCE_BASE
            .iter()
            .rposition(|&base| base <= distance)
            .unwrap_or(0);
        self.code(index as u32, 5);
        self.push(
            u32::from(distance - DISTANCE_BASE[index]),
            u32::from(DISTANCE_EXTRA[index]),
        );
    }

    fn finish(mut self) -> Vec<u8> {
        if self.used > 0 {
            self.out.push((self.acc & 0xff) as u8);
        }
        self.out
    }
}

#[cfg(test)]
mod tests {
    use super::{adler32, base64, crc32};

    #[test]
    fn the_checksums_match_their_references() {
        // RFC 1950's and the PNG spec's check values for "123456789".
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(adler32(b"123456789"), 0x091E_01DE);
        assert_eq!(adler32(b""), 1);
    }

    #[test]
    fn base64_pads_like_rfc_4648() {
        const CASES: &[(&str, &str)] = &[
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foobar", "Zm9vYmFy"),
        ];
        for (plain, want) in CASES {
            assert_eq!(base64(plain.as_bytes()), *want, "{plain}");
        }
    }
}
