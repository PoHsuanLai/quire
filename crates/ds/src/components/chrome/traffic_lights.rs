//! The traffic lights (design/04-COMPONENTS.md "Window frame"): close, minimize and zoom, 12 px
//! discs 8 px apart at the titlebar's start. Coloured at rest in the active window, grey in an
//! inactive one, and their marks appear while the pointer is over the group, as on macOS. The
//! green one zooms on a click, and always with Option held; held for `menu_press` or rested on for `menu_hover` (or
//! right-clicked, or ArrowDown with the keyboard on it) it opens the tiling menu, whose
//! placements the host cannot make are shown unavailable. Each light keeps its press: a press on
//! a light never starts the titlebar's move, and a double-click on one never zooms.

use crate::components::chrome::light_mark::{LightMark, Mark};
use crate::components::menus::item::item::{AfterPick, MenuImage, MenuItem};
use crate::components::menus::menu::menu::Menu;
use crate::components::menus::menu::placement::MenuPlacement;
use crate::host::measure::{Anchor, MountedRef};
use crate::window::hold::{Hold, HoldIn, HoldOut, HoldTiming, Wait};
use crate::window::host::{WindowHost, use_window_host, use_window_state};
use crate::window::timing::FrameTiming;
use crate::window::vocab::{Maximized, Support, WindowTile, Zoom};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use ds_core::vocab::{Availability, Shown};
use ds_core::word::Word;
use ds_motion::machine::use_machine;
use ds_style::icon::Icon;

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
    let opener = Opener {
        menu,
        host: host.clone(),
    };
    let hold = use_machine(
        |_| Hold::default(),
        HoldTiming {
            press: timing.menu_press,
            hover: timing.menu_hover,
        },
        || (),
        {
            let (opener, zoom) = (opener.clone(), host.clone());
            move |out, _| match out {
                HoldOut::OpenMenu => opener.open(),
                HoldOut::ToggleZoom => with(zoom.as_ref(), |host| host.host().zoom(Zoom::Toggle)),
            }
        },
    );
    let (close, minimize, pick) = (host.clone(), host.clone(), host);
    let zoom_mark = match state.maximized {
        Maximized::On => Mark::Restore,
        Maximized::Off => Mark::Zoom,
    };
    let expanded = match menu() {
        TileMenu::Open(_) => Shown::Visible,
        TileMenu::Closed => Shown::Hidden,
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
                onpointerdown: move |event: PointerEvent| {
                    event.stop_propagation();
                    if event.trigger_button() == Some(MouseButton::Primary) && !zooms(&event) {
                        hold.send(HoldIn::Wait(Wait::Press));
                    }
                },
                onpointerup: move |_| hold.send(HoldIn::Cancel),
                onpointerenter: move |event: PointerEvent| {
                    if !zooms(&event) {
                        hold.send(HoldIn::Wait(Wait::Hover));
                    }
                },
                onpointerleave: move |_| hold.send(HoldIn::Cancel),
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
                onclick: move |_| hold.send(HoldIn::Click),
                LightMark { mark: zoom_mark }
            }
        }
        if let (TileMenu::Open(support), Some(anchor)) = (menu(), anchor()) {
            Menu::<WindowTile> {
                placement: MenuPlacement::Popup,
                anchor: Anchor::Mounted(anchor),
                items: entries(support),
                onpick: move |tile| with(pick.as_ref(), |host| place(host, tile)),
                onclose: move |()| menu.set(TileMenu::Closed),
            }
        }
    }
}

/// Whether Option is held: the green light then only zooms, and neither a hold nor a rest on it
/// opens the tiling menu (design/30 section 2.7).
fn zooms(event: &PointerEvent) -> bool {
    event.modifiers().contains(Modifiers::ALT)
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
    std::array::from_fn(|index| {
        host.map_or(Support::No, |host| {
            host.host().supports(WindowTile::ALL[index])
        })
    })
}

/// The menu's rows: a header, then each placement with its glyph, unavailable where the host
/// said it cannot.
fn entries(support: [Support; 4]) -> Vec<MenuItem<WindowTile>> {
    let rows = WindowTile::ALL
        .iter()
        .copied()
        .zip(support)
        .map(|(tile, support)| MenuItem::Item {
            value: tile,
            title: tile.label().to_owned(),
            image: Some(MenuImage::Icon(glyph(tile))),
            key: None,
            hint: None,
            check: None,
            after: AfterPick::Close,
            text: Default::default(),
            availability: match support {
                Support::Yes => Availability::Enabled,
                Support::No => Availability::Disabled,
            },
        });
    std::iter::once(MenuItem::Header("Move & Resize".to_owned()))
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
