//! `data:` URIs: the wallpaper the materials sit on, and the contact sheet's pictures.

/// The standard base64 alphabet (RFC 4648 section 4).
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// `bytes` in base64, padded.
pub fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let [a, b, c] = [0, 1, 2].map(|i| chunk.get(i).copied().unwrap_or(0));
        let group = (u32::from(a) << 16) | (u32::from(b) << 8) | u32::from(c);
        let symbols = [18, 12, 6, 0].map(|shift| ALPHABET[((group >> shift) & 63) as usize]);
        let kept = chunk.len() + 1;
        for (index, symbol) in symbols.into_iter().enumerate() {
            out.push(if index < kept {
                char::from(symbol)
            } else {
                '='
            });
        }
    }
    out
}

/// A PNG as a `data:` URI.
pub fn png(bytes: &[u8]) -> String {
    format!("data:image/png;base64,{}", base64(bytes))
}

#[cfg(test)]
mod tests {
    use super::base64;

    #[test]
    fn rfc_4648_vectors() {
        const CASES: &[(&str, &str)] = &[
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ];
        for (plain, encoded) in CASES {
            assert_eq!(base64(plain.as_bytes()), *encoded, "{plain:?}");
        }
    }
}
