//! How the stylesheet is laid out as text: a header, each section under `/* == name == */`, each
//! component sheet under `/* -- name -- */`. The assembly decides which sections and sheets go
//! in, and in what order.

use crate::style::css::UTILITIES;
use crate::style::css::grain::grain_uri;
use crate::style::emit::{property, rule};

/// The whole stylesheet: the header, then each `(section, body)` in the order given.
pub(crate) fn document(sections: &[(&str, String)]) -> String {
    let mut css =
        String::from("/* ds::stylesheet(), generated from the token table. Do not edit. */\n");
    for (name, body) in sections {
        css.push_str(&format!("/* == {name} == */\n{}\n", body.trim_end()));
    }
    css
}

/// Component sheets, each under its marker, in the order given.
pub fn sheets(sheets: &[(&str, &str)]) -> String {
    sheets
        .iter()
        .map(|(name, css)| format!("/* -- {name} -- */\n{}\n", css.trim_end()))
        .collect()
}

/// `.ds-truncate` and the other utilities, then the grain tile on `.ds-grain`.
pub fn utilities_css() -> String {
    let grain = rule(
        ".ds-grain",
        &[property(
            "background-image",
            &format!("url(\"{}\")", grain_uri()),
        )],
    );
    format!("{}\n{grain}", UTILITIES.trim_end())
}
