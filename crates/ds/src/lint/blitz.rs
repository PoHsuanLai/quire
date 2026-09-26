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
        "scroll-behavior" if value == "smooth" => Some(
            "scroll-behavior: smooth is banned; Blitz's 300 ms ScrollTo fights the host's scroll \
             engine (design/11-BEHAVIOUR-scroll.md#11-3-1-ownership); drive a programmatic scroll \
             through the host's ScrollCmd instead",
        ),
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

#[cfg(test)]
mod tests {
    use super::unsupported;

    struct Case {
        name: &'static str,
        property: &'static str,
        value: &'static str,
        expect: bool,
    }

    /// One row per entry in [`unsupported`]'s table, plus the values that must fall through.
    const CASES: &[Case] = &[
        Case {
            name: "backdrop-filter fires",
            property: "backdrop-filter",
            value: "blur(10px)",
            expect: true,
        },
        Case {
            name: "filter fires",
            property: "filter",
            value: "saturate(2)",
            expect: true,
        },
        Case {
            name: "mix-blend-mode fires",
            property: "mix-blend-mode",
            value: "multiply",
            expect: true,
        },
        Case {
            name: "position: sticky fires",
            property: "position",
            value: "sticky",
            expect: true,
        },
        Case {
            name: "position: sticky, one of several comma values, fires",
            property: "position",
            value: "relative, sticky",
            expect: true,
        },
        Case {
            name: "position: relative passes",
            property: "position",
            value: "relative",
            expect: false,
        },
        Case {
            name: "scroll-behavior: smooth fires",
            property: "scroll-behavior",
            value: "smooth",
            expect: true,
        },
        Case {
            name: "scroll-behavior: smooth, other case, fires",
            property: "Scroll-Behavior",
            value: "Smooth",
            expect: true,
        },
        Case {
            name: "scroll-behavior: auto passes",
            property: "scroll-behavior",
            value: "auto",
            expect: false,
        },
        Case {
            name: "text-overflow fires",
            property: "text-overflow",
            value: "ellipsis",
            expect: true,
        },
        Case {
            name: "line-clamp fires",
            property: "line-clamp",
            value: "2",
            expect: true,
        },
        Case {
            name: "-webkit-line-clamp fires",
            property: "-webkit-line-clamp",
            value: "2",
            expect: true,
        },
        Case {
            name: "text-shadow fires",
            property: "text-shadow",
            value: "0 0 1px #000",
            expect: true,
        },
        Case {
            name: "an ordinary property passes",
            property: "color",
            value: "var(--ink)",
            expect: false,
        },
    ];

    #[test]
    fn the_table_matches_every_case() {
        for case in CASES {
            let got = unsupported(case.property, case.value).is_some();
            assert_eq!(got, case.expect, "{}", case.name);
        }
    }
}
