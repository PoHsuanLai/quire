//! The panes of the Settings window: each a stack of `FieldGroup`s with real, live controls.

use super::Category;
use dioxus::prelude::*;
use ds::components::chrome::tab_view::TabView;
use ds::components::controls::checkbox::Checkbox;
use ds::components::controls::radio_group::Arrangement;
use ds::components::controls::segmented::Tracking;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::fields::stepper::model::StepRange;
use ds::components::fields::stepper::view::Stepper;
use ds::components::menus::pop_up_button::PopUpButton;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;

/// The pane for `category`.
pub(super) fn pane(category: Category) -> Element {
    match category {
        Category::WiFi => rsx! { WifiPane {} },
        Category::Appearance => rsx! { AppearancePane {} },
        Category::Displays => rsx! { DisplaysPane {} },
    }
}

fn label(text: &'static str) -> TextLine {
    TextLine::from(text)
}

#[component]
fn WifiPane() -> Element {
    let mut on = use_signal(|| Check::On);
    let mut network = use_signal(|| "home");
    let mut ask = use_signal(|| Check::On);
    let mut name = use_signal(|| "Dana's Laptop".to_owned());
    let networks = vec![
        MenuItem::new("home", "Home 5 GHz"),
        MenuItem::new("office", "Office"),
        MenuItem::new("guest", "Guest"),
    ];
    let live = if on() == Check::On {
        Availability::Enabled
    } else {
        Availability::Disabled
    };
    rsx! {
        FieldGroup {
            FieldRow { label: label("Wi-Fi"), help: Some(label("Join known networks automatically.")),
                Toggle { label: "Wi-Fi", value: on(), onchange: move |next| on.set(next) }
            }
            FieldRow { label: label("Network"), availability: live,
                PopUpButton::<&'static str> { items: networks, value: Some(network()), onpick: move |next| network.set(next), availability: live }
            }
            FieldRow { label: label("Ask to join networks"), help: Some(label("Known networks are joined automatically; otherwise you are asked.")), availability: live,
                Toggle { label: "Ask to join networks", value: ask(), onchange: move |next| ask.set(next), availability: live }
            }
        }
        FieldGroup { title: "This computer",
            FieldRow { label: label("Name"),
                TextField { label: "Name", value: name(), oninput: move |next| name.set(next) }
            }
        }
    }
}

#[component]
fn AppearancePane() -> Element {
    let mut theme = use_signal(|| 0u8);
    let mut accent = use_signal(|| "blue");
    let mut size = use_signal(|| 13i32);
    let mut tint = use_signal(|| Check::On);
    let mut sidebar = use_signal(|| 1u8);
    let accents = Choice::pairs([
        ("blue", "Blue"),
        ("purple", "Purple"),
        ("pink", "Pink"),
        ("green", "Green"),
    ]);
    rsx! {
        FieldGroup {
            FieldRow { label: label("Appearance"),
                SegmentedControl::<u8> {
                    label: "Appearance",
                    choices: Choice::pairs([(0u8, "Light"), (1, "Dark"), (2, "Auto")]),
                    tracking: Tracking::SelectOne(theme()),
                    size: ControlSize::Small,
                    onchange: move |next| theme.set(next),
                }
            }
            FieldRow { label: label("Accent colour"),
                RadioGroup::<&'static str> { label: "Accent colour", choices: accents, value: accent(), arrangement: Arrangement::Column, onchange: move |next| accent.set(next) }
            }
            FieldRow { label: label("Text size"), help: Some(label("Points, from 9 to 24.")),
                Stepper { label: "Text size", value: size(), range: StepRange::new(9, 24, 1), onchange: move |next| size.set(next) }
            }
            FieldRow { label: label("Tint window backgrounds"),
                Checkbox { label: "Tint", value: tint(), onchange: move |next| tint.set(next) }
            }
        }
        FieldGroup { title: "Sidebar",
            FieldRow { label: label("Sidebar icon size"),
                TabView::<u8> {
                    label: "Sidebar icon size",
                    tabs: Choice::pairs([(0u8, "Small"), (1, "Medium"), (2, "Large")]),
                    value: sidebar(),
                    size: ControlSize::Small,
                    onchange: move |next| sidebar.set(next),
                    p { "Rows are 28, 32 or 36 points tall." }
                }
            }
        }
    }
}

#[component]
fn DisplaysPane() -> Element {
    let mut brightness = use_signal(|| Fraction(700));
    let mut scale = use_signal(|| 1u8);
    let mut night = use_signal(|| Check::Off);
    rsx! {
        FieldGroup {
            FieldRow { label: label("Brightness"),
                div { style: "width:200px",
                    Slider { label: "Brightness", value: brightness(), onchange: move |next| brightness.set(next) }
                }
            }
            FieldRow { label: label("Scale"),
                SegmentedControl::<u8> {
                    label: "Scale",
                    choices: Choice::pairs([(0u8, "100%"), (1, "125%"), (2, "150%")]),
                    tracking: Tracking::SelectOne(scale()),
                    size: ControlSize::Small,
                    onchange: move |next| scale.set(next),
                }
            }
            FieldRow { label: label("Night shift"), help: Some(label("Warms the display after sunset.")),
                Toggle { label: "Night shift", value: night(), onchange: move |next| night.set(next) }
            }
        }
    }
}
