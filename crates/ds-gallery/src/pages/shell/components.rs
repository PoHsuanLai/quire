//! The Shell page (design/30 section 2.10, step 4a.7): the shell-only components rebuilt on the
//! survivors, each in every state it can express: `MenuBarItem` (title and glyph, plain, strong,
//! open, pressed, disabled, toggled), `WorkspacePills`, `ModuleTile` (off, on, busy, disabled,
//! with and without the chevron, open), `BatteryRing` (tone, power, readout), `DockTile` (side,
//! running, badge, progress, label, tint), `WidgetFrame` (size, host, lift, shown) and
//! `AnimatedEmoji` (once through, still).

use crate::pages::shell::chrome_targets::Root;
use crate::pages::{Section, Specimen};
use crate::wallpaper;
use dioxus::prelude::*;
use ds::components::content::status::battery_state::{BatteryPower, BatteryState, LowAt};
use ds::components::controls::badge::BadgeContent;
use ds::components::controls::button_model::ImagePosition;
use ds::prelude::*;
use ds::root::common::Common;
use ds_core::vocab::Activity;
use ds_shell::battery::device_glyph::Device;
use ds_shell::battery::ring::Readout;
use ds_shell::control_center::module_panel::PanelPlate;
use ds_shell::control_center::module_tile_kind::TileSpan;
use ds_shell::emoji::disc::EmojiPlayback;
use ds_shell::emoji::id::EmojiId;
use ds_shell::prelude::*;
use ds_shell::user_picture::size::PictureSize;
use ds_shell::widget::kind::{CardTint, Lift, WidgetTitle};
use ds_style::icon::family::PlateFamily;
use ds_style::icon::plate_tint::PlateTint;

/// The page.
#[component]
pub fn ShellPage() -> Element {
    rsx! {
        MenuBarItems {}
        ModuleTiles {}
        BatteryRings {}
        DockTiles {}
        WidgetFrames {}
        Emoji {}
    }
}

/// `MenuBarItem` and `WorkspacePills` on a Bar root over the wallpaper.
#[component]
fn MenuBarItems() -> Element {
    rsx! {
        Section { title: "MenuBarItem and WorkspacePills", note: "MenuBarItem is the bar's item button, `NSStatusItem`'s or a menu's title: a title at the shell scale (13/500, the app name Strong at 700) or a glyph in the bar's status slot, on a 22 pt pill: `--surface` under the pointer, `--surface-2` while pressed or while its menu is open (`shown`), .35 when disabled, and `value` makes it a toggle. Left to right: plain, Strong, open, toggled on, disabled; then the glyph items: idle, open, toggled on, disabled. WorkspacePills is Mission Control's Spaces bar: one raised segment for the selected workspace (`data-selected`), the rest quiet.",
            div { class: "g-wall g-chrome-wall", style: "background-image:url(\"{wallpaper::uri()}\")",
                Root { material: Material::Bar, style: "width:100%",
                    div { class: "g-chrome-bar",
                        WorkspacePills { label: "Workspaces",
                            WorkspacePill { label: "1", current: Selection::Selected, onclick: |_| {} }
                            WorkspacePill { label: "2", onclick: |_| {} }
                            WorkspacePill { label: "3", onclick: |_| {} }
                        }
                        MenuBarItem { label: "Files", emphasis: Emphasis::Strong, onclick: |_| {} }
                        MenuBarItem { label: "File", onclick: |_| {} }
                        MenuBarItem { label: "Edit", shown: Shown::Visible, onclick: |_| {} }
                        MenuBarItem { label: "View", value: Some(Check::On), onclick: |_| {} }
                        MenuBarItem { label: "Help", availability: Availability::Disabled, onclick: |_| {} }
                        span { class: "g-spacer" }
                        MenuBarItem { image: ImagePosition::Only, icon: Icon::Wifi, label: "Wi-Fi", onclick: |_| {} }
                        MenuBarItem { image: ImagePosition::Only, icon: Icon::Volume2, label: "Volume", shown: Shown::Visible, onclick: |_| {} }
                        MenuBarItem { image: ImagePosition::Only, icon: Icon::Bluetooth, label: "Bluetooth", value: Some(Check::On), onclick: |_| {} }
                        MenuBarItem { image: ImagePosition::Only, icon: Icon::BatteryFull, label: "Battery", availability: Availability::Disabled, onclick: |_| {} }
                    }
                }
            }
        }
    }
}

/// `ModuleGrid`, `ModuleTile` and `ModulePanel` in every state.
#[component]
fn ModuleTiles() -> Element {
    rsx! {
        Section { title: "ModuleGrid, ModuleTile and ModulePanel", note: "Control Center modules. A tile is a toggle (`value: Check`) with its `Availability`: Enabled, Disabled (.35, no press) and Busy (the disc turns its progress ring, `aria-pressed` reads mixed, no press). `on_detail` draws the chevron, its own hit target; `expanded` is its `aria-expanded`. A Full tile spans the grid; a ModulePanel holds content of its own on the tile's frame (Tile plate or Bare). The tile does not lift under the pointer.",
            Root { material: Material::Popover, style: "width:344px",
                ModuleGrid {
                    ModuleTile { glyph: Icon::Wifi, title: "Wi-Fi", status: "Home", value: Check::On, on_detail: |_| {}, onclick: |_| {} }
                    ModuleTile { glyph: Icon::Bluetooth, title: "Bluetooth", status: "Off", value: Check::Off, on_detail: |_| {}, onclick: |_| {} }
                    ModuleTile { glyph: Icon::Moon, title: "Focus", status: "Do Not Disturb", value: Check::On, onclick: |_| {} }
                    ModuleTile { glyph: Icon::Link, title: "Hotspot", status: "Connecting…", value: Check::Off, availability: Availability::Busy, onclick: |_| {} }
                    ModuleTile { glyph: Icon::Moon, title: "Focus off-limits", status: "Unavailable", value: Check::Off, availability: Availability::Disabled, on_detail: |_| {}, onclick: |_| {} }
                    ModuleTile { glyph: Icon::Wifi, title: "Wi-Fi details open", status: "Home", value: Check::On, on_detail: |_| {}, expanded: Shown::Visible, onclick: |_| {} }
                    ModuleTile { glyph: Icon::Play, title: "Nocturne in E-flat", status: "Paused", value: Check::Off, span: TileSpan::Full, onclick: |_| {} }
                    ModulePanel { glyph: Icon::Volume2, title: "Panel, Tile plate", trailing: rsx! { "40%" },
                        span { class: "g-name", "content of its own" }
                    }
                    ModulePanel { glyph: Icon::Volume2, title: "Panel, Bare", trailing: rsx! { "40%" }, plate: PanelPlate::Bare, availability: Availability::Disabled,
                        span { class: "g-name", "disabled, no plate: the glyph and figure dim, the title names it" }
                    }
                }
            }
        }
    }
}

/// One battery specimen: its caption and state.
fn cell(name: &str, level: u16, power: BatteryPower, device: Device, readout: Readout) -> Element {
    let state = BatteryState {
        level: Fraction(level),
        power,
        low_at: LowAt::default(),
    };
    rsx! {
        Specimen { name: name.to_string(),
            BatteryRing { state, label: name.to_string(), readout,
                DeviceGlyph { device, size: IconSize::Base }
            }
        }
    }
}

/// `BatteryRing`: one arc geometry, one `BatteryState`.
#[component]
fn BatteryRings() -> Element {
    rsx! {
        Section { title: "BatteryRing", note: "A `ProgressIndicator` ring with the device's filled glyph in the middle, drawn from one `BatteryState` (level, power, where low begins). Draining at or under a fifth reads low (`--battery-low`); charging notches the ring at twelve for the bolt and is never low; Held is plugged in and draws as a normal battery. `Readout::Under` puts the percentage under the ring in a Label.",
            Root { material: Material::Widget, chrome: Some(RootChrome::Transparent),
                div { class: "g-row g-row-top",
                    {cell("Draining 82%", 820, BatteryPower::Battery, Device::Laptop, Readout::Under)}
                    {cell("Low 15%", 150, BatteryPower::Battery, Device::Phone, Readout::Under)}
                    {cell("Charging 45%", 450, BatteryPower::Charging, Device::Headphones, Readout::Under)}
                    {cell("Held 100%", 1000, BatteryPower::Held, Device::Mouse, Readout::Under)}
                    {cell("Empty", 0, BatteryPower::Battery, Device::Keyboard, Readout::Alone)}
                }
            }
        }
    }
}

/// `DockTile` at its rest and magnified sides, with each accessory.
#[component]
fn DockTiles() -> Element {
    let files = IconSource::Glyph(Icon::Folder);
    rsx! {
        Section { title: "DockTile", note: "One Dock tile: the plated icon (80.5 % of the tile's side), a Badge (count or dot), a `ProgressIndicator` bar while the app works, the running dot on the baseline and the DockLabel with the name. Magnification and the bounce are sill's: it hands the tile its `side`. Left to right: at rest (48), magnified (72 and 96), running with a count and its label shown, working (40%), and a muted dock with a dot.",
            Root { material: Material::Dock, style: "width:100%",
                div { class: "g-row g-row-top g-dock-tiles",
                    DockTile { icon: files.clone(), plate: Some(PlateFamily::Blue), label: "Files", onclick: |_| {} }
                    DockTile { icon: files.clone(), plate: Some(PlateFamily::Blue), side: Px(72.0), label: "Files", onclick: |_| {} }
                    DockTile { icon: files.clone(), plate: Some(PlateFamily::Blue), side: Px(96.0), label: "Files", onclick: |_| {} }
                    DockTile {
                        icon: IconSource::Glyph(Icon::Mail),
                        plate: Some(PlateFamily::Blue),
                        running: Activity::Active,
                        badge: Some(BadgeContent::Number(3)),
                        label: "Mail",
                        label_shown: Some(Shown::Visible),
                        onclick: |_| {},
                    }
                    DockTile {
                        icon: IconSource::Glyph(Icon::Download),
                        plate: Some(PlateFamily::Green),
                        progress: Some(Fraction(400)),
                        label: "Downloads",
                        onclick: |_| {},
                    }
                    DockTile {
                        icon: files,
                        plate: Some(PlateFamily::Neutral),
                        plate_tint: Some(PlateTint::Muted),
                        badge: Some(BadgeContent::Dot),
                        running: Activity::Active,
                        label: "Files",
                        onclick: |_| {},
                    }
                }
            }
        }
    }
}

/// `WidgetFrame` at each size, in each host, lifted, and hidden.
#[component]
fn WidgetFrames() -> Element {
    let title = || Some(WidgetTitle::new(Icon::Clock, "Widget"));
    rsx! {
        Section { title: "WidgetFrame", note: "The card every widget is drawn on: Small is one cell, Medium two by one, Large two by two, in the Desktop host (the Widget material's plate with the Space's tint) or as a Tile on the notification center's Popover. `lift: Lifted` swaps the resting drop for the drag shadow. A frame leaves by Presence: `shown: Hidden` fades it out over --t-quick and calls `on_hidden`; hidden and settled it lays nothing out (the last cell is one, empty).",
            Root { material: Material::Widget, chrome: Some(RootChrome::Transparent), style: "width:100%",
                div { class: "g-row g-row-top",
                    Specimen { name: "Small, Desktop".to_string(),
                        WidgetFrame { size: WidgetSize::Small, title: title(), span { class: "g-name", "content" } }
                    }
                    Specimen { name: "Small, Desktop, Material tint".to_string(),
                        WidgetFrame { size: WidgetSize::Small, tint: CardTint::Material, title: title(), span { class: "g-name", "content" } }
                    }
                    Specimen { name: "Small, lifted".to_string(),
                        WidgetFrame { size: WidgetSize::Small, lift: Lift::Lifted, title: title(), span { class: "g-name", "content" } }
                    }
                    Specimen { name: "Small, Tile".to_string(),
                        WidgetFrame { size: WidgetSize::Small, host: WidgetHost::Tile, title: title(), span { class: "g-name", "content" } }
                    }
                    Specimen { name: "Medium".to_string(),
                        WidgetFrame { size: WidgetSize::Medium, title: title(), span { class: "g-name", "content" } }
                    }
                    Specimen { name: "Hidden (nothing laid out)".to_string(),
                        WidgetFrame { size: WidgetSize::Small, shown: Shown::Hidden, common: Common::default(), span { class: "g-name", "content" } }
                    }
                }
            }
        }
    }
}

/// `AnimatedEmoji`: the asset's own animation, once, or still.
#[component]
fn Emoji() -> Element {
    rsx! {
        Section { title: "AnimatedEmoji", note: "The asset's own animation: it plays once through when it appears (and on each new wake stamp or pick), then rests on its first frame and paints nothing; there is no loop of quire's and no mood. `EmojiPlayback::Still`, and Reduced motion, show the rest frame only. A snapshot is taken at rest.",
            div { class: "g-row g-row-top",
                Specimen { name: "Once through".to_string(),
                    AnimatedEmoji { emoji: EmojiId::Wink, size: PictureSize::Large }
                }
                Specimen { name: "Still".to_string(),
                    AnimatedEmoji { emoji: EmojiId::Wink, size: PictureSize::Large, playback: EmojiPlayback::Still }
                }
            }
        }
    }
}
