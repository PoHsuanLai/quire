//! StylesOffNotice: the one line shown when the crash guard started a program without the
//! person's styles (customization design section 7.4): "Your styles were turned off because
//! <program> could not start with them." and a **Turn On Again** button. It is an
//! [`InlineBanner`](super::inline_banner) with a warning and one action, so it has no markup or
//! rules of its own. Nothing opens Settings by itself: the button is the whole way forward.

use crate::components::controls::button::Button;
use crate::components::overlays::inline_banner::InlineBanner;
use dioxus::prelude::*;
use ds_core::vocab::Severity;
use ds_style::tokens::control_size::ControlSize;

/// The sentence, with the program's name in it.
pub fn styles_off_text(program: &str) -> String {
    format!("Your styles were turned off because {program} could not start with them.")
}

/// The notice for `program`. `onturnon` runs when the person presses **Turn On Again**: the app
/// clears the guard (`ds_blitz::StylesGuard::turn_on_again`) and draws with its styles again.
#[component]
pub fn StylesOffNotice(#[props(into)] program: String, onturnon: EventHandler<()>) -> Element {
    rsx! {
        InlineBanner {
            severity: Severity::Warn,
            text: styles_off_text(&program),
            actions: rsx! {
                Button {
                    label: "Turn On Again",
                    size: ControlSize::Small,
                    onclick: move |_| onturnon.call(()),
                }
            },
        }
    }
}
