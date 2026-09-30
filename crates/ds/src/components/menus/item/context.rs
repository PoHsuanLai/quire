//! What a context menu leaves out (design/30 section 2.4, design/27): an item that cannot be
//! picked is not shown, and a header, status line or rule left with nothing to head or divide goes
//! with it.

use crate::components::menus::item::item::MenuItem;
use ds_core::vocab::Availability;

/// `items` without the disabled ones, tidied: no rule first, last or doubled, no header or status
/// line whose group is empty.
pub(crate) fn available<T: Clone>(items: &[MenuItem<T>]) -> Vec<MenuItem<T>> {
    let kept: Vec<MenuItem<T>> = items
        .iter()
        .filter(|item| item.availability() != Some(Availability::Disabled))
        .map(|item| match item {
            MenuItem::Submenu {
                title,
                image,
                availability,
                children,
            } => MenuItem::Submenu {
                title: title.clone(),
                image: image.clone(),
                availability: *availability,
                children: available(children),
            },
            other => other.clone(),
        })
        .collect();
    tidy(kept)
}

/// A line that stands for a group: a rule between two, or a header over its own.
fn is_group_mark<T>(item: &MenuItem<T>) -> bool {
    matches!(item, MenuItem::Separator | MenuItem::Header(_))
}

/// Drop the rules and headers that group nothing.
fn tidy<T>(items: Vec<MenuItem<T>>) -> Vec<MenuItem<T>> {
    let mut out: Vec<MenuItem<T>> = Vec::with_capacity(items.len());
    for item in items {
        let doubled = matches!(item, MenuItem::Separator)
            && out
                .last()
                .is_none_or(|last| matches!(last, MenuItem::Separator));
        if !doubled {
            if matches!(out.last(), Some(MenuItem::Header(_))) && is_group_mark(&item) {
                out.pop();
            }
            out.push(item);
        }
    }
    while out.last().is_some_and(is_group_mark) {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::available;
    use crate::components::menus::item::item::MenuItem;
    use ds_core::vocab::Availability::Disabled;

    fn shape(items: &[MenuItem<u8>]) -> String {
        items
            .iter()
            .map(|item| match item {
                MenuItem::Item { value, .. } => value.to_string(),
                MenuItem::Submenu { .. } => "S".to_string(),
                MenuItem::Header(_) => "H".to_string(),
                MenuItem::Info { .. } => "I".to_string(),
                MenuItem::Separator => "-".to_string(),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn a_context_menu_drops_the_disabled_and_the_groups_they_leave_empty() {
        let on = |n: u8| MenuItem::new(n, "x");
        let off = |n: u8| MenuItem::new(n, "x").with_availability(Disabled);
        let cases: Vec<(&str, Vec<MenuItem<u8>>, &str)> = vec![
            ("all on", vec![on(1), MenuItem::Separator, on(2)], "1 - 2"),
            ("a disabled item goes", vec![on(1), off(2), on(3)], "1 3"),
            (
                "a rule left first goes",
                vec![off(1), MenuItem::Separator, on(2)],
                "2",
            ),
            (
                "a rule left last goes",
                vec![on(1), MenuItem::Separator, off(2)],
                "1",
            ),
            (
                "two rules become one",
                vec![
                    on(1),
                    MenuItem::Separator,
                    off(2),
                    MenuItem::Separator,
                    on(3),
                ],
                "1 - 3",
            ),
            (
                "a header over nothing goes",
                vec![
                    on(1),
                    MenuItem::Separator,
                    MenuItem::Header("h".into()),
                    off(2),
                ],
                "1",
            ),
            (
                "a header over items stays",
                vec![MenuItem::Header("h".into()), on(1)],
                "H 1",
            ),
            ("nothing left", vec![off(1)], ""),
        ];
        for (name, items, want) in cases {
            assert_eq!(shape(&available(&items)), want, "{name}");
        }
    }
}
