//! A palette group growing or shrinking in place (design/26-DETAILS.md section 5.6):
//! pure, beside `palette_motion`. Between two result sets, one group of rows either kept its
//! first rows and gained more after them (Show More) or kept its first rows and lost the rest
//! (Show Less), with every other group as it was. Anything else is a new result set, which
//! replaces in place with no motion (R1, R12).

use crate::components::menus::menu_entry::MenuEntry;
use crate::components::menus::palette::palette_group::{GroupEntries, GroupsKey};

/// How one group's rows changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Resize {
    /// It kept its first `kept` choices and gained `added` after them.
    Grew { kept: usize, added: usize },
    /// It kept its first `kept` choices and lost the `removed` after them.
    Shrank { kept: usize, removed: usize },
}

/// Which group changed, by title, and how.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GroupResize {
    /// The group's header.
    pub title: String,
    /// How its rows changed.
    pub resize: Resize,
}

/// The one group of rows that grew or shrank at its end from `before` to `after`, the others
/// (titles, order and entries; action labels may change) as they were; `None` for any other
/// change.
pub(crate) fn resized<T: PartialEq>(
    before: &GroupsKey<T>,
    after: &GroupsKey<T>,
) -> Option<GroupResize> {
    if before.len() != after.len() {
        return None;
    }
    let mut differing =
        before
            .iter()
            .zip(after)
            .filter(|((was, was_entries, _), (now, now_entries, _))| {
                was != now || was_entries != now_entries
            });
    let ((title, was, _), (now_title, now, _)) = differing.next()?;
    if differing.next().is_some() || title != now_title {
        return None;
    }
    let (GroupEntries::List(was), GroupEntries::List(now)) = (was, now) else {
        return None;
    };
    let resize = match (was.len(), now.len()) {
        (w, n) if w < n && now.starts_with(was) => Resize::Grew {
            kept: choice_count(was),
            added: choice_count(&now[w..]),
        },
        (w, n) if n < w && was.starts_with(now) => Resize::Shrank {
            kept: choice_count(now),
            removed: choice_count(&was[n..]),
        },
        _ => return None,
    };
    Some(GroupResize {
        title: title.clone(),
        resize,
    })
}

/// How many of `entries` are rows the cursor rests on.
fn choice_count<T>(entries: &[MenuEntry<T>]) -> usize {
    entries
        .iter()
        .filter(|entry| {
            matches!(
                entry,
                MenuEntry::Item { .. } | MenuEntry::Row(_) | MenuEntry::Submenu { .. }
            )
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::{GroupResize, Resize, resized};
    use crate::components::lists::emoji_grid::grid::{EMOJI_CELL, EmojiCells};
    use crate::components::menus::menu_entry::{MenuEntry, MenuTrail};
    use crate::components::menus::palette::palette_group::{GroupEntries, GroupsKey};
    use ds_core::vocab::Availability;

    fn item(value: u8) -> MenuEntry<u8> {
        MenuEntry::Item {
            value,
            title: format!("row {value}"),
            detail: None,
            tile: None,
            trail: MenuTrail::None,
            check: None,
            availability: Availability::Enabled,
        }
    }

    fn rows(
        title: &str,
        values: &[u8],
        action: &str,
    ) -> (String, GroupEntries<u8>, Option<String>) {
        (
            title.to_string(),
            GroupEntries::List(values.iter().copied().map(item).collect()),
            Some(action.to_string()),
        )
    }

    fn grid(title: &str) -> (String, GroupEntries<u8>, Option<String>) {
        let cells = EmojiCells {
            cells: Vec::new(),
            columns: 8,
            cell: EMOJI_CELL,
        };
        (title.to_string(), GroupEntries::Grid(cells), None)
    }

    fn resize(title: &str, resize: Resize) -> Option<GroupResize> {
        Some(GroupResize {
            title: title.to_string(),
            resize,
        })
    }

    #[test]
    fn only_one_group_growing_or_shrinking_at_its_end_is_a_resize() {
        let short: GroupsKey<u8> =
            vec![rows("Apps", &[1, 2], "Show More"), rows("Files", &[9], "")];
        let long: GroupsKey<u8> = vec![
            rows("Apps", &[1, 2, 3, 4], "Show Less"),
            rows("Files", &[9], ""),
        ];
        let other: GroupsKey<u8> = vec![
            rows("Apps", &[1, 5, 3], "Show Less"),
            rows("Files", &[9], ""),
        ];
        let both: GroupsKey<u8> = vec![
            rows("Apps", &[1, 2, 3, 4], "Show Less"),
            rows("Files", &[9, 8], ""),
        ];
        let renamed: GroupsKey<u8> = vec![
            rows("Tools", &[1, 2, 3, 4], "Show Less"),
            rows("Files", &[9], ""),
        ];
        let fewer: GroupsKey<u8> = vec![rows("Apps", &[1, 2, 3, 4], "Show Less")];
        let emoji: GroupsKey<u8> = vec![grid("Emoji")];
        type Case<'a> = (
            &'a str,
            &'a GroupsKey<u8>,
            &'a GroupsKey<u8>,
            Option<GroupResize>,
        );
        let cases: &[Case<'_>] = &[
            (
                "more",
                &short,
                &long,
                resize("Apps", Resize::Grew { kept: 2, added: 2 }),
            ),
            (
                "less",
                &long,
                &short,
                resize(
                    "Apps",
                    Resize::Shrank {
                        kept: 2,
                        removed: 2,
                    },
                ),
            ),
            ("same", &short, &short, None),
            ("not a prefix", &short, &other, None),
            ("two groups", &short, &both, None),
            ("renamed", &short, &renamed, None),
            ("a group went", &long, &fewer, None),
            ("a grid", &emoji, &emoji, None),
        ];
        for (name, before, after, want) in cases {
            assert_eq!(&resized(before, after), want, "{name}");
        }
    }
}
