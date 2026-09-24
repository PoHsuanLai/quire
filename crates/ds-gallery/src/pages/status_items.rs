//! The Controls page's status items on the frame: `IconButton { Status }` on a Bar root, as
//! sill's bar draws them (bar gaps, sill Q9 and Q12). The root draws the Space gradient at the
//! bar's tint alpha (`data-frame=tinted`) and stamps the frame ground (`data-ground=frame`), so
//! the items, the app name and the clock take the `--f-*` inks with no variant of their own.

use super::{Scope, Section, Specimen};
use crate::axes::Axes;
use crate::wallpaper;
use dioxus::prelude::*;
use ds::{
    BlurState, Button, ButtonVariant, Count, CountPlace, Icon, IconButton, IconButtonVariant,
    Material, Px, StatusMetrics, Switch, use_env,
};

/// The two metrics a shell writes from its settings: the keys' defaults (a 22 px box, a 16 px
/// glyph), and `bar.glyph_size_policy = IconSizeBar22`, where the glyph fills the box.
const METRICS: [(&str, StatusMetrics); 2] = [
    (
        "StatusIcon16 (22 box, 16 glyph)",
        StatusMetrics {
            box_size: Px(22.0),
            glyph: Px(16.0),
        },
    ),
    (
        "IconSizeBar22 (22 box, 22 glyph)",
        StatusMetrics {
            box_size: Px(22.0),
            glyph: Px(22.0),
        },
    ),
];

/// The status items row.
#[component]
pub fn StatusItems() -> Element {
    let axes = use_context::<Signal<Axes>>();
    let accent = axes.read().accent;
    let scheme = use_env().scheme;
    rsx! {
        Section { title: "Status items on the frame", note: "IconButton {{ Status }} on a Bar root over the wallpaper, blur on and off: the Space gradient at the bar's tint, and the frame ground, so every item, the app name and the clock draw in --f-ink*. Each row: rest, open (its menu showing, --f-pill), pressed, disabled. The box and glyph come from StatusMetrics, which a shell fills from bar.status_icon_box_px, bar.status_glyph_px and bar.glyph_size_policy.",
            div { class: "g-wall", style: "background-image:url(\"{wallpaper::uri()}\")",
                for (policy , metrics) in METRICS {
                    for blur in [BlurState::Available, BlurState::Unavailable] {
                        Specimen { name: format!("{policy}, {}", blur_label(blur)),
                            Scope { scheme, accent, material: Material::Bar, blur,
                                div { class: "g-panel g-panel-bar g-row", style: metrics.style_attr(),
                                    Button { variant: ButtonVariant::Quiet, label: "Files", onclick: |_| {} }
                                    span { class: "g-spacer" }
                                    IconButton { variant: IconButtonVariant::Status, icon: Icon::Ethernet, label: "Wired network", onclick: |_| {} }
                                    IconButton { variant: IconButtonVariant::Status, icon: Icon::Wifi, label: "Wi-Fi", expanded: Some(Switch::On), onclick: |_| {} }
                                    IconButton { variant: IconButtonVariant::Status, icon: Icon::Volume2, label: "Volume", pressed: Some(Switch::On), onclick: |_| {} }
                                    IconButton { variant: IconButtonVariant::Status, icon: Icon::BatteryCharging, label: "Battery", availability: ds::Availability::Disabled, onclick: |_| {} }
                                    Count { value: 3, place: CountPlace::Item }
                                    span { class: "ds-tabular", "09:41" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn blur_label(blur: BlurState) -> &'static str {
    match blur {
        BlurState::Available => "blur on",
        BlurState::Unavailable => "blur off",
    }
}
