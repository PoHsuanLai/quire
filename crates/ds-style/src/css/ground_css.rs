//! The frame ground (`data-ground="frame"`, `Ground::Frame`): under it the paper inks and
//! fills are the frame's, so every component (Button, Chip, Badge, a menu's
//! trigger, plain text) draws in the `--f-*` inks without a variant of its own
//! (design/03-COLOR.md section 4, design/04-COMPONENTS.md's sidebar item).
//!
//! Overlays opened from the frame (`.ds-overlay`, rendered at the end of the root) are paper
//! cards, so they take the paper values back, per scheme, from the token table.

use crate::appearance::theme::Scheme;
use crate::emit::{attr_selector, declaration, rule};
use crate::look::Look;
use crate::tokens::{colour::ColourToken, name::VarName};
use ds_core::word::Word;

/// Each paper token a frame ground redirects, and the frame variable it reads instead: the text
/// inks to the frame inks, the hover fill (`--surface`, Tool's hover) to `--f-pill-hover`, the
/// pressed and raised fills (`--surface-2`, `--raise`) to `--f-pill`, the hairlines to `--f-line`.
pub(crate) const REMAP: [(ColourToken, VarName); 8] = [
    (ColourToken::Ink, VarName("--f-ink")),
    (ColourToken::InkSoft, VarName("--f-ink-soft")),
    (ColourToken::InkFaint, VarName("--f-ink-faint")),
    (ColourToken::Surface, VarName("--f-pill-hover")),
    (ColourToken::Surface2, VarName("--f-pill")),
    (ColourToken::Raise, VarName("--f-pill")),
    (ColourToken::Line, VarName("--f-line")),
    (ColourToken::LineSoft, VarName("--f-line")),
];

/// The frame ground's redirect, then the paper values given back to overlays in each scheme.
pub fn ground_css() -> String {
    let ground = format!(".ds{}", attr_selector("data-ground", "frame"));
    let mut css = rule(
        &ground,
        &REMAP
            .iter()
            .map(|(token, frame)| declaration(token.var(), &frame.reference()))
            .collect::<Vec<_>>(),
    );
    for scheme in Scheme::ALL.iter().copied() {
        let theme = match scheme {
            Scheme::Light => String::new(),
            Scheme::Dark => attr_selector("data-theme", scheme.slug()),
        };
        let paper = REMAP
            .iter()
            .map(|(token, _)| declaration(token.var(), &token.value(Look::default(), scheme).css()))
            .collect::<Vec<_>>();
        css.push_str(&rule(
            &format!(
                ".ds{theme}{} .ds-overlay",
                attr_selector("data-ground", "frame")
            ),
            &paper,
        ));
    }
    css
}

#[cfg(test)]
mod tests {
    use super::ground_css;

    #[test]
    fn the_frame_redirects_the_paper_inks_and_overlays_take_them_back() {
        let css = ground_css();
        const WANT: &[&str] = &[
            ".ds[*|data-ground=frame]{--ink:var(--f-ink);--ink-soft:var(--f-ink-soft);--ink-faint:var(--f-ink-faint);--surface:var(--f-pill-hover);--surface-2:var(--f-pill);",
            ".ds[*|data-ground=frame] .ds-overlay{--ink:#",
            ".ds[*|data-theme=dark][*|data-ground=frame] .ds-overlay{--ink:#",
        ];
        for want in WANT {
            assert!(css.contains(want), "missing {want}\n{css}");
        }
    }
}
