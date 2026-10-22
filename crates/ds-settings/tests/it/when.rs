//! A list cell's modified time: today with the time, yesterday, a short date.

use ds_settings::{ResolvedHours, UnixSecs, UtcOffset, list_date};

/// 2026-10-22 14:05:00 UTC.
const NOW: i64 = 1_792_677_900;

#[test]
fn a_modified_time_is_worded_by_how_long_ago_its_day_was() {
    /// name, seconds before now, utc offset, hours, wording.
    type Case = (&'static str, i64, i32, ResolvedHours, &'static str);
    const CASES: &[Case] = &[
        (
            "this morning",
            3 * 3600,
            0,
            ResolvedHours::H24,
            "Today at 11:05",
        ),
        (
            "a 12-hour clock",
            3600,
            0,
            ResolvedHours::H12,
            "Today at 1:05 PM",
        ),
        (
            "midnight is 12 AM",
            14 * 3600 + 300,
            0,
            ResolvedHours::H12,
            "Today at 12:00 AM",
        ),
        (
            "the day before",
            24 * 3600,
            0,
            ResolvedHours::H24,
            "Yesterday",
        ),
        (
            "a week back",
            7 * 24 * 3600,
            0,
            ResolvedHours::H24,
            "Oct 15, 2026",
        ),
        (
            "an earlier month",
            236 * 24 * 3600,
            0,
            ResolvedHours::H24,
            "Feb 28, 2026",
        ),
        (
            "the zone moves the day",
            15 * 3600,
            36_000,
            ResolvedHours::H24,
            "Yesterday",
        ),
        (
            "in the future",
            -2 * 24 * 3600,
            0,
            ResolvedHours::H24,
            "Oct 24, 2026",
        ),
    ];
    for &(name, before, offset, hours, want) in CASES {
        let got = list_date(
            UnixSecs(NOW - before),
            UnixSecs(NOW),
            UtcOffset(offset),
            hours,
        );
        assert_eq!(got, want, "{name}");
    }
}
