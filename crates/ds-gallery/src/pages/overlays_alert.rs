//! The Overlays page's alerts: the Mac's pre-Liquid-Glass alert, "Turn Bluetooth
//! off?", drawn in place inside a 320 px control-center popover (`Flow::Inline`) and centred in a
//! whole window (`Flow::Floating`), in light and dark, and a destructive one whose default is
//! Cancel. Live, Cancel, Escape, the scrim and the action close it and the button opens it again.

use super::Section;
use crate::axes::Axes;
use dioxus::prelude::*;
use ds::{
    Alert, AlertEmphasis, Appearance, Button, ButtonVariant, Chevron, Ds, Flow, Icon, IconSource,
    Inject, Material, ModuleGrid, ModuleState, ModuleTile, Px, TextLine, Theme,
};

const TITLE: &str = "Turn Bluetooth off?";
const MESSAGE: &str = "Bluetooth devices such as keyboards and mice will be disconnected.";

/// The alerts section.
#[component]
pub fn Alerts() -> Element {
    rsx! {
        Section { title: "Alert", note: "Alert {{ title, message, action, cancel, emphasis, flow }}: the narrow sheet (340, or 88 % of a smaller root) over the modal scrim, one centred column (optional icon at 48, title 16/700, message in the soft ink), Cancel and the action in two equal columns, the action on the right. Enters with peek-in (--t-big --e-spring); hidden, it springs out as a sheet does. The default button is filled: the action, or Cancel when the action is destructive (its label red), and the keyboard starts there. Return presses the default, Escape and the scrim cancel, Space presses the focused button, Tab moves between the two. Flow::Inline stands in the nearest positioned ancestor: here a 320 px control-center popover. Flow::Floating centres it in the whole root.",
            div { class: "g-row g-alert-row",
                for theme in [Theme::Light, Theme::Dark] {
                    InPopover { theme }
                }
            }
            div { class: "g-row g-alert-row",
                Windowed { theme: Theme::Light, emphasis: AlertEmphasis::Default }
                Windowed { theme: Theme::Dark, emphasis: AlertEmphasis::Destructive }
            }
        }
    }
}

/// The appearance for `theme` under the gallery's axes.
fn appearance(theme: Theme) -> Appearance {
    let axes = use_context::<Signal<Axes>>();
    let axes = axes.read();
    Appearance {
        theme,
        accent: axes.accent,
        motion: axes.motion,
    }
}

/// The alert in place over a control-center popover.
#[component]
fn InPopover(theme: Theme) -> Element {
    let appearance = appearance(theme);
    let mut open = use_signal(|| true);
    rsx! {
        div { class: "g-alert-cc",
            Ds { appearance, material: Material::Popover, stylesheet: Inject::Host,
                div { class: "g-alert-cc-body",
                    ModuleGrid { padding: Px(0.0),
                        ModuleTile { glyph: Icon::Wifi, title: "Wi-Fi", status: "Home", state: ModuleState::On, chevron: Chevron::Detail, onclick: |_| {}, on_detail: |_| {} }
                        ModuleTile { glyph: Icon::Bluetooth, title: "Bluetooth", status: "On", state: ModuleState::On, chevron: Chevron::Detail,
                            onclick: move |_| open.set(true), on_detail: |_| {} }
                        ModuleTile { glyph: Icon::Moon, title: "Focus", status: "Off", state: ModuleState::Off, onclick: |_| {} }
                        ModuleTile { glyph: Icon::Link, title: "Hotspot", status: "Off", state: ModuleState::Off, onclick: |_| {} }
                    }
                    if open() {
                        Alert {
                            title: TITLE,
                            message: Some(TextLine::from(MESSAGE)),
                            action: "Turn Off",
                            flow: Flow::Inline,
                            onaction: move |_| open.set(false),
                            oncancel: move |_| open.set(false),
                        }
                    }
                }
            }
        }
    }
}

/// The alert centred in a small window.
#[component]
fn Windowed(theme: Theme, emphasis: AlertEmphasis) -> Element {
    let appearance = appearance(theme);
    let mut open = use_signal(|| true);
    let (title, message, action, icon) = match emphasis {
        AlertEmphasis::Default => (TITLE, MESSAGE, "Turn Off", None),
        AlertEmphasis::Destructive => (
            "Erase “Backup”?",
            "Everything on the disk will be lost. This can’t be undone.",
            "Erase",
            Some(IconSource::Glyph(Icon::Trash)),
        ),
    };
    rsx! {
        div { class: "g-modal",
            Ds { appearance, material: Material::Sheet, stylesheet: Inject::Host,
                div { class: "g-modal-stage g-alert-stage",
                    Button { variant: ButtonVariant::Secondary, label: "Show the alert", onclick: move |_| open.set(true) }
                }
                if open() {
                    Alert {
                        title,
                        message: Some(TextLine::from(message)),
                        action,
                        emphasis,
                        icon,
                        onaction: move |_| open.set(false),
                        oncancel: move |_| open.set(false),
                    }
                }
            }
        }
    }
}
