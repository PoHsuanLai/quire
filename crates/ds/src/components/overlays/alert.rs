//! Alert: a short question or notice with one to three or more buttons, as the Mac draws an
//! `NSAlert` before Liquid Glass (design/30 section 2.5, design/04-COMPONENTS.md section 55): a
//! narrow panel, everything centred in one column (an optional hero icon at 64, a bold title, the
//! message in the soft ink), then the buttons. It stands in a [`Sheet`] and dims nothing.
//!
//! Buttons (`AlertButton`): the first is the default unless it is destructive, when the first
//! that is not is (`default_button`); one that cancels answers Escape. One or two lie side by side
//! at equal width with the default on the right, three or more stack with the default on top
//! (`FooterLayout`).
//!
//! Keys (design/06-INTERACTIONS.md sections 17 and 18): Escape presses the Cancel button; Return
//! presses the default button, whichever button has the keyboard; Space presses the button that
//! has it; Tab and Shift+Tab move between the buttons, wrapping, since the alert is modal. The
//! keyboard starts on the default button. Blitz synthesises no click from a key, and each key
//! taken here is prevented, so a browser's synthesised click cannot press a button twice.
//!
//! Where it stands: `Flow::Floating` (the default) is a `Sheet` in the overlay, centred in the
//! whole root (a full window); `Flow::Inline` draws the same panel where the caller renders it,
//! covering the nearest positioned ancestor (a control-center popover) and catching the pointer
//! for the popover under it, with no layer of its own on the stack.
//!
//! An alert may carry a suppression `Checkbox` under its message ("Do not show this again") and a
//! help button (the `Help` bezel) at the foot's leading corner.

use crate::components::content::icon_source::IconSource;
use crate::components::content::icon_view::IconView;
use crate::components::content::text_runs::{TextLine, text};
use crate::components::controls::button::Button;
use crate::components::controls::button_model::{Answers, Bezel, ButtonRole};
use crate::components::controls::checkbox::Checkbox;
use crate::components::overlays::alert_model::{
    AlertButton, AlertRole, AlertStyle, FooterLayout, Suppression, default_button, escape_button,
    tab_target,
};
use crate::components::overlays::flow::Flow;
use crate::components::overlays::sheet::Sheet;
use crate::components::overlays::sheet_attach::Attach;
use crate::components::overlays::sheet_width::SheetWidth;
use crate::focus::soon::focus_soon;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::Shown;
use ds_core::word::Word;
use ds_motion::anim::Anim;
use ds_motion::presence::{
    Exit,
    spec::PresenceSpec,
    use_presence::{Presented, use_presence},
};
use ds_style::icon::Icon;
use ds_style::icon::render::{IconPx, IconSize};
use ds_style::tokens::control_size::ControlSize;
use std::rc::Rc;

/// The hero icon's side (design/34 section 3.7).
const HERO_PX: u8 = 64;
/// The Critical badge's side, at the hero's lower right.
const BADGE_PX: u8 = 20;

/// What an alert says and what its buttons do.
#[derive(Debug, Clone, PartialEq)]
struct Words {
    title: String,
    message: Option<TextLine>,
    style: AlertStyle,
    buttons: Vec<AlertButton>,
    icon: Option<IconSource>,
    suppression: Option<Suppression>,
    help: Option<EventHandler<()>>,
}

impl Words {
    fn roles(&self) -> Vec<AlertRole> {
        self.buttons.iter().map(|button| button.role).collect()
    }

    /// Press button `index`, if there is one.
    fn press(&self, index: Option<usize>) {
        if let Some(button) = index.and_then(|index| self.buttons.get(index)) {
            button.onpress.call(());
        }
    }
}

/// A question or notice with buttons.
///
/// `title` is the question ("Turn Bluetooth off?"), `message` what follows from it; `buttons` are
/// its answers (see the module doc for their order and roles); `style` is `data-style`; `icon` is
/// drawn at 64 above the title, the caller's (an app's or provider's icon; none by default, and
/// then nothing is reserved for it); a `Critical` alert badges it with a warning glyph.
///
/// `flow: Flow::Inline` draws it where the caller renders it, over the nearest positioned
/// ancestor, for a surface too small for a sheet's root (a 320 px control-center popover);
/// render it last in that container. `Flow::Floating` (the default) centres it in the root's
/// overlay, which needs a root with a height (`RootExtent::Viewport`). `shown` and `on_hidden`
/// are the sheet's, for a host that hides it with its exit rather than unmounting it.
///
/// `common` goes on the panel (the sheet), so a host whose blur region resolves an id finds it.
#[component]
pub fn Alert(
    #[props(into)] title: String,
    #[props(default)] message: Option<TextLine>,
    buttons: Vec<AlertButton>,
    #[props(default)] style: AlertStyle,
    #[props(default)] icon: Option<IconSource>,
    #[props(default)] suppression: Option<Suppression>,
    #[props(default)] help: Option<EventHandler<()>>,
    #[props(default)] flow: Flow,
    #[props(default)] shown: Option<Shown>,
    #[props(default)] on_hidden: Option<EventHandler<()>>,
    #[props(default)] common: Common,
) -> Element {
    let words = Words {
        title: title.clone(),
        message,
        style,
        buttons,
        icon,
        suppression,
        help,
    };
    match flow {
        Flow::Floating => {
            let escape = words.clone();
            rsx! {
                Sheet {
                    label: title,
                    onclose: move |()| escape.press(escape_button(&escape.roles())),
                    shown,
                    on_hidden,
                    attach: Attach::Centre,
                    width: SheetWidth::Narrow,
                    common,
                    AlertBody { words }
                }
            }
        }
        Flow::Inline => rsx! {
            InlineAlert { words, shown, on_hidden, common }
        },
    }
}

/// The alert drawn in place: a stage over the nearest positioned ancestor holding the sheet's
/// panel, centred, with the sheet's entrance and exit.
#[component]
fn InlineAlert(
    words: Words,
    shown: Option<Shown>,
    on_hidden: Option<EventHandler<()>>,
    common: Common,
) -> Element {
    let Presented { presence, alias } = use_presence(
        shown.unwrap_or(Shown::Visible),
        PresenceSpec {
            enter: Anim::SheetIn,
            exit: Exit::SheetOut,
        },
        on_hidden,
    );
    let Some(drawn) = presence.drawn_slug() else {
        return rsx! {};
    };
    let class = common.class("ds-sheet");
    let data = common.data_attributes();
    rsx! {
        div { class: "ds-alert-stage", "data-flow": Flow::Inline.attr(),
            div { class: "ds-sheet-stage",
                div {
                    id: common.id.clone(),
                    class,
                    "data-presence": drawn,
                    "data-pulse": alias.slug(),
                    "data-attach": Attach::Centre.slug(),
                    "data-width": SheetWidth::Narrow.attribute(),
                    role: "alertdialog",
                    "aria-label": "{words.title}",
                    onmounted: move |event| common.mounted(event),
                    ..data,
                    AlertBody { words: words.clone() }
                }
            }
        }
    }
}

/// The column: icon, title, message, and the buttons, with the keys.
#[component]
fn AlertBody(words: Words) -> Element {
    let roles = words.roles();
    let default = default_button(&roles);
    let escape = escape_button(&roles);
    let count = words.buttons.len();
    let elements = use_hook(|| CopyValue::new(Vec::<Option<Rc<MountedData>>>::new()));
    let press = {
        let words = words.clone();
        EventHandler::new(move |index: Option<usize>| words.press(index))
    };
    rsx! {
        div {
            class: "ds-alert",
            "data-style": words.style.slug(),
            onkeydown: move |event| match event.key() {
                Key::Escape => {
                    event.stop_propagation();
                    event.prevent_default();
                    press.call(escape);
                }
                Key::Enter => {
                    event.stop_propagation();
                    event.prevent_default();
                    press.call(default);
                }
                _ => {}
            },
            if let Some(icon) = words.icon {
                div { class: "ds-alert-icon",
                    IconView { source: icon, size: IconSize::Px(IconPx(HERO_PX)) }
                    if words.style == AlertStyle::Critical {
                        span { class: "ds-alert-badge",
                            IconView { source: IconSource::Glyph(Icon::TriangleAlert), size: IconSize::Px(IconPx(BADGE_PX)) }
                        }
                    }
                }
            }
            div { class: "ds-alert-title", "{words.title}" }
            if let Some(message) = &words.message {
                div { class: "ds-alert-body", {text(message)} }
            }
            if let Some(suppression) = words.suppression.clone() {
                div { class: "ds-alert-suppression",
                    Checkbox {
                        label: suppression.label,
                        value: suppression.value,
                        size: ControlSize::Small,
                        onchange: suppression.onchange,
                    }
                }
            }
            if let Some(help) = words.help {
                div { class: "ds-alert-help",
                    Button { bezel: Bezel::Help, label: "Help", onclick: move |_| help.call(()) }
                }
            }
            div { class: "ds-alert-footer", "data-layout": FooterLayout::of(count).slug(),
                for (index , button) in words.buttons.iter().enumerate() {
                    {slot(Slot { index, count, button, default, press, elements })}
                }
            }
        }
    }
}

/// What one slot draws.
struct Slot<'a> {
    index: usize,
    count: usize,
    button: &'a AlertButton,
    default: Option<usize>,
    press: EventHandler<Option<usize>>,
    elements: CopyValue<Vec<Option<Rc<MountedData>>>>,
}

/// One button in its slot: the slot takes Space for the button inside it and Tab to the next
/// button, and the default button takes the keyboard as it mounts.
fn slot(slot: Slot<'_>) -> Element {
    let Slot {
        index,
        count,
        button,
        default,
        press,
        mut elements,
    } = slot;
    let is_default = default == Some(index);
    let (answers, role) = face(button.role, is_default);
    rsx! {
        span {
            key: "{index}",
            class: "ds-alert-slot",
            onkeydown: move |event| {
                let key = event.key();
                if is_space(&key) {
                    event.stop_propagation();
                    event.prevent_default();
                    press.call(Some(index));
                } else if key == Key::Tab {
                    event.stop_propagation();
                    event.prevent_default();
                    let target = tab_target(count, index, event.modifiers().shift());
                    if let Some(Some(next)) = elements.peek().get(target).cloned() {
                        focus_soon(next);
                    }
                }
            },
            Button {
                common: Common { mounted: Some(EventHandler::new(move |event: MountedEvent| {
                    let mut kept = elements.write();
                    if kept.len() <= index {
                        kept.resize(index + 1, None);
                    }
                    kept[index] = Some(event.data());
                    if is_default {
                        focus_soon(event.data());
                    }
                })), ..button.common.clone() },
                answers,
                role,
                size: ControlSize::Large,
                label: button.label.clone(),
                onclick: move |_| press.call(Some(index)),
            }
        }
    }
}

/// How a button is drawn: the default answers Return and takes the accent, a Cancel that is not
/// the default answers Escape, a destructive action that is not the default is a destructive
/// button (its red label is the alert's rule).
fn face(role: AlertRole, is_default: bool) -> (Answers, ButtonRole) {
    match (role, is_default) {
        (_, true) => (Answers::Return, ButtonRole::Normal),
        (AlertRole::Destructive, false) => (Answers::Nothing, ButtonRole::Destructive),
        (AlertRole::Cancel, false) => (Answers::Escape, ButtonRole::Normal),
        (AlertRole::Normal, false) => (Answers::Nothing, ButtonRole::Normal),
    }
}

/// Whether a key is Space.
fn is_space(key: &Key) -> bool {
    *key == Key::Character(" ".into())
}

#[cfg(test)]
mod tests {
    use super::{face, is_space};
    use crate::components::controls::button_model::{Answers, ButtonRole};
    use crate::components::overlays::alert_model::AlertRole::{Cancel, Destructive, Normal};
    use dioxus::prelude::Key;

    #[test]
    fn space_is_the_space_character() {
        assert!(is_space(&Key::Character(" ".into())));
        assert!(!is_space(&Key::Character("a".into())));
    }

    #[test]
    fn the_default_answers_return_and_a_destructive_action_is_red() {
        let cases = [
            (Cancel, false, (Answers::Escape, ButtonRole::Normal)),
            (Normal, true, (Answers::Return, ButtonRole::Normal)),
            (Normal, false, (Answers::Nothing, ButtonRole::Normal)),
            (Cancel, true, (Answers::Return, ButtonRole::Normal)),
            (
                Destructive,
                false,
                (Answers::Nothing, ButtonRole::Destructive),
            ),
        ];
        for (role, is_default, want) in cases {
            assert_eq!(face(role, is_default), want, "{role:?} {is_default}");
        }
    }
}
