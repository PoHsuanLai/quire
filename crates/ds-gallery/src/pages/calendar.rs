//! The Overlays page's calendar (design/04-COMPONENTS.md section 39; design/20 section 1.12;
//! sill Q180): a `MonthGrid` on a Popover panel of the Work Space over the wallpaper, light
//! without week numbers and dark with them. The month is laid out by the same stand-in for the
//! shell's grid the tests use (September 2026, today the 26th, three busy days and one on the
//! next month's padding); the header's buttons step it live, sliding each new month in, and the
//! dark panel's days are pressable and report the day picked. Last, the compact grid (sill
//! Q190): `MonthDensity::Auto` inside a small desktop `WidgetFrame`, a six-week August.

#[path = "../../../ds/tests/support/month_sample.rs"]
mod month_sample;

use super::Section;
use crate::axes::Axes;
use crate::wallpaper;
use dioxus::prelude::*;
use ds::{
    Appearance, CardAccent, DayKey, Ds, FrameTint, Grain, Inject, Material, MonthGrid, RootChrome,
    Step, Theme, WeekNumbers, WidgetFrame, WidgetMetrics, WidgetSize, default_look,
};
use month_sample::{AUGUST, First, SEPTEMBER, month as lay_out, sample, shift};

/// The small widget's today and busy days, so its August shows the disc and the dots.
const AUGUST_TODAY: DayKey = DayKey {
    year: 2026,
    month: 8,
    day: 14,
};
const AUGUST_BUSY: [DayKey; 3] = [
    DayKey {
        year: 2026,
        month: 8,
        day: 5,
    },
    AUGUST_TODAY,
    DayKey {
        year: 2026,
        month: 8,
        day: 27,
    },
];

/// The calendar section.
#[component]
pub fn Calendar() -> Element {
    rsx! {
        Section { title: "Calendar", note: "A MonthGrid on a Popover panel, light and dark: the month's title in the data face, upper, in --accent; the previous and next Tool IconButtons (live: a later month slides in from the right, an earlier one from the left, slide-r/slide-l at --t-big --e-spring, once per change); weekday initials in the data face at --fs-micro, tracked; seven 32 px columns of days, the neighbours' days in --ink-faint, today on an --accent disc in --accent-ink, a busy day's 4 px dot (--ink-soft on a neighbour's day). Right: week numbers shown (WeekNumbers::Show, a quieter 24 px column) and pressable days (onpick) that report the day picked. Last: MonthDensity::Auto inside a Small desktop WidgetFrame resolves to the compact grid (sill Q190): a six-week August in 126 x 132 of the frame's 132 x 132, seven 18 px columns, the title at --fs-micro, 14 px glyph step buttons, 16 px today disc, 3 px dots, no week column.",
            div { class: "g-wall g-polish-cards", style: "background-image:url(\"{wallpaper::uri()}\")",
                Month { theme: Theme::Light, weeks: WeekNumbers::Hide }
                Month { theme: Theme::Dark, weeks: WeekNumbers::Show }
                SmallWidget {}
            }
        }
    }
}

/// One calendar panel in `theme`.
#[component]
fn Month(theme: Theme, weeks: WeekNumbers) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (accent, motion, blur) = {
        let axes = axes.read();
        (axes.accent, axes.motion, axes.blur)
    };
    let mut month = use_signal(|| SEPTEMBER);
    let mut picked = use_signal(|| None::<DayKey>);
    let onpick = match weeks {
        WeekNumbers::Show => Some(EventHandler::new(move |day: DayKey| picked.set(Some(day)))),
        WeekNumbers::Hide => None,
    };
    let note = picked().map(|day| format!("Picked {}-{:02}-{:02}", day.year, day.month, day.day));
    rsx! {
        div { class: "g-cc",
            Ds {
                appearance: Appearance { theme, accent, motion },
                look: ds::SpaceLook { theme, ..default_look(0, Grain(35), CardAccent::SpaceHue) },
                material: Material::Popover,
                blur,
                stylesheet: Inject::Host,
                chrome: Some(RootChrome::Painted),
                frame: Some(FrameTint::Tinted),
                div { class: "g-cc-body",
                    MonthGrid {
                        data: sample(month(), First::Monday),
                        weeks,
                        onstep: move |step: Step| month.set(shift(month(), step)),
                        onpick,
                    }
                    if let Some(note) = note {
                        p { class: "g-note", "{note}" }
                    }
                }
            }
        }
    }
}

/// The compact grid: `Auto` in a small desktop widget, stepping live.
#[component]
fn SmallWidget() -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (accent, motion, blur) = {
        let axes = axes.read();
        (axes.accent, axes.motion, axes.blur)
    };
    let mut month = use_signal(|| AUGUST);
    rsx! {
        div { class: "g-col",
            Ds {
                appearance: Appearance { theme: Theme::Light, accent, motion },
                material: Material::Widget,
                blur,
                chrome: Some(RootChrome::Transparent),
                stylesheet: Inject::Host,
                div { style: WidgetMetrics::default().style_attr(),
                    WidgetFrame { size: WidgetSize::Small,
                        MonthGrid {
                            data: lay_out(month(), First::Monday, AUGUST_TODAY, &AUGUST_BUSY),
                            onstep: move |step: Step| month.set(shift(month(), step)),
                        }
                    }
                }
            }
        }
    }
}
