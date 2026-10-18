//! A FieldRow's help behind a small ⓘ button (design/04-COMPONENTS.md, "FieldRow help"): where the
//! help sits, the id the row's `aria-describedby` points at, and the button with its popover.
//!
//! The button is a borderless `Bezel::Toolbar` icon button named "About <label>" with the tip
//! "About". Pressing it opens a [`Popover`] anchored to the button: the label as a bold title,
//! then the help. The popover is `Dismiss::Transient`, so the layer stack closes it on Escape, on
//! an outside press (which includes a second press on the button, and a press on another row's
//! button: the first press closes the open one) and its own fade; nothing here times anything.

use crate::components::content::icon_source::IconSource;
use crate::components::content::text_runs::{TextLine, text};
use crate::components::controls::button::Button;
use crate::components::controls::button_model::{Bezel, ImagePosition};
use crate::components::overlays::popover::{Arrow, Popover};
use crate::host::measure::{Anchor, MountedRef};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::placement::{Align, Placement, Side};
use ds_core::geometry::units::Px;
use ds_core::press::Press;
use ds_core::vocab::Shown;
use ds_core::word::Word;
use ds_style::icon::Icon;
use ds_style::tokens::control_size::ControlSize;

/// Where a row's help text is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum HelpShown {
    /// Under the label, always (`span.ds-field-row-help`).
    #[default]
    Inline,
    /// Behind a small ⓘ button after the label; the text stays in the accessibility tree.
    OnRequest,
}

/// The id of the row's help element: the row's own `id` and `-help`, else a slug of its label.
pub(crate) fn help_id(row_id: Option<&str>, label: &str) -> String {
    if let Some(id) = row_id {
        format!("{id}-help")
    } else {
        let slug: String = label
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect();
        format!("ds-field-help-{}", slug.trim_matches('-'))
    }
}

/// The ⓘ button of a row labelled `label`, and the popover of `help` it opens.
#[component]
pub(crate) fn HelpButton(label: TextLine, help: TextLine) -> Element {
    let mut shown = use_signal(|| Shown::Hidden);
    let mut element = use_signal(|| None::<MountedRef>);
    let name = format!("About {}", label.plain_text());
    rsx! {
        span {
            class: "ds-field-row-about",
            onkeydown: move |event| {
                if event.key() == Key::Escape && shown() == Shown::Visible {
                    event.stop_propagation();
                    shown.set(Shown::Hidden);
                }
            },
            Button {
                label: name,
                bezel: Bezel::Toolbar,
                image: ImagePosition::Only,
                icon: Some(IconSource::Glyph(Icon::Info)),
                size: ControlSize::Small,
                title: Some("About".to_owned()),
                shown: Some(shown()),
                onclick: move |_: Press| shown.set(shown().flipped()),
                common: Common {
                    mounted: Some(EventHandler::new(move |event: MountedEvent| {
                        element.set(Some(MountedRef(event.data())));
                    })),
                    ..Common::default()
                },
            }
        }
        if shown() == Shown::Visible && let Some(anchor) = element() {
            Popover {
                anchor: Anchor::Mounted(anchor),
                placement: Placement::new(Side::Bottom, Align::Center),
                gap: Px(4.0),
                arrow: Arrow::Arrow,
                onclose: move |()| shown.set(Shown::Hidden),
                div { class: "ds-field-help-card",
                    span { class: "ds-field-help-title", {text(&label)} }
                    span { class: "ds-field-help-text", {text(&help)} }
                }
            }
        }
    }
}
