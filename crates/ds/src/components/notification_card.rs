//! NotificationCard: one notification, as a banner or a row of the notification center (sill
//! Q120; design/20 section 1.6, design/13 section 13.3.6).
//!
//! The card is a plate in its material (Toast by default) inside a scope of that material whose
//! own box paints nothing (`Surface { chrome: Some(RootChrome::Transparent) }`), so the same card
//! paints the Toast material in a Toast root or in a Popover-rooted center. The plate holds the
//! app's icon at 32, the summary (13/700, one line) with the group's count chip and the age (the
//! data face at the caption size) on its first line, the body under it, and the actions. A
//! press on the plate opens the notification (`on_open`); the close button and every action and
//! link keep their press, so none of them also opens it.
//!
//! Under the pointer (`data-hover="on"`, told to `on_hover` so the caller's hold timer can
//! pause) the body opens from two lines to six over `--t-move --e-out`, the actions row opens
//! under it, and the 18 px close button shows at the top left corner, as macOS draws it. Keyboard
//! focus on the close button shows it too.
//!
//! A group (`count`) shows its count in a chip and draws `layers` offset plates behind the card,
//! each `--notifications-group-offset` lower and a little narrower.

use crate::components::button::{Button, ButtonVariant};
use crate::components::icon_view::IconView;
use crate::components::notification_body::NotificationBody;
use crate::components::notification_parts::{AppMark, CardAction, GroupCount, Hover};
use crate::components::press::{Press, PressListeners, Propagation};
use crate::components::rich_text::Rich;
use crate::components::text_runs::{Text, text};
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconPx, IconSize};
use crate::material::Material;
use crate::root::chrome::RootChrome;
use crate::root::surface::Surface;
use dioxus::prelude::*;

/// One notification. `app`, `age` (the caller's words: "now", "2m") and `summary` are its first
/// line; `body` (runs and links, clamped) is optional, as are the group's `count` and the
/// `actions`. `on_open` hears a press on the card, Enter or Space; `on_close` a press on the
/// close button; `on_link` a press on a link in the body. `icon_size` is the icon's side
/// (`notifications.icon_px`, 32); `id` names the plate for its blur region
/// (`Element("toast-<id>")`).
#[component]
pub fn NotificationCard(
    app: AppMark,
    #[props(into)] age: Text,
    #[props(into)] summary: Text,
    #[props(default)] body: Option<Rich>,
    #[props(default)] count: Option<GroupCount>,
    #[props(default)] actions: Vec<CardAction>,
    on_close: EventHandler<Press>,
    on_open: EventHandler<Press>,
    #[props(default)] on_link: Option<EventHandler<String>>,
    #[props(default)] on_hover: Option<EventHandler<Hover>>,
    #[props(default = Material::Toast)] material: Material,
    #[props(default = IconPx(32))] icon_size: IconPx,
    #[props(default)] id: Option<String>,
) -> Element {
    let mut hover = use_signal(Hover::default);
    let mut point = move |next: Hover| {
        if *hover.peek() != next {
            hover.set(next);
            if let Some(on_hover) = on_hover {
                on_hover.call(next);
            }
        }
    };
    let layers = count.map_or(0, |group| group.layers.drawn());
    let label = format!("{}: {}", app.name.plain_text(), summary.plain_text());
    let open = PressListeners::new(on_open);
    rsx! {
        Surface { material, chrome: RootChrome::Transparent,
            div {
                class: "ds-notification",
                "data-hover": hover().slug(),
                style: (layers > 0).then(|| format!("--layers:{layers}")),
                onpointerenter: move |_| point(Hover::Over),
                onpointerleave: move |_| point(Hover::Away),
                for n in (1..=layers).rev() {
                    div { key: "{n}", class: "ds-notification-layer", "aria-hidden": "true", style: "--i:{n}" }
                }
                div {
                    class: "ds-notification-plate",
                    id,
                    role: "button",
                    tabindex: "0",
                    "aria-label": "{label}",
                    onclick: move |event| open.click(&event),
                    onkeydown: move |event| {
                        if opens(&event.key()) {
                            event.prevent_default();
                            on_open.call(Press::primary());
                        }
                    },
                    span { class: "ds-notification-icon",
                        IconView { source: app.icon.clone(), size: IconSize::Px(icon_size) }
                    }
                    div { class: "ds-notification-words",
                        {head(&summary, &age, count)}
                        if let Some(body) = body {
                            NotificationBody { key: "{body.plain_text()}", body: body.clone(), on_link }
                        }
                        if !actions.is_empty() {
                            {action_row(&actions)}
                        }
                    }
                }
                {close_button(on_close)}
            }
        }
    }
}

/// Whether a key on the plate opens the notification: Enter or Space, a button's own keys.
fn opens(key: &Key) -> bool {
    matches!(key, Key::Enter) || *key == Key::Character(" ".into())
}

/// The first line: the summary, the group's count from two, and the age.
fn head(summary: &Text, age: &Text, count: Option<GroupCount>) -> Element {
    let shown = count.map(|group| group.count).filter(|&count| count > 1);
    rsx! {
        div { class: "ds-notification-head",
            span { class: "ds-notification-summary ds-truncate", {text(summary)} }
            if let Some(count) = shown {
                span { class: "ds-notification-count", "{count}" }
            }
            span { class: "ds-notification-age", {text(age)} }
        }
    }
}

/// The actions, as Mini buttons that keep their press.
fn action_row(actions: &[CardAction]) -> Element {
    rsx! {
        div { class: "ds-notification-actions",
            for (n , action) in actions.iter().cloned().enumerate() {
                Button {
                    key: "{n}",
                    variant: ButtonVariant::Mini,
                    label: action.label,
                    propagation: Propagation::Stop,
                    onclick: move |press| action.on_press.call(press),
                }
            }
        }
    }
}

/// The close button at the top left corner: its own hit target, keeping its press.
fn close_button(on_close: EventHandler<Press>) -> Element {
    let close = PressListeners::new(on_close).with_propagation(Propagation::Stop);
    rsx! {
        button {
            r#type: "button",
            class: "ds-notification-close",
            "aria-label": "Close",
            onclick: move |event| close.click(&event),
            Glyph { icon: Icon::X, size: IconSize::Micro }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::opens;
    use dioxus::prelude::Key;

    #[test]
    fn enter_and_space_open_the_card() {
        let cases = [
            (Key::Enter, true),
            (Key::Character(" ".into()), true),
            (Key::Escape, false),
            (Key::Tab, false),
        ];
        for (key, want) in cases {
            assert_eq!(opens(&key), want, "{key:?}");
        }
    }
}
