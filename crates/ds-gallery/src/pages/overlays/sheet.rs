//! The Overlays page's sheets: the power menu (a centred Sheet) in light and dark, and a
//! window-attached Sheet hanging from the top edge, each in a Sheet root of its own over the
//! wallpaper. A sheet dims nothing. Cancel,
//! a Danger Restart and a Primary Shut Down at the Regular size, an unavailable Suspend drawn
//! disabled, and the key hints with the arrow caps at the Small size.

use crate::axes::Axes;
use crate::pages::Section;
use crate::wallpaper;
use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::components::controls::button_model::{Answers, ButtonRole};
use ds::components::controls::key_equivalent::{KeyEquivalent, KeyStyle};
use ds::components::overlays::sheet_attach::Attach;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;

/// The power menu section.
#[component]
pub fn PowerMenu() -> Element {
    rsx! {
        Section { title: "Sheet and power menu", note: "Sheet {{ attach }} in a Sheet root, dimming nothing (macOS draws no scrim). Attach::Centre, light and dark: centred both ways, fading in with a scale over --t-quick. Attach::Window: a card hanging 12 below the top edge. Buttons at ButtonSize::Regular: Cancel (Secondary), Restart (Danger), Shut Down (Primary), and Suspend unavailable (Availability::Disabled: .35, no hover, no press). The hints are Small Kbd caps; the left and right arrows come from Space Mono like the up and down. A sheet its host hides fades out (sheet-out, --t-quick) and reports on_hidden at settle(SheetOut).",
            div { class: "g-wall g-chrome-cards", style: "background-image:url(\"{wallpaper::uri()}\")",
                for (theme , attach) in [(Theme::Light, Attach::Centre), (Theme::Dark, Attach::Centre), (Theme::Light, Attach::Window)] {
                    Dialog { theme, attach }
                }
            }
        }
    }
}

/// One power menu in `theme`, in a stage the size of a small screen.
#[component]
fn Dialog(theme: Theme, attach: Attach) -> Element {
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
                    attach,
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
