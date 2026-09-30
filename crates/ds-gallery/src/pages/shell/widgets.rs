//! The Overlays page's widgets (design/23 section 9): over the wallpaper, widget
//! cards on the desktop (the Widget material's card, light, tinted by the Space) and as tiles in
//! the notification center (a Popover panel, dark), every one a `WidgetCard`: the world clock
//! (analog dials on the desktop, digits in the tile) and the batteries at four levels, one
//! charging. Live, a button drains the small battery a tenth at a time: its provider sends a new
//! timeline, and the arc sweeps down as its percentage counts with it.

use crate::axes::Axes;
use crate::pages::Section;
use crate::pages::shell::widget_reference::cell;
use crate::wallpaper;
use dioxus::prelude::*;
use ds::ControlSize;
use ds::{Appearance, BatteryPower, Button, Ds, Fraction, Inject, Material, RootChrome, Theme};
use ds_shell::{
    BatteryEntry, BatteryWidget, ClockCity, ClockEntry, ClockTime, DayPhase, Device, Seconds,
    Timeline, WidgetCard, WidgetHost, WidgetMetrics, WidgetSize, WorldClockWidget,
};

const TAIPEI: ClockTime = ClockTime {
    hour: 9,
    minute: 50,
    second: Seconds::Hidden,
};

const LONDON: ClockTime = ClockTime {
    hour: 2,
    minute: 50,
    second: Seconds::Shown(42),
};

fn city(name: &str, time: ClockTime, phase: DayPhase, note: &str) -> ClockCity {
    ClockCity {
        name: name.to_owned(),
        time,
        phase,
        notes: vec![note.to_owned()],
    }
}

fn cities() -> ClockEntry {
    ClockEntry::Cities(vec![
        city("Taipei", TAIPEI, DayPhase::Day, "Today"),
        city("London", LONDON, DayPhase::Night, "-7HRS"),
    ])
}

fn batteries() -> BatteryEntry {
    BatteryEntry::Devices(vec![
        cell("Mouse", Device::Mouse, 80, BatteryPower::Battery),
        cell("Headphones", Device::Headphones, 450, BatteryPower::Battery),
        cell("Keyboard", Device::Keyboard, 1000, BatteryPower::Battery),
        cell("This computer", Device::Laptop, 150, BatteryPower::Charging),
    ])
}

/// The widgets section.
#[component]
pub fn Widgets() -> Element {
    rsx! {
        Section { title: "Widgets", note: "WidgetCard on the grid unit WidgetMetrics writes (widgets.desktop_cell_px 164, desktop_gap_px 16): Small one cell, Medium 2x1. Left, the desktop: the Widget material's card (its own 20 corner, padded 12) tinted by the Space, over the wallpaper, light. Right, the notification center: tiles on the Popover (--surface-2, a hairline, --r-tile 12, padded 12), no second tint, dark. Inside: WorldClockWidget (analog dials on the desktop, digits in the tile, bumping on each new minute) and BatteryWidget at 8 % (one red for low and critical), 45 %, 100 % and 15 % charging (never low while charging). Live: Drain sends a new timeline a tenth lower, and the arc sweeps down from the old level as its percentage counts down with it.",
            div { class: "g-wall g-widgets", style: "background-image:url(\"{wallpaper::uri()}\")",
                Host { theme: Theme::Light, host: WidgetHost::Desktop }
                Host { theme: Theme::Dark, host: WidgetHost::Tile }
            }
        }
    }
}

/// The cards for `host`, in `theme`.
#[component]
fn Host(theme: Theme, host: WidgetHost) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (accent, motion, blur) = {
        let axes = axes.read();
        (axes.accent, axes.motion, axes.blur)
    };
    let (material, chrome, class) = match host {
        WidgetHost::Desktop => (Material::Widget, RootChrome::Transparent, "g-widgets-desk"),
        WidgetHost::Tile => (Material::Popover, RootChrome::Painted, "g-widgets-center"),
    };
    rsx! {
        div { class,
            Ds {
                appearance: Appearance { theme, accent, motion },
                material,
                blur,
                chrome: Some(chrome),
                stylesheet: Inject::Host,
                div { class: "g-widgets-grid", style: WidgetMetrics::default().style_attr(),
                    Battery { host }
                    WidgetCard { widget: WorldClockWidget, timeline: Timeline::now(cities()), size: WidgetSize::Medium, host }
                    WidgetCard { widget: BatteryWidget, timeline: Timeline::now(batteries()), size: WidgetSize::Medium, host }
                }
            }
        }
    }
}

/// The small battery widget, its provider drained a tenth at a time by its button.
#[component]
fn Battery(host: WidgetHost) -> Element {
    let mut level = use_signal(|| Fraction(840));
    let entry = BatteryEntry::Devices(vec![cell(
        "This computer",
        Device::Laptop,
        level().0,
        BatteryPower::Battery,
    )]);
    rsx! {
        div { class: "g-col",
            WidgetCard { widget: BatteryWidget, timeline: Timeline::now(entry), size: WidgetSize::Small, host }
            Button { size: ControlSize::Mini, label: "Drain",
                onclick: move |_| level.set(Fraction(level().0.saturating_sub(100))) }
        }
    }
}
