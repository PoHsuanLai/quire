//! The Spaces bar as markup: `div.ds-spaces-bar[role=group]` holding a row of
//! `button.ds-spaces-bar-space` (a thumbnail when expanded, the name), and a plus at the right
//! edge when the host can add a Space.
//!
//! Shown, the bar drops in from the top edge over `--t-overview --e-out`; hidden, it lifts away
//! over `--t-overview --e-exit` and `on_hidden` runs once the lift has settled, when the host
//! unmaps its surface. The pointer over the bar expands it to thumbnails, and off it collapses it
//! again, unless the caller holds `expansion` itself.

use super::model::{BarExpansion, BarSpaceKey, PointerOver, SpaceTile};
use crate::components::content::icon_source::IconSource;
use crate::components::controls::button::Button;
use crate::components::controls::button_model::{Bezel, ImagePosition};
use crate::components::controls::press::{ActivationKeys, PressListeners};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::press::{PointerButton, Press};
use ds_core::vocab::Shown;
use ds_core::word::Word;
use ds_motion::anim::Anim;
use ds_motion::presence::{
    Exit,
    spec::PresenceSpec,
    use_presence::{Presented, use_presence},
};
use ds_style::icon::Icon;

/// The name of the plus when the caller gives none.
const ADD_LABEL: &str = "Add Space";

/// Mission Control's Spaces bar. `spaces` lists the Spaces left to right; a primary press on one
/// reports its key through `on_pick`, and the plus (drawn only when `on_add` is given, labelled
/// `add_label`) reports through `on_add`. `shown` is whether the bar is up: it drops in and lifts
/// away, and `on_hidden` runs at the end of the lift. `expansion` is how much shows: `None`
/// follows the pointer, `Some` is what the caller says, and `on_expansion_change` hears each
/// change the pointer asks for. `common.aria_label` names the group in place of `label`.
#[component]
pub fn SpacesBar(
    label: String,
    spaces: Vec<SpaceTile>,
    shown: Shown,
    on_pick: EventHandler<BarSpaceKey>,
    #[props(default)] on_add: Option<EventHandler<()>>,
    #[props(default = ADD_LABEL.to_owned())] add_label: String,
    #[props(default)] on_hidden: EventHandler<()>,
    #[props(default)] expansion: Option<BarExpansion>,
    #[props(default)] on_expansion_change: Option<EventHandler<BarExpansion>>,
    #[props(default)] common: Common,
) -> Element {
    let Presented { presence, alias } = use_presence(
        shown,
        PresenceSpec {
            enter: Anim::DropIn,
            exit: Exit::LiftOut,
        },
        Some(on_hidden),
    );
    let mut own = use_signal(BarExpansion::default);
    let now = expansion.unwrap_or_else(|| own());
    let mut ask = move |over: PointerOver| {
        let next = BarExpansion::under_pointer(over);
        if expansion.is_none() {
            own.set(next);
        }
        if next != now
            && let Some(listen) = on_expansion_change
        {
            listen.call(next);
        }
    };
    let Some(drawn) = presence.drawn_slug() else {
        return rsx! {};
    };
    let class = common.class("ds-spaces-bar");
    let data = common.data_attributes();
    let name = common.aria_label.clone().unwrap_or(label);
    rsx! {
        div {
            id: common.id.clone(),
            class,
            role: "group",
            "aria-label": "{name}",
            "data-expansion": now.slug(),
            "data-presence": drawn,
            "data-pulse": alias.slug(),
            onmounted: move |event| common.mounted(event),
            onpointerenter: move |_| ask(PointerOver::Bar),
            onpointerleave: move |_| ask(PointerOver::Elsewhere),
            ..data,
            div { class: "ds-spaces-bar-side" }
            div { class: "ds-spaces-bar-row",
                for tile in spaces {
                    Tile { key: "{tile.key.0}", tile, expansion: now, on_pick }
                }
            }
            div { class: "ds-spaces-bar-side",
                if let Some(on_add) = on_add {
                    span { class: "ds-spaces-bar-add",
                        Button {
                            label: add_label,
                            bezel: Bezel::Toolbar,
                            image: ImagePosition::Only,
                            icon: Some(IconSource::from(Icon::Plus)),
                            onclick: move |press: Press| {
                                if press.button == PointerButton::Primary {
                                    on_add.call(());
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

/// One Space: its thumbnail while the bar is expanded, and its name.
#[component]
fn Tile(tile: SpaceTile, expansion: BarExpansion, on_pick: EventHandler<BarSpaceKey>) -> Element {
    let SpaceTile {
        key,
        name,
        current,
        thumbnail,
        drop,
    } = tile;
    let listen = PressListeners::new(EventHandler::new(move |press: Press| {
        if press.button == PointerButton::Primary {
            on_pick.call(key.clone());
        }
    }));
    rsx! {
        button {
            r#type: "button",
            class: "ds-spaces-bar-space",
            "data-selected": current.slug(),
            "data-drop": drop.drop_attr(),
            "data-drag": drop.drag_attr(),
            "aria-current": current.aria_current(),
            onclick: move |event| listen.click(&event),
            onkeydown: move |event| listen.key_down(&event, ActivationKeys::ReturnAndSpace),
            oncontextmenu: move |event| listen.context_menu(&event),
            if expansion == BarExpansion::Expanded {
                span { class: "ds-spaces-bar-thumb",
                    if let Some(picture) = thumbnail {
                        img { class: "ds-spaces-bar-picture", alt: "", src: picture.0, draggable: "false" }
                    }
                }
            }
            span { class: "ds-spaces-bar-name", "{name}" }
        }
    }
}
