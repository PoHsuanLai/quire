//! Materials: each of the eight as the chrome of the surface that wears it (design/20-SURFACES.md),
//! over a wallpaper, blur on and off, with the card ink's four legibility floors measured live.

use crate::axes::{Axes, material_label};
use crate::legibility::floors;
use crate::pages::{Scope, Section};
use crate::wallpaper;
use dioxus::prelude::*;
use ds::components::content::avatar::{AvatarSize, AvatarTone};
use ds::components::controls::button_model::{Bezel, ImagePosition};
use ds::components::controls::chip::{Chip, ChipVariant};
use ds::prelude::*;
use ds::style::appearance::blur::BlurState;
use ds::style::icon::render::Glyph;
use ds::style::material::recipe::DEFAULT_TINT_ALPHA;
use ds::style::tokens::control_size::ControlSize;

/// Which surface wears each material (design/20-SURFACES.md section 3's table).
fn wearer(material: Material) -> &'static str {
    match material {
        Material::Window => "App windows: the Space gradient, its layers and grain",
        Material::Bar => {
            "The menu bar: the Space gradient at the bar's tint (data-frame=tinted), status items and text on the frame ground"
        }
        Material::Dock => {
            "The dock pill: the Space gradient at the dock's tint, on the frame ground"
        }
        Material::Popover => {
            "Menus and popups off the bar and dock; as a panel root (the launcher), the Space gradient at its tint"
        }
        Material::Sheet => "Control center, notification center, power menu",
        Material::Toast => "Notification banners",
        Material::Osd => "Volume and brightness: the Space gradient at its tint",
        Material::Widget => "Desktop widgets, the quick note: the Space gradient at its tint",
    }
}

/// The materials page.
#[component]
pub fn MaterialsPage() -> Element {
    rsx! {
        div { class: "g-grid2",
            for material in Material::ALL.iter().copied() {
                Chrome { material }
            }
        }
    }
}

/// One material: its wearer, the panel blur on and off over the wallpaper, and the floors.
#[component]
fn Chrome(material: Material) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let accent = axes.read().accent;
    let scheme = use_scope().scheme;
    let measured = floors(material, scheme, DEFAULT_TINT_ALPHA);
    rsx! {
        Section { title: material_label(material), note: wearer(material),
            div { class: "g-wall", style: "background-image:url(\"{wallpaper::uri()}\")",
                if material == Material::Window {
                    Surface { material,
                        Panel { material }
                    }
                } else {
                    for blur in [BlurState::Available, BlurState::Unavailable] {
                        Scope { scheme, accent, material, blur,
                            Panel { material }
                        }
                    }
                }
            }
            if measured.is_empty() {
                p { class: "g-note", "No floors: the window paints the Space gradient, never a tint over blur. Its frame's own gates are on the Space page." }
            } else {
                div { class: "g-floors",
                    for floor in measured {
                        Chip {
                            variant: ChipVariant::Status(floor.verdict()),
                            text: format!("{}, over {}: {:.2} (needs {})", floor.tint.label(), floor.backdrop.label(), floor.measured, floor.floor),
                        }
                    }
                }
            }
        }
    }
}

/// What sits in the chrome: the specimen that surface draws.
#[component]
fn Panel(material: Material) -> Element {
    let blur = use_scope().blur;
    let state = match (material, blur) {
        (Material::Window, _) => "no blur, ever",
        (_, BlurState::Available) => "data-blur=on",
        (_, BlurState::Unavailable) => "data-blur=off",
    };
    match material {
        Material::Bar => rsx! {
            div { class: "g-panel g-panel-bar g-row", style: "--bar-status-box:22px;--bar-status-glyph:16px;",
                Glyph { icon: Icon::Grid, size: IconSize::Bar }
                span { class: "g-name", "Files" }
                span { class: "g-code", "{state}" }
                span { class: "g-spacer" }
                span { class: "ds-tabular", "09:41" }
            }
        },
        Material::Dock => rsx! {
            div { class: "g-panel g-panel-dock g-row",
                for icon in [Icon::Mail, Icon::Folder, Icon::Terminal, Icon::Camera] {
                    Button { bezel: Bezel::Toolbar, size: ControlSize::Large, image: ImagePosition::Only, icon, label: "App", onclick: |_| {} }
                }
                span { class: "g-code", "{state}" }
            }
        },
        Material::Osd => rsx! {
            div { class: "g-panel",
                Glyph { icon: Icon::Volume, size: IconSize::Bar }
                Slider { label: "Volume", value: Fraction(620), onchange: |_| {} }
                span { class: "g-code", "{state}" }
            }
        },
        Material::Toast => rsx! {
            div { class: "g-panel g-panel-row",
                Avatar { initial: 'D', size: AvatarSize::Size34, tone: AvatarTone::Ink }
                div { class: "g-col",
                    span { class: "g-name", "Dana Okafor" }
                    span { "Re: UIDL stability across servers" }
                    span { class: "g-code", "{state}" }
                }
            }
        },
        Material::Window | Material::Popover | Material::Sheet | Material::Widget => rsx! {
            div { class: "g-panel",
                span { class: "g-name", "{material_label(material)}" }
                span { "Body text in --ink over the tint." }
                span { class: "g-code", "{state}" }
            }
        },
    }
}
