//! One `.ds[data-accent=…]` block per accent and scheme, and one more of each under
//! `data-contrast=high` (the desktop asks for increased contrast), which sets the same seven
//! properties from the increased-contrast shade of the hue.
//!
//! The dark blocks carry `data-theme=dark` as well, one attribute more specific than both the
//! light accent blocks and the dark token block, so a dark root takes its accent's dark quad
//! whatever the source order. Each block sets exactly the seven accent properties (design/03-COLOR.md
//! section 20). A Space never overrides them: the card wears the desktop's accent.
//!
//! The `--swatch-<accent>` properties on `.ds` are every accent's own `--accent`, for a picker
//! that has to paint all six at once: a swatch written `var(--accent)` would show the selected
//! hue on every button.

use crate::appearance::{accent::Accent, system::Contrast, theme::Scheme};
use crate::emit::{attr_selector, declaration, rule};
use crate::tokens::{
    accent_band::roles::AccentRoles,
    accent_table::{accent_in, accent_of},
    colour::ColourToken,
};
use ds_core::word::Word;

/// The accent quads, light and dark.
pub fn accents_css() -> String {
    let mut css = String::new();
    for scheme in Scheme::ALL.iter().copied() {
        css.push_str(&rule(&swatch_selector(scheme), &swatches(scheme)));
    }
    for scheme in Scheme::ALL.iter().copied() {
        for accent in Accent::ALL.iter().copied() {
            css.push_str(&rule(
                &selector(accent, scheme, Contrast::Normal),
                &seven(accent_of(accent, scheme)),
            ));
        }
    }
    for scheme in Scheme::ALL.iter().copied() {
        for accent in Accent::ALL.iter().copied() {
            css.push_str(&rule(
                &selector(accent, scheme, Contrast::High),
                &seven(accent_in(accent, scheme, Contrast::High)),
            ));
        }
    }
    css
}

fn swatch_selector(scheme: Scheme) -> String {
    match scheme {
        Scheme::Light => ".ds".to_owned(),
        Scheme::Dark => format!(".ds{}", attr_selector("data-theme", scheme.slug())),
    }
}

fn swatches(scheme: Scheme) -> Vec<String> {
    Accent::ALL
        .iter()
        .copied()
        .map(|accent| {
            format!(
                "{}:{};",
                swatch_var(accent),
                accent_of(accent, scheme).fill.css()
            )
        })
        .collect()
}

/// `--swatch-<accent>`: that accent's own `--accent`, whatever the root selects.
pub fn swatch_var(accent: Accent) -> String {
    format!("--swatch-{}", accent.slug())
}

fn selector(accent: Accent, scheme: Scheme, contrast: Contrast) -> String {
    let theme = match scheme {
        Scheme::Light => String::new(),
        Scheme::Dark => attr_selector("data-theme", scheme.slug()),
    };
    let contrast = match contrast {
        Contrast::Normal => String::new(),
        Contrast::High => attr_selector("data-contrast", "high"),
    };
    format!(
        ".ds{theme}{contrast}{}",
        attr_selector("data-accent", accent.slug())
    )
}

fn seven(roles: AccentRoles) -> Vec<String> {
    vec![
        declaration(ColourToken::Accent.var(), &roles.fill.css()),
        declaration(ColourToken::AccentInk.var(), &roles.ink.css()),
        declaration(ColourToken::AccentSoft.var(), &roles.wash_colour().css()),
        declaration(ColourToken::AccentText.var(), &roles.text.css()),
        declaration(
            ColourToken::AccentTextMaterial.var(),
            &roles.text_material.css(),
        ),
        declaration(ColourToken::AccentRing.var(), &roles.ring_colour().css()),
        declaration(ColourToken::Seal.var(), &roles.fill.css()),
    ]
}
