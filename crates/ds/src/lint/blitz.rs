//! What Blitz does not paint, for [`super::Rule::BlitzUnsupported`] (the plan's Blitz risk
//! table, FINDINGS.md spike S12-S16).

/// Why `property: value` will not paint on Blitz, or `None` when it will.
pub fn unsupported(property: &str, value: &str) -> Option<&'static str> {
    let property = property.trim().to_ascii_lowercase();
    let value = value.trim().to_ascii_lowercase();
    match property.as_str() {
        "backdrop-filter" => Some(
            "backdrop-filter is not painted on anyrender_vello_cpu or _hybrid (FINDINGS S15); \
             ask the compositor to blur behind the surface through a Material instead",
        ),
        "filter" => Some(
            "filter (saturate/blur/...) is dropped by the pinned anyrender backends when \
             multithreading is on, and saturate() is unimplemented even on hybrid (FINDINGS \
             S16); precompute the effect into a colour token instead",
        ),
        "mix-blend-mode" => Some(
            "mix-blend-mode is not exercised by the Blitz spike; use a PNG grain layer or a precomputed blend instead",
        ),
        "position" if value.split(',').any(|part| part.trim() == "sticky") => {
            Some("position: sticky is banned outright; keep headers outside the scroller instead")
        }
        "text-overflow" => Some(
            "text-overflow: ellipsis does not truncate on Blitz (FINDINGS S13); use \
             `.ds-truncate`'s mask-image fade, or `text::clip_chars` for a real ellipsis",
        ),
        "-webkit-line-clamp" | "line-clamp" => {
            Some("line-clamp has no Blitz spike coverage; use `text::clip_chars` instead")
        }
        "text-shadow" => Some("text-shadow has no Blitz spike coverage and is assumed unsupported"),
        _ => None,
    }
}
