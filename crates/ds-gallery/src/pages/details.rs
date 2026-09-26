//! Details: every small-state primitive of design/26-DETAILS.md in its own cell, with the
//! buttons that move its state. Nothing here loops: each settles to 0 frames.

use super::Section;
use crate::details_states::{Bell, Charge, Net, Seal};
use crate::details_views::{
    CheckView, NetView, NudgeView, SealView, ShakeView, SlashView, SweepCountView,
};
use dioxus::prelude::*;
use ds::components::press::Press;
use ds::detail::{
    Deadline, EventStamp, FirstShow, MorphGlyph, MorphStyle, Operation, PendingToken, Reveal,
    RollDigits, Slashed, Touch,
};
use ds::{Button, ButtonVariant, Glyph, Icon, IconSize, Spinner, SpinnerKind};
use std::time::Duration;

/// The page.
#[component]
pub fn DetailsPage() -> Element {
    rsx! {
        TimeSection {}
        StateSection {}
        MorphSection {}
        super::details_status::StatusSection {}
    }
}

/// One primitive: its well, its caption, and the buttons that move it.
#[component]
pub(super) fn Cell(name: String, code: String, controls: Element, children: Element) -> Element {
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
pub(super) fn mini(label: &'static str, onclick: impl FnMut(Press) + 'static) -> Element {
    rsx! { Button { variant: ButtonVariant::Mini, label, onclick } }
}

/// The page opens at rest (a snapshot runs no Rust timer, so an Appear would be caught mid-sweep);
/// each replay mounts the specimen again as a surface just opened.
fn opened(round: u32) -> FirstShow {
    match round {
        0 => FirstShow::Still,
        _ => FirstShow::Animate,
    }
}

#[component]
fn TimeSection() -> Element {
    let mut appear = use_signal(|| 0u32);
    let mut charge = use_signal(|| Charge::Level(80));
    let mut reveal = use_signal(|| 0u32);
    rsx! {
        Section { title: "Sweep, CountUp, Reveal", note: "Appear sweeps from zero over --t-sweep with the count in step; a change sweeps from where it is over --t-quick; Reveal staggers a list's rows the first time it shows, capped at 12. Reduced shows the target at once.",
            div { class: "g-detail-grid",
                Cell { name: "Sweep + CountUp", code: "use_sweep, use_count_up(InStep)",
                    controls: rsx! {
                        {mini("Replay appear", move |_| *appear.write() += 1)}
                        {mini("80 %", move |_| charge.set(Charge::Level(80)))}
                        {mini("35 %", move |_| charge.set(Charge::Level(35)))}
                    },
                    for round in [appear()] {
                        SweepCountView { key: "{round}", charge: charge(), first: opened(round) }
                    }
                }
                Cell { name: "Reveal", code: "Reveal {{ first }}",
                    controls: rsx! { {mini("Replay reveal", move |_| *reveal.write() += 1)} },
                    for round in [reveal()] {
                        div { key: "{round}", class: "g-detail g-detail-list",
                            Reveal { first: opened(round),
                                for n in 1..=4 {
                                    div { key: "{n}", class: "g-reveal-row", "Result {n}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StateSection() -> Element {
    let mut net = use_signal(|| Net::Off);
    let mut check = use_signal(|| Net::Off);
    let mut failures = use_signal(|| 0u32);
    let mut bell = use_signal(|| Bell::Quiet);
    let mut seal = use_signal(|| (Seal::Open, Touch::Remote));
    let mut operation = use_signal(|| Operation::Idle);
    let failed = move || match failures() {
        0 => Net::Joined,
        n => Net::Failed(EventStamp(n)),
    };
    let next_seal = move || match seal().0 {
        Seal::Sealed(EventStamp(n)) => Seal::Sealed(EventStamp(n + 1)),
        Seal::Open => Seal::Sealed(EventStamp(1)),
    };
    rsx! {
        Section { title: "Pending, Settle, Shake, Nudge", note: "A pending loop waits out PendingGrace (400 ms), steps every --t-pending-step and holds its still frame at its token's deadline (never past PendingCap, 10 s). A success fills or draws once and rests; a failure shakes once per new stamp, never again for the same one; attention nudges once per request. Only a press springs.",
            div { class: "g-detail-grid",
                Cell { name: "Pending (Iterate), Settle (Fill)", code: "use_pending, use_settle(Fill)",
                    controls: rsx! {
                        {mini("Join", move |_| net.set(Net::Joining))}
                        {mini("Joined", move |_| net.set(Net::Joined))}
                        {mini("Off", move |_| net.set(Net::Off))}
                    },
                    NetView { net: net() }
                }
                Cell { name: "Spinner (Spin, Breathe)", code: "Spinner {{ kind, operation }}",
                    controls: rsx! {
                        {mini("Start", move |_| operation.set(Operation::Running(PendingToken::start(Deadline::cap()))))}
                        {mini("Start, 3 s", move |_| operation.set(Operation::Running(PendingToken::start(Deadline::within(Duration::from_secs(3))))))}
                        {mini("End", move |_| operation.set(Operation::Idle))}
                    },
                    div { class: "g-detail",
                        span { class: "g-spin-well",
                            Glyph { icon: Icon::Refresh, size: IconSize::Small }
                            Spinner { kind: SpinnerKind::Spin, operation: operation() }
                        }
                        span { class: "g-spin-well",
                            Glyph { icon: Icon::Wifi, size: IconSize::Small }
                            Spinner { kind: SpinnerKind::Breathe, operation: operation() }
                        }
                    }
                }
                Cell { name: "Settle (Check)", code: "use_settle(Check)",
                    controls: rsx! {
                        {mini("Try", move |_| check.set(Net::Joining))}
                        {mini("Succeed", move |_| check.set(Net::Joined))}
                    },
                    CheckView { net: check() }
                }
                Cell { name: "Shake", code: "use_shake",
                    controls: rsx! {
                        {mini("Fail", move |_| *failures.write() += 1)}
                        {mini("Same failure", move |_| failures.set(failures()))}
                    },
                    ShakeView { net: failed() }
                }
                Cell { name: "Nudge", code: "use_nudge",
                    controls: rsx! {
                        {mini("Ask", move |_| {
                            let next = match bell() { Bell::Asking(EventStamp(n)) => n + 1, Bell::Quiet => 1 };
                            bell.set(Bell::Asking(EventStamp(next)));
                        })}
                    },
                    NudgeView { bell: bell() }
                }
                Cell { name: "Settle (LockIn)", code: "press the disc: Touch::Contact springs",
                    controls: rsx! {
                        {mini("Seal from elsewhere", move |_| seal.set((next_seal(), Touch::Remote)))}
                    },
                    div {
                        onclick: move |event| seal.set((next_seal(), Touch::from_event(&event))),
                        SealView { seal: seal().0, touch: seal().1 }
                    }
                }
            }
        }
    }
}

#[component]
fn MorphSection() -> Element {
    let mut playing = use_signal(|| Playing::Paused);
    let mut bars = use_signal(|| 0usize);
    let mut muted = use_signal(|| Slashed::Off);
    let mut percent = use_signal(|| 79i32);
    const STRENGTHS: [Icon; 3] = [Icon::WifiLow, Icon::Wifi, Icon::WifiHigh];
    let play = move || playing().icon();
    rsx! {
        Section { title: "MorphGlyph, RollDigits", note: "A glyph gives way by its style: DownUp for a change of state, OffUp for the next action, CrossFade within one family, Slash drawn on and off. A readout's changed digits roll; the rest stand.",
            div { class: "g-detail-grid",
                Cell { name: "DownUp, OffUp", code: "MorphGlyph {{ style }}",
                    controls: rsx! { {mini("Play / pause", move |_| playing.set(playing().toggled()))} },
                    div { class: "g-detail",
                        MorphGlyph { icon: play(), size: IconSize::Bar, style: MorphStyle::DownUp }
                        MorphGlyph { icon: play(), size: IconSize::Bar, style: MorphStyle::OffUp }
                    }
                }
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
                Cell { name: "RollDigits", code: "RollDigits {{ value }}",
                    controls: rsx! {
                        {mini("+1", move |_| *percent.write() += 1)}
                        {mini("-7", move |_| *percent.write() -= 7)}
                    },
                    div { class: "g-detail", span { class: "g-figure", RollDigits { value: format!("{}%", percent()) } } }
                }
            }
        }
    }
}

/// The play/pause specimen's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Playing {
    Playing,
    Paused,
}

impl Playing {
    /// The glyph for the next action: pause while playing, play while paused.
    fn icon(self) -> Icon {
        match self {
            Playing::Playing => Icon::Pause,
            Playing::Paused => Icon::Play,
        }
    }

    fn toggled(self) -> Playing {
        match self {
            Playing::Playing => Playing::Paused,
            Playing::Paused => Playing::Playing,
        }
    }
}
