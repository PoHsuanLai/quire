//! InlineBanner: a message in a pane's own flow, above the content it is about (design/30 section
//! 2.9): "remote images are blocked" over a message, "a read receipt was asked for", "you
//! accepted this invitation". It is not a notification (that is [`Toast`](super::toast) and the
//! shell's banners): it takes a place in the layout, stays until what it says stops being true or
//! the person closes it, and never floats.
//!
//! It is static, as Mail's remote-content bar is: it appears and disappears in place with no
//! motion (design/30 section 2.9, design/00 rule 10: a concession is dull). `shown` is only a
//! render switch: `Hidden` draws nothing, and what follows takes its place at once. It needs no
//! `Ds` root of its own.
//!
//! Markup: `div.ds-inline-banner[data-severity]` of `span.ds-inline-banner-icon`,
//! `div.ds-inline-banner-body` (the `text`, the `detail` under it), `div.ds-inline-banner-actions`
//! (the caller's buttons) and, when it can be closed, a close button. A `Danger` banner is
//! `role="alert"`, the others `role="status"`.

use crate::components::content::icon_view::IconView;
use crate::components::content::label::{Label, LabelRole, LabelStyle};
use crate::components::content::text_runs::TextLine;
use crate::components::controls::button::Button;
use crate::components::controls::button_model::{Bezel, ImagePosition};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::{Severity, Shown};
use ds_core::word::Word;
use ds_style::icon::Icon;
use ds_style::tokens::control_size::ControlSize;

/// The glyph a severity draws when the caller brings none.
fn glyph_of(severity: Severity) -> Icon {
    match severity {
        Severity::Info => Icon::Info,
        Severity::Ok => Icon::CircleCheck,
        Severity::Warn => Icon::TriangleAlert,
        Severity::Danger => Icon::OctagonAlert,
    }
}

/// The ARIA role a severity takes: only a failure interrupts.
fn role_of(severity: Severity) -> &'static str {
    match severity {
        Severity::Danger => "alert",
        Severity::Info | Severity::Ok | Severity::Warn => "status",
    }
}

/// A banner in the flow. `text` says what is so and `detail` what follows from it; `icon`
/// overrides the severity's glyph; `actions` is the caller's buttons (a `Button` at
/// `ControlSize::Small` reads best) and `onclose` adds the close button. `common` puts the
/// consumer's `id`, `data-*` and classes on the root and `aria_label` names it. `shown` (default
/// `Visible`) is whether it is drawn at all; there is no motion either way.
#[component]
pub fn InlineBanner(
    #[props(default)] severity: Severity,
    #[props(into)] text: TextLine,
    #[props(default)] detail: Option<TextLine>,
    #[props(default)] icon: Option<Icon>,
    #[props(default)] actions: Option<Element>,
    #[props(default)] onclose: Option<EventHandler<()>>,
    #[props(default = Shown::Visible)] shown: Shown,
    #[props(default)] common: Common,
) -> Element {
    if shown == Shown::Hidden {
        return rsx! {};
    }
    let class = common.class("ds-inline-banner");
    let data = common.data_attributes();
    let icon = icon.unwrap_or_else(|| glyph_of(severity));
    rsx! {
        div {
            id: common.id.clone(),
            class,
            role: role_of(severity),
            "aria-label": common.aria_label.clone(),
            "data-severity": severity.slug(),
            onmounted: move |event| common.mounted(event),
            ..data,
            span { class: "ds-inline-banner-icon",
                IconView { source: icon.into() }
            }
            div { class: "ds-inline-banner-body",
                Label { text, style: LabelStyle::Headline }
                if let Some(detail) = detail {
                    Label { text: detail, role: LabelRole::Secondary, style: LabelStyle::Footnote }
                }
            }
            if let Some(actions) = actions {
                div { class: "ds-inline-banner-actions", {actions} }
            }
            if let Some(onclose) = onclose {
                Button {
                    bezel: Bezel::Toolbar,
                    size: ControlSize::Small,
                    image: ImagePosition::Only,
                    icon: Icon::X,
                    label: "Close",
                    onclick: move |_| onclose.call(()),
                }
            }
        }
    }
}
