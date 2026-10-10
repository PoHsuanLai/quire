//! PullDownButton: a button that opens a menu of actions (`NSPopUpButton` in its pull-down
//! style, design/30 section 2.1). The button wears an icon, a label or both and a chevron down,
//! and keeps them whatever is picked; its menu marks nothing and hangs from the button's lower
//! edge. It is the foot's one chevron in Dia's sidebar: New Space, history, Settings and Hide
//! sidebar behind a single control. It shares `PopUpButton`'s open-state props and its keys.

use crate::components::content::icon_source::IconSource;
use crate::components::controls::button::Button;
use crate::components::controls::button_marks::Trailing;
use crate::components::controls::button_model::{Bezel, ImagePosition};
use crate::components::menus::item::item::MenuItem;
use crate::components::menus::menu::menu::Menu;
use crate::components::menus::menu::placement::MenuPlacement;
use crate::host::measure::{Anchor, MountedRef, use_rect};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::press::Press;
use ds_core::vocab::{Availability, Shown};
use ds_core::word::Word;
use ds_style::icon::Icon;
use ds_style::tokens::control_size::ControlSize;

/// Which face a pull-down button wears.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum PullDownFace {
    /// `icon` and `title` (either or both), and the chevron after them.
    #[default]
    Labelled,
    /// The chevron alone, Dia's foot control; `icon` and `title` are not drawn, and
    /// `common.aria_label`, else "Menu", is its name and its tooltip.
    Chevron,
}

/// A pull-down button over `items`: `icon` and `title` are its face (either or both; with no
/// `title` the icon stands alone and `common.aria_label`, else "Menu", names it), and a chevron
/// follows; `face: PullDownFace::Chevron` draws the chevron alone. `onpick` hears the value a pick asks for; the face never changes with it.
///
/// Return, Space and a click open the menu; Down and Up do too. Escape or a pick closes it and
/// the menu hands the keyboard back to the button.
///
/// `start`, `open`, `on_open_change` and `anchor` mean what they do on `PopUpButton`: the open
/// state is the button's own unless `open: Some(shown)` hands it to the caller, who then stores
/// every state `on_open_change` reports; `anchor` places the menu somewhere other than the
/// button (a document with no renderer needs one to open it).
#[component]
pub fn PullDownButton<T: Clone + PartialEq + 'static>(
    items: Vec<MenuItem<T>>,
    onpick: EventHandler<T>,
    #[props(default)] face: PullDownFace,
    #[props(default)] icon: Option<IconSource>,
    #[props(default)] title: Option<String>,
    #[props(default = Bezel::Toolbar)] bezel: Bezel,
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
    let live = availability == Availability::Enabled;
    let (icon, title) = match face {
        PullDownFace::Labelled => (icon, title),
        PullDownFace::Chevron => (None, common.aria_label.clone()),
    };
    let image = match (face, &title, &icon) {
        (PullDownFace::Chevron, ..) | (PullDownFace::Labelled, None, Some(_)) => {
            ImagePosition::Only
        }
        (PullDownFace::Labelled, Some(_), _) | (PullDownFace::Labelled, None, None) => {
            ImagePosition::Leading
        }
    };
    let label = title.unwrap_or_else(|| "Menu".to_owned());
    let mounted = common.clone();
    let onkey = move |event: KeyboardEvent| match event.key() {
        Key::ArrowDown | Key::ArrowUp if live => {
            event.prevent_default();
            set_open(Shown::Visible);
        }
        _ => {}
    };
    let anchor = anchor.or_else(|| element().map(Anchor::Mounted));
    rsx! {
        span {
            class: "ds-popup",
            "data-kind": "pull-down",
            "data-size": size.slug(),
            onkeydown: onkey,
            Button {
                label: label.clone(),
                bezel,
                image,
                icon,
                size,
                availability,
                trailing: Some(Trailing::Glyph(Icon::ChevronDown)),
                title: match image {
                    ImagePosition::Only => Some(label.clone()),
                    ImagePosition::Leading => None,
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
        }
        if shown == Shown::Visible && let Some(anchor) = anchor {
            Menu::<T> {
                placement: MenuPlacement::Popup,
                anchor,
                measured: button.rect(),
                items,
                onpick: move |picked: T| {
                    set_open(Shown::Hidden);
                    onpick.call(picked);
                },
                onclose: move |()| set_open(Shown::Hidden),
            }
        }
    }
}
