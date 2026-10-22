//! How a list cell words a file's modified time, the way Finder's list view does: "Today at
//! 14:05", "Yesterday", or a short date. Pure: the caller gives the instants, the zone's offset
//! and the clock ([`ResolvedHours`], from `region.hours`); nothing here reads a clock.

use crate::region::ResolvedHours;

/// Whole seconds since 1970-01-01 00:00 UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnixSecs(pub i64);

/// How far the local zone is ahead of UTC, in whole seconds (negative west of Greenwich). The
/// offset at `now` is used for both instants, so a change of daylight time between them can move
/// a time by an hour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UtcOffset(pub i32);

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

const DAY: i64 = 86_400;

/// A local calendar day and the minute of that day.
struct Local {
    days: i64,
    year: i64,
    month: usize,
    day: i64,
    minute: i64,
}

impl Local {
    fn of(at: UnixSecs, offset: UtcOffset) -> Local {
        let local = at.0 + i64::from(offset.0);
        let days = local.div_euclid(DAY);
        let (year, month, day) = civil(days);
        Local {
            days,
            year,
            month,
            day,
            minute: local.rem_euclid(DAY) / 60,
        }
    }
}

/// The civil date of `days` since 1970-01-01 (Howard Hinnant's `civil_from_days`); the month is
/// an index from 0.
fn civil(days: i64) -> (i64, usize, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, usize::try_from(month - 1).unwrap_or(0), day)
}

/// The time of day: `14:05` on a 24-hour clock, `2:05 PM` on a 12-hour one.
fn time_of_day(minute: i64, hours: ResolvedHours) -> String {
    let (hour, minute) = (minute / 60, minute % 60);
    match hours {
        ResolvedHours::H24 => format!("{hour:02}:{minute:02}"),
        ResolvedHours::H12 => {
            let shown = match hour % 12 {
                0 => 12,
                other => other,
            };
            let half = if hour < 12 { "AM" } else { "PM" };
            format!("{shown}:{minute:02} {half}")
        }
    }
}

/// `modified` as a list cell words it at `now`: "Today at 14:05" for the same local day,
/// "Yesterday" for the day before, otherwise a short date with its year ("Oct 5, 2026"). A time
/// in the future is a date too. Month names are English until the desktop has a locale seam.
pub fn list_date(
    modified: UnixSecs,
    now: UnixSecs,
    offset: UtcOffset,
    hours: ResolvedHours,
) -> String {
    let (then, today) = (Local::of(modified, offset), Local::of(now, offset));
    match today.days - then.days {
        0 => format!("Today at {}", time_of_day(then.minute, hours)),
        1 => "Yesterday".to_owned(),
        _ => format!("{} {}, {}", MONTHS[then.month], then.day, then.year),
    }
}
