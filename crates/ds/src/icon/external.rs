//! Icons quire does not draw: a status item's pixmap, an icon theme's file, a third party's
//! symbolic SVG (design/08-ICONS.md section 1.5). They arrive as a URL the document loads, and
//! draw either recoloured to the text colour (symbolic) or as they are (an image).

use super::Icon;
use super::render::IconSize;
use crate::components::space_editor::png::base64;
use crate::error::DsError;
use std::path::Path;

/// Where an external icon's pixels come from: a `data:` or `file:` URL, the two schemes a
/// quire document's net provider answers. Parsed once, so what reaches the stylesheet can
/// always sit inside a quoted CSS `url("…")`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IconUrl(String);

impl IconUrl {
    /// `url`, which must be `data:` or `file:` (any case). A character that would end or break
    /// the CSS string (`"`, `\`, a control character) is percent-encoded, which leaves the URL
    /// meaning the same thing.
    pub fn parse(url: &str) -> Result<Self, DsError> {
        let scheme = url
            .split_once(':')
            .map(|(scheme, _)| scheme.to_ascii_lowercase());
        match scheme.as_deref() {
            Some("data" | "file") => Ok(IconUrl(escape_for_css(url))),
            _ => Err(DsError::IconScheme {
                url: url.to_string(),
            }),
        }
    }

    /// A PNG's bytes as a `data:image/png;base64,…` URL (a status item's `IconPixmap`, once the
    /// caller has encoded it).
    pub fn png(bytes: &[u8]) -> Self {
        IconUrl(format!("data:image/png;base64,{}", base64(bytes)))
    }

    /// An SVG document as a `data:image/svg+xml,…` URL, percent-encoded (spike S7's form).
    pub fn svg(document: &str) -> Self {
        IconUrl(format!(
            "data:image/svg+xml,{}",
            percent(document, SVG_KEEP)
        ))
    }

    /// An absolute path as a `file://` URL, percent-encoded.
    pub fn file(path: &Path) -> Result<Self, DsError> {
        if !path.is_absolute() {
            return Err(DsError::IconPathRelative {
                path: path.to_path_buf(),
            });
        }
        let text = path.to_string_lossy();
        Ok(IconUrl(format!("file://{}", percent(&text, PATH_KEEP))))
    }

    /// The URL as written into the page.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// An external icon and the size it is drawn at, in the glyph scale.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalIcon {
    /// Where its pixels come from.
    pub url: IconUrl,
    /// How big it is drawn (a square).
    pub size: IconSize,
}

/// What an icon slot shows: one of quire's glyphs, or an external icon.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IconSource {
    /// A quire glyph, stroked in the text colour.
    Glyph(Icon),
    /// An external icon used as a mask: only its alpha counts, painted in the text colour, so
    /// it follows `--ink`, hover and `--f-ink*` exactly as a glyph does (a freedesktop
    /// `*-symbolic` icon, or a monochrome tray pixmap).
    Symbolic(ExternalIcon),
    /// An external icon drawn as it is: a coloured tray icon states a fact about its app.
    Image(ExternalIcon),
}

impl From<Icon> for IconSource {
    fn from(icon: Icon) -> Self {
        IconSource::Glyph(icon)
    }
}

/// The characters an SVG `data:` URL keeps as they are; everything else is percent-encoded
/// (`#` above all, which would start a fragment).
const SVG_KEEP: &[u8] = b"-._~!$&'()*+,;=:@/? ";

/// The characters a `file://` path keeps as they are.
const PATH_KEEP: &[u8] = b"-._~!$&'()*+,;=:@/";

/// `text` with every byte that is not ASCII alphanumeric or in `keep` percent-encoded. A space
/// in `keep` is written `%20` all the same: it is kept only in the sense that it is expected.
fn percent(text: &str, keep: &[u8]) -> String {
    text.bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || (keep.contains(&byte) && byte != b' ') {
                char::from(byte).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}

/// `url` with the characters that would break a quoted CSS string percent-encoded.
fn escape_for_css(url: &str) -> String {
    url.chars()
        .map(|c| {
            if c == '"' || c == '\\' || c.is_control() {
                let mut bytes = [0u8; 4];
                c.encode_utf8(&mut bytes)
                    .bytes()
                    .map(|byte| format!("%{byte:02X}"))
                    .collect()
            } else {
                c.to_string()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::IconUrl;
    use crate::error::DsError;
    use std::path::Path;

    #[test]
    fn only_data_and_file_urls_are_icons() {
        // (url, accepted as)
        const CASES: &[(&str, Option<&str>)] = &[
            (
                "data:image/png;base64,AAAA",
                Some("data:image/png;base64,AAAA"),
            ),
            (
                "DATA:image/png;base64,AAAA",
                Some("DATA:image/png;base64,AAAA"),
            ),
            (
                "file:///usr/share/icons/a.svg",
                Some("file:///usr/share/icons/a.svg"),
            ),
            ("file:///a \"b\".svg", Some("file:///a %22b%22.svg")),
            ("data:x,a\\b\nc", Some("data:x,a%5Cb%0Ac")),
            ("https://example.com/a.png", None),
            ("/usr/share/icons/a.svg", None),
            ("", None),
        ];
        for (url, want) in CASES {
            let got = IconUrl::parse(url).ok();
            assert_eq!(got.as_ref().map(IconUrl::as_str), *want, "{url:?}");
        }
    }

    #[test]
    fn an_svg_is_percent_encoded_like_spike_s7() {
        let url = IconUrl::svg("<svg fill='#000'><circle r=\"1\"/></svg>");
        assert_eq!(
            url.as_str(),
            "data:image/svg+xml,%3Csvg%20fill='%23000'%3E%3Ccircle%20r=%221%22/%3E%3C/svg%3E"
        );
    }

    #[test]
    fn a_png_is_base64() {
        assert_eq!(IconUrl::png(b"foo").as_str(), "data:image/png;base64,Zm9v");
    }

    #[test]
    fn a_file_url_needs_an_absolute_path() {
        let url = IconUrl::file(Path::new("/usr/share/icons/hi color/a#1.svg"));
        assert_eq!(
            url.as_ref().map(IconUrl::as_str),
            Ok("file:///usr/share/icons/hi%20color/a%231.svg")
        );
        assert_eq!(
            IconUrl::file(Path::new("icons/a.svg")),
            Err(DsError::IconPathRelative {
                path: "icons/a.svg".into()
            })
        );
    }
}
