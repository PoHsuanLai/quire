//! The voice orb component.

use super::io::use_turn;
use super::model::{OrbColour, OrbColours, OrbMask, OrbMetrics, Turn};
use crate::core::geometry::units::Px;
use crate::core::vocab::Activity;
use crate::core::word::Word;
use crate::style::tokens::colour::ColourToken;
use crate::style::tokens::orb as vars;
use dioxus::prelude::*;
use std::time::Duration;

/// How long the glows take to turn once, unless the caller says.
pub const ORB_PERIOD: Duration = Duration::from_secs(20);

/// The orb's default size across.
pub const ORB_SIZE: Px = Px(192.0);

/// A round field of drifting colour: three glows behind a fine dot grid, turning slowly while
/// the orb is `Active` and standing still when it is not.
///
/// The turn is driven from Rust (a frame timer that runs only while `activity` is `Active`, and
/// not under Reduced motion), because Blitz cannot animate a registered custom property. On
/// Blitz the dots are a plain layer at reduced opacity under the mask, where a browser
/// overlay-blends and backdrop-blurs them; `filter: blur()` paints on the GPU renderer only and
/// `contrast()` on none (FINDINGS "CSS filter"). Decorative, so it is hidden from assistive
/// technology unless `aria_label` names it.
#[component]
pub fn VoiceOrb(
    #[props(default = ORB_SIZE)] size: Px,
    #[props(default)] colours: OrbColours,
    #[props(default = ORB_PERIOD)] period: Duration,
    #[props(default)] activity: Activity,
    #[props(default)] aria_label: Option<String>,
) -> Element {
    let turn = use_turn(activity, period);
    let metrics = OrbMetrics::of(size);
    let mask = match metrics.mask {
        OrbMask::Off => "off",
        OrbMask::Radius(_) => "radius",
    };
    let (role, hidden) = match aria_label {
        Some(_) => (Some("img"), None),
        None => (None, Some("true")),
    };
    rsx! {
        div {
            class: "ds-voice-orb",
            "data-activity": activity.slug(),
            "data-mask": mask,
            role,
            "aria-label": aria_label,
            "aria-hidden": hidden,
            style: style_attr(size, &metrics, &colours, turn),
            span { class: "ds-voice-orb-glow" }
            span { class: "ds-voice-orb-dots" }
        }
    }
}

/// A number as CSS writes it: up to three decimals, no trailing zeros.
fn number(value: f32) -> String {
    let text = format!("{value:.3}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// The custom properties the stylesheet reads, as an inline style: the size, the metrics, the
/// turn, and a colour only where the caller brought one (the tokens answer for the rest).
pub(super) fn style_attr(
    size: Px,
    metrics: &OrbMetrics,
    colours: &OrbColours,
    turn: Turn,
) -> String {
    let mut out = format!(
        "{}:{}px;{}:{}px;{}:{};{}:{}px;{}:{}px;{}:{}deg;",
        vars::SIZE.as_str(),
        number(size.0),
        vars::BLUR.as_str(),
        number(metrics.blur.0),
        vars::CONTRAST.as_str(),
        number(metrics.contrast.0),
        vars::DOT.as_str(),
        number(metrics.dot.0),
        vars::SHADOW.as_str(),
        number(metrics.shadow.0),
        vars::TURN.as_str(),
        number(turn.degrees()),
    );
    if let OrbMask::Radius(radius) = metrics.mask {
        out.push_str(&format!("{}:{}%;", vars::MASK.as_str(), radius.0));
    }
    for (token, colour) in [
        (ColourToken::OrbBg, colours.bg),
        (ColourToken::OrbC1, colours.c1),
        (ColourToken::OrbC2, colours.c2),
        (ColourToken::OrbC3, colours.c3),
    ] {
        if let OrbColour::Custom(hex) = colour {
            out.push_str(&format!("{}:{};", token.var().as_str(), hex.css()));
        }
    }
    out
}
