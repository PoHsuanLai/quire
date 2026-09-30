//! The mail-app overlay cases: a hover card's flag whose words are runs.

use crate::cases::{Case, PartsCard};
use dioxus::prelude::*;
use ds::components::content::text_runs::RunTone;
use ds::components::overlays::hover_card::parts::{FlagTone, HoverCardPart};
use ds::prelude::*;
use std::time::Duration;

/// Past the 500 ms hover intent.
const INTENT: Duration = Duration::from_millis(570);

/// The spoof warning: the brand and the domain in the strong tone.
fn spoof(tone: FlagTone) -> HoverCardPart {
    HoverCardPart::flag(
        tone,
        Icon::OctagonAlert,
        TextLine::Runs(vec![
            TextRun::new("Not ", RunTone::Plain),
            TextRun::new("Acme", RunTone::Strong),
            TextRun::new(": this was sent from ", RunTone::Plain),
            TextRun::new("acme-billing.example", RunTone::Strong),
            TextRun::new(".", RunTone::Plain),
        ]),
    )
}

pub const HOVER_CARD_FLAG_CASES: &[Case] = &[
    Case {
        component: "hover_card",
        state: "part-flag-runs-danger",
        make: || rsx! { PartsCard { parts: vec![spoof(FlagTone::Danger)] } },
        wait: INTENT,
    },
    Case {
        component: "hover_card",
        state: "part-flag-runs-info",
        make: || rsx! { PartsCard { parts: vec![spoof(FlagTone::Info)] } },
        wait: INTENT,
    },
    Case {
        component: "hover_card",
        state: "part-flag-text-plain",
        make: || rsx! { PartsCard { parts: vec![HoverCardPart::flag(FlagTone::Danger, Icon::OctagonAlert, "Not the address Dana usually writes from.")] } },
        wait: INTENT,
    },
];
