//! PopUpButton: a button that opens a menu of choices (`NSPopUpButton`, design/30 section 2.1).
//! A pop-up shows the chosen item and marks it in its menu; a pull-down keeps a fixed title and
//! marks nothing; an overflow is a pull-down drawn as the ⋯ image alone (Finder's Action button),
//! for the extra commands of a row or a toolbar. It owns its chevrons, opens a `Menu`, and selects
//! by typing while it holds the keyboard.

use crate::components::content::icon_source::IconSource;
use crate::components::controls::button::Button;
use crate::components::controls::button_marks::Trailing;
use crate::components::controls::button_model::{Bezel, ImagePosition};
use crate::components::menus::item::item::MenuItem;
use crate::components::menus::menu::menu::Menu;
use crate::components::menus::menu::placement::MenuPlacement;
use crate::host::measure::{Anchor, MountedRef, use_rect};
use crate::root::common::Common;
use crate::stack::typeahead::Typeahead;
use dioxus::prelude::*;
use ds_core::command::types_text;
use ds_core::press::Press;
use ds_core::time::clock::now;
use ds_core::vocab::{Availability, Check, Shown};
use ds_core::word::Word;
use ds_style::icon::Icon;
use ds_style::tokens::control_size::ControlSize;

/// Which pop-up button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum PopUpKind {
    /// It shows the chosen item; its menu marks it.
    #[default]
    PopUp,
    /// It shows a fixed title; its menu marks nothing.
    PullDown,
    /// It shows the ⋯ glyph alone, no title and no chevrons; its menu marks nothing. `title` is
    /// its accessible name ("More" when absent).
    Overflow,
}

/// The title of the item `value` names, if any.
fn chosen_title<T: PartialEq>(items: &[MenuItem<T>], value: Option<&T>) -> Option<String> {
    let value = value?;
    items.iter().find_map(|item| match item {
        MenuItem::Item {
            value: own, title, ..
        } if own == value => Some(title.clone()),
        MenuItem::Item { .. }
        | MenuItem::Submenu { .. }
        | MenuItem::Header(_)
        | MenuItem::Info { .. }
        | MenuItem::Separator => None,
    })
}

/// `items` with the chosen one marked `On` and the others `Off`: a pop-up's menu.
fn marked<T: Clone + PartialEq>(items: &[MenuItem<T>], value: Option<&T>) -> Vec<MenuItem<T>> {
    items
        .iter()
        .map(|item| match item {
            MenuItem::Item { value: own, .. } => item.clone().with_check(if Some(own) == value {
                Check::On
            } else {
                Check::Off
            }),
            other => other.clone(),
        })
        .collect()
}

/// A pop-up button over `items`. `value` is the chosen item's value for a pop-up (its title
/// is the button's); `title` is a pull-down's fixed label and a pop-up's while nothing is
/// chosen. `onpick` hears the value a pick asks for; the caller's `value` decides what is
/// chosen. While the button holds the keyboard, letters choose the next item that starts with
/// them.
///
/// `start` is whether the menu is open as the button mounts (closed unless asked). `open`
/// hands the open state to the caller: `Some(shown)` is the state (the button's own is then
/// ignored, `start` included), and every change the person asks for (a press, an arrow key, a
/// pick, Escape or a click outside) reaches `on_open_change` as the state it wants, so the
/// caller stores it and passes it back. With `open: None` the button keeps its own state and
/// `on_open_change` only listens.
///
/// The menu is placed against the button's own element, which a renderer reports once it has
/// laid the button out; `anchor` names another place instead (a point or a rect), and then the menu
/// needs no element at all. A document with no renderer (a unit test over a `VirtualDom` and
/// `dioxus-ssr`) never reports one, so it opens a pop-up with
/// `start: Shown::Visible, anchor: Some(Anchor::Point(..))`, or by clicking the button once
/// `anchor` is set.
#[component]
pub fn PopUpButton<T: Clone + PartialEq + 'static>(
    items: Vec<MenuItem<T>>,
    onpick: EventHandler<T>,
    #[props(default)] kind: PopUpKind,
    #[props(default)] value: Option<T>,
    #[props(default)] title: Option<String>,
    #[props(default)] size: ControlSize,
    #[props(default)] start: Shown,
    #[props(default)] open: Option<Shown>,
    #[props(default)] on_open_change: Option<EventHandler<Shown>>,
    #[props(default)] anchor: Option<Anchor>,
    #[props(default)] availability: Availability,
    #[props(default)] common: Common,
) -> Element {
    let mut own = use_signal(move || start);
    let shown = open.unwrap_or(own());
    let mut set_open = move |to: Shown| {
        if open.is_none() {
            own.set(to);
        }
        if let Some(heard) = on_open_change {
            heard.call(to);
        }
    };
    let mut element = use_signal(|| None::<MountedRef>);
    let button = use_rect();
    let typeahead = use_hook(|| CopyValue::new(Typeahead::default()));
    let live = availability == Availability::Enabled;
    let label = match kind {
        PopUpKind::PopUp => chosen_title(&items, value.as_ref()).or(title.clone()),
        PopUpKind::PullDown => title.clone(),
        PopUpKind::Overflow => title.clone().or_else(|| Some("More".to_string())),
    }
    .unwrap_or_default();
    let widths: Vec<String> = match kind {
        PopUpKind::PopUp => items
            .iter()
            .filter_map(|item| match item {
                MenuItem::Item { title, .. } => Some(title.clone()),
                _ => None,
            })
            .collect(),
        PopUpKind::PullDown | PopUpKind::Overflow => Vec::new(),
    };
    let listed = match kind {
        PopUpKind::PopUp => marked(&items, value.as_ref()),
        PopUpKind::PullDown | PopUpKind::Overflow => items.clone(),
    };
    let (bezel, image, icon, trailing) = match kind {
        PopUpKind::PopUp => (
            Bezel::Push,
            ImagePosition::Leading,
            None,
            Some(Trailing::Glyph(Icon::ChevronsUpDown)),
        ),
        PopUpKind::PullDown => (
            Bezel::Push,
            ImagePosition::Leading,
            None,
            Some(Trailing::Glyph(Icon::ChevronDown)),
        ),
        PopUpKind::Overflow => (
            Bezel::Toolbar,
            ImagePosition::Only,
            Some(IconSource::Glyph(Icon::Ellipsis)),
            None,
        ),
    };
    let mounted = common.clone();
    let typed = items.clone();
    let onkey = move |event: KeyboardEvent| match event.key() {
        Key::ArrowDown | Key::ArrowUp if live => {
            event.prevent_default();
            set_open(Shown::Visible);
        }
        Key::Character(text)
            if live
                && kind == PopUpKind::PopUp
                && !text.trim().is_empty()
                && types_text(&event.key(), event.modifiers()) =>
        {
            let choices: Vec<(&T, &str)> = typed
                .iter()
                .filter_map(|item| match item {
                    MenuItem::Item {
                        value,
                        title,
                        availability: Availability::Enabled,
                        ..
                    } => Some((value, title.as_str())),
                    _ => None,
                })
                .collect();
            let labels: Vec<&str> = choices.iter().map(|(_, title)| *title).collect();
            let from = choices
                .iter()
                .position(|(own, _)| Some(*own) == value.as_ref())
                .unwrap_or(choices.len().saturating_sub(1));
            let mut buffer = typeahead;
            let (next, found) = buffer.peek().clone().typed(&text, now(), &labels, from);
            buffer.set(next);
            if let Some((picked, _)) = found.and_then(|at| choices.get(at)) {
                onpick.call((*picked).clone());
            }
        }
        _ => {}
    };
    let anchor = anchor.or_else(|| element().map(Anchor::Mounted));
    rsx! {
        span {
            class: "ds-popup",
            "data-kind": kind.slug(),
            "data-size": size.slug(),
            onkeydown: onkey,
            Button {
                label: label.clone(),
                bezel,
                image,
                icon,
                size,
                availability,
                trailing,
                // An overflow draws no words, so its name is its tooltip too.
                title: match kind {
                    PopUpKind::Overflow => Some(label.clone()),
                    PopUpKind::PopUp | PopUpKind::PullDown => None,
                },
                shown: Some(shown),
                onclick: move |_: Press| {
                    if live {
                        set_open(shown.flipped());
                    }
                },
                common: Common {
                    mounted: Some(EventHandler::new(move |event: MountedEvent| {
                        element.set(Some(MountedRef(event.data())));
                        button.on_mounted(event.clone());
                        mounted.mounted(event);
                    })),
                    ..common.clone()
                },
            }
            // A pop-up is as wide as its widest item, so choosing never resizes it.
            for widest in widths {
                span { class: "ds-popup-sizer", "aria-hidden": "true", "{widest}" }
            }
        }
        if shown == Shown::Visible && let Some(anchor) = anchor {
            Menu::<T> {
                placement: MenuPlacement::Popup,
                anchor,
                measured: button.rect(),
                items: listed,
                onpick: move |picked: T| {
                    set_open(Shown::Hidden);
                    onpick.call(picked);
                },
                onclose: move |()| set_open(Shown::Hidden),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{chosen_title, marked};
    use crate::components::menus::item::item::MenuItem;
    use ds_core::vocab::Check;

    fn items() -> Vec<MenuItem<u8>> {
        vec![
            MenuItem::new(1, "Archive"),
            MenuItem::Separator,
            MenuItem::new(2, "Move"),
        ]
    }

    #[test]
    fn a_pop_up_shows_the_chosen_title_and_marks_only_that_item() {
        assert_eq!(chosen_title(&items(), Some(&2)).as_deref(), Some("Move"));
        assert_eq!(chosen_title(&items(), Some(&9)), None);
        assert_eq!(chosen_title(&items(), None), None);
        let checks: Vec<Option<Check>> = marked(&items(), Some(&2))
            .iter()
            .filter_map(|item| match item {
                MenuItem::Item { check, .. } => Some(*check),
                _ => None,
            })
            .collect();
        assert_eq!(checks, [Some(Check::Off), Some(Check::On)]);
    }
}
