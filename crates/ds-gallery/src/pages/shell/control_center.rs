//! The Overlays page's control center: a Popover panel of the Work
//! Space, tinted over the wallpaper, in light and dark. Its root pane is a `ModuleGrid` of
//! `ModuleTile`s (Wi-Fi on, Bluetooth off, both with a chevron; Focus; a busy Hotspot; a
//! full-span Now Playing), then the Sound module and the compact Appearance picker on
//! `ModulePanel`s; a chevron pushes the module's detail, a `SettingsRow` list, through
//! the `PaneSwitcher`, and the back button returns. Posed, the dark panel shows the detail.

use crate::axes::{Axes, Showcase};
use crate::pages::Section;
use crate::wallpaper;
use dioxus::prelude::*;
use ds::Availability;
use ds::Bezel;
use ds::{
    Accessory, Appearance, Button, CardAccent, Check, Ds, FrameTint, Grain, Icon, Inject, List,
    ListItem, Material, Pane, PaneSwitcher, RootChrome, Row, RowLeading, RowSize, TextLine, Theme,
    default_look,
};
use ds::{AppearancePicker, Fraction, LevelGlyph, Muting, PickerLayout, Px, SystemPrefs};
use ds::{Slider, SliderLook};
use ds_shell::ModulePanel;
use ds_shell::{ModuleGrid, ModuleTile, TileSpan};

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
        Section { title: "Control center", note: "A Popover panel of the Work Space over the wallpaper, light and dark: a ModuleGrid (2 columns, gap 10) of 56 px ModuleTiles (--cc-module-r 8, inside the panel's 18 at padding 10; On paints the disc --accent on the same plate (the tile does not tint), Off a paper disc, Busy breathes), a Full tile spanning both columns, then ModulePanels on the tile's frame holding the Sound level, a disabled Display level (the capsule and its header glyph and figure at .35, no knob, the not-allowed cursor), (glyph, title, its percentage in the trailing slot; a 22 capsule with a 20 knob, the module 64 tall) and the compact AppearancePicker. A tile toggles; its chevron (its own hit target, Enter or Right) pushes the module's detail through the PaneSwitcher: slide-r in, the grid out to the left, both at --t-move, the height following the pane. The detail lists SettingsRows (44 px, hairlines, the text menu's type); the back button slides it out to the right.",
            div { class: "g-wall g-chrome-cards", style: "background-image:url(\"{wallpaper::uri()}\")",
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
    let mut wifi = use_signal(|| Check::On);
    let mut bluetooth = use_signal(|| Check::Off);
    let mut focus = use_signal(|| Check::Off);
    let mut volume = use_signal(|| Fraction(400));
    let mut appearance = use_signal(Appearance::default);
    let percent = volume().0 / 10;
    rsx! {
        // The panel's body pads both panes, so the grid adds none of its own.
        ModuleGrid { padding: Px(0.0),
            ModuleTile {
                glyph: Icon::Wifi, title: "Wi-Fi", status: "Home", value: wifi(),
                onclick: move |_| wifi.set(wifi().flipped()), on_detail: move |_| on_open.call(Module::WiFi),
            }
            ModuleTile {
                glyph: Icon::Bluetooth, title: "Bluetooth", status: "Off", value: bluetooth(),
                onclick: move |_| bluetooth.set(bluetooth().flipped()), on_detail: move |_| on_open.call(Module::Bluetooth),
            }
            ModuleTile { glyph: Icon::Moon, title: "Focus", status: "Do Not Disturb", value: focus(), onclick: move |_| focus.set(focus().flipped()) }
            ModuleTile { glyph: Icon::Link, title: "Hotspot", status: "Connecting…", value: Check::Off, availability: Availability::Busy, onclick: |_| {} }
            ModuleTile { glyph: Icon::Play, title: "Nocturne in E-flat", status: "Paused", value: Check::Off, span: TileSpan::Full, onclick: |_| {} }
            ModulePanel { glyph: Icon::Volume2, title: "Speakers", trailing: rsx! { "{percent}%" },
                Slider { label: "Volume", value: volume(), glyph: LevelGlyph::Volume(Muting::Audible), look: SliderLook::CapsuleKnob, onchange: move |next| volume.set(next) }
            }
            // A level that cannot move (no brightness control on this display) says so.
            ModulePanel { glyph: Icon::Sun, title: "Display", trailing: rsx! { "0%" }, availability: ds::Availability::Disabled,
                Slider { label: "Brightness", value: Fraction(0), glyph: LevelGlyph::Brightness, look: SliderLook::CapsuleKnob, availability: ds::Availability::Disabled }
            }
            ModulePanel {
                AppearancePicker { value: appearance(), system: SystemPrefs::default(), onchange: move |next| appearance.set(next), layout: PickerLayout::Compact }
            }
        }
    }
}

/// The detail pane: a back button and the module's list.
#[component]
fn Detail(module: Module, on_back: EventHandler<ds::Press>) -> Element {
    let mut chosen = use_signal(|| 0usize);
    let mut headphones = use_signal(|| Check::On);
    let check = move |index: usize| {
        Accessory::Check(if chosen() == index {
            Check::On
        } else {
            Check::Off
        })
    };
    let item = |title: &'static str, row: Element| ListItem::row(title, title, row);
    let list = match module {
        Module::WiFi => rsx! {
            List::<&'static str> {
                label: "Networks",
                items: vec![
                    item("Home", rsx! { Row { leading: RowLeading::Icon(Icon::Wifi), title: "Home", detail: TextLine::from("Connected"), accessory: check(0), size: RowSize::Settings, onclick: move |_| chosen.set(0) } }),
                    item("Studio 5G", rsx! { Row { leading: RowLeading::Icon(Icon::WifiHigh), title: "Studio 5G", detail: TextLine::from("Secured"), accessory: check(1), size: RowSize::Settings, onclick: move |_| chosen.set(1) } }),
                    item("Café Guest", rsx! { Row { leading: RowLeading::Icon(Icon::WifiLow), title: TextLine::from("Café Guest"), accessory: check(2), size: RowSize::Settings, onclick: move |_| chosen.set(2) } }),
                ],
            }
        },
        Module::Bluetooth => rsx! {
            List::<&'static str> {
                label: "Devices",
                items: vec![
                    item("Headphones", rsx! {
                        Row {
                            leading: RowLeading::Icon(Icon::Headphones), title: "Headphones", detail: TextLine::from("Battery 84%"),
                            accessory: Accessory::Toggle { value: headphones(), on_toggle: EventHandler::new(move |next| headphones.set(next)) },
                            size: RowSize::Settings,
                        }
                    }),
                    item("Mouse", rsx! { Row { leading: RowLeading::Icon(Icon::Mouse), title: "Mouse", detail: TextLine::from("Not connected"), accessory: Accessory::Chevron, size: RowSize::Settings } }),
                    item("Phone", rsx! { Row { leading: RowLeading::Icon(Icon::Phone), title: "Phone", accessory: Accessory::Text("Paired".to_string()), size: RowSize::Settings } }),
                ],
            }
        },
    };
    rsx! {
        div { class: "g-col",
            div { class: "g-row",
                Button { bezel: Bezel::Inline, label: module.title(), icon: Some(Icon::ChevronLeft), onclick: on_back }
            }
            div { {list} }
        }
    }
}
