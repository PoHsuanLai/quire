//! Which suggestion row is highlighted, and how the caller reads and sets it: pure. A row is
//! named by its value, as a controlled selection is (`SearchCursor`); the menu numbers its choices.

use crate::components::menus::menu::choices::{Act, Choice};
use crate::components::menus::search::model::{InitialHighlight, SearchCursor};
use ds_core::vocab::Availability;

/// The choice a fresh list of results highlights: the first pickable one under `TopHit`.
pub(crate) fn initial(highlight: InitialHighlight, live: &[Availability]) -> Option<usize> {
    match highlight {
        InitialHighlight::None => None,
        InitialHighlight::TopHit => live.iter().position(|row| *row == Availability::Enabled),
    }
}

/// The value of choice `at`, if it picks one: what the caller is told.
pub(crate) fn value_at<T: Clone>(picks: &[Choice<T>], at: Option<usize>) -> Option<T> {
    match picks.get(at?).map(|choice| &choice.act) {
        Some(Act::Pick(value, _)) => Some(value.clone()),
        Some(Act::Open(_)) | None => None,
    }
}

/// The choice that picks `value`: what the caller's cursor names.
fn index_of<T: PartialEq>(picks: &[Choice<T>], value: &T) -> Option<usize> {
    picks
        .iter()
        .position(|choice| matches!(&choice.act, Act::Pick(own, _) if own == value))
}

/// The highlighted choice: the caller's, when it holds the cursor, else the field's `own`.
pub(crate) fn shown_at<T: PartialEq>(
    cursor: &SearchCursor<T>,
    picks: &[Choice<T>],
    own: Option<usize>,
) -> Option<usize> {
    match cursor {
        SearchCursor::Own => own,
        SearchCursor::Is(value) => value.as_ref().and_then(|value| index_of(picks, value)),
    }
}

#[cfg(test)]
mod tests {
    use super::{initial, shown_at, value_at};
    use crate::components::menus::item::item::AfterPick;
    use crate::components::menus::menu::choices::{Act, Choice};
    use crate::components::menus::search::model::{InitialHighlight, SearchCursor};
    use ds_core::vocab::Availability::{Disabled as D, Enabled as E};

    fn pick(
        value: &'static str,
        availability: ds_core::vocab::Availability,
    ) -> Choice<&'static str> {
        Choice {
            act: Act::Pick(value, AfterPick::default()),
            availability,
            title: value.to_owned(),
        }
    }

    #[test]
    fn the_top_hit_is_the_first_enabled_choice() {
        assert_eq!(initial(InitialHighlight::None, &[E, E]), None);
        assert_eq!(initial(InitialHighlight::TopHit, &[D, E, E]), Some(1));
        assert_eq!(initial(InitialHighlight::TopHit, &[D]), None);
        assert_eq!(initial(InitialHighlight::TopHit, &[]), None);
    }

    #[test]
    fn the_callers_cursor_names_a_row_by_value() {
        let picks = [pick("a", E), pick("b", E)];
        let is = |value| SearchCursor::Is(value);
        assert_eq!(shown_at(&SearchCursor::Own, &picks, Some(1)), Some(1));
        assert_eq!(shown_at(&is(Some("b")), &picks, Some(0)), Some(1));
        assert_eq!(shown_at(&is(Some("z")), &picks, Some(0)), None);
        assert_eq!(shown_at(&is(None), &picks, Some(0)), None);
        assert_eq!(value_at(&picks, Some(1)), Some("b"));
        assert_eq!(value_at(&picks, Some(2)), None);
        assert_eq!(value_at(&picks, None), None);
    }
}
