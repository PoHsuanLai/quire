//! Wall time as plain data, so the model stays pure and a test names the instant.

use serde::{Deserialize, Deserializer, Serialize};
use std::time::Duration;

/// Seconds since the Unix epoch.
///
/// Written as a number. Read from a number, or from an RFC 3339 string (`2026-10-08T01:02:03Z`,
/// with optional fraction and offset), which is how mailo's `today.json` stored `last_opened`
/// before the kit, so an existing file keeps its entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct Epoch(pub i64);

#[derive(Deserialize)]
#[serde(untagged)]
enum EpochWire {
    Seconds(i64),
    Text(String),
}

impl<'de> Deserialize<'de> for Epoch {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match EpochWire::deserialize(deserializer)? {
            EpochWire::Seconds(secs) => Ok(Epoch(secs)),
            EpochWire::Text(text) => rfc3339(&text)
                .map(Epoch)
                .ok_or_else(|| serde::de::Error::custom("not a unix time or an RFC 3339 time")),
        }
    }
}

/// `YYYY-MM-DDTHH:MM:SS[.fraction](Z|+HH:MM|-HH:MM)` as unix seconds; the fraction is dropped.
fn rfc3339(text: &str) -> Option<i64> {
    let (date, rest) = text.split_once(['T', 't', ' '])?;
    let mut ymd = date.splitn(3, '-').map(|part| part.parse::<i64>().ok());
    let (year, month, day) = (ymd.next()??, ymd.next()??, ymd.next()??);
    let (clock, offset) = match rest.find(['Z', 'z', '+', '-']) {
        Some(at) => (&rest[..at], &rest[at..]),
        None => (rest, "Z"),
    };
    let mut hms = clock.split(':');
    let hour = hms.next()?.parse::<i64>().ok()?;
    let minute = hms.next()?.parse::<i64>().ok()?;
    let second = hms.next()?.split('.').next()?.parse::<i64>().ok()?;
    let shift = match offset.as_bytes().first()? {
        b'Z' | b'z' => 0,
        sign => {
            let (h, m) = offset[1..].split_once(':')?;
            let minutes = h.parse::<i64>().ok()? * 60 + m.parse::<i64>().ok()?;
            if *sign == b'-' { -minutes } else { minutes }
        }
    };
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(
        days_from_civil(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second
            - shift * 60,
    )
}

/// Days from 1970-01-01 to the civil date (Howard Hinnant's algorithm).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// How long an item stays in Today after it was last opened.
pub const IDLE: Duration = Duration::from_secs(12 * 60 * 60);

impl Epoch {
    /// The wall clock now.
    pub fn now() -> Epoch {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_secs());
        Epoch(i64::try_from(secs).unwrap_or(i64::MAX))
    }

    /// `self` moved later by `by`.
    pub fn plus(self, by: Duration) -> Epoch {
        Epoch(
            self.0
                .saturating_add(i64::try_from(by.as_secs()).unwrap_or(i64::MAX)),
        )
    }

    /// How long after `earlier` this is, or `None` when it is not after it.
    pub fn since(self, earlier: Epoch) -> Option<Duration> {
        u64::try_from(self.0.saturating_sub(earlier.0))
            .ok()
            .map(Duration::from_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::Epoch;

    #[test]
    fn a_time_reads_from_seconds_or_rfc3339() {
        // (json, seconds)
        const CASES: &[(&str, Option<i64>)] = &[
            ("1700000000", Some(1_700_000_000)),
            (r#""2023-11-14T22:13:20Z""#, Some(1_700_000_000)),
            (r#""2023-11-14T22:13:20.123456789Z""#, Some(1_700_000_000)),
            (r#""2023-11-15T06:13:20+08:00""#, Some(1_700_000_000)),
            (r#""2023-11-14T17:13:20-05:00""#, Some(1_700_000_000)),
            (r#""1970-01-01T00:00:00Z""#, Some(0)),
            (r#""2024-02-29T00:00:00Z""#, Some(1_709_164_800)),
            (r#""yesterday""#, None),
            (r#""2023-13-14T22:13:20Z""#, None),
            ("null", None),
        ];
        for &(json, want) in CASES {
            let got = serde_json::from_str::<Epoch>(json)
                .ok()
                .map(|epoch| epoch.0);
            assert_eq!(got, want, "{json}");
        }
        assert_eq!(serde_json::to_string(&Epoch(5)).ok().as_deref(), Some("5"));
    }
}
