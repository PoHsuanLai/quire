//! Details: every small-state primitive of design/26-DETAILS.md, each with a button that plays its
//! moment again. Nothing here loops: each settles to 0 frames.

use super::{Section, Specimen};
use crate::details_states::{Bell, Charge, Net, Seal};
use crate::details_views::{
    CheckView, NetView, NudgeView, SealView, ShakeView, SlashView, SweepCountView,
};
use dioxus::prelude::*;
use ds::detail::{
    Deadline, EventStamp, FirstShow, MorphGlyph, MorphStyle, Operation, PendingToken, Reveal,
    RollDigits, Slashed, Touch,
};
use ds::{Button, ButtonVariant, Icon, IconSize, Spinner, SpinnerKind};

/// The page.
#[component]
pub fn DetailsPage() -> Element {
    rsx! {
        TimeSection {}
        StateSection {}
        MorphSection {}
    }
}

/// A mini button.
fn mini(
    label: &'static str,
    onclick: impl FnMut(ds::components::press::Press) + 'static,
) -> Element {
    rsx! { Button { variant: ButtonVariant::Mini, label, onclick } }
}

#[component]
fn TimeSection() -> Element {
    let mut appear = use_signal(|| 0u32);
    let mut charge = use_signal(|| Charge::Level(80));
    let mut reveal = use_signal(|| 0u32);
    rsx! {
        Section { title: "Sweep, CountUp, Reveal", note: "Appear sweeps from zero over --t-sweep with the count in step; a change sweeps from where it is over --t-quick; Reveal staggers the rows of a list the first time it shows, capped at 12.",
            div { class: "g-row",
                for round in [appear()] {
                    Specimen { key: "{round}", name: "Sweep + CountUp", code: "use_sweep, use_count_up(InStep)",
                        SweepCountView { charge: charge(), first: FirstShow::Animate }
                    }
                }
                {mini("Replay appear", move |_| *appear.write() += 1)}
                {mini("80 %", move |_| charge.set(Charge::Level(80)))}
                {mini("35 %", move |_| charge.set(Charge::Level(35)))}
            }
            div { class: "g-row g-row-top",
                for round in [reveal()] {
                    Specimen { key: "{round}", name: "Reveal", code: "Reveal {{ first: Animate }}",
                        Reveal { first: FirstShow::Animate,
                            for n in 1..=6 {
                                div { key: "{n}", class: "g-reveal-row", "Result {n}" }
                            }
                        }
                    }
                }
                {mini("Replay reveal", move |_| *reveal.write() += 1)}
            }
        }
    }
}

#[component]
fn StateSection() -> Element {
    let mut net = use_signal(|| Net::Off);
    let mut check = use_signal(|| Net::Joining);
    let mut failures = use_signal(|| 0u32);
    let mut bell = use_signal(|| Bell::Quiet);
    let mut seal = use_signal(|| (Seal::Open, Touch::Remote));
    let mut operation = use_signal(|| Operation::Idle);
    let stamp = move || EventStamp(failures());
    rsx! {
        Section { title: "Pending, Settle, Shake, Nudge", note: "A pending loop waits out PendingGrace (400 ms), steps every --t-pending-step and holds its still frame at its deadline (PendingCap, 10 s). Success fills or draws once and rests; a failure shakes once per new stamp and never for the same one; attention nudges once.",
            div { class: "g-row",
                Specimen { name: "Pending (Iterate) then Settle (Fill)", code: "use_pending, use_settle(Fill)",
                    NetView { net: net() }
                }
                {mini("Join", move |_| net.set(Net::Joining))}
                {mini("Joined", move |_| net.set(Net::Joined))}
                {mini("Off", move |_| net.set(Net::Off))}
            }
            div { class: "g-row",
                Specimen { name: "Spinner (Spin)", code: "Spinner {{ operation }}",
                    span { class: "g-spin-well", Spinner { kind: SpinnerKind::Spin, operation: operation() } }
                }
                Specimen { name: "Spinner (Breathe)",
                    span { class: "g-spin-well", Spinner { kind: SpinnerKind::Breathe, operation: operation() } }
                }
                {mini("Start", move |_| operation.set(Operation::Running(PendingToken::start(Deadline::cap()))))}
                {mini("Start, 3 s cap", move |_| operation.set(Operation::Running(PendingToken::start(Deadline::within(std::time::Duration::from_secs(3))))))}
                {mini("End", move |_| operation.set(Operation::Idle))}
            }
            div { class: "g-row",
                Specimen { name: "Settle (Check)", code: "use_settle(Check)", CheckView { net: check() } }
                {mini("Try", move |_| check.set(Net::Joining))}
                {mini("Succeed", move |_| check.set(Net::Joined))}
            }
            div { class: "g-row",
                Specimen { name: "Shake", code: "use_shake", ShakeView { net: Net::Failed(stamp()) } }
                {mini("Fail", move |_| *failures.write() += 1)}
                {mini("Same failure", move |_| failures.set(failures()))}
            }
            div { class: "g-row",
                Specimen { name: "Nudge", code: "use_nudge", NudgeView { bell: bell() } }
                {mini("Ask", move |_| {
                    let next = match bell() { Bell::Asking(EventStamp(n)) => n + 1, Bell::Quiet => 1 };
                    bell.set(Bell::Asking(EventStamp(next)));
                })}
            }
            div { class: "g-row",
                Specimen { name: "Settle (LockIn)", code: "press the disc: Touch::Contact springs",
                    div {
                        onclick: move |event| {
                            let next = match seal().0 { Seal::Sealed(EventStamp(n)) => n + 1, Seal::Open => 1 };
                            seal.set((Seal::Sealed(EventStamp(next)), Touch::from_event(&event)));
                        },
                        SealView { seal: seal().0, touch: seal().1 }
                    }
                }
                {mini("Seal from elsewhere", move |_| {
                    let next = match seal().0 { Seal::Sealed(EventStamp(n)) => n + 1, Seal::Open => 1 };
                    seal.set((Seal::Sealed(EventStamp(next)), Touch::Remote));
                })}
            }
        }
    }
}

#[component]
fn MorphSection() -> Element {
    let mut playing = use_signal(|| false);
    let mut bars = use_signal(|| 0usize);
    let mut muted = use_signal(|| Slashed::Off);
    let mut percent = use_signal(|| 79i32);
    const STRENGTHS: [Icon; 3] = [Icon::WifiLow, Icon::Wifi, Icon::WifiHigh];
    let play = move || if playing() { Icon::Pause } else { Icon::Play };
    rsx! {
        Section { title: "MorphGlyph, RollDigits", note: "A glyph gives way by its style: DownUp for a change of state, OffUp for the next action, CrossFade within one family, Slash drawn on and off. A readout's changed digits roll; the rest stand.",
            div { class: "g-row",
                Specimen { name: "DownUp", div { class: "g-detail g-detail-glyph", MorphGlyph { icon: play(), size: IconSize::Bar, style: MorphStyle::DownUp } } }
                Specimen { name: "OffUp", div { class: "g-detail g-detail-glyph", MorphGlyph { icon: play(), size: IconSize::Bar, style: MorphStyle::OffUp } } }
                {mini("Play / pause", move |_| playing.set(!playing()))}
                Specimen { name: "CrossFade", div { class: "g-detail g-detail-glyph", MorphGlyph { icon: STRENGTHS[bars() % 3], size: IconSize::Bar, style: MorphStyle::CrossFade } } }
                {mini("Next strength", move |_| *bars.write() += 1)}
                Specimen { name: "Slash", SlashView { slashed: muted() } }
                {mini("Mute", move |_| muted.set(match muted() { Slashed::Off => Slashed::On, Slashed::On => Slashed::Off }))}
            }
            div { class: "g-row",
                Specimen { name: "RollDigits", div { class: "g-detail g-figure", RollDigits { value: format!("{}%", percent()) } } }
                {mini("+1", move |_| *percent.write() += 1)}
                {mini("-7", move |_| *percent.write() -= 7)}
            }
        }
    }
}
