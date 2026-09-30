//! The Overlays page's alerts: the Mac's pre-Liquid-Glass alert, "Turn Bluetooth
//! off?", drawn in place inside a 320 px control-center popover (`Flow::Inline`) and centred in a
//! whole window (`Flow::Floating`), in light and dark; a destructive one whose default is Cancel;
//! one button; three buttons, stacked, in the Warning style. Live, every button and Escape (Cancel)
//! close it and the button opens it again.

use crate::axes::Axes;
use crate::pages::Section;
use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::components::overlays::alert_model::{AlertButton, AlertRole, AlertStyle};
use ds::prelude::*;
use ds::root::chrome::FrameTint;
use ds::style::space::look::CardAccent;
use ds::style::space::presets::default_look;
use ds_shell::prelude::*;

const TITLE: &str = "Turn Bluetooth off?";
const MESSAGE: &str = "Bluetooth devices such as keyboards and mice will be disconnected.";

/// The alerts section.
#[component]
pub fn Alerts() -> Element {
    rsx! {
        Section { title: "Alert", note: "Alert {{ title, message, buttons, style, icon, flow }}: the narrow sheet (340, or 88 % of a smaller root), dimming nothing, one centred column (optional icon at 48, title 16/700, body in the soft ink), then the buttons: one or two side by side with the default on the right, three or more stacked with the default on top. It slides in as a sheet does. The default button is filled: the first that is not destructive (a destructive one has its label red), and the keyboard starts there. Return presses the default, Escape the Cancel button, Space presses the focused button, Tab and Shift+Tab move between the buttons. Flow::Inline stands in the nearest positioned ancestor: here a 320 px control-center popover. Flow::Floating centres it in the whole root.",
            div { class: "g-row g-alert-row",
                for theme in [Theme::Light, Theme::Dark] {
                    InPopover { theme }
                }
            }
            div { class: "g-row g-alert-row",
                Windowed { theme: Theme::Light, case: Case::Ask }
                Windowed { theme: Theme::Dark, case: Case::Erase }
            }
            div { class: "g-row g-alert-row",
                Windowed { theme: Theme::Light, case: Case::Notice }
                Windowed { theme: Theme::Dark, case: Case::Save }
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
            Ds {
                appearance,
                look: SpaceLook { theme, ..default_look(0, CardAccent::SpaceHue) },
                material: Material::Popover,
                stylesheet: Inject::Host,
                chrome: Some(RootChrome::Painted),
                frame: Some(FrameTint::Tinted),
                div { class: "g-alert-cc-body",
                    ModuleGrid { padding: Px(0.0),
                        ModuleTile { glyph: Icon::Wifi, title: "Wi-Fi", status: "Home", value: Check::On, onclick: |_| {}, on_detail: |_| {} }
                        ModuleTile { glyph: Icon::Bluetooth, title: "Bluetooth", status: "On", value: Check::On,
                            onclick: move |_| open.set(true), on_detail: |_| {} }
                        ModuleTile { glyph: Icon::Moon, title: "Focus", status: "Off", value: Check::Off, onclick: |_| {} }
                        ModuleTile { glyph: Icon::Link, title: "Hotspot", status: "Off", value: Check::Off, onclick: |_| {} }
                    }
                    if open() {
                        Alert {
                            title: TITLE,
                            message: Some(TextLine::from(MESSAGE)),
                            buttons: vec![
                                AlertButton::new("Turn Off", AlertRole::Normal, EventHandler::new(move |()| open.set(false))),
                                AlertButton::new("Cancel", AlertRole::Cancel, EventHandler::new(move |()| open.set(false))),
                            ],
                            flow: Flow::Inline,
                        }
                    }
                }
            }
        }
    }
}

/// Which alert a window shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Case {
    /// Two buttons, the action the default.
    Ask,
    /// A destructive action: Cancel is the default.
    Erase,
    /// One button.
    Notice,
    /// Three buttons, stacked, in the Warning style.
    Save,
}

/// The alert centred in a small window.
#[component]
fn Windowed(theme: Theme, case: Case) -> Element {
    let appearance = appearance(theme);
    let mut open = use_signal(|| true);
    let close = EventHandler::new(move |()| open.set(false));
    let button = move |label: &str, role| AlertButton::new(label, role, close);
    let (title, message, style, icon, buttons) = match case {
        Case::Ask => (
            TITLE,
            MESSAGE,
            AlertStyle::Informational,
            None,
            vec![
                button("Turn Off", AlertRole::Normal),
                button("Cancel", AlertRole::Cancel),
            ],
        ),
        Case::Erase => (
            "Erase “Backup”?",
            "Everything on the disk will be lost. This can’t be undone.",
            AlertStyle::Critical,
            Some(IconSource::Glyph(Icon::Trash)),
            vec![
                button("Erase", AlertRole::Destructive),
                button("Cancel", AlertRole::Cancel),
            ],
        ),
        Case::Notice => (
            "Bluetooth is off",
            "Turn it on in Control Center to connect a keyboard.",
            AlertStyle::Informational,
            None,
            vec![button("OK", AlertRole::Normal)],
        ),
        Case::Save => (
            "Do you want to save the changes?",
            "Your changes will be lost if you don’t save them.",
            AlertStyle::Warning,
            None,
            vec![
                button("Save", AlertRole::Normal),
                button("Don’t Save", AlertRole::Destructive),
                button("Cancel", AlertRole::Cancel),
            ],
        ),
    };
    rsx! {
        div { class: "g-modal",
            Ds { appearance, material: Material::Sheet, stylesheet: Inject::Host,
                div { class: "g-modal-stage g-alert-stage",
                    Button { label: "Show the alert", onclick: move |_| open.set(true) }
                }
                if open() {
                    Alert {
                        title,
                        message: Some(TextLine::from(message)),
                        buttons,
                        style,
                        icon,
                    }
                }
            }
        }
    }
}
