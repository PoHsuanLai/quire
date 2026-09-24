//! Materials: each of the eight as the chrome of the surface that wears it (design/20-SURFACES.md),
//! over a wallpaper, blur on and off, with the card ink's four legibility floors measured live.

use super::{Scope, Section};
use crate::axes::{Axes, material_label};
use crate::legibility::{FLOOR, floors};
use crate::wallpaper;
use dioxus::prelude::*;
use ds::tokens::Alpha;
use ds::{
    Avatar, AvatarSize, AvatarTone, BlurState, Button, ButtonVariant, Chip, ChipVariant, Fraction,
    Glyph, Icon, IconButton, IconButtonVariant, IconSize, Material, Slider, StatusMetrics, Surface,
    use_env,
};

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
    let mut tint = use_context::<Signal<Alpha>>();
    let percent = tint().0 / 10;
    rsx! {
        Section {
            title: "Tint alpha",
            note: "appearance.material_tint_alpha scales every translucent tint (the default 80 is design/03-COLOR.md section 17.2 as written). The bar, dock, popover panel, OSD and widget draw the Space gradient at that alpha, cross-fading on a Space switch (design/21-SPACES.md sections 3 and 5). The floors below follow it and measure the flat tint a Surface or a floating card paints; the gradient's own gates are ds's legibility tests. The solid fallback never moves.",
            div { class: "g-row",
                div { class: "g-col", style: "width:320px",
                    Slider {
                        label: "Material tint alpha",
                        value: Fraction(tint().0),
                        step: Fraction(10),
                        onchange: move |next: Fraction| tint.set(Alpha(next.0.max(100))),
                    }
                }
                span { class: "g-code", "material_tint_alpha = {percent}" }
                Button { variant: ButtonVariant::Mini, label: "Reset to 80", onclick: move |_| tint.set(Alpha(800)) }
            }
        }
        div { class: "g-grid2",
            for material in Material::ALL {
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
    let scheme = use_env().scheme;
    let tint = use_context::<Signal<Alpha>>()();
    let measured = floors(material, scheme, tint);
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
                            text: format!("{}, over {}: {:.2} (needs {FLOOR})", floor.tint.label(), floor.backdrop.label(), floor.measured),
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
    let blur = use_env().blur;
    let state = match (material, blur) {
        (Material::Window, _) => "no blur, ever",
        (_, BlurState::Available) => "data-blur=on",
        (_, BlurState::Unavailable) => "data-blur=off",
    };
    match material {
        Material::Bar => rsx! {
            div { class: "g-panel g-panel-bar g-row", style: StatusMetrics::default().style_attr(),
                Glyph { icon: Icon::Grid, size: IconSize::Bar }
                span { class: "g-name", "Files" }
                span { class: "g-code", "{state}" }
                span { class: "g-spacer" }
                IconButton { variant: IconButtonVariant::Status, icon: Icon::Wifi, label: "Wi-Fi", onclick: |_| {} }
                IconButton { variant: IconButtonVariant::Status, icon: Icon::BatteryFull, label: "Battery", expanded: Some(ds::Switch::On), onclick: |_| {} }
                span { class: "ds-tabular", "09:41" }
            }
        },
        Material::Dock => rsx! {
            div { class: "g-panel g-panel-dock g-row",
                for icon in [Icon::Mail, Icon::Folder, Icon::Terminal, Icon::Camera] {
                    IconButton { variant: IconButtonVariant::Pin, icon, label: "App", onclick: |_| {} }
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
