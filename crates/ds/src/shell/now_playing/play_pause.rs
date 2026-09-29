//! The Now Playing play/pause button (design/26-DETAILS.md 5.2.10): an `IconButton { Tool }`
//! whose glyph is the next action, replaced off-up (the old glyph goes at once, the next grows
//! in: "emphasizes the next available state or action", A5), springing when the person's own
//! press caused the change (R5).

use crate::components::controls::button_size::disabled;
use crate::components::controls::press::PressListeners;
use crate::core::press::Press;
use crate::core::vocab::Availability;
use crate::motion::detail::touch::Touch;
use crate::motion::detail::{
    armed::use_armed, first_show::FirstShow, morph::MorphStyle, morph_glyph::MorphGlyph,
    use_detail::use_detail,
};
use crate::shell::now_playing::kind::Playback;
use crate::style::icon::render::IconSize;
use dioxus::prelude::*;

/// Play or pause as `playback` offers next, named for that action ("Play", "Pause"). A press is
/// kept for the change it causes: that change's glyph springs in, one from elsewhere (a media
/// key, another app) grows in at `--e-out`. `Disabled` when the player cannot do it.
#[component]
pub fn PlayPauseButton(
    playback: Playback,
    onclick: EventHandler<Press>,
    #[props(default)] availability: Availability,
) -> Element {
    let armed = use_armed();
    let detail = use_detail(playback, FirstShow::Still, armed.touch());
    armed.spend(detail.cue());
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
                    armed.arm(Touch::from_event(&event));
                    listen.click(&event);
                }
            },
            MorphGlyph {
                icon: playback.next_action(),
                size: IconSize::Base,
                style: MorphStyle::OffUp,
                touch: detail.cue().touch(),
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
