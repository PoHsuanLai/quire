//! What a `Button` shows besides its icon (mailo gaps 4): its face, a word or a mark drawn in
//! its own style, and a trailing glyph after it.

use crate::components::text_runs::{Text, text};
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use dioxus::prelude::*;

/// How a button's label is drawn. Every face but `Label` draws a one-letter mark in the style
/// it names (the selection bubble's B, I, U and S, design/04 section 30) and names the button
/// to assistive technology by its `label` ("Bold") instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ButtonFace {
    /// The label itself, as words.
    #[default]
    Label,
    /// `B` in bold.
    Bold,
    /// `i` in the serif italic (S's Georgia is `--font-serif`, design/02).
    Italic,
    /// `U` underlined.
    Underline,
    /// `S` struck through.
    Strike,
}

/// What names a button to assistive technology when the caller gave no `aria_label`: the
/// label's characters when a mark face hides them, or when runs split them into pieces (so the
/// name is one string, whatever spans the tones need); nothing for a plain label, which names
/// the button by its own text.
pub(crate) fn spoken_label(face: ButtonFace, label: &Text) -> Option<String> {
    match (face.is_mark(), label) {
        (true, _) | (false, Text::Runs(_)) => Some(label.plain_text()),
        (false, Text::Plain(_)) => None,
    }
}

impl ButtonFace {
    /// The letter a mark face shows and its `data-face` word; `None` for `Label`.
    fn mark(self) -> Option<(&'static str, &'static str)> {
        match self {
            ButtonFace::Label => None,
            ButtonFace::Bold => Some(("B", "bold")),
            ButtonFace::Italic => Some(("i", "italic")),
            ButtonFace::Underline => Some(("U", "underline")),
            ButtonFace::Strike => Some(("S", "strike")),
        }
    }

    /// Whether the face shows a mark rather than the label's words: then the label becomes the
    /// button's `aria-label`.
    pub(crate) fn is_mark(self) -> bool {
        self.mark().is_some()
    }
}

/// A mark after the label: a dropdown's caret, or any glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trailing {
    /// The dropdown caret: `chevron-down` at 12, in the label's colour.
    Caret,
    /// Another glyph, at the button's icon size.
    Glyph(Icon),
}

impl Trailing {
    /// The glyph and its size, given the button's own icon size.
    fn glyph(self, icon_size: IconSize) -> (Icon, IconSize) {
        match self {
            Trailing::Caret => (Icon::ChevronDown, IconSize::Tiny),
            Trailing::Glyph(icon) => (icon, icon_size),
        }
    }
}

/// A mark before the label (mailo gaps 5), the counterpart of [`Trailing`]: a glyph, or an
/// element the caller draws with quire's own components. The From dropdown shows the chosen
/// account's provider as `Leading::Mark(rsx! { ProviderMark { size: MarkSize::Inline, .. } })`
/// inside its value. The slot takes an `Element` rather than a `Provider` so any quire mark (an
/// avatar, a person colour's dot) fits without a variant per kind; what goes in it is the
/// caller's to keep to quire components, as for any children.
#[derive(Debug, Clone, PartialEq)]
pub enum Leading {
    /// A glyph at the button's icon size, in the label's colour.
    Glyph(Icon),
    /// A mark the caller built: a `ProviderMark`, an `Avatar`.
    Mark(Element),
}

/// The leading mark, in its own span so the sheet can size and space it.
pub(crate) fn leading(mark: Leading, icon_size: IconSize) -> Element {
    match mark {
        Leading::Glyph(icon) => rsx! {
            span { class: "ds-button-lead",
                Glyph { icon, size: icon_size }
            }
        },
        Leading::Mark(element) => rsx! {
            span { class: "ds-button-lead", {element} }
        },
    }
}

/// The label drawn in `face`: a span of words (runs in their tones, mailo gaps 5), or the
/// face's letter in its style. The mark is `aria-hidden`, since the button is named by its
/// label. Usable on its own as a `BubbleButton`'s `label`, so the bubble's marks need no raw
/// `b`, `i`, `u` or `s`.
#[component]
pub fn FaceMark(face: ButtonFace, #[props(into)] label: Text) -> Element {
    match face.mark() {
        None => rsx! {
            span { {text(&label)} }
        },
        Some((letter, slug)) => rsx! {
            span { class: "ds-button-face", "data-face": slug, "aria-hidden": "true", "{letter}" }
        },
    }
}

/// The trailing glyph, in its own span so the sheet can set it apart from the label.
pub(crate) fn trailing(mark: Trailing, icon_size: IconSize) -> Element {
    let (icon, size) = mark.glyph(icon_size);
    rsx! {
        span { class: "ds-button-trail",
            Glyph { icon, size }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ButtonFace, spoken_label};
    use crate::components::text_runs::{Run, RunTone, Text};

    #[test]
    fn a_hidden_or_split_label_is_spoken_whole() {
        let runs = Text::Runs(vec![
            Run::new("Dana Okafor", RunTone::Strong),
            Run::new(" wrote on Tue 22 Sep", RunTone::Faint),
        ]);
        let cases = [
            (ButtonFace::Label, Text::from("Send"), None),
            (ButtonFace::Bold, Text::from("Bold"), Some("Bold")),
            (
                ButtonFace::Label,
                runs,
                Some("Dana Okafor wrote on Tue 22 Sep"),
            ),
        ];
        for (face, label, want) in cases {
            assert_eq!(spoken_label(face, &label).as_deref(), want, "{label:?}");
        }
    }

    #[test]
    fn only_the_label_face_draws_words() {
        const CASES: &[(ButtonFace, bool)] = &[
            (ButtonFace::Label, false),
            (ButtonFace::Bold, true),
            (ButtonFace::Italic, true),
            (ButtonFace::Underline, true),
            (ButtonFace::Strike, true),
        ];
        for &(face, mark) in CASES {
            assert_eq!(face.is_mark(), mark, "{face:?}");
        }
    }
}
