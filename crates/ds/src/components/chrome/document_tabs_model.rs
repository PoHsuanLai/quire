//! DocumentTabs' rules, pure so they are tested without a pointer or a key: when a strip is
//! worth showing, what a standard action asks of it, where an arrow lands and what a drop does.

use crate::stack::roving::{Edge, Rove, Wrap, moved};
use chordkit::{Action, StandardAction};

/// How many tabs make a strip worth showing: one document needs no strip, as in Finder and
/// Safari. The strip does not hide itself; the app decides with [`strip_shown`].
pub const SHOWN_FROM: usize = 2;

/// Whether an app with `count` tabs draws the strip.
pub fn strip_shown(count: usize) -> bool {
    count >= SHOWN_FROM
}

/// What a standard action asks of the strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabCommand {
    /// Open a new tab (New).
    New,
    /// Close the selected tab (Close).
    CloseSelected,
}

/// The command `action` is for the strip, if any. chordkit has no next or previous tab action,
/// so the strip moves between its tabs with the arrows, Home and End only.
pub fn command_of(action: &Action) -> Option<TabCommand> {
    match action {
        Action::Standard(StandardAction::New) => Some(TabCommand::New),
        Action::Standard(StandardAction::Close) => Some(TabCommand::CloseSelected),
        _ => None,
    }
}

/// The tab `rove` selects from `selected` among `count`: an arrow steps and stops at the ends,
/// Home and End jump to them.
pub fn rove_to(rove: Rove, selected: usize, count: usize) -> usize {
    match rove {
        Rove::Step(step) => moved(Wrap::Stops, selected, count, step),
        Rove::Edge(Edge::First) => 0,
        Rove::Edge(Edge::Last) => count.saturating_sub(1),
    }
}

/// The reorder a drop of tab `from` on tab `to` asks for: `from` takes `to`'s place and the
/// tabs between shift toward the gap. Dropping a tab on itself asks for nothing.
pub fn reorder(from: usize, to: usize) -> Option<(usize, usize)> {
    (from != to).then_some((from, to))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack::roving::Step;

    #[test]
    fn a_strip_shows_from_two_tabs() {
        const CASES: &[(usize, bool)] = &[(0, false), (1, false), (2, true), (9, true)];
        for &(count, want) in CASES {
            assert_eq!(strip_shown(count), want, "{count}");
        }
    }

    #[test]
    fn only_new_and_close_are_the_strips() {
        const CASES: &[(StandardAction, Option<TabCommand>)] = &[
            (StandardAction::New, Some(TabCommand::New)),
            (StandardAction::Close, Some(TabCommand::CloseSelected)),
            (StandardAction::Find, None),
            (StandardAction::NextWindow, None),
        ];
        for &(action, want) in CASES {
            assert_eq!(command_of(&Action::Standard(action)), want, "{action:?}");
        }
    }

    #[test]
    fn arrows_stop_at_the_ends_and_home_end_jump() {
        // (rove, selected, count, result)
        const CASES: &[(Rove, usize, usize, usize)] = &[
            (Rove::Step(Step::Down), 0, 3, 1),
            (Rove::Step(Step::Down), 2, 3, 2),
            (Rove::Step(Step::Up), 0, 3, 0),
            (Rove::Step(Step::Up), 2, 3, 1),
            (Rove::Edge(Edge::First), 2, 3, 0),
            (Rove::Edge(Edge::Last), 0, 3, 2),
            (Rove::Edge(Edge::Last), 0, 0, 0),
        ];
        for &(rove, selected, count, want) in CASES {
            assert_eq!(rove_to(rove, selected, count), want, "{rove:?} {selected}/{count}");
        }
    }

    #[test]
    fn a_drop_on_itself_reorders_nothing() {
        assert_eq!(reorder(1, 3), Some((1, 3)));
        assert_eq!(reorder(2, 0), Some((2, 0)));
        assert_eq!(reorder(2, 2), None);
    }
}
