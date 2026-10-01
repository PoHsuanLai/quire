//! InlineBanner: a message in a pane's own flow, above the content it is about (design/30 section
//! 2.9): "remote images are blocked" over a message, "a read receipt was asked for", "you
//! accepted this invitation". It is not a notification (that is [`Toast`](super::toast) and the
//! shell's banners): it takes a place in the layout, stays until what it says stops being true or
//! the person closes it, and never floats.
//!
//! It comes and goes with the caller's `shown` (design/30 section 1.3: Presence and Collapse): it
//! fades in over `--t-quick` while its height opens over `--t-move`, so what is under it slides
//! down; hidden, it fades out and its height closes over `--t-move`, and `on_hidden` runs once
//! it has settled so the caller can drop it. Under Reduced the height snaps and the banner
//! cross-fades. Mounted `Visible` (the default) it simply stands in its place.
//!
//! Markup: `div.ds-inline-banner-frame` (the collapsing height) around
//! `div.ds-inline-banner[data-severity][data-presence]` of `span.ds-inline-banner-icon`,
//! `div.ds-inline-banner-body` (the `text`, the `detail` under it), `div.ds-inline-banner-actions`
//! (the caller's buttons) and, when it can be closed, a close button. A `Danger` banner is
//! `role="alert"`, the others `role="status"`.

use crate::components::content::icon_view::IconView;
use crate::components::content::label::{Label, LabelRole, LabelStyle};
use crate::components::content::text_runs::TextLine;
use crate::components::controls::button::Button;
use crate::components::controls::button_model::{Bezel, ImagePosition};
use crate::host::measure::client_rect;
use crate::root::common::Common;
use dioxus::core::current_scope_id;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::vocab::{Severity, Shown};
use ds_core::word::Word;
use ds_motion::anim::Anim;
use ds_motion::detail::level::use_level;
use ds_motion::presence::{
    Exit, Presence,
    spec::PresenceSpec,
    use_presence::{Presented, use_presence},
};
use ds_motion::use_collapse::{Collapse, use_collapse};
use ds_style::appearance::motion::MotionLevel;
use ds_style::icon::Icon;
use ds_style::task::spawn_in;
use ds_style::tokens::control_size::ControlSize;
use std::rc::Rc;

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

/// What the banner's height follows. It closes as soon as the banner leaves, except under
/// Reduced, where it stays open while the banner cross-fades and goes with it when it is dropped.
fn height_target(presence: Presence, level: MotionLevel) -> Shown {
    match (presence, level) {
        (Presence::Hidden, _) | (Presence::Leaving(_), MotionLevel::Standard) => Shown::Hidden,
        (Presence::Leaving(_), MotionLevel::Reduced)
        | (Presence::Entering | Presence::Present, _) => Shown::Visible,
    }
}

/// The frame's `style`: nothing at rest open (the banner takes its natural height), else that
/// share of the measured height, clipped. The fade is the banner's own animation, so under
/// Reduced (where the height jumps) it is still seen.
fn frame_style(collapse: Collapse, measured: Px) -> Option<String> {
    match collapse.open().0 {
        1000.. => None,
        0 => Some("height:0px;overflow:hidden".to_owned()),
        open => Some(format!(
            "height:{:.2}px;overflow:hidden",
            measured.0 * f32::from(open) / 1000.0
        )),
    }
}

/// Read the banner's natural height into `measured`. A banner that is not laid out (not mounted,
/// or a stale element) reads as nothing, and nothing never replaces a height already known.
async fn remeasure(element: &MountedData, mut measured: Signal<Px>) {
    if let Some(rect) = client_rect(element).await
        && rect.size.height.0 > 0.0
    {
        measured.set(rect.size.height);
    }
}

/// A banner in the flow. `text` says what is so and `detail` what follows from it; `icon`
/// overrides the severity's glyph; `actions` is the caller's buttons (a `Button` at
/// `ControlSize::Small` reads best) and `onclose` adds the close button. `common` puts the
/// consumer's `id`, `data-*` and classes on the root and `aria_label` names it.
///
/// `shown` is the caller's (`Visible` by default): shown after being hidden, the banner fades in
/// and opens its height; hidden, it fades out, closes, and `on_hidden` runs once it has settled
/// (the caller then drops it). A show while it closes takes the hide back.
#[component]
pub fn InlineBanner(
    #[props(default)] severity: Severity,
    #[props(into)] text: TextLine,
    #[props(default)] detail: Option<TextLine>,
    #[props(default)] icon: Option<Icon>,
    #[props(default)] actions: Option<Element>,
    #[props(default)] onclose: Option<EventHandler<()>>,
    #[props(default = Shown::Visible)] shown: Shown,
    #[props(default)] on_hidden: Option<EventHandler<()>>,
    #[props(default)] common: Common,
) -> Element {
    let Presented { presence, alias } = use_presence(
        shown,
        PresenceSpec {
            enter: Anim::PaletteFade,
            exit: Exit::OsdOut,
        },
        on_hidden,
    );
    let level = use_level().now();
    let collapse = use_collapse(height_target(presence, level));
    let mut content = use_signal(|| None::<Rc<MountedData>>);
    let measured = use_signal(|| Px(0.0));
    let scope = use_hook(current_scope_id);
    use_effect(use_reactive!(|shown| {
        let _ = shown;
        if let Some(element) = content.peek().clone() {
            spawn_in(scope, async move {
                remeasure(&element, measured).await;
            });
        }
    }));
    let Some(drawn) = presence.drawn_slug() else {
        return rsx! {};
    };
    let hidden_from_readers = matches!(presence, Presence::Leaving(_)).then_some("true");
    let class = common.class("ds-inline-banner");
    let data = common.data_attributes();
    let icon = icon.unwrap_or_else(|| glyph_of(severity));
    rsx! {
        div { class: "ds-inline-banner-frame", style: frame_style(collapse, measured()),
            div {
                id: common.id.clone(),
                class,
                role: role_of(severity),
                "aria-label": common.aria_label.clone(),
                "aria-hidden": hidden_from_readers,
                "data-severity": severity.slug(),
                "data-presence": drawn,
                "data-pulse": alias.slug(),
                onmounted: move |event| {
                    let element = event.data();
                    content.set(Some(element.clone()));
                    spawn_in(scope, async move {
                        remeasure(&element, measured).await;
                    });
                    common.mounted(event);
                },
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
}
