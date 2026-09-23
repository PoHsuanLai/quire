//! One `.ds[data-accent=…]` block per accent and scheme.
//!
//! The dark blocks carry `data-theme=dark` as well, one attribute more specific than both the
//! light accent blocks and the dark token block, so a dark root takes its accent's dark quad
//! whatever the source order. Each block sets exactly the four accent properties. A Space that
//! lends the card its hue writes `--accent` and its pair inline on the root, which beats all of
//! these.
//!
//! The `--swatch-<accent>` properties on `.ds` are every accent's own `--accent`, for a picker
//! that has to paint all six at once: a swatch written `var(--accent)` would show the selected
//! hue on every button.

use super::emit::{attr_selector, declaration, rule};
use crate::appearance::{Accent, Scheme};
use crate::tokens::{AccentQuad, ColourToken, quad};

/// The accent quads, light and dark.
pub fn accents_css() -> String {
    let mut css = String::new();
    for scheme in Scheme::ALL {
        css.push_str(&rule(&swatch_selector(scheme), &swatches(scheme)));
    }
    for scheme in Scheme::ALL {
        for accent in Accent::ALL {
            css.push_str(&rule(
                &selector(accent, scheme),
                &four(quad(accent, scheme)),
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
        .into_iter()
        .map(|accent| {
            format!(
                "--swatch-{}:{};",
                accent.slug(),
                quad(accent, scheme).accent.css()
            )
        })
        .collect()
}

fn selector(accent: Accent, scheme: Scheme) -> String {
    let theme = match scheme {
        Scheme::Light => String::new(),
        Scheme::Dark => attr_selector("data-theme", scheme.slug()),
    };
    format!(".ds{theme}{}", attr_selector("data-accent", accent.slug()))
}

fn four(quad: AccentQuad) -> Vec<String> {
    vec![
        declaration(ColourToken::Accent.var(), &quad.accent.css()),
        declaration(ColourToken::AccentInk.var(), &quad.ink.css()),
        declaration(ColourToken::AccentSoft.var(), &quad.soft.css()),
        declaration(ColourToken::Seal.var(), &quad.seal.css()),
    ]
}
