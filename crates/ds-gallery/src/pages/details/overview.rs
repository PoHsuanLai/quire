//! Details: every small-state primitive of design/26-DETAILS.md in its own cell, with the
//! buttons that move its state.

use crate::details_states::Net;
use crate::details_views::{NetView, ShakeView, SlashView};
use crate::pages::Section;
use dioxus::prelude::*;
use ds::Press;
use ds::detail::{EventStamp, MorphGlyph, MorphStyle, Operation, PendingToken, Slashed};
use ds::{Button, ControlSize, Glyph, Icon, IconSize, Progress, ProgressIndicator, ProgressStyle};

/// The page.
#[component]
pub fn DetailsPage() -> Element {
    rsx! {
        StateSection {}
        MorphSection {}
        crate::pages::details::status::StatusSection {}
        crate::pages::details::center::CenterSection {}
    }
}

/// One primitive: its well, its caption, and the buttons that move it.
#[component]
pub(crate) fn Cell(name: String, code: String, controls: Element, children: Element) -> Element {
    rsx! {
        div { class: "g-detail-cell",
            {children}
            div { class: "g-col",
                span { class: "g-name", "{name}" }
                span { class: "g-code", "{code}" }
            }
            div { class: "g-row", {controls} }
        }
    }
}

/// A mini button.
pub(crate) fn mini(label: &'static str, onclick: impl FnMut(Press) + 'static) -> Element {
    rsx! { Button { size: ControlSize::Mini, label, onclick } }
}

#[component]
fn StateSection() -> Element {
    let mut net = use_signal(|| Net::Off);
    let mut failures = use_signal(|| 0u32);
    let mut operation = use_signal(|| Operation::Idle);
    let failed = move || match failures() {
        0 => Net::Joined,
        n => Net::Failed(EventStamp(n)),
    };
    rsx! {
        Section { title: "Pending, Shake", note: "A pending loop spins at once, a step every --t-spin-step (a turn a second), for as long as its operation runs, and keeps turning under Reduced. A failure shakes once per new stamp, never again for the same one.",
            div { class: "g-detail-grid",
                Cell { name: "Pending (Iterate)", code: "use_pending",
                    controls: rsx! {
                        {mini("Join", move |_| net.set(Net::Joining))}
                        {mini("Joined", move |_| net.set(Net::Joined))}
                        {mini("Off", move |_| net.set(Net::Off))}
                    },
                    NetView { net: net() }
                }
                Cell { name: "Spinner", code: "ProgressIndicator {{ Spinner, Unknown(operation) }}",
                    controls: rsx! {
                        {mini("Start", move |_| operation.set(Operation::Running(PendingToken::start())))}
                        {mini("End", move |_| operation.set(Operation::Idle))}
                    },
                    div { class: "g-detail",
                        span { class: "g-spin-well",
                            Glyph { icon: Icon::Refresh, size: IconSize::Small }
                            ProgressIndicator { style: ProgressStyle::Spinner, progress: Progress::Unknown(operation()), size: ControlSize::Small }
                        }
                    }
                }
                Cell { name: "Shake", code: "use_shake",
                    controls: rsx! {
                        {mini("Fail", move |_| *failures.write() += 1)}
                        {mini("Same failure", move |_| failures.set(failures()))}
                    },
                    ShakeView { net: failed() }
                }
            }
        }
    }
}

#[component]
fn MorphSection() -> Element {
    let mut bars = use_signal(|| 0usize);
    let mut muted = use_signal(|| Slashed::Off);
    const STRENGTHS: [Icon; 3] = [Icon::WifiLow, Icon::Wifi, Icon::WifiHigh];
    rsx! {
        Section { title: "MorphGlyph", note: "A glyph gives way by cross-fading within one family, or by drawing a slash on and off.",
            div { class: "g-detail-grid",
                Cell { name: "CrossFade", code: "MorphGlyph {{ style: CrossFade }}",
                    controls: rsx! { {mini("Next strength", move |_| *bars.write() += 1)} },
                    div { class: "g-detail",
                        MorphGlyph { icon: STRENGTHS[bars() % 3], size: IconSize::Bar, style: MorphStyle::CrossFade }
                    }
                }
                Cell { name: "Slash", code: "MorphGlyph {{ style: Slash, slashed }}",
                    controls: rsx! { {mini("Mute", move |_| muted.set(match muted() { Slashed::Off => Slashed::On, Slashed::On => Slashed::Off }))} },
                    SlashView { slashed: muted() }
                }
            }
        }
    }
}
