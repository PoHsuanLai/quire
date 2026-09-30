//! TabView: four tabs, each with its own body, at two sizes.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::{Check, Checkbox, Choice, ControlSize, TabView, Toggle};

/// The TabView section.
#[component]
pub fn TabViewSection() -> Element {
    let mut tab = use_signal(|| 0u8);
    let mut small = use_signal(|| 1u8);
    let mut wifi = use_signal(|| Check::On);
    let mut remember = use_signal(|| Check::Off);
    let tabs = || {
        Choice::pairs([
            (0u8, "General"),
            (1, "Network"),
            (2, "Privacy"),
            (3, "Advanced"),
        ])
    };
    rsx! {
        Section { title: "TabView", note: "NSTabView: at most six tabs in a segmented strip over the body of the selected one; the arrows move between tabs.",
            div { class: "g-row g-row-top",
                div { style: "width:340px",
                    TabView::<u8> {
                        label: "Settings",
                        tabs: tabs(),
                        value: tab(),
                        onchange: move |next| tab.set(next),
                        match tab() {
                            0 => rsx! { p { "Name, language and the region a document opens in." } },
                            1 => rsx! {
                                div { class: "g-row",
                                    Toggle { label: "Wi-Fi", value: wifi(), onchange: move |next| wifi.set(next) }
                                    span { "Wi-Fi" }
                                }
                            },
                            2 => rsx! { Checkbox { label: "Remember this device", value: remember(), onchange: move |next| remember.set(next) } },
                            _ => rsx! { p { "Anything that changes how the app behaves under load." } },
                        }
                    }
                }
                div { style: "width:340px",
                    TabView::<u8> {
                        label: "Settings, small",
                        tabs: tabs(),
                        value: small(),
                        size: ControlSize::Small,
                        onchange: move |next| small.set(next),
                        p { "The same strip at the small rung." }
                    }
                }
            }
        }
    }
}
