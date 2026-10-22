//! A table's outline: the first column of a `VirtualTable` as a tree flattened into rows, as in
//! Finder's list view. The caller lists the rows that show (the children of an open branch follow
//! it) and says, for each, how deep it sits and whether it is a branch, open or closed
//! ([`OutlineRow`]). The table draws the indent and a disclosure triangle in front of the first
//! cell and reports each toggle ([`OutlineToggle`]); the app keeps which branches are open and
//! changes the rows it lists. The Right and Left arrows open and close the cursor's branch
//! (Option widens it to the subtree). Only the mounted rows are asked about, so the table stays
//! virtualised.

use crate::components::controls::disclosure::indicator;
use crate::focus::click::kept_click;
use dioxus::prelude::*;
use ds_core::vocab::Shown;

/// How far each level of the outline is indented, in logical pixels (the triangle's width, so a
/// child's triangle sits under its parent's name).
const INDENT: u32 = 16;

/// Whether a row has children to show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutlineKind {
    /// No children: no triangle.
    Leaf,
    /// A branch, its children listed after it (`Visible`) or not (`Hidden`).
    Branch(Shown),
}

/// Where a row stands in the outline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutlineRow {
    /// How many branches it sits inside; zero for a top-level row.
    pub depth: u16,
    /// Whether it is a branch.
    pub kind: OutlineKind,
}

impl OutlineRow {
    /// A row with no children, `depth` levels in.
    pub fn leaf(depth: u16) -> Self {
        OutlineRow {
            depth,
            kind: OutlineKind::Leaf,
        }
    }

    /// A branch `depth` levels in, `shown` open or closed.
    pub fn branch(depth: u16, shown: Shown) -> Self {
        OutlineRow {
            depth,
            kind: OutlineKind::Branch(shown),
        }
    }

    /// The row's `aria-level`: one for a top-level row.
    pub(crate) fn level(self) -> String {
        (u32::from(self.depth) + 1).to_string()
    }

    /// The row's `aria-expanded`: absent on a leaf.
    pub(crate) fn expanded(self) -> Option<&'static str> {
        match self.kind {
            OutlineKind::Leaf => None,
            OutlineKind::Branch(shown) => Some(shown.aria()),
        }
    }
}

/// How far a toggle reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleReach {
    /// The row alone.
    Row,
    /// The row and every branch under it (Option held, as in Finder).
    Subtree,
}

/// A request to open or close a branch, for the app to carry out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutlineToggle {
    /// The branch's row.
    pub row: usize,
    /// The state asked for: `Visible` opens it, `Hidden` closes it.
    pub shown: Shown,
    /// How far it reaches.
    pub reach: ToggleReach,
}

/// The reach of a press or key with `modifiers` held: Option widens it to the whole subtree.
pub(crate) fn reach_of(modifiers: Modifiers) -> ToggleReach {
    match modifiers.contains(Modifiers::ALT) {
        true => ToggleReach::Subtree,
        false => ToggleReach::Row,
    }
}

/// The first cell of a row in an outline: the indent, the triangle (a gap of its width on a leaf,
/// so the names line up), then the cell.
pub(crate) fn outline_cell(
    at: OutlineRow,
    row: usize,
    ontoggle: Option<EventHandler<OutlineToggle>>,
    cell: Element,
) -> Element {
    let indent = u32::from(at.depth) * INDENT;
    rsx! {
        span { class: "ds-vtable-outline", style: "padding-left:{indent}px",
            match at.kind {
                OutlineKind::Leaf => rsx! {
                    span { class: "ds-vtable-twisty-gap", "aria-hidden": "true" }
                },
                OutlineKind::Branch(shown) => rsx! {
                    Twisty { row, shown, ontoggle }
                },
            }
            span { class: "ds-vtable-outline-cell", {cell} }
        }
    }
}

/// The triangle of a branch row. A press asks `ontoggle` for the other state, and for the whole
/// subtree with Option held. It never reaches the row.
#[component]
fn Twisty(row: usize, shown: Shown, ontoggle: Option<EventHandler<OutlineToggle>>) -> Element {
    let label = match shown {
        Shown::Hidden => "Expand",
        Shown::Visible => "Collapse",
    };
    let toggle = move |reach: ToggleReach| {
        if let Some(ontoggle) = ontoggle {
            ontoggle.call(OutlineToggle {
                row,
                shown: shown.flipped(),
                reach,
            });
        }
    };
    rsx! {
        span {
            class: "ds-vtable-twisty",
            role: "button",
            "aria-label": label,
            "aria-expanded": shown.aria(),
            onclick: move |event| {
                event.stop_propagation();
                toggle(reach_of(event.modifiers()));
                kept_click(&event);
            },
            ondoubleclick: |event| event.stop_propagation(),
            {indicator(shown)}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{OutlineRow, ToggleReach, reach_of};
    use dioxus::prelude::Modifiers;
    use ds_core::vocab::Shown;

    #[test]
    fn option_widens_a_toggle_to_the_subtree_and_nothing_else_does() {
        let cases = [
            (Modifiers::empty(), ToggleReach::Row),
            (Modifiers::ALT, ToggleReach::Subtree),
            (Modifiers::SHIFT, ToggleReach::Row),
            (Modifiers::ALT | Modifiers::SHIFT, ToggleReach::Subtree),
        ];
        for (held, want) in cases {
            assert_eq!(reach_of(held), want, "{held:?}");
        }
    }

    #[test]
    fn a_row_says_its_level_from_one_and_expanded_only_when_it_is_a_branch() {
        let deep = OutlineRow::branch(2, Shown::Visible);
        assert_eq!(
            (deep.level().as_str(), deep.expanded()),
            ("3", Some("true"))
        );
        let closed = OutlineRow::branch(0, Shown::Hidden);
        assert_eq!(
            (closed.level().as_str(), closed.expanded()),
            ("1", Some("false"))
        );
        let leaf = OutlineRow::leaf(1);
        assert_eq!((leaf.level().as_str(), leaf.expanded()), ("2", None));
    }
}
