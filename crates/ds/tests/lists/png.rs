//! Reading the editor's field plane back: base64, the PNG container and a DEFLATE inflater
//! (RFC 1951: stored and fixed-Huffman blocks, the two the plane can use), written here so the
//! test does not trust the encoder it checks.

/// Standard base64 (RFC 4648 section 4) back to bytes.
pub fn unbase64(text: &str) -> Vec<u8> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let values: Vec<u32> = text
        .bytes()
        .filter(|byte| *byte != b'=')
        .map(|byte| {
            ALPHABET
                .iter()
                .position(|a| *a == byte)
                .unwrap_or_else(|| panic!("{byte} is not base64")) as u32
        })
        .collect();
    values
        .chunks(4)
        .flat_map(|chunk| {
            let n = chunk
                .iter()
                .enumerate()
                .fold(0u32, |n, (i, v)| n | v << (18 - 6 * i));
            (0..chunk.len() - 1).map(move |i| (n >> (16 - 8 * i)) as u8)
        })
        .collect()
}

/// A decoded 8-bit RGB or RGBA PNG.
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub depth: u8,
    pub colour_type: u8,
    pub rows: Vec<Vec<u8>>,
}

impl Image {
    /// Bytes per pixel: 3 for RGB, 4 for RGBA.
    fn channels(&self) -> usize {
        if self.colour_type == 6 { 4 } else { 3 }
    }

    /// The pixel at `(x, y)` as `#rrggbb`, alpha left out.
    pub fn hex(&self, x: usize, y: usize) -> String {
        let row = &self.rows[y];
        let at = x * self.channels();
        let [r, g, b] = [row[at], row[at + 1], row[at + 2]];
        format!("#{r:02x}{g:02x}{b:02x}")
    }

    /// The alpha at `(x, y)`: 255 for an RGB image.
    pub fn alpha(&self, x: usize, y: usize) -> u8 {
        match self.channels() {
            4 => self.rows[y][x * 4 + 3],
            _ => 255,
        }
    }
}

/// Parse a PNG whose every scanline uses filter 0, as the plane's do.
pub fn decode(png: &[u8]) -> Image {
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "PNG signature");
    let mut at = 8;
    let mut header = None;
    let mut idat = Vec::new();
    while at + 8 <= png.len() {
        let length = u32::from_be_bytes([png[at], png[at + 1], png[at + 2], png[at + 3]]) as usize;
        let kind = &png[at + 4..at + 8];
        let data = &png[at + 8..at + 8 + length];
        match kind {
            b"IHDR" => header = Some(data.to_vec()),
            b"IDAT" => idat.extend_from_slice(data),
            _ => {}
        }
        at += 12 + length;
    }
    let header = header.unwrap_or_else(|| panic!("no IHDR"));
    let width = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
    let height = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
    assert_eq!(
        (idat[0], idat[1] & 0x20),
        (0x78, 0),
        "zlib header, no dictionary"
    );
    assert_eq!(
        (u16::from(idat[0]) << 8 | u16::from(idat[1])) % 31,
        0,
        "zlib check bits"
    );
    let raw = inflate(&idat[2..]);
    let channels = if header[9] == 6 { 4 } else { 3 };
    let stride = 1 + width * channels;
    assert_eq!(raw.len(), stride * height, "scanline bytes");
    let rows = raw
        .chunks(stride)
        .map(|line| {
            assert_eq!(line[0], 0, "filter type");
            line[1..].to_vec()
        })
        .collect();
    Image {
        width,
        height,
        depth: header[8],
        colour_type: header[9],
        rows,
    }
}

struct Reader<'a> {
    data: &'a [u8],
    bit: usize,
}

impl Reader<'_> {
    fn bit(&mut self) -> u32 {
        let byte = self.data[self.bit / 8];
        let value = u32::from(byte >> (self.bit % 8) & 1);
        self.bit += 1;
        value
    }

    fn bits(&mut self, count: u32) -> u32 {
        (0..count).fold(0, |value, i| value | self.bit() << i)
    }

    /// A fixed-code literal/length symbol, read a Huffman bit at a time.
    fn literal(&mut self) -> u32 {
        let mut code = 0;
        for length in 1..=9 {
            code = code << 1 | self.bit();
            match (length, code) {
                (7, 0..=0b001_0111) => return 256 + code,
                (8, 0x30..=0xBF) => return code - 0x30,
                (8, 0xC0..=0xC7) => return 280 + code - 0xC0,
                (9, 0x190..=0x1FF) => return 144 + code - 0x190,
                _ => {}
            }
        }
        panic!("no fixed code at bit {}", self.bit)
    }
}

const LENGTH_BASE: [u32; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_EXTRA: [u32; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DISTANCE_BASE: [u32; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DISTANCE_EXTRA: [u32; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

/// Inflate a raw DEFLATE stream of stored and fixed-Huffman blocks.
pub fn inflate(data: &[u8]) -> Vec<u8> {
    let mut reader = Reader { data, bit: 0 };
    let mut out: Vec<u8> = Vec::new();
    loop {
        let last = reader.bits(1);
        match reader.bits(2) {
            0 => {
                reader.bit = reader.bit.div_ceil(8) * 8;
                let length = reader.bits(16) as usize;
                reader.bits(16);
                let start = reader.bit / 8;
                out.extend_from_slice(&data[start..start + length]);
                reader.bit += length * 8;
            }
            1 => loop {
                let symbol = reader.literal();
                match symbol {
                    0..=255 => out.push(symbol as u8),
                    256 => break,
                    _ => {
                        let i = (symbol - 257) as usize;
                        let length = LENGTH_BASE[i] + reader.bits(LENGTH_EXTRA[i]);
                        let code = (0..5).fold(0, |code, _| code << 1 | reader.bit()) as usize;
                        let distance =
                            (DISTANCE_BASE[code] + reader.bits(DISTANCE_EXTRA[code])) as usize;
                        for _ in 0..length {
                            out.push(out[out.len() - distance]);
                        }
                    }
                }
            },
            other => panic!("block type {other} is not one the plane uses"),
        }
        if last == 1 {
            return out;
        }
    }
}
