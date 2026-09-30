//! Label: `NSTextField` as a label, the one way to draw plain text (design/30 section 2.8).
//! Markup: `span.ds-label[data-role][data-style]`, with `data-availability` and `aria-disabled`
//! when it is not enabled; the text is a [`TextLine`], whole or as runs in their tones.

use crate::components::content::text_runs::{TextLine, text};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::Availability;
use ds_core::word::Word;

/// How much a label matters (`NSColor.labelColor` and its three secondary levels):
/// `data-role`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum LabelRole {
    /// The text itself: full ink.
    #[default]
    Primary,
    /// Beside or under the primary text: a step quieter.
    Secondary,
    /// Hints and captions: quieter again.
    Tertiary,
    /// Text that only orients: the faintest that still reads.
    Quaternary,
}

/// The step of the type scale the label is set in (design/02-TYPE.md), `data-style`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum LabelStyle {
    /// `--fs-caption`: a row's time, a shortcut.
    Caption,
    /// `--fs-help`: the help text under a field.
    Footnote,
    /// `--fs-control`: the text of a control and a row.
    #[default]
    Body,
    /// `--fs-control` at 700: a row's title, a group's name.
    Headline,
    /// `--fs-title`: a pane's or a card's title.
    Title,
    /// `--fs-display`: a figure or a page's name.
    Display,
}

/// A label: `text` in `role` at `style`. `Disabled` draws it at the disabled .35 and writes
/// `aria-disabled`; a label is never busy, so `Busy` reads as `Enabled`.
#[component]
pub fn Label(
    #[props(into)] text: TextLine,
    #[props(default)] role: LabelRole,
    #[props(default)] style: LabelStyle,
    #[props(default)] availability: Availability,
    #[props(default)] common: Common,
) -> Element {
    let class = common.class("ds-label");
    let data = common.data_attributes();
    let disabled = availability == Availability::Disabled;
    rsx! {
        span {
            id: common.id.clone(),
            class,
            "data-role": role.slug(),
            "data-style": style.slug(),
            "data-availability": if disabled { Some(availability.slug()) } else { None },
            "aria-label": common.aria_label.clone(),
            "aria-disabled": availability.aria_disabled(),
            onmounted: move |event| common.mounted(event),
            ..data,
            {self::text(&text)}
        }
    }
}
