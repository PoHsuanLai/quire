//! Scans of the control stylesheets: which classes they style, and that they use tokens only.

/// The CSS files that style a component's markup: its own, then the components it renders.
pub const STYLES: &[(&str, &[&str])] = &[
    (
        "button",
        &[
            include_str!("../../src/components/button.css"),
            include_str!("../../src/components/icon_view.css"),
        ],
    ),
    (
        "icon_button",
        &[
            include_str!("../../src/components/icon_button.css"),
            include_str!("../../src/components/icon_view.css"),
        ],
    ),
    (
        "icon_view",
        &[include_str!("../../src/components/icon_view.css")],
    ),
    (
        "segmented",
        &[include_str!("../../src/components/segmented.css")],
    ),
    ("toggle", &[include_str!("../../src/components/toggle.css")]),
    ("slider", &[include_str!("../../src/components/slider.css")]),
    (
        "text_input",
        &[
            include_str!("../../src/components/text_input.css"),
            include_str!("../../src/components/icon_button.css"),
            include_str!("../../src/components/icon_view.css"),
        ],
    ),
    (
        "search_field",
        &[
            include_str!("../../src/components/search_field.css"),
            include_str!("../../src/components/text_input.css"),
            include_str!("../../src/components/chip.css"),
        ],
    ),
    ("kbd", &[include_str!("../../src/components/kbd.css")]),
    (
        "chip",
        &[
            include_str!("../../src/components/chip.css"),
            include_str!("../../src/components/avatar.css"),
        ],
    ),
    ("avatar", &[include_str!("../../src/components/avatar.css")]),
    ("tabs", &[include_str!("../../src/components/tabs.css")]),
    (
        "section_header",
        &[include_str!("../../src/components/section_header.css")],
    ),
    ("count", &[include_str!("../../src/components/count.css")]),
    (
        "spinner",
        &[include_str!("../../src/components/spinner.css")],
    ),
];

/// Every value of every `class="..."` attribute in `html`.
pub fn classes(html: &str) -> Vec<&str> {
    html.split("class=\"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .flat_map(str::split_whitespace)
        .collect()
}

/// Whether `css` has a `.class` selector for exactly this class, not one it prefixes.
pub fn styles_class(css: &str, class: &str) -> bool {
    let needle = format!(".{class}");
    css.match_indices(&needle).any(|(at, _)| {
        !css[at + needle.len()..]
            .starts_with(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    })
}

/// `css` with its comments removed.
fn without_comments(css: &str) -> String {
    let mut out = String::new();
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        rest = rest[start..]
            .find("*/")
            .map_or("", |end| &rest[start + end + 2..]);
    }
    out.push_str(rest);
    out
}

/// What a control stylesheet may not contain (ORCHESTRATION coherence rule 1; spike S2, S12).
pub fn token_violations(css: &str) -> Vec<String> {
    let css = without_comments(css);
    let mut found = Vec::new();
    for banned in [
        "rgb(",
        "rgba(",
        "hsl(",
        "cubic-bezier",
        ":focus-visible",
        ":focus-within",
        "!important",
        "@keyframes",
        ":root",
        "text-overflow",
        "filter",
        "color-mix",
    ] {
        if css.contains(banned) {
            found.push(format!("contains {banned}"));
        }
    }
    for (at, _) in css.match_indices('#') {
        found.push(format!(
            "a # at {at}: {}",
            &css[at..(at + 8).min(css.len())]
        ));
    }
    for (at, _) in css.match_indices('[') {
        if !css[at + 1..].starts_with("*|") {
            found.push(format!(
                "attribute selector without *|: {}",
                &css[at..(at + 24).min(css.len())]
            ));
        }
    }
    for (at, _) in css.match_indices("font-family:") {
        let value = css[at + "font-family:".len()..].trim_start();
        if !(value.starts_with("var(--") || value.starts_with("inherit")) {
            found.push(format!(
                "literal font-family: {}",
                &value[..value.len().min(24)]
            ));
        }
    }
    for word in css.split(|c: char| !(c.is_ascii_alphanumeric() || c == '.')) {
        let number = word.strip_suffix("ms").or_else(|| word.strip_suffix('s'));
        if number
            .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit() || c == '.'))
        {
            found.push(format!("literal duration {word}"));
        }
    }
    found
}
