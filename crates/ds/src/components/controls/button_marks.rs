//! What a `Button` shows besides its icon and label: a mark before the label and a trailing
//! glyph after it.

use crate::components::content::text_runs::TextLine;
use dioxus::prelude::*;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};

/// What names a button to assistive technology when the caller gave no `aria_label`: the
/// label's characters when runs split them into pieces (so the name is one string, whatever
/// spans the tones need); nothing for a plain label, which names the button by its own text.
pub(crate) fn spoken_label(label: &TextLine) -> Option<String> {
    match label {
        TextLine::Runs(_) => Some(label.plain_text()),
        TextLine::Plain(_) => None,
    }
}

/// A mark after the label: a glyph at the button's icon size (`Icon::ChevronDown` for a button
/// that opens a menu).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trailing {
    /// A glyph, at the button's icon size.
    Glyph(Icon),
}

impl Trailing {
    /// The glyph and its size, given the button's own icon size.
    fn glyph(self, icon_size: IconSize) -> (Icon, IconSize) {
        match self {
            Trailing::Glyph(icon) => (icon, icon_size),
        }
    }
}

/// A mark before the label, the counterpart of [`Trailing`]: a glyph, or an
/// element the caller draws with quire's own components. The From dropdown shows the chosen
/// account's provider as `Leading::Mark(rsx! { ProviderMark { size: ControlSize::Small, .. } })`
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
    use super::spoken_label;
    use crate::components::content::text_runs::{RunTone, TextLine, TextRun};

    #[test]
    fn a_split_label_is_spoken_whole() {
        let runs = TextLine::Runs(vec![
            TextRun::new("Dana Okafor", RunTone::Strong),
            TextRun::new(" wrote on Tue 22 Sep", RunTone::Faint),
        ]);
        let cases = [
            (TextLine::from("Send"), None),
            (runs, Some("Dana Okafor wrote on Tue 22 Sep")),
        ];
        for (label, want) in cases {
            assert_eq!(spoken_label(&label).as_deref(), want, "{label:?}");
        }
    }
}
