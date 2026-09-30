//! The Overlays page's power menu (sheet and modal parts): a centred Sheet over a
//! modal scrim, in light and dark, each in a Sheet root of its own over the wallpaper. Cancel,
//! a Danger Restart and a Primary Shut Down at the Regular size, an unavailable Suspend drawn
//! disabled, and the key hints with the arrow caps at the Small size.

use super::Section;
use crate::axes::Axes;
use crate::wallpaper;
use dioxus::prelude::*;
use ds::{Answers, ButtonRole, ControlSize};
use ds::{
    Appearance, Availability, Button, Ds, Inject, Material, ScrimStrength, Sheet, SheetPlacement,
    Shortcut, ShortcutKey, Theme,
};
use ds::{KeyEquivalent, KeyStyle};

/// The power menu section.
#[component]
pub fn PowerMenu() -> Element {
    rsx! {
        Section { title: "Power menu", note: "Sheet {{ placement: SheetPlacement::Centre, scrim: ScrimStrength::Modal }} in a Sheet root, light and dark: centred both ways, over --scrim-modal (.40 light, .55 dark). Buttons at ControlSize::Regular: Cancel, Restart (destructive), Shut Down (the default button), and Suspend unavailable (Availability::Disabled: .35, no hover, no press). The hints are Mini key caps; the left and right arrows come from Space Mono like the up and down. A sheet its host hides plays sheet-out and reports on_hidden at settle(SheetOut).",
            div { class: "g-wall g-polish-cards", style: "background-image:url(\"{wallpaper::uri()}\")",
                for theme in [Theme::Light, Theme::Dark] {
                    Dialog { theme }
                }
            }
        }
    }
}

/// One power menu in `theme`, in a stage the size of a small screen.
#[component]
fn Dialog(theme: Theme) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (accent, motion) = {
        let axes = axes.read();
        (axes.accent, axes.motion)
    };
    rsx! {
        div { class: "g-modal",
            Ds {
                appearance: Appearance { theme, accent, motion },
                material: Material::Sheet,
                stylesheet: Inject::Host,
                div { class: "g-modal-stage" }
                Sheet {
                    label: "Power",
                    onclose: |_| {},
                    placement: SheetPlacement::Centre,
                    scrim: ScrimStrength::Modal,
                    div { class: "g-panel",
                        h3 { "Shut down this computer?" }
                        p { class: "g-note", "Open windows close. Unsaved work may be lost." }
                        div { class: "g-row",
                            Button { label: "Cancel", onclick: |_| {} }
                            Button { role: ButtonRole::Destructive, size: ControlSize::Regular, label: "Suspend", availability: Availability::Disabled, onclick: |_| {} }
                            Button { role: ButtonRole::Destructive, size: ControlSize::Regular, label: "Restart", onclick: |_| {} }
                            Button { answers: Answers::Return, label: "Shut Down", onclick: |_| {} }
                        }
                        div { class: "g-row g-note",
                            KeyEquivalent { shortcut: Shortcut(vec![ShortcutKey::Left]), style: KeyStyle::Cap, size: ControlSize::Mini}
                            KeyEquivalent { shortcut: Shortcut(vec![ShortcutKey::Right]), style: KeyStyle::Cap, size: ControlSize::Mini}
                            span { "move" }
                            KeyEquivalent { shortcut: Shortcut(vec![ShortcutKey::Enter]), style: KeyStyle::Cap, size: ControlSize::Mini}
                            span { "confirm" }
                            KeyEquivalent { shortcut: Shortcut(vec![ShortcutKey::Escape]), style: KeyStyle::Cap, size: ControlSize::Mini}
                            span { "close" }
                        }
                    }
                }
            }
        }
    }
}
