//! The whole stylesheet, generated once.

use super::{
    GRAIN_PNG, RESET, UTILITIES, accents_css::accents_css, emit::property, emit::rule,
    ground_css::ground_css, materials_css::materials_css, motion_css::motion_css,
    tokens_css::tokens_css,
};
use crate::components::CSS;
use std::sync::LazyLock;

/// Every rule the design system draws with, in cascade order. Built once, on first use.
pub fn stylesheet() -> &'static str {
    static SHEET: LazyLock<String> = LazyLock::new(build);
    SHEET.as_str()
}

/// Reset, tokens (with the level blocks), accents, materials, the frame ground, keyframes with their aliases and
/// pulse classes, utilities with the grain tile, then every component in its fixed order.
fn build() -> String {
    let grain = rule(
        ".ds-grain",
        &[property(
            "background-image",
            &format!("url(\"{GRAIN_PNG}\")"),
        )],
    );
    let sections: [(&str, String); 8] = [
        ("reset", RESET.trim_end().to_owned()),
        ("tokens", tokens_css()),
        ("accents", accents_css()),
        ("materials", materials_css()),
        ("ground", ground_css()),
        ("motion", motion_css()),
        ("utilities", format!("{}\n{grain}", UTILITIES.trim_end())),
        ("components", components()),
    ];
    let mut css =
        String::from("/* ds::stylesheet(), generated from the token table. Do not edit. */\n");
    for (name, body) in sections {
        css.push_str(&format!("/* == {name} == */\n{}\n", body.trim_end()));
    }
    css
}

fn components() -> String {
    CSS.iter()
        .map(|(name, css)| format!("/* -- {name} -- */\n{}\n", css.trim_end()))
        .collect()
}
