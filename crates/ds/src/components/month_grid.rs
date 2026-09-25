//! MonthGrid: a month of days in seven columns, the calendar widget's month (design/04-COMPONENTS.md
//! section 39; design/20-SURFACES.md section 1.12; sill Q180). The shell computes the month;
//! this draws it: a header with the month's title and, when the caller steps months, the
//! previous and next `IconButton { Tool }`; the weekday heads; then the weeks, optionally led by
//! their ISO week numbers, the neighbours' days quieter, today on the accent disc and a busy
//! day's dot. A change of month slides the weeks in once (`month_grid_weeks`).

use crate::components::icon_button::{IconButton, IconButtonVariant};
use crate::components::month_grid_data::{DayKey, MonthGridData, MonthKey, Step, WeekNumbers};
use crate::components::month_grid_weeks::{MonthSlide, MonthWeeks};
use crate::components::text_runs::{Text, text};
use crate::icon::Icon;
use dioxus::prelude::*;

/// A month. `data` is the month as the shell laid it out; `weeks` whether each row leads with
/// its ISO week. With `onstep` the header ends in the previous and next buttons, which call it
/// with their `Step`; without it the header is the title alone. With `onpick` every day is a
/// `button` that calls it with the day's key; without it the days are plain text.
///
/// Changing `data.month` slides the new month's weeks in once: from the right for a later
/// month (`slide-r`), from the left for an earlier one (`slide-l`); the first month drawn and
/// a render that keeps the month play nothing.
#[component]
pub fn MonthGrid(
    data: MonthGridData,
    #[props(default)] weeks: WeekNumbers,
    #[props(default)] onstep: Option<EventHandler<Step>>,
    #[props(default)] onpick: Option<EventHandler<DayKey>>,
) -> Element {
    let slide = use_month_slide(data.month);
    let label = data.title.plain_text();
    rsx! {
        div { class: "ds-month", "data-weeks": weeks.slug(), "aria-label": "{label}",
            div { class: "ds-month-header",
                span { class: "ds-month-title", {text(&data.title)} }
                if let Some(onstep) = onstep {
                    {step_button(Step::Previous, Icon::ChevronLeft, onstep)}
                    {step_button(Step::Next, Icon::ChevronRight, onstep)}
                }
            }
            {heads(&data.heads, weeks)}
            // A keyed list of one: a key only remounts inside a list, so a new month is a new
            // body (and a new slide) while a render that keeps the month keeps the body.
            for month in std::iter::once(data.month) {
                MonthWeeks {
                    key: "{month.slug()}",
                    weeks: data.weeks.clone(),
                    numbers: weeks,
                    slide,
                    onpick,
                }
            }
        }
    }
}

/// The weekday heads, behind an empty week-number cell when the numbers show.
fn heads(heads: &[Text; 7], weeks: WeekNumbers) -> Element {
    rsx! {
        div { class: "ds-month-row", "data-row": "heads",
            if weeks == WeekNumbers::Show {
                span { class: "ds-month-week" }
            }
            for (at , head) in heads.iter().enumerate() {
                span { key: "{at}", class: "ds-month-head", {text(head)} }
            }
        }
    }
}

/// One of the header's step buttons.
fn step_button(step: Step, icon: Icon, onstep: EventHandler<Step>) -> Element {
    rsx! {
        IconButton {
            variant: IconButtonVariant::Tool,
            icon,
            label: step.label(),
            tooltip: step.label().to_owned(),
            onclick: move |_| onstep.call(step),
        }
    }
}

/// Which way the weeks slide in this render: the order of `month` against the month drawn
/// before, held until the month changes again. The first month drawn is still.
fn use_month_slide(month: MonthKey) -> MonthSlide {
    let mut seen = use_hook(|| CopyValue::new((month, MonthSlide::Still)));
    let (last, slide) = *seen.peek();
    let now = MonthSlide::between(last, month).unwrap_or(slide);
    seen.set((month, now));
    now
}
