//! Details, the control center's detail-pane rows (design/26-DETAILS.md 5.2.2, 5.2.3, 5.2.8):
//! a network joined, a device connected, an output switched, each through its `RowPhase`.

use super::details::{Cell, mini};
use dioxus::prelude::*;
use ds::detail::EventStamp;
use ds::{Fraction, Icon, RowDisc, RowPhase, RowTrailing, RowWork, SettingsRow, Switch, Text};

/// A row's operation over a run of stamps: each press mints the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Run {
    phase: RowPhase,
    stamp: u32,
}

impl Run {
    fn start() -> Run {
        Run {
            phase: RowPhase::Rest,
            stamp: 0,
        }
    }

    fn pending(self) -> Run {
        let stamp = self.stamp + 1;
        Run {
            phase: RowPhase::Pending(EventStamp(stamp)),
            stamp,
        }
    }

    fn succeeded(self) -> Run {
        Run {
            phase: RowPhase::Succeeded(EventStamp(self.stamp)),
            ..self
        }
    }

    fn failed(self) -> Run {
        let stamp = self.stamp + 1;
        Run {
            phase: RowPhase::Failed(EventStamp(stamp)),
            stamp,
        }
    }

    /// Whether the item ended in use.
    fn in_use(self) -> Switch {
        match self.phase {
            RowPhase::Succeeded(_) => Switch::On,
            RowPhase::Rest | RowPhase::Pending(_) | RowPhase::Failed(_) => Switch::Off,
        }
    }
}

/// The words under a row's title (R8: the still state carries the moment).
fn detail(run: Run, done: &str) -> Option<Text> {
    match run.phase {
        RowPhase::Rest => None,
        RowPhase::Pending(_) => Some(Text::from("Connecting…")),
        RowPhase::Succeeded(_) => Some(Text::from(done)),
        RowPhase::Failed(_) => Some(Text::from("Couldn't connect")),
    }
}

#[component]
pub fn NetworkRows() -> Element {
    let mut home = use_signal(|| Run::start().pending().succeeded());
    let mut cafe = use_signal(Run::start);
    let disc = |run: Run| match run.in_use() {
        Switch::On => RowDisc::On,
        Switch::Off => RowDisc::Off,
    };
    rsx! {
        Cell { name: "Network rows", code: "SettingsRow {{ phase, work: RowWork::Trailing, disc }}",
            controls: rsx! {
                {mini("Join Café", move |_| cafe.set(cafe().pending()))}
                {mini("Joined", move |_| cafe.set(cafe().succeeded()))}
                {mini("Wrong password", move |_| cafe.set(cafe().failed()))}
            },
            div { class: "g-detail g-detail-list",
                SettingsRow { glyph: Some(Icon::Wifi), title: "Home", detail: detail(home(), "Connected"), phase: home().phase, work: RowWork::Trailing, disc: disc(home()), trailing: RowTrailing::Glyph(Icon::Lock), onclick: move |_| home.set(home().pending()) }
                SettingsRow { glyph: Some(Icon::Wifi), title: "Café", detail: detail(cafe(), "Connected"), phase: cafe().phase, work: RowWork::Trailing, disc: disc(cafe()), trailing: RowTrailing::Glyph(Icon::Lock), onclick: move |_| cafe.set(cafe().pending()) }
            }
        }
    }
}

#[component]
pub fn DeviceRows() -> Element {
    let mut phones = use_signal(|| Run::start().pending().succeeded());
    let mut mouse = use_signal(Run::start);
    let row = |run: Run, level: u16| match run.in_use() {
        Switch::On => (RowDisc::On, RowTrailing::Battery(Fraction(level))),
        Switch::Off => (RowDisc::Off, RowTrailing::None),
    };
    let (phones_disc, phones_trail) = row(phones(), 840);
    let (mouse_disc, mouse_trail) = row(mouse(), 420);
    rsx! {
        Cell { name: "Device rows", code: "SettingsRow {{ phase, disc, trailing: RowTrailing::Battery }}",
            controls: rsx! {
                {mini("Connect mouse", move |_| mouse.set(mouse().pending()))}
                {mini("Connected", move |_| mouse.set(mouse().succeeded()))}
                {mini("Fail", move |_| mouse.set(mouse().failed()))}
            },
            div { class: "g-detail g-detail-list",
                SettingsRow { glyph: Some(Icon::Headphones), title: "Headphones", detail: detail(phones(), "Connected"), phase: phones().phase, disc: phones_disc, trailing: phones_trail, onclick: move |_| phones.set(phones().pending()) }
                SettingsRow { glyph: Some(Icon::Mouse), title: "Mouse", detail: detail(mouse(), "Connected"), phase: mouse().phase, disc: mouse_disc, trailing: mouse_trail, onclick: move |_| mouse.set(mouse().pending()) }
            }
        }
    }
}

#[component]
pub fn OutputRows() -> Element {
    let mut speakers = use_signal(Run::start);
    let chosen = move || speakers().in_use();
    let other = move || match chosen() {
        Switch::On => Switch::Off,
        Switch::Off => Switch::On,
    };
    rsx! {
        Cell { name: "Output rows", code: "SettingsRow {{ phase, trailing: RowTrailing::Check }}",
            controls: rsx! {
                {mini("Switch to Speakers", move |_| speakers.set(speakers().pending()))}
                {mini("Switched", move |_| speakers.set(speakers().succeeded()))}
            },
            div { class: "g-detail g-detail-list",
                SettingsRow { glyph: Some(Icon::Monitor), title: "Display Audio", detail: None, trailing: RowTrailing::Check(other()), onclick: move |_| speakers.set(Run::start()) }
                SettingsRow { glyph: Some(Icon::Speaker), title: "Speakers", detail: detail(speakers(), "In use"), phase: speakers().phase, trailing: RowTrailing::Check(chosen()), onclick: move |_| speakers.set(speakers().pending()) }
            }
        }
    }
}
