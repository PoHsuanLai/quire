//! The Details page's specimens as components of their state, so the page (with its replay
//! buttons) and the frame strips (driven by a harness) draw the same thing.

use crate::details_states::Net;
use dioxus::prelude::*;
use ds::components::content::status::wifi_state::{WifiBars, WifiReach, WifiState};
use ds::motion::detail::morph::MorphStyle;
use ds::motion::detail::morph::Slashed;
use ds::motion::detail::morph_glyph::MorphGlyph;
use ds::motion::detail::once::use_shake;
use ds::motion::detail::touch::Touch;
use ds::prelude::*;

/// `class` with a pulse's class and alias when it plays.
fn pulsed(class: &str, key: Option<(String, &'static str)>) -> (String, Option<&'static str>) {
    match key {
        Some((anim, alias)) => (format!("{class} {anim}"), Some(alias)),
        None => (class.to_owned(), None),
    }
}

/// What a Wi-Fi state says in words, so no moment is carried by motion alone (R8).
fn words(net: &Net) -> &'static str {
    match net {
        Net::Off => "Off",
        Net::Joining => "Joining…",
        Net::Joined => "Joined",
        Net::Failed(_) => "Couldn't join",
    }
}

/// The Wi-Fi glyph a demo state stands for.
fn wifi(net: &Net) -> WifiState {
    match net {
        Net::Off => WifiState::Off,
        Net::Joining => WifiState::Joining(ds::motion::detail::stamp::EventStamp(1)),
        Net::Joined => WifiState::Joined {
            bars: WifiBars::Three,
            reach: WifiReach::Internet,
        },
        Net::Failed(stamp) => WifiState::Failed(*stamp),
    }
}

/// A Wi-Fi glyph: its layers searching while joining, a step every `--t-spin-step`.
#[component]
pub fn NetView(net: Net) -> Element {
    let detail = use_detail(net, Touch::Remote);
    rsx! {
        div { class: "g-detail",
            WifiGlyph { state: wifi(detail.state()), size: IconSize::Bar }
            span { class: "g-detail-word", {words(detail.state())} }
        }
    }
}

/// A field that shakes once per new failure (Shake, R6).
#[component]
pub fn ShakeView(net: Net) -> Element {
    let detail = use_detail(net, Touch::Remote);
    let (class, alias) = pulsed("g-shake-field", use_shake(detail.cue()).attrs());
    rsx! {
        div { class: "g-detail",
            div { class, "data-pulse": alias, "Password" }
        }
    }
}

/// A speaker whose slash draws on and off (MorphGlyph Slash).
#[component]
pub fn SlashView(slashed: Slashed) -> Element {
    rsx! {
        div { class: "g-detail",
            MorphGlyph { icon: Icon::Volume, size: IconSize::Bar, style: MorphStyle::Slash, slashed }
        }
    }
}
