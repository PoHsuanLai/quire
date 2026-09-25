//! The words a `MonthGrid` is drawn from (design/04-COMPONENTS.md section 39; sill Q180). The
//! shell computes the month; these mirror its grid field for field, with the civil dates as its
//! calendar library gives them (`i16` year, `i8` month and day), so its mapping is a plain
//! `From` and quire needs no calendar of its own.

use crate::components::text_runs::Text;

/// A calendar month: which month a grid shows. Ordered, so a change of month knows its way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MonthKey {
    /// The year.
    pub year: i16,
    /// 1-12.
    pub month: i8,
}

impl MonthKey {
    /// `2026-09`: the key the grid's weeks are mounted under.
    pub(crate) fn slug(self) -> String {
        format!("{:04}-{:02}", self.year, self.month)
    }
}

/// A civil date: which day a cell is, handed back to `onpick`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DayKey {
    /// The year.
    pub year: i16,
    /// 1-12.
    pub month: i8,
    /// 1-31: also the cell's label.
    pub day: i8,
}

/// An ISO 8601 week number, 1-53: the week of the row's Thursday.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IsoWeek(pub i8);

/// Whether a cell's day is in the month shown or pads the first or last week.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DayPlace {
    /// The previous month's, before the first: drawn quieter.
    Before,
    /// The month shown.
    InMonth,
    /// The next month's, after the last: drawn quieter.
    After,
}

impl DayPlace {
    /// The `data-place` word.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            DayPlace::Before => "before",
            DayPlace::InMonth => "in",
            DayPlace::After => "after",
        }
    }
}

/// Whether a cell is today: today sits on the accent disc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DayMark {
    /// Today: `aria-current="date"`.
    Today,
    /// Any other day.
    Plain,
}

impl DayMark {
    /// `aria-current`: written only on today.
    pub(crate) fn aria_current(self) -> Option<&'static str> {
        match self {
            DayMark::Today => Some("date"),
            DayMark::Plain => None,
        }
    }
}

/// Whether a day has an event: a busy day carries a dot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Eventful {
    /// At least one event.
    Busy,
    /// None.
    Free,
}

impl Eventful {
    /// The `data-events` word.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            Eventful::Busy => "busy",
            Eventful::Free => "free",
        }
    }
}

/// One cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MonthDay {
    /// The date; its day is the label.
    pub key: DayKey,
    /// In the month shown, or a neighbour's.
    pub place: DayPlace,
    /// Today or not.
    pub mark: DayMark,
    /// Whether it has an event.
    pub events: Eventful,
}

/// One row: its ISO week and its seven days, from the first weekday.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MonthWeek {
    /// The row's ISO week, drawn only with `WeekNumbers::Show`.
    pub number: IsoWeek,
    /// Seven days, the first weekday first.
    pub days: [MonthDay; 7],
}

/// A month laid out in weeks, with the words the shell formatted for it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MonthGridData {
    /// Which month: the grid's identity, and the order a change slides by.
    pub month: MonthKey,
    /// The header's words: "September 2026".
    pub title: Text,
    /// The column heads, starting at the first weekday: weekday initials.
    pub heads: [Text; 7],
    /// As many rows as the month touches (four to six), padded with the neighbours' days.
    pub weeks: Vec<MonthWeek>,
}

/// Whether the grid leads each row with its ISO week (`calendar.week_numbers`, design/22
/// section 3.21).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeekNumbers {
    /// Seven columns only.
    #[default]
    Hide,
    /// A quieter column of week numbers first.
    Show,
}

impl WeekNumbers {
    /// The `data-weeks` word.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            WeekNumbers::Hide => "hide",
            WeekNumbers::Show => "show",
        }
    }
}

/// Which way the header's buttons step the month.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Step {
    /// The month before.
    Previous,
    /// The month after.
    Next,
}

impl Step {
    /// The button's name.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Step::Previous => "Previous month",
            Step::Next => "Next month",
        }
    }
}
