//! The traffic lights (design/04-COMPONENTS.md "Window frame"): close, minimize and zoom, 12 px
//! discs 8 px apart at the titlebar's start. Coloured at rest in the active window, grey in an
//! inactive one, and their marks appear while the pointer is over the group, as on macOS. The
//! green one zooms on a click; held for `menu_press` or rested on for `menu_hover` (or
//! right-clicked, or ArrowDown with the keyboard on it) it opens the tiling menu, whose
//! placements the host cannot make are shown unavailable. Each light keeps its press: a press on
//! a light never starts the titlebar's move, and a double-click on one never zooms.

use crate::components::light_mark::{LightMark, Mark};
use crate::components::menu::{Menu, MenuKind};
use crate::components::menu_entry::{MenuEntry, Tile, Trail};
use crate::components::vocab::{Availability, Expanded};
use crate::geometry::{Anchor, MountedRef};
use crate::icon::Icon;
use crate::time::sleep;
use crate::window::hold::{Click, Hold, Opens, Waiting};
use crate::window::{
    FrameTiming, Maximized, Support, WindowHost, WindowTile, Zoom, use_window_host,
    use_window_state,
};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use std::time::Duration;

/// Whether the tiling menu is open as the lights mount: `Open` poses it (a gallery, a picture).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TilePose {
    /// Closed until asked for.
    #[default]
    Closed,
    /// Open from the first frame.
    Open,
}

/// The tiling menu: closed, or open with what the host said it can do, asked once as it opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TileMenu {
    Closed,
    Open([Support; 4]),
}

/// The three lights, and the tiling menu the green one opens.
#[component]
pub(crate) fn TrafficLightGroup(timing: FrameTiming, pose: TilePose) -> Element {
    let host = use_window_host();
    let state = use_window_state();
    let mut menu = use_signal(|| match pose {
        TilePose::Open => TileMenu::Open(support(host.as_ref())),
        TilePose::Closed => TileMenu::Closed,
    });
    let mut anchor = use_signal(|| None::<MountedRef>);
    let hold = use_hook(|| CopyValue::new(Hold::default()));
    let opener = Opener {
        menu,
        host: host.clone(),
    };
    let (close, minimize, zoom, pick) = (host.clone(), host.clone(), host.clone(), host);
    let zoom_mark = match state.maximized {
        Maximized::On => Mark::Restore,
        Maximized::Off => Mark::Zoom,
    };
    let expanded = match menu() {
        TileMenu::Open(_) => Expanded::Open,
        TileMenu::Closed => Expanded::Closed,
    };
    rsx! {
        div { class: "ds-lights", role: "group", "aria-label": "Window",
            Light { mark: Mark::Close, label: "Close",
                onclick: move |()| with(close.as_ref(), |host| host.host().close()),
            }
            Light { mark: Mark::Minimize, label: "Minimize",
                onclick: move |()| with(minimize.as_ref(), |host| host.host().minimize()),
            }
            button {
                r#type: "button",
                class: "ds-light",
                "data-light": "zoom",
                "data-first-mouse": "",
                "aria-label": zoom_mark.label(),
                "aria-haspopup": "menu",
                "aria-expanded": expanded.aria(),
                onmounted: move |event: MountedEvent| anchor.set(Some(MountedRef(event.data()))),
                onpointerdown: {
                    let opener = opener.clone();
                    move |event: PointerEvent| {
                        event.stop_propagation();
                        if event.trigger_button() == Some(MouseButton::Primary) {
                            after(hold, Waiting::Press, timing.menu_press, opener.clone());
                        }
                    }
                },
                onpointerup: move |_| cancel(hold),
                onpointerenter: {
                    let opener = opener.clone();
                    move |_| after(hold, Waiting::Hover, timing.menu_hover, opener.clone())
                },
                onpointerleave: move |_| cancel(hold),
                ondoubleclick: move |event: MouseEvent| event.stop_propagation(),
                oncontextmenu: {
                    let opener = opener.clone();
                    move |event: MouseEvent| {
                        event.prevent_default();
                        opener.open();
                    }
                },
                onkeydown: move |event: KeyboardEvent| {
                    if event.key() == Key::ArrowDown {
                        event.prevent_default();
                        opener.open();
                    }
                },
                onclick: move |_| {
                    let mut hold = hold;
                    let (next, click) = hold.peek().click();
                    hold.set(next);
                    if click == Click::Zoom {
                        with(zoom.as_ref(), |host| host.host().zoom(Zoom::Toggle));
                    }
                },
                LightMark { mark: zoom_mark }
            }
        }
        if let (TileMenu::Open(support), Some(anchor)) = (menu(), anchor()) {
            Menu::<WindowTile> {
                kind: MenuKind::Slim,
                anchor: Anchor::Mounted(anchor),
                entries: entries(support),
                onpick: move |tile| with(pick.as_ref(), |host| place(host, tile)),
                onclose: move |()| menu.set(TileMenu::Closed),
            }
        }
    }
}

/// A light that only acts on a click: close and minimize.
#[component]
fn Light(mark: Mark, label: &'static str, onclick: EventHandler<()>) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "ds-light",
            "data-light": mark.slug(),
            "data-first-mouse": "",
            "aria-label": label,
            onpointerdown: move |event: PointerEvent| event.stop_propagation(),
            ondoubleclick: move |event: MouseEvent| event.stop_propagation(),
            onclick: move |_| onclick.call(()),
            LightMark { mark }
        }
    }
}

/// Opens the tiling menu, asking the host once what it can place.
#[derive(Clone)]
struct Opener {
    menu: Signal<TileMenu>,
    host: Option<WindowHost>,
}

impl Opener {
    fn open(&self) {
        let mut menu = self.menu;
        menu.set(TileMenu::Open(support(self.host.as_ref())));
    }
}

/// Run `act` with the host, if there is one.
fn with(host: Option<&WindowHost>, act: impl FnOnce(&WindowHost)) {
    if let Some(host) = host {
        act(host);
    }
}

/// Fill is the zoom's maximize; the others are the host's placements.
fn place(host: &WindowHost, tile: WindowTile) {
    match tile {
        WindowTile::Fill => host.host().zoom(Zoom::Maximize),
        WindowTile::LeftHalf | WindowTile::RightHalf | WindowTile::Centre => {
            // The row was unavailable when the host said so: an error here is the host
            // changing its mind between the menu opening and the pick, and nothing is placed.
            let _ = host.host().tile(tile);
        }
    }
}

/// What the host can do, per placement in the menu's order; nothing without a host.
fn support(host: Option<&WindowHost>) -> [Support; 4] {
    WindowTile::ALL.map(|tile| host.map_or(Support::No, |host| host.host().supports(tile)))
}

/// Start waiting for `what`, and open the menu after `delay` unless the wait went stale.
fn after(mut hold: CopyValue<Hold>, what: Waiting, delay: Duration, opener: Opener) {
    let (next, ticket) = hold.peek().wait(what);
    hold.set(next);
    spawn(async move {
        sleep(delay).await;
        let (next, opens) = hold.peek().fire(ticket);
        hold.set(next);
        if opens == Opens::Yes {
            opener.open();
        }
    });
}

/// Stop waiting.
fn cancel(mut hold: CopyValue<Hold>) {
    let next = hold.peek().cancel();
    hold.set(next);
}

/// The menu's rows: a header, then each placement with its glyph, unavailable where the host
/// said it cannot.
fn entries(support: [Support; 4]) -> Vec<MenuEntry<WindowTile>> {
    let rows = WindowTile::ALL
        .into_iter()
        .zip(support)
        .map(|(tile, support)| MenuEntry::Item {
            value: tile,
            title: tile.title().to_owned(),
            detail: None,
            tile: Some(Tile::Icon(glyph(tile))),
            trail: Trail::None,
            check: None,
            availability: match support {
                Support::Yes => Availability::Enabled,
                Support::No => Availability::Disabled,
            },
        });
    std::iter::once(MenuEntry::Header("Move & Resize".to_owned()))
        .chain(rows)
        .collect()
}

/// Each placement's glyph.
fn glyph(tile: WindowTile) -> Icon {
    match tile {
        WindowTile::Fill => Icon::Maximize,
        WindowTile::LeftHalf => Icon::PanelLeft,
        WindowTile::RightHalf => Icon::Panel,
        WindowTile::Centre => Icon::Square,
    }
}
