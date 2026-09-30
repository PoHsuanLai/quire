//! The Now Playing play/pause button (design/26-DETAILS.md 5.2.10): an `IconButton { Tool }`
//! whose glyph is the next action, cross-faded into the next ("emphasizes the next available
//! state or action", A5).

use crate::components::controls::button_size::disabled;
use crate::components::controls::press::PressListeners;
use ds_motion::detail::{morph::MorphStyle, morph_glyph::MorphGlyph};
use crate::shell::now_playing::kind::Playback;
use dioxus::prelude::*;
use ds_core::press::Press;
use ds_core::vocab::Availability;
use ds_style::icon::render::IconSize;

/// Play or pause as `playback` offers next, named for that action ("Play", "Pause").
/// `Disabled` when the player cannot do it.
#[component]
pub fn PlayPauseButton(
    playback: Playback,
    onclick: EventHandler<Press>,
    #[props(default)] availability: Availability,
) -> Element {
    let listen = PressListeners::new(onclick);
    let live = availability == Availability::Enabled;
    let label = playback.label();
    rsx! {
        button {
            r#type: "button",
            class: "ds-icon-button",
            "data-variant": "tool",
            "data-playback": slug(playback),
            "aria-label": label,
            "aria-disabled": availability.aria_disabled(),
            disabled: disabled(availability),
            onclick: move |event| {
                if live {
                    listen.click(&event);
                }
            },
            MorphGlyph {
                icon: playback.next_action(),
                size: IconSize::Base,
                style: MorphStyle::CrossFade,
            }
        }
    }
}

/// The `data-playback` word.
fn slug(playback: Playback) -> &'static str {
    match playback {
        Playback::Paused => "paused",
        Playback::Playing => "playing",
        Playback::Buffering(_) => "buffering",
    }
}
