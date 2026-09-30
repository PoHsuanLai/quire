//! The Widget looks page (design/23-WIDGETS.md): the widgets drawn flat, bright and measured
//! (section 2), in the page's scheme over a calm wallpaper, every card through the widget
//! contract (section 9) and tinted by the Space (settled 2026-09-27): the batteries with the
//! filled device glyphs, the world clock following the scheme, the month; one card on the bare
//! material beside its tinted twin for comparison; the filled device set on its own; and the
//! registry's placeholders, what a widget picker shows before any provider speaks.

use crate::axes::Axes;
use crate::pages::Section;
use crate::pages::shell::calendar::month_sample::{First, SEPTEMBER, sample, shift};
use crate::pages::shell::widget_reference::cell;
use crate::wallpaper;
use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::components::content::status::battery_state::BatteryPower;
use ds::prelude::*;
use ds::style::icon::render::IconPx;
use ds::style::tokens::label_hue::LabelHue;
use ds_shell::battery::device_glyph::Device;
use ds_shell::clock::kind::{ClockTime, DayPhase, Seconds};
use ds_shell::month_grid::data::WeekNumbers;
use ds_shell::prelude::*;
use ds_shell::tokens::widgets::WidgetMetrics;
use ds_shell::widget::battery::{BatteryEntry, BatteryWidget};
use ds_shell::widget::calendar::{EventLine, MonthFace, TodayLine};
use ds_shell::widget::calendar::{MonthEntry, MonthIntent, MonthWidget};
use ds_shell::widget::clock::{ClockCity, ClockEntry, WorldClockWidget};
use ds_shell::widget::kind::{CardTint, Lift};
use ds_shell::widget::timeline::Timeline;

/// The medium widget's four: critical, half, full, and low but charging.
fn devices() -> BatteryEntry {
    BatteryEntry::Devices(vec![
        cell("Mouse", Device::Mouse, 80, BatteryPower::Battery),
        cell("Headphones", Device::Headphones, 450, BatteryPower::Battery),
        cell("Keyboard", Device::Keyboard, 1000, BatteryPower::Battery),
        cell("This computer", Device::Laptop, 150, BatteryPower::Charging),
    ])
}

const fn at(hour: u8, minute: u8) -> ClockTime {
    ClockTime {
        hour,
        minute,
        second: Seconds::Shown(30),
    }
}

fn city(name: &str, time: ClockTime, phase: DayPhase, offset: &str) -> ClockCity {
    ClockCity {
        name: name.to_owned(),
        time,
        phase,
        notes: vec![offset.to_owned()],
    }
}

fn cities() -> ClockEntry {
    ClockEntry::Cities(vec![
        city("Taipei", at(10, 9), DayPhase::Day, "Today"),
        city("London", at(3, 9), DayPhase::Night, "-7HRS"),
        city("New York", at(22, 9), DayPhase::Night, "-12HRS"),
        city("Tokyo", at(11, 9), DayPhase::Day, "+1HRS"),
    ])
}

/// The page.
#[component]
pub fn WidgetLooksPage() -> Element {
    let small = BatteryEntry::Devices(vec![cell(
        "This computer",
        Device::Laptop,
        840,
        BatteryPower::Battery,
    )]);
    let taipei = ClockEntry::Cities(vec![city("Taipei", at(10, 9), DayPhase::Day, "Today")]);
    rsx! {
        Section { title: "Battery", note: "BatteryWidget on WidgetCard: a bright ring on a track of the plate darkened, the device's filled glyph in it, the percentage (display face 500) under it. Small: this computer at 84 %. Medium: 8 % and 15 % charging share one red for low and critical except while charging (the bolt in the ring's gap). Every card carries the Space's tint.",
            Wall {
                WidgetCard { widget: BatteryWidget, timeline: Timeline::now(small), size: WidgetSize::Small }
                WidgetCard { widget: BatteryWidget, timeline: Timeline::now(devices()), size: WidgetSize::Medium }
            }
        }
        Section { title: "World clock", note: "WorldClockWidget on WidgetCard: Small, one large dial with sixty ticks; Medium, four dials, Taipei and Tokyo by day (white), London and New York by night (dark), each with an orange seconds hand. The card follows the scheme: light here in the light scheme, dark in the dark.",
            Wall {
                WidgetCard { widget: WorldClockWidget, timeline: Timeline::now(taipei), size: WidgetSize::Small }
                WidgetCard { widget: WorldClockWidget, timeline: Timeline::now(cities()), size: WidgetSize::Medium }
            }
        }
        Section { title: "Calendar", note: "MonthWidget on WidgetCard in the reference's layouts (design/23 section 5.2): Small, the compact month filling the card at its own pitch; Medium, the today column (weekday, date, the next event, or No events today) beside the compact month at the small card's metrics; Large, the regular month at its own 32 px pitch, centred, over the day's events; last, the Large card as the notification center's tile with no events. On the desktop card the month title is --ink-soft (option (b), pending the user's pick); today's disc and the dots keep the accent. The steps send MonthIntent::Step to the provider.",
            Wall {
                MonthCard { size: WidgetSize::Small, host: WidgetHost::Desktop, busy: Busy::Events }
                MonthCard { size: WidgetSize::Medium, host: WidgetHost::Desktop, busy: Busy::Events }
                MonthCard { size: WidgetSize::Medium, host: WidgetHost::Desktop, busy: Busy::Free }
                MonthCard { size: WidgetSize::Large, host: WidgetHost::Desktop, busy: Busy::Events }
                MonthCard { size: WidgetSize::Large, host: WidgetHost::Tile, busy: Busy::Free }
            }
        }
        Section { title: "Space tint against the bare material", note: "The same entry twice: left, the card every widget gets (the Space's gradient over the Widget material at its frame alpha); right, the bare material, drawn by WidgetFrame with the tint CardTint::Material, for this comparison only.",
            Wall {
                WidgetCard { widget: BatteryWidget, timeline: Timeline::now(devices()), size: WidgetSize::Medium }
                WidgetFrame { size: WidgetSize::Medium, tint: CardTint::Material,
                    {BatteryWidget::view(&devices(), WidgetContext { size: WidgetSize::Medium, host: WidgetHost::Desktop, wake: Default::default(), act: None })}
                }
            }
        }
        Section { title: "Device glyphs", note: "DeviceGlyph: the filled device set for the battery rings (design/23 section 4.4), abstract solids on the Lucide 24 grid, one path each, holes cut by the even-odd rule; at 16 (the ring's) and 32.",
            div { class: "g-wl-devices",
                for device in Device::ALL.iter().copied() {
                    div { key: "{device.slug()}", class: "g-wl-device",
                        DeviceGlyph { device, size: IconSize::Base }
                        DeviceGlyph { device, size: IconSize::Px(IconPx(32)) }
                        span { class: "g-wl-device-name", "{device.slug()}" }
                    }
                }
            }
        }
        Section { title: "Pick up and drop", note: "Lift: a desktop widget picked up grows by --pickup (1.04) and trades its resting drop for --shadow-drag on --z-drag, over --t-quick at --e-out, and settles back the same way. WidgetSlotGuide: the footprint of the size at the snap cell, a quiet plate faded in, where the widget will land.",
            Wall {
                WidgetCard { widget: BatteryWidget, timeline: Timeline::now(devices()), size: WidgetSize::Medium, lift: Lift::Lifted }
                WidgetSlotGuide { size: WidgetSize::Small }
                WidgetSlotGuide { size: WidgetSize::Medium }
            }
        }
        Section { title: "Edit Widgets", note: "The desktop with its widgets placed from the right and Edit Widgets as a sheet at the bottom (Panel with PanelEdge::Bottom), never taller than half the desktop, so the top rows where a new widget lands stay in view. WidgetGallery over the registry: the widgets listed with their descriptions; the one looked at drawn once, at the one size it takes (no size picker); Add to Desktop and Add to Notification Center hand the host a WidgetEdit at the size the widget takes on that surface, which it applies to the layout it keeps as data and passes back; at the right, what is placed on each surface and Remove. Live.",
            crate::pages::shell::widget_edit::EditWidgetsStage {}
        }
    }
}

/// Whether the sample day has events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Busy {
    Events,
    Free,
}

/// The month widget at `size` in `host`, its provider stepping the month on each intent.
#[component]
fn MonthCard(size: WidgetSize, host: WidgetHost, busy: Busy) -> Element {
    let mut month = use_signal(|| SEPTEMBER);
    let event = |time: &str, title: &str, hue| EventLine {
        time: time.to_owned(),
        title: title.to_owned(),
        hue,
    };
    let events = match busy {
        Busy::Events => vec![
            event("10:00", "Design review", LabelHue::Blue),
            event("13:30", "Lunch with Mei", LabelHue::Green),
            event("17:00", "Climbing", LabelHue::Amber),
        ],
        Busy::Free => Vec::new(),
    };
    let entry = MonthEntry::Month(Box::new(MonthFace {
        today: Some(TodayLine {
            weekday: "Saturday".to_owned(),
            day: 26,
        }),
        events,
        no_events: "No events today".to_owned(),
        ..MonthFace::of(sample(month(), First::Monday), WeekNumbers::Hide)
    }));
    rsx! {
        WidgetCard { widget: MonthWidget, timeline: Timeline::now(entry), size, host,
            onintent: move |intent: MonthIntent| match intent {
                MonthIntent::Step(step) => month.set(shift(month(), step)),
            }
        }
    }
}

/// The wallpaper, with a desktop's Widget scope over it in the page's scheme.
#[component]
pub(super) fn Wall(children: Element) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (theme, accent, motion, blur, look) = {
        let axes = axes.read();
        (
            axes.theme,
            axes.accent,
            axes.motion,
            axes.blur,
            axes.look.clone(),
        )
    };
    let scheme = use_scope().scheme;
    rsx! {
        div { class: "g-wall g-wl-wall", style: "background-image:url(\"{wallpaper::calm_uri(scheme)}\")",
            Ds {
                appearance: Appearance { theme, accent, motion },
                look: SpaceLook { theme, ..look },
                material: Material::Widget,
                blur,
                chrome: Some(RootChrome::Transparent),
                stylesheet: Inject::Host,
                div { class: "g-wl-cards", style: WidgetMetrics::default().style_attr(), {children} }
            }
        }
    }
}
