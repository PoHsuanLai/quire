//! The Overlays page's control center (sill FINDINGS Q78-Q80): a Popover panel of the Work
//! Space, tinted over the wallpaper, in light and dark. Its root pane is a `ModuleGrid` of
//! `ModuleTile`s (Wi-Fi on, Bluetooth off, both with a chevron; Focus; a busy Hotspot; a
//! full-span Now Playing); a chevron pushes the module's detail, a `SettingsRow` list, through
//! the `PaneSwitcher`, and the back button returns. Posed, the dark panel shows the detail.

use super::Section;
use crate::axes::{Axes, Showcase};
use crate::wallpaper;
use dioxus::prelude::*;
use ds::{
    Appearance, Button, ButtonVariant, CardAccent, Chevron, Ds, FrameTint, Grain, Icon, Inject,
    Material, ModuleGrid, ModuleState, ModuleTile, Pane, PaneSwitcher, RootChrome, RowTrailing,
    SettingsRow, Switch, Text, Theme, TileSpan, default_look,
};

/// The module whose detail a chevron opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Module {
    WiFi,
    Bluetooth,
}

impl Module {
    fn title(self) -> &'static str {
        match self {
            Module::WiFi => "Wi-Fi",
            Module::Bluetooth => "Bluetooth",
        }
    }
}

/// The two schemes, and the pane each opens on when posed.
const PANELS: [(Theme, Pane); 2] = [(Theme::Light, Pane::Root), (Theme::Dark, Pane::Detail)];

/// The control center section.
#[component]
pub fn ControlCenter() -> Element {
    rsx! {
        Section { title: "Control center", note: "A Popover panel of the Work Space over the wallpaper, light and dark: a ModuleGrid (2 columns, gap 8) of ModuleTiles (--r-tile 12; On paints the disc --accent on an --accent-soft plate, Off a paper disc, Busy breathes), a Full tile spanning both columns. A tile toggles; its chevron (its own hit target, Enter or Right) pushes the module's detail through the PaneSwitcher: slide-r in, the grid out to the left, both at --t-move, the height following the pane. The detail lists SettingsRows (44 px, hairlines, the text menu's type); the back button slides it out to the right.",
            div { class: "g-wall g-polish-cards", style: "background-image:url(\"{wallpaper::uri()}\")",
                for (theme , posed) in PANELS {
                    Panel { theme, posed }
                }
            }
        }
    }
}

/// One control center in `theme`, opening on `posed` in a snapshot.
#[component]
fn Panel(theme: Theme, posed: Pane) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (accent, motion, blur, showcase) = {
        let axes = axes.read();
        (axes.accent, axes.motion, axes.blur, axes.showcase)
    };
    let start = match showcase {
        Showcase::Posed => posed,
        Showcase::Live => Pane::Root,
    };
    let mut shown = use_signal(|| start);
    let mut module = use_signal(|| Module::WiFi);
    let open = move |which: Module| {
        module.set(which);
        shown.set(Pane::Detail);
    };
    rsx! {
        div { class: "g-cc",
            Ds {
                appearance: Appearance { theme, accent, motion },
                look: ds::SpaceLook { theme, ..default_look(0, Grain(35), CardAccent::SpaceHue) },
                material: Material::Popover,
                blur,
                stylesheet: Inject::Host,
                chrome: Some(RootChrome::Painted),
                frame: Some(FrameTint::Tinted),
                div { class: "g-cc-body",
                    PaneSwitcher {
                        shown: shown(),
                        root: rsx! { Modules { on_open: open } },
                        detail: rsx! { Detail { module: module(), on_back: move |_| shown.set(Pane::Root) } },
                    }
                }
            }
        }
    }
}

/// The root pane: the module tiles.
#[component]
fn Modules(on_open: EventHandler<Module>) -> Element {
    let mut wifi = use_signal(|| ModuleState::On);
    let mut bluetooth = use_signal(|| ModuleState::Off);
    let mut focus = use_signal(|| ModuleState::Off);
    let flip = |state: ModuleState| match state {
        ModuleState::On => ModuleState::Off,
        ModuleState::Off | ModuleState::Busy => ModuleState::On,
    };
    rsx! {
        ModuleGrid {
            ModuleTile {
                glyph: Icon::Wifi, title: "Wi-Fi", status: "Home", state: wifi(), chevron: Chevron::Detail,
                onclick: move |_| wifi.set(flip(wifi())), on_detail: move |_| on_open.call(Module::WiFi),
            }
            ModuleTile {
                glyph: Icon::Bluetooth, title: "Bluetooth", status: "Off", state: bluetooth(), chevron: Chevron::Detail,
                onclick: move |_| bluetooth.set(flip(bluetooth())), on_detail: move |_| on_open.call(Module::Bluetooth),
            }
            ModuleTile { glyph: Icon::Moon, title: "Focus", status: "Do Not Disturb", state: focus(), onclick: move |_| focus.set(flip(focus())) }
            ModuleTile { glyph: Icon::Link, title: "Hotspot", status: "Connecting…", state: ModuleState::Busy, onclick: |_| {} }
            ModuleTile { glyph: Icon::Play, title: "Nocturne in E-flat", status: "Paused", state: ModuleState::Off, span: TileSpan::Full, onclick: |_| {} }
        }
    }
}

/// The detail pane: a back button and the module's list.
#[component]
fn Detail(module: Module, on_back: EventHandler<ds::Press>) -> Element {
    let mut chosen = use_signal(|| 0usize);
    let mut headphones = use_signal(|| Switch::On);
    let check = move |index: usize| {
        RowTrailing::Check(if chosen() == index {
            Switch::On
        } else {
            Switch::Off
        })
    };
    let list = match module {
        Module::WiFi => rsx! {
            SettingsRow { glyph: Icon::Wifi, title: "Home", detail: "Connected", trailing: check(0), onclick: move |_| chosen.set(0) }
            SettingsRow { glyph: Icon::WifiHigh, title: "Studio 5G", detail: "Secured", trailing: check(1), onclick: move |_| chosen.set(1) }
            SettingsRow { glyph: Icon::WifiLow, title: Text::from("Café Guest"), trailing: check(2), onclick: move |_| chosen.set(2) }
        },
        Module::Bluetooth => rsx! {
            SettingsRow {
                glyph: Icon::Headphones, title: "Headphones", detail: "Battery 84%",
                trailing: RowTrailing::Toggle { value: headphones(), on_toggle: EventHandler::new(move |next| headphones.set(next)) },
                onclick: |_| {},
            }
            SettingsRow { glyph: Icon::Mouse, title: "Mouse", detail: "Not connected", trailing: RowTrailing::Chevron, onclick: |_| {} }
            SettingsRow { glyph: Icon::Phone, title: "Phone", trailing: RowTrailing::Text(Text::from("Paired")), onclick: |_| {} }
        },
    };
    rsx! {
        div { class: "g-col",
            div { class: "g-row",
                Button { variant: ButtonVariant::Quiet, label: module.title(), icon: Some(Icon::ChevronLeft), onclick: on_back }
            }
            div { {list} }
        }
    }
}
