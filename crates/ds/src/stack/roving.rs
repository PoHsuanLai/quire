//! Roving focus (design/30-CATALOGUE.md section 1.4): one item of a set is the focused one; the
//! arrows move it, Home and End jump to the ends, a disabled item is passed over, and only a
//! menu wraps. Pure: the menu, the command palette and a list step through it, and
//! [`Roving`] holds the same rules over keyed items.

use dioxus::prelude::Key;
use ds_core::vocab::Availability;

/// How a step moves past the ends: a floating menu wraps, everything else stops
/// (design/30-CATALOGUE.md section 1.4, Roving focus).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wrap {
    /// Past the last is the first.
    Wraps,
    /// Past the last stays on the last.
    Stops,
}

/// One step of the selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Down: the next item.
    Down,
    /// Up: the previous item.
    Up,
}

/// The selection after `step` from `selected` among `count` items.
pub(crate) fn moved(nav: Wrap, selected: usize, count: usize, step: Step) -> usize {
    if count == 0 {
        return 0;
    }
    let last = count - 1;
    let selected = selected.min(last);
    match (nav, step) {
        (Wrap::Wraps, Step::Down) => (selected + 1) % count,
        (Wrap::Wraps, Step::Up) => (selected + last) % count,
        (Wrap::Stops, Step::Down) => (selected + 1).min(last),
        (Wrap::Stops, Step::Up) => selected.saturating_sub(1),
    }
}

/// The selection after `step` from `selected`, skipping disabled choices (design/13 section
/// 13.3.3). With no enabled choice the selection stays; clamping at an end with only disabled
/// choices past it stays too.
pub(crate) fn moved_live(nav: Wrap, selected: usize, live: &[Availability], step: Step) -> usize {
    let mut at = selected.min(live.len().saturating_sub(1));
    for _ in 0..live.len() {
        let next = moved(nav, at, live.len(), step);
        if next == at {
            return selected;
        }
        if live[next] == Availability::Enabled {
            return next;
        }
        at = next;
    }
    selected
}

/// The enabled choice the selection shows as: `selected` itself, or the first enabled one
/// after it (wrapping), or `selected` when none is enabled.
pub(crate) fn settled(selected: usize, live: &[Availability]) -> usize {
    let count = live.len();
    if count == 0 {
        return 0;
    }
    let start = selected.min(count - 1);
    (0..count)
        .map(|offset| (start + offset) % count)
        .find(|&index| live[index] == Availability::Enabled)
        .unwrap_or(start)
}

/// An end of the set: Home and End.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    /// Home: the first enabled item.
    First,
    /// End: the last enabled item.
    Last,
}

/// The first or last enabled item of `live`; `selected` when none is enabled.
pub fn edge_live(edge: Edge, selected: usize, live: &[Availability]) -> usize {
    let mut enabled = live
        .iter()
        .enumerate()
        .filter(|(_, availability)| **availability == Availability::Enabled)
        .map(|(index, _)| index);
    let found = match edge {
        Edge::First => enabled.next(),
        Edge::Last => enabled.next_back(),
    };
    found.unwrap_or(selected)
}

/// One key's move through the set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rove {
    /// An arrow.
    Step(Step),
    /// Home or End.
    Edge(Edge),
}

impl Rove {
    /// The move `key` asks for along one axis: Left or Up is back, Right or Down is on, Home
    /// and End are the ends; any other key asks for none.
    pub fn of(key: &Key) -> Option<Rove> {
        match key {
            Key::ArrowLeft | Key::ArrowUp => Some(Rove::Step(Step::Up)),
            Key::ArrowRight | Key::ArrowDown => Some(Rove::Step(Step::Down)),
            Key::Home => Some(Rove::Edge(Edge::First)),
            Key::End => Some(Rove::Edge(Edge::Last)),
            _ => None,
        }
    }
}

/// A set of keyed items with one focused: the machine the menu, segmented control, radio group,
/// tabs and lists share.
#[derive(Debug, Clone, PartialEq)]
pub struct Roving<K> {
    items: Vec<(K, Availability)>,
    wrap: Wrap,
    focused: Option<usize>,
}

impl<K: Clone + PartialEq> Roving<K> {
    /// `items` in order, none focused yet.
    pub fn new(items: Vec<(K, Availability)>, wrap: Wrap) -> Self {
        Roving {
            items,
            wrap,
            focused: None,
        }
    }

    /// The focused item, if any.
    pub fn focused(&self) -> Option<&K> {
        self.focused
            .and_then(|index| self.items.get(index))
            .map(|(key, _)| key)
    }

    /// Focus `key`, if it is listed and enabled; otherwise nothing changes.
    pub fn focus(mut self, key: &K) -> Self {
        let found = self
            .items
            .iter()
            .position(|(item, availability)| item == key && *availability == Availability::Enabled);
        self.focused = found.or(self.focused);
        self
    }

    fn live(&self) -> Vec<Availability> {
        self.items
            .iter()
            .map(|(_, availability)| *availability)
            .collect()
    }

    /// One step. From nothing focused, Down lands on the first enabled item and Up on the last.
    pub fn step(mut self, step: Step) -> Self {
        let live = self.live();
        self.focused = match self.focused {
            Some(at) => Some(moved_live(self.wrap, at, &live, step)),
            None => Some(edge_live(
                match step {
                    Step::Down => Edge::First,
                    Step::Up => Edge::Last,
                },
                0,
                &live,
            )),
        };
        self
    }

    /// Move as `rove` says.
    pub fn rove(self, rove: Rove) -> Self {
        match rove {
            Rove::Step(step) => self.step(step),
            Rove::Edge(edge) => self.edge(edge),
        }
    }

    /// Home or End.
    pub fn edge(mut self, edge: Edge) -> Self {
        let live = self.live();
        self.focused = Some(edge_live(edge, self.focused.unwrap_or(0), &live));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{Edge, Roving, Step, Wrap, edge_live, moved, moved_live, settled};
    use ds_core::vocab::Availability::{Disabled as D, Enabled as E};

    #[test]
    fn a_menu_wraps_and_the_palette_stops() {
        // (nav, selected, count, step, want)
        #[rustfmt::skip]
        const CASES: &[(Wrap, usize, usize, Step, usize)] = &[
            (Wrap::Wraps, 0, 3, Step::Down, 1),
            (Wrap::Wraps, 2, 3, Step::Down, 0),
            (Wrap::Wraps, 0, 3, Step::Up, 2),
            (Wrap::Wraps, 1, 3, Step::Up, 0),
            (Wrap::Wraps, 0, 1, Step::Down, 0),
            (Wrap::Wraps, 0, 0, Step::Down, 0),
            (Wrap::Wraps, 7, 3, Step::Down, 0),
            (Wrap::Stops, 0, 3, Step::Down, 1),
            (Wrap::Stops, 2, 3, Step::Down, 2),
            (Wrap::Stops, 0, 3, Step::Up, 0),
            (Wrap::Stops, 2, 3, Step::Up, 1),
            (Wrap::Stops, 0, 0, Step::Up, 0),
        ];
        for &(nav, selected, count, step, want) in CASES {
            assert_eq!(
                moved(nav, selected, count, step),
                want,
                "{nav:?} {step:?} from {selected} of {count}"
            );
        }
    }

    #[test]
    fn up_and_down_skip_disabled_choices() {
        // (nav, selected, live, step, want)
        #[rustfmt::skip]
        let cases: &[(Wrap, usize, &[_], Step, usize)] = &[
            (Wrap::Wraps, 0, &[E, D, E], Step::Down, 2),
            (Wrap::Wraps, 2, &[E, D, E], Step::Up, 0),
            (Wrap::Wraps, 2, &[D, E, E], Step::Down, 1),
            (Wrap::Wraps, 0, &[E, D, D], Step::Down, 0),
            (Wrap::Wraps, 0, &[D, D, D], Step::Down, 0),
            (Wrap::Stops, 1, &[E, E, D], Step::Down, 1),
            (Wrap::Stops, 2, &[E, D, E], Step::Up, 0),
            (Wrap::Wraps, 0, &[], Step::Down, 0),
        ];
        for &(nav, selected, live, step, want) in cases {
            assert_eq!(
                moved_live(nav, selected, live, step),
                want,
                "{nav:?} {step:?} from {selected} in {live:?}"
            );
        }
    }

    #[test]
    fn the_selection_settles_on_an_enabled_choice() {
        #[rustfmt::skip]
        let cases: &[(usize, &[_], usize)] = &[
            (0, &[E, E], 0),
            (0, &[D, E], 1),
            (1, &[E, D], 0),
            (0, &[D, D], 0),
            (5, &[E, D, E], 2),
            (0, &[], 0),
        ];
        for &(selected, live, want) in cases {
            assert_eq!(settled(selected, live), want, "{selected} in {live:?}");
        }
    }

    #[test]
    fn keys_ask_for_moves() {
        use super::Rove;
        use dioxus::prelude::Key;
        assert_eq!(Rove::of(&Key::ArrowRight), Some(Rove::Step(Step::Down)));
        assert_eq!(Rove::of(&Key::ArrowLeft), Some(Rove::Step(Step::Up)));
        assert_eq!(Rove::of(&Key::Home), Some(Rove::Edge(Edge::First)));
        assert_eq!(Rove::of(&Key::Enter), None);
    }

    #[test]
    fn home_and_end_jump_to_the_enabled_ends() {
        #[rustfmt::skip]
        let cases: &[(Edge, usize, &[_], usize)] = &[
            (Edge::First, 2, &[E, E, E], 0),
            (Edge::Last, 0, &[E, E, E], 2),
            (Edge::First, 2, &[D, E, E], 1),
            (Edge::Last, 0, &[E, E, D], 1),
            (Edge::First, 1, &[D, D], 1),
            (Edge::Last, 0, &[], 0),
        ];
        for &(edge, selected, live, want) in cases {
            assert_eq!(
                edge_live(edge, selected, live),
                want,
                "{edge:?} in {live:?}"
            );
        }
    }

    #[test]
    fn a_roving_set_focuses_steps_and_jumps_by_key() {
        let items = vec![("a", E), ("b", D), ("c", E), ("d", E)];
        let set = Roving::new(items.clone(), Wrap::Wraps);
        assert_eq!(set.focused(), None);
        let set = set.step(Step::Down);
        assert_eq!(set.focused(), Some(&"a"), "Down from nothing is the first");
        let set = set.step(Step::Down);
        assert_eq!(set.focused(), Some(&"c"), "the disabled one is passed over");
        let set = set.edge(Edge::Last);
        assert_eq!(set.focused(), Some(&"d"));
        assert_eq!(
            set.clone().step(Step::Down).focused(),
            Some(&"a"),
            "a menu wraps"
        );
        let stops = Roving::new(items, Wrap::Stops)
            .edge(Edge::Last)
            .step(Step::Down);
        assert_eq!(stops.focused(), Some(&"d"), "a list stops");
        assert_eq!(
            stops.clone().focus(&"b").focused(),
            Some(&"d"),
            "a disabled key is refused"
        );
        assert_eq!(stops.focus(&"c").focused(), Some(&"c"));
        let up = Roving::new(vec![("a", E), ("b", E)], Wrap::Wraps).step(Step::Up);
        assert_eq!(up.focused(), Some(&"b"), "Up from nothing is the last");
    }
}
