//! DatePicker: the textual style with a date, a time and both, the graphical style, at each size,
//! disabled.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds_shell::date_picker::model::{DateValue, Elements, PickerStyle, TimeOfDay};
use ds_shell::month_grid::data::DayKey;
use ds_shell::prelude::*;

/// The moment the pickers start on.
fn start() -> DateValue {
    DateValue {
        day: DayKey {
            year: 2026,
            month: 9,
            day: 30,
        },
        time: TimeOfDay {
            hour: 14,
            minute: 41,
        },
    }
}

/// The DatePicker section.
#[component]
pub fn DatePickerSection() -> Element {
    let mut date = use_signal(start);
    let mut both = use_signal(start);
    let mut time = use_signal(start);
    let mut grid = use_signal(start);
    rsx! {
        Section { title: "DatePicker", note: "NSDatePicker: the textual style selects a segment on a press, Up and Down or the stepper change it (December steps to January), Left and Right move between segments; the graphical style is the month grid, its arrows stepping months and a press picking the day, above the time field.",
            div { class: "g-row",
                for size in ControlSize::ALL.iter().copied() {
                    Specimen { key: "{size.slug()}", name: format!("textual, date, {}", size.slug()),
                        DatePicker { label: "Date", value: date(), size, onchange: move |next| date.set(next) }
                    }
                }
            }
            div { class: "g-row",
                Specimen { name: "textual, date and time".to_owned(),
                    DatePicker { label: "Date and time", value: both(), elements: Elements::DateAndTime, onchange: move |next| both.set(next) }
                }
                Specimen { name: "textual, time".to_owned(),
                    DatePicker { label: "Time", value: time(), elements: Elements::Time, onchange: move |next| time.set(next) }
                }
                Specimen { name: "disabled".to_owned(),
                    DatePicker { label: "Date", value: start(), availability: Availability::Disabled, onchange: |_| {} }
                }
            }
            div { class: "g-row g-row-top",
                Specimen { name: "graphical, date and time".to_owned(),
                    div { style: "width:260px",
                        DatePicker { label: "Date and time", value: grid(), style: PickerStyle::Graphical, elements: Elements::DateAndTime, onchange: move |next| grid.set(next) }
                    }
                }
                Specimen { name: "graphical, date".to_owned(),
                    div { style: "width:260px",
                        DatePicker { label: "Date", value: grid(), style: PickerStyle::Graphical, onchange: move |next| grid.set(next) }
                    }
                }
            }
        }
    }
}
