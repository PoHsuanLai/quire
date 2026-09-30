//! Details, the control center's detail-pane rows (design/26-DETAILS.md 5.2.2, 5.2.3, 5.2.8; design/30
//! section 2.9): a network joined, a device connected, an output switched. A row working on its
//! item is `Availability::Busy`: it takes no press and shows the small spinner where its accessory
//! is; when the work ends it is enabled again with no flourish and shows its accessory.

use crate::pages::details::overview::{Cell, mini};
use dioxus::prelude::*;
use ds::{
    Accessory, Availability, BatteryState, Check, Fraction, Icon, List, ListItem, Row, RowLeading,
    RowState, Selection, TextLine,
};

/// Where an operation on a row's item stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Run {
    Rest,
    Working,
    Done,
    Failed,
}

impl Run {
    /// The row's availability: working is busy.
    fn availability(self) -> Availability {
        match self {
            Run::Working => Availability::Busy,
            Run::Rest | Run::Done | Run::Failed => Availability::Enabled,
        }
    }

    /// Whether the item ended in use.
    fn in_use(self) -> Check {
        match self {
            Run::Done => Check::On,
            Run::Rest | Run::Working | Run::Failed => Check::Off,
        }
    }

    /// The words under a row's title (R8: the still state carries the reason).
    fn detail(self, done: &str) -> Option<TextLine> {
        match self {
            Run::Rest => None,
            Run::Working => Some(TextLine::from("Connecting…")),
            Run::Done => Some(TextLine::from(done)),
            Run::Failed => Some(TextLine::from("Couldn't connect")),
        }
    }
}

/// A row of the pane.
fn pane_row(
    key: &'static str,
    leading: RowLeading,
    run: Run,
    done: &str,
    accessory: Accessory,
    press: EventHandler<()>,
) -> ListItem<&'static str> {
    ListItem::row(
        key,
        key,
        rsx! {
            Row {
                title: key,
                detail: run.detail(done),
                leading,
                accessory,
                state: RowState { availability: run.availability(), selection: Selection::Unselected, ..RowState::default() },
                size: ds::RowSize::Settings,
                onclick: move |_| press.call(()),
            }
        },
    )
}

fn disc(run: Run) -> Selection {
    match run.in_use() {
        Check::On => Selection::Selected,
        Check::Off | Check::Mixed => Selection::Unselected,
    }
}

#[component]
pub fn NetworkRows() -> Element {
    let mut home = use_signal(|| Run::Done);
    let mut cafe = use_signal(|| Run::Rest);
    rsx! {
        Cell { name: "Network rows", code: "Row {{ state.availability: Busy, leading: Disc }}",
            controls: rsx! {
                {mini("Join Café", move |_| cafe.set(Run::Working))}
                {mini("Joined", move |_| cafe.set(Run::Done))}
                {mini("Wrong password", move |_| cafe.set(Run::Failed))}
            },
            div { class: "g-detail g-detail-list",
                List::<&'static str> {
                    label: "Networks",
                    items: vec![
                        pane_row("Home", RowLeading::Disc(Icon::Wifi, disc(home())), home(), "Connected", Accessory::Glyph(Icon::Lock), EventHandler::new(move |()| home.set(Run::Working))),
                        pane_row("Café", RowLeading::Disc(Icon::Wifi, disc(cafe())), cafe(), "Connected", Accessory::Glyph(Icon::Lock), EventHandler::new(move |()| cafe.set(Run::Working))),
                    ],
                }
            }
        }
    }
}

#[component]
pub fn DeviceRows() -> Element {
    let mut phones = use_signal(|| Run::Done);
    let mut mouse = use_signal(|| Run::Rest);
    let battery = |run: Run, level: u16| match run.in_use() {
        Check::On => Accessory::Battery(BatteryState {
            level: Fraction(level),
            ..BatteryState::default()
        }),
        Check::Off | Check::Mixed => Accessory::None,
    };
    rsx! {
        Cell { name: "Device rows", code: "Row {{ accessory: Accessory::Battery }}",
            controls: rsx! {
                {mini("Connect mouse", move |_| mouse.set(Run::Working))}
                {mini("Connected", move |_| mouse.set(Run::Done))}
                {mini("Fail", move |_| mouse.set(Run::Failed))}
            },
            div { class: "g-detail g-detail-list",
                List::<&'static str> {
                    label: "Devices",
                    items: vec![
                        pane_row("Headphones", RowLeading::Disc(Icon::Headphones, disc(phones())), phones(), "Connected", battery(phones(), 840), EventHandler::new(move |()| phones.set(Run::Working))),
                        pane_row("Mouse", RowLeading::Disc(Icon::Mouse, disc(mouse())), mouse(), "Connected", battery(mouse(), 420), EventHandler::new(move |()| mouse.set(Run::Working))),
                    ],
                }
            }
        }
    }
}

#[component]
pub fn OutputRows() -> Element {
    let mut speakers = use_signal(|| Run::Rest);
    rsx! {
        Cell { name: "Output rows", code: "Row {{ accessory: Accessory::Check }}",
            controls: rsx! {
                {mini("Switch to Speakers", move |_| speakers.set(Run::Working))}
                {mini("Switched", move |_| speakers.set(Run::Done))}
            },
            div { class: "g-detail g-detail-list",
                List::<&'static str> {
                    label: "Outputs",
                    items: vec![
                        pane_row("Display Audio", RowLeading::Icon(Icon::Monitor), Run::Rest, "", Accessory::Check(speakers().in_use().flipped()), EventHandler::new(move |()| speakers.set(Run::Rest))),
                        pane_row("Speakers", RowLeading::Icon(Icon::Speaker), speakers(), "In use", Accessory::Check(speakers().in_use()), EventHandler::new(move |()| speakers.set(Run::Working))),
                    ],
                }
            }
        }
    }
}
