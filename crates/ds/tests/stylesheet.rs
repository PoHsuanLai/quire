//! The generated stylesheet: pinned by a golden file, free of the constructs Blitz cannot use,
//! with every attribute selector in the namespaced form Blitz matches, in the documented
//! cascade order.
//!
//! `DS_BLESS=1 cargo test -p ds --test stylesheet` rewrites the golden file.

use ds::css::GRAIN_PNG;
use ds::stylesheet;
use std::path::PathBuf;

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/stylesheet.css")
}

#[test]
fn the_stylesheet_matches_its_golden_file() {
    let path = golden_path();
    if std::env::var_os("DS_BLESS").is_some() {
        std::fs::write(&path, stylesheet())
            .unwrap_or_else(|why| panic!("writing {}: {why}", path.display()));
        return;
    }
    let golden = std::fs::read_to_string(&path)
        .unwrap_or_else(|why| panic!("{}: {why}; run with DS_BLESS=1", path.display()));
    if golden != stylesheet() {
        let first = golden
            .lines()
            .zip(stylesheet().lines())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| golden.lines().count().min(stylesheet().lines().count()));
        panic!(
            "the stylesheet differs from {} from line {}; rerun with DS_BLESS=1 if intended\n\
             golden:    {:?}\ngenerated: {:?}",
            path.display(),
            first + 1,
            golden.lines().nth(first),
            stylesheet().lines().nth(first)
        );
    }
}

fn strip_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        match rest[start + 2..].find("*/") {
            Some(end) => rest = &rest[start + 2 + end + 2..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

/// Every declared property name, taken at declaration boundaries: after `{` or `;`, up to `:`.
/// Selectors and values are never mistaken for one, so `.ds-menu-filter` would not count.
fn properties(css: &str) -> Vec<String> {
    css.split(['{', ';', '}'])
        .filter_map(|piece| piece.split_once(':'))
        .map(|(name, _)| name.trim().to_ascii_lowercase())
        .filter(|name| name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
        .collect()
}

#[test]
fn nothing_blitz_cannot_draw_or_the_lint_bans() {
    let css = strip_comments(stylesheet());
    for construct in [
        ":root",
        ":focus-visible",
        ":focus-within",
        "@media",
        "prefers-",
    ] {
        assert!(!css.contains(construct), "the stylesheet uses {construct}");
    }
    let properties = properties(&css);
    for banned in [
        "filter",
        "backdrop-filter",
        "text-overflow",
        "line-clamp",
        "-webkit-line-clamp",
    ] {
        assert!(
            !properties.iter().any(|p| p == banned),
            "the stylesheet declares {banned}"
        );
    }
    assert!(
        properties.iter().any(|p| p == "mask-image"),
        "the parser found no declarations"
    );
}

#[test]
fn every_attribute_selector_is_namespaced() {
    // `[` appears in this stylesheet only in selectors: no value uses brackets.
    let css = strip_comments(stylesheet());
    let unprefixed: Vec<String> = css
        .match_indices('[')
        .filter(|(index, _)| !css[index + 1..].starts_with("*|"))
        .map(|(index, _)| css[index..(index + 40).min(css.len())].to_owned())
        .collect();
    assert!(
        unprefixed.is_empty(),
        "attribute selectors without *|: {unprefixed:#?}"
    );
    assert!(
        css.contains("[*|data-theme=dark]"),
        "no attribute selector was checked"
    );
}

#[test]
fn the_cascade_is_in_the_documented_order() {
    let css = stylesheet();
    let order = [
        "/* == reset == */",
        "/* == tokens == */",
        "/* == accents == */",
        "/* == materials == */",
        "/* == motion == */",
        "/* == utilities == */",
        "/* == components == */",
    ];
    let positions: Vec<usize> = order
        .iter()
        .map(|marker| {
            css.find(marker)
                .unwrap_or_else(|| panic!("{marker} missing"))
        })
        .collect();
    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "{positions:?}"
    );
    // Inside the tokens section: the light block, the dark block, then the levels.
    let tokens = [
        ".ds{--paper:",
        ".ds[*|data-theme=dark]{--paper:",
        ".ds[*|data-motion=calm]{",
        ".ds[*|data-motion=extra]{",
        ".ds[*|data-motion=reduced]{",
    ];
    let at: Vec<usize> = tokens
        .iter()
        .map(|needle| {
            css.find(needle)
                .unwrap_or_else(|| panic!("{needle} missing"))
        })
        .collect();
    assert!(at.windows(2).all(|pair| pair[0] < pair[1]), "{at:?}");
    // The components, in the fixed list's order, all of them.
    let components: Vec<usize> = ds::components::CSS
        .iter()
        .map(|(name, _)| {
            let marker = format!("/* -- {name} -- */");
            css.find(&marker)
                .unwrap_or_else(|| panic!("{marker} missing"))
        })
        .collect();
    assert!(components.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(
        components
            .first()
            .is_some_and(|first| *first > positions[6])
    );
}

/// Standard base64 (RFC 4648 section 4) back to bytes.
fn unbase64(text: &str) -> Vec<u8> {
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

#[test]
fn the_grain_is_the_prototypes_tile_as_alpha_noise() {
    let data = GRAIN_PNG
        .strip_prefix("data:image/png;base64,")
        .unwrap_or_else(|| panic!("not a PNG data URI: {:.40}", GRAIN_PNG));
    let png = unbase64(data);
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(&png[12..16], b"IHDR");
    assert_eq!(&png[16..20], 128u32.to_be_bytes());
    assert_eq!(&png[20..24], 128u32.to_be_bytes());
    assert_eq!(&png[24..26], [8, 4], "8-bit grey with alpha");
    // The first scanline, from the stored deflate block after the zlib header: filter 0, then
    // the prototype's first draws. Seed 7: 7 x 16807 = 117649, floor(117649 / (2^31 - 1) x 255)
    // = 0, a full black; the second draw, 117649 x 16807 mod (2^31 - 1) = 1977326743, is
    // grey 234, white at alpha 2 x 234 - 255 = 213.
    let idat = png
        .windows(4)
        .position(|w| w == b"IDAT")
        .unwrap_or_else(|| panic!("no IDAT"));
    let raw = &png[idat + 4 + 2 + 5..];
    assert_eq!(&raw[..5], [0, 0, 255, 255, 213]);
    assert!(
        stylesheet().contains(GRAIN_PNG),
        "the stylesheet does not paint the grain"
    );
}

/// The declarations of the first rule whose selector is exactly `selector`.
fn rule_body(css: &str, selector: &str) -> Option<String> {
    css.split('}')
        .filter_map(|rule| rule.split_once('{'))
        .find(|(head, _)| head.trim() == selector)
        .map(|(_, body)| body.to_owned())
}

/// `banner-in` and `banner-out` read `--banner-dx`/`--banner-dy`, so every entry edge (and the
/// swiped row, which leaves along the swipe) must set both, or a banner would inherit the other
/// axis from an enclosing stack or fall back to the right edge (design/05 section 12 item 7).
#[test]
fn every_banner_entry_edge_sets_both_axes_of_its_vector() {
    let css = strip_comments(stylesheet());
    for selector in [
        ".ds-banner-stack[*|data-entry=right]",
        ".ds-banner-stack[*|data-entry=below]",
        ".ds-banner[*|data-flight=swipe]",
    ] {
        let body = rule_body(&css, selector).unwrap_or_else(|| panic!("no rule for {selector}"));
        let declared = properties(&body);
        for axis in ["--banner-dx", "--banner-dy"] {
            assert!(
                declared.iter().any(|name| name == axis),
                "{selector} does not set {axis}: {body}"
            );
        }
    }
    for keyframes in ["@keyframes banner-in{", "@keyframes banner-out{"] {
        let at = css
            .find(keyframes)
            .unwrap_or_else(|| panic!("no {keyframes}"));
        let block = css[at..].split("} }").next().unwrap_or_default();
        for axis in ["var(--banner-dx", "var(--banner-dy"] {
            assert!(block.contains(axis), "{keyframes} does not read {axis}");
        }
    }
}
