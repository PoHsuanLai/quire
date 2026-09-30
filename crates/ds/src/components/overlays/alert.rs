//! Alert: a short question with Cancel and one action, as the Mac draws an `NSAlert` before
//! Liquid Glass (design/04-COMPONENTS.md section 55): a narrow panel over a modal
//! scrim, everything centred in one column (an optional icon at 48, a bold title, the message
//! in the soft ink), then Cancel and the action side by side at equal width, the action on the
//! right. It enters with the sheet's `peek-in` and, hidden by its host, springs out as a sheet
//! does.
//!
//! Keys (design/06-INTERACTIONS.md sections 17 and 18): Escape, or a press on the scrim, is
//! Cancel; Return presses the default button, whichever button has the keyboard; Space presses
//! the button that has it; Tab moves between the two. The default button is the action, unless
//! the action is destructive, when it is Cancel (`AlertEmphasis::default_button`), and the
//! keyboard starts on the default button. Blitz synthesises no click from a key, and each key
//! taken here is prevented, so a browser's synthesised click cannot press a button twice.
//!
//! Where it stands: `Flow::Floating` (the default) is a `Sheet` in the overlay, centred in the
//! whole root (a full window); `Flow::Inline` draws the same panel and scrim where the caller
//! renders it, covering the nearest positioned ancestor (a control-center popover), with no
//! layer of its own on the stack.

use crate::components::content::icon_source::IconSource;
use crate::components::content::icon_view::IconView;
use crate::components::content::text_runs::{TextLine, text};
use crate::components::controls::button::{Button, ButtonVariant};
use crate::components::controls::button_size::ButtonSize;
use crate::components::overlays::alert_vocab::{AlertButton, AlertEmphasis};
use crate::components::overlays::flow::Flow;
use crate::components::overlays::scrim::{ScrimLook, scrim_button_as};
use crate::components::overlays::scrim_strength::ScrimStrength;
use crate::components::overlays::{
    sheet::Sheet, sheet_placement::SheetPlacement, sheet_width::SheetWidth,
};
use crate::focus::soon::focus_soon;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::Shown;
use ds_motion::anim::Anim;
use ds_motion::presence::spring::use_spring_presence;
use ds_style::icon::render::IconSize;
use std::rc::Rc;

/// What an alert says and what its buttons do.
#[derive(Debug, Clone, PartialEq)]
struct Words {
    title: String,
    message: Option<TextLine>,
    action: String,
    cancel: String,
    emphasis: AlertEmphasis,
    icon: Option<IconSource>,
    onaction: EventHandler<()>,
    oncancel: EventHandler<()>,
}

/// A question with Cancel and one action.
///
/// `title` is the question ("Turn Bluetooth off?"), `message` what follows from it; `action`
/// labels the action button ("Turn Off") and `cancel` the other ("Cancel"). `emphasis:
/// AlertEmphasis::Destructive` draws the action's label red and makes Cancel the default (see
/// [`AlertEmphasis`]). `onaction` hears the action; `oncancel` hears Cancel, Escape and a press
/// on the scrim. `icon` is drawn at 48 above the title (an app's icon; none by default).
///
/// `flow: Flow::Inline` draws it where the caller renders it, over the nearest positioned
/// ancestor, for a surface too small for a sheet's root (a 320 px control-center popover);
/// render it last in that container. `Flow::Floating` (the default) centres it in the root's
/// overlay, which needs a root with a height (`RootExtent::Viewport`). `shown` and `on_hidden`
/// are the sheet's, for a host that hides it with its exit rather than unmounting it.
/// `panel_id` names the panel, for a host whose blur region resolves an id.
#[component]
pub fn Alert(
    #[props(into)] title: String,
    #[props(default)] message: Option<TextLine>,
    #[props(into)] action: String,
    #[props(default = "Cancel".to_owned(), into)] cancel: String,
    #[props(default)] emphasis: AlertEmphasis,
    onaction: EventHandler<()>,
    oncancel: EventHandler<()>,
    #[props(default)] icon: Option<IconSource>,
    #[props(default)] flow: Flow,
    #[props(default)] shown: Option<Shown>,
    #[props(default)] on_hidden: Option<EventHandler<()>>,
    #[props(default)] panel_id: Option<String>,
) -> Element {
    let words = Words {
        title: title.clone(),
        message,
        action,
        cancel,
        emphasis,
        icon,
        onaction,
        oncancel,
    };
    match flow {
        Flow::Floating => rsx! {
            Sheet {
                label: title,
                onclose: move |()| oncancel.call(()),
                shown,
                on_hidden,
                placement: SheetPlacement::Centre,
                scrim: ScrimStrength::Modal,
                width: SheetWidth::Narrow,
                id: panel_id,
                AlertBody { words }
            }
        },
        Flow::Inline => rsx! {
            InlineAlert { words, shown, on_hidden, panel_id }
        },
    }
}

/// The alert drawn in place: a stage over the nearest positioned ancestor holding a modal scrim
/// and the sheet's panel, centred, with the sheet's entrance and exit.
#[component]
fn InlineAlert(
    words: Words,
    shown: Option<Shown>,
    on_hidden: Option<EventHandler<()>>,
    panel_id: Option<String>,
) -> Element {
    let showing = use_spring_presence(shown, on_hidden, Anim::PeekIn);
    if !showing.drawn() {
        return rsx! {};
    }
    let oncancel = words.oncancel;
    let look = ScrimLook {
        presence: showing.leaving().then_some("leaving"),
        strength: ScrimStrength::Modal,
    };
    let close = format!("Close {}", words.title);
    rsx! {
        div { class: "ds-alert-stage", "data-flow": Flow::Inline.attr(),
            {scrim_button_as(&close, look, || true, oncancel)}
            div { class: "ds-sheet-stage",
                div {
                    class: "ds-sheet",
                    id: panel_id,
                    "data-presence": showing.slug(),
                    "data-drive": showing.drive(),
                    style: showing.style(),
                    "data-placement": SheetPlacement::Centre.attribute(),
                    "data-width": SheetWidth::Narrow.attribute(),
                    role: "alertdialog",
                    "aria-label": "{words.title}",
                    AlertBody { words: words.clone() }
                }
            }
        }
    }
}

/// The column: icon, title, message, and the two buttons, with the keys.
#[component]
fn AlertBody(words: Words) -> Element {
    let default = words.emphasis.default_button();
    let buttons = use_hook(|| CopyValue::new(Buttons::default()));
    let press = move |button: AlertButton| match button {
        AlertButton::Cancel => words.oncancel.call(()),
        AlertButton::Action => words.onaction.call(()),
    };
    let press = EventHandler::new(press);
    rsx! {
        div {
            class: "ds-alert",
            "data-emphasis": words.emphasis.attribute(),
            onkeydown: move |event| {
                if let Some(button) = alert_key(&event.key(), default) {
                    event.stop_propagation();
                    event.prevent_default();
                    press.call(button);
                }
            },
            if let Some(icon) = words.icon {
                div { class: "ds-alert-icon",
                    IconView { source: icon, size: IconSize::Tile48 }
                }
            }
            div { class: "ds-alert-title", "{words.title}" }
            if let Some(message) = &words.message {
                div { class: "ds-alert-message", {text(message)} }
            }
            div { class: "ds-alert-actions",
                {slot(Slot { button: AlertButton::Cancel, label: &words.cancel, emphasis: words.emphasis, press, buttons })}
                {slot(Slot { button: AlertButton::Action, label: &words.action, emphasis: words.emphasis, press, buttons })}
            }
        }
    }
}

/// The two buttons' elements, so Tab can keep the keyboard inside the alert (it is modal).
#[derive(Default)]
struct Buttons {
    cancel: Option<Rc<MountedData>>,
    action: Option<Rc<MountedData>>,
}

impl Buttons {
    fn keep(&mut self, button: AlertButton, element: Rc<MountedData>) {
        match button {
            AlertButton::Cancel => self.cancel = Some(element),
            AlertButton::Action => self.action = Some(element),
        }
    }

    /// The element of the button that is not `button`: with two, Tab and Shift+Tab both go
    /// there.
    fn other(&self, button: AlertButton) -> Option<Rc<MountedData>> {
        match button {
            AlertButton::Cancel => self.action.clone(),
            AlertButton::Action => self.cancel.clone(),
        }
    }
}

/// What one slot draws.
struct Slot<'a> {
    button: AlertButton,
    label: &'a str,
    emphasis: AlertEmphasis,
    press: EventHandler<AlertButton>,
    buttons: CopyValue<Buttons>,
}

/// One button in its slot: the slot takes Space for the button inside it and Tab to the other
/// button, and the default button takes the keyboard as it mounts.
fn slot(slot: Slot<'_>) -> Element {
    let Slot {
        button,
        label,
        emphasis,
        press,
        mut buttons,
    } = slot;
    let (variant, size) = face(button, emphasis);
    let starts = emphasis.default_button() == button;
    rsx! {
        span {
            class: "ds-alert-slot",
            onkeydown: move |event| {
                let key = event.key();
                if is_space(&key) {
                    event.stop_propagation();
                    event.prevent_default();
                    press.call(button);
                } else if key == Key::Tab {
                    event.stop_propagation();
                    event.prevent_default();
                    if let Some(other) = buttons.peek().other(button) {
                        focus_soon(other);
                    }
                }
            },
            Button {
                common: Common { mounted: Some(EventHandler::new(move |event: MountedEvent| {
                    buttons.write().keep(button, event.data());
                    if starts {
                        focus_soon(event.data());
                    }
                })), ..Common::default() },
                variant,
                size,
                label: label.to_owned(),
                onclick: move |_| press.call(button),
            }
        }
    }
}

/// How a button is drawn: the default accent-filled, Cancel otherwise the ghost, a destructive
/// action a Danger at the Regular size (its red label is the alert's rule).
fn face(button: AlertButton, emphasis: AlertEmphasis) -> (ButtonVariant, Option<ButtonSize>) {
    match (button, emphasis.default_button() == button, emphasis) {
        (_, true, _) => (ButtonVariant::Primary, None),
        (AlertButton::Action, false, AlertEmphasis::Destructive) => {
            (ButtonVariant::Danger, Some(ButtonSize::Regular))
        }
        (AlertButton::Cancel | AlertButton::Action, false, _) => (ButtonVariant::Secondary, None),
    }
}

/// The button a key presses anywhere in the alert: Escape is Cancel, Return the default.
fn alert_key(key: &Key, default: AlertButton) -> Option<AlertButton> {
    match key {
        Key::Escape => Some(AlertButton::Cancel),
        Key::Enter => Some(default),
        _ => None,
    }
}

/// Whether a key is Space.
fn is_space(key: &Key) -> bool {
    *key == Key::Character(" ".into())
}

#[cfg(test)]
mod tests {
    use super::{alert_key, face, is_space};
    use crate::components::controls::button::ButtonVariant;
    use crate::components::controls::button_size::ButtonSize;
    use crate::components::overlays::alert_vocab::{AlertButton, AlertEmphasis};
    use dioxus::prelude::Key;

    #[test]
    fn escape_cancels_and_return_presses_the_default() {
        let cases = [
            (Key::Escape, AlertButton::Action, Some(AlertButton::Cancel)),
            (Key::Escape, AlertButton::Cancel, Some(AlertButton::Cancel)),
            (Key::Enter, AlertButton::Action, Some(AlertButton::Action)),
            (Key::Enter, AlertButton::Cancel, Some(AlertButton::Cancel)),
            (Key::Tab, AlertButton::Action, None),
        ];
        for (key, default, want) in cases {
            assert_eq!(alert_key(&key, default), want, "{key:?} {default:?}");
        }
        assert!(is_space(&Key::Character(" ".into())));
        assert!(!is_space(&Key::Character("a".into())));
    }

    #[test]
    fn the_default_is_filled_and_a_destructive_action_is_red() {
        use AlertButton::{Action, Cancel};
        use AlertEmphasis::{Default, Destructive};
        let cases = [
            (Cancel, Default, ButtonVariant::Secondary, None),
            (Action, Default, ButtonVariant::Primary, None),
            (Cancel, Destructive, ButtonVariant::Primary, None),
            (
                Action,
                Destructive,
                ButtonVariant::Danger,
                Some(ButtonSize::Regular),
            ),
        ];
        for (button, emphasis, variant, size) in cases {
            assert_eq!(
                face(button, emphasis),
                (variant, size),
                "{button:?} {emphasis:?}"
            );
        }
    }
}
