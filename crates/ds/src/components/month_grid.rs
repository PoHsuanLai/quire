//! MonthGrid: a month of days in seven columns, the calendar widget's month (design/04-COMPONENTS.md
//! section 39; design/20-SURFACES.md section 1.12; sill Q180). The shell computes the month;
//! this draws it: a header with the month's title and, when the caller steps months, the
//! previous and next `IconButton { Tool }`; the weekday heads; then the weeks, optionally led by
//! their ISO week numbers, the neighbours' days quieter, today on the accent disc and a busy
//! day's dot. A change of month slides the weeks in once (`month_grid_weeks`).
//!
//! Two densities (sill Q190): the regular grid, and a compact one that fits a small desktop
//! widget's 140 x 140 content box, which the regular grid (224 x 254 for six weeks) overflows.
//! `MonthDensity::Auto` picks between them by the enclosing `WidgetFrame`.

use crate::components::month_grid_data::{DayKey, MonthGridData, MonthKey, Step, WeekNumbers};
use crate::components::month_grid_density::{Drawn, MonthDensity};
use crate::components::month_grid_header::header;
use crate::components::month_grid_weeks::{MonthSlide, MonthWeeks};
use crate::components::text_runs::{Text, text};
use crate::components::widget_scope::use_enclosing_frame;
use dioxus::prelude::*;

/// A month. `data` is the month as the shell laid it out; `weeks` whether each row leads with
/// its ISO week. With `onstep` the header ends in the previous and next buttons, which call it
/// with their `Step`; without it the header is the title alone. With `onpick` every day is a
/// `button` that calls it with the day's key; without it the days are plain text.
///
/// Changing `data.month` slides the new month's weeks in once: from the right for a later
/// month (`slide-r`), from the left for an earlier one (`slide-l`); the first month drawn and
/// a render that keeps the month play nothing.
///
/// `density` (`Auto`) is written as `data-density`: `Auto` draws compact inside a
/// `WidgetFrame { size: Small }` and regular inside a Medium or Large one or outside any frame;
/// `Regular` and `Compact` force it. The compact grid is seven 20 px columns of 19 px rows, a
/// 14 px header of `--fs-micro` title and 14 px glyph buttons, 10 px heads, today on a 20 px
/// disc (twice `--fs-caption`, sill Q361) and a 3 px dot: 140 x 138 for a six-week month,
/// measured on Blitz (the `month_grid_density` harness test), inside the small frame's
/// 140 x 140. It never draws
/// week numbers, whatever `weeks` says: a week column would not fit.
#[component]
pub fn MonthGrid(
    data: MonthGridData,
    #[props(default)] weeks: WeekNumbers,
    #[props(default)] density: MonthDensity,
    #[props(default)] onstep: Option<EventHandler<Step>>,
    #[props(default)] onpick: Option<EventHandler<DayKey>>,
) -> Element {
    let slide = use_month_slide(data.month);
    let drawn = density.resolve(use_enclosing_frame());
    let weeks = shown_weeks(weeks, drawn);
    let label = data.title.plain_text();
    rsx! {
        div { class: "ds-month", "data-weeks": weeks.slug(), "data-density": drawn.slug(), "aria-label": "{label}",
            {header(&data.title, drawn, onstep)}
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

/// The week numbers drawn: the caller's, except never at the compact density.
fn shown_weeks(weeks: WeekNumbers, drawn: Drawn) -> WeekNumbers {
    match drawn {
        Drawn::Regular => weeks,
        Drawn::Compact => WeekNumbers::Hide,
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
