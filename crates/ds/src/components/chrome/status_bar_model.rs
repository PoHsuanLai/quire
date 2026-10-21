//! The words of a status bar: Finder's "12 items, 40 GB available".

/// The count of things shown, and what the app wants said beside it.
///
/// `available` is the app's own words for the room left ("40 GB"), already formatted for the
/// reader's units; the line then ends in "available". Counts are plural except one.
pub fn item_count_text(count: usize, available: Option<&str>) -> String {
    let noun = if count == 1 { "item" } else { "items" };
    match available {
        Some(room) => format!("{count} {noun}, {room} available"),
        None => format!("{count} {noun}"),
    }
}

#[cfg(test)]
mod tests {
    use super::item_count_text;

    #[test]
    fn the_count_is_plural_except_one_and_names_the_room_left() {
        // (name, count, available, want)
        const CASES: &[(&str, usize, Option<&str>, &str)] = &[
            ("none", 0, None, "0 items"),
            ("one", 1, None, "1 item"),
            ("many", 12, None, "12 items"),
            (
                "many with room",
                12,
                Some("40 GB"),
                "12 items, 40 GB available",
            ),
            (
                "one with room",
                1,
                Some("512 MB"),
                "1 item, 512 MB available",
            ),
            (
                "none with room",
                0,
                Some("1.2 TB"),
                "0 items, 1.2 TB available",
            ),
        ];
        for &(name, count, available, want) in CASES {
            assert_eq!(item_count_text(count, available), want, "{name}");
        }
    }
}
