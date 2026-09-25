//! What the host does when a resolve moved the hover by itself (sill Q170).
//!
//! Blitz ends every `resolve` by hit-testing the last pointer position against the fresh layout
//! and storing the result as the hovered node, with no events (its `refresh_hover` carries a TODO
//! for enter/leave). Its event driver diffs the next pointer move against that stored node, so
//! an element that slid in under a resting pointer never hears `pointerenter`, and the element
//! the pointer left never hears `pointerleave`. The host repairs this after each resolve: it puts
//! Blitz's hover back on the element it held before, by hit-testing a point still inside that
//! element, then replays the last pointer move so the driver sees, and dispatches, the real
//! change. This module is the decision; `crate::hover_replay` carries it out.

use blitz_dom::NodeId;
use ds::{Point, Px, Rect};

/// What to do after a resolve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum HoverSync {
    /// Nothing: the hover did not move, or no pointer rests on the document.
    Unchanged,
    /// Hover this point (it hits the element hovered before), then replay the pointer move.
    Restore(Point),
    /// The element hovered before cannot be hit any more (it left the tree, or is covered):
    /// clear the hover, then replay the pointer move.
    Clear,
}

/// The hovered element before a resolve and after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HoverShift {
    pub(crate) before: Option<NodeId>,
    pub(crate) after: Option<NodeId>,
}

/// Whether the host knows where the pointer rests: only then can it replay a move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Rest {
    /// A pointer event has arrived, so its position is known.
    Known,
    /// None has.
    Unknown,
}

/// A point inside the element hovered before, and the element that point hits now (mapped as
/// Blitz maps a hit to the node it stores as hovered).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Probe {
    pub(crate) at: Point,
    pub(crate) element: Option<NodeId>,
}

/// Decide from `shift`, `rest` and the `probes`, taken in order and only as far as needed.
pub(crate) fn decide(
    shift: HoverShift,
    rest: Rest,
    probes: impl IntoIterator<Item = Probe>,
) -> HoverSync {
    match (rest, shift.before) {
        _ if shift.before == shift.after => HoverSync::Unchanged,
        (Rest::Unknown, _) => HoverSync::Unchanged,
        (Rest::Known, None) => HoverSync::Clear,
        (Rest::Known, Some(old)) => probes
            .into_iter()
            .find(|probe| probe.element == Some(old))
            .map_or(HoverSync::Clear, |probe| HoverSync::Restore(probe.at)),
    }
}

/// How far inside each corner a probe sits: on the edge itself a hit may fall to a neighbour.
const INSET: f32 = 1.0;

/// Where to probe `rect`: its centre first (where an element is least likely to be covered at
/// its edge), then its four corners inset by [`INSET`], for an element whose centre something
/// now covers.
pub(crate) fn probe_points(rect: Rect) -> [Point; 5] {
    let (x, y) = (rect.origin.x.0, rect.origin.y.0);
    let (width, height) = (rect.size.width.0, rect.size.height.0);
    let at = |x: f32, y: f32| Point { x: Px(x), y: Px(y) };
    let (left, right) = (x + INSET, x + width - INSET);
    let (top, bottom) = (y + INSET, y + height - INSET);
    [
        at(x + width / 2.0, y + height / 2.0),
        at(left, top),
        at(right, top),
        at(left, bottom),
        at(right, bottom),
    ]
}

#[cfg(test)]
mod tests {
    use super::{HoverShift, HoverSync, Probe, Rest, decide, probe_points};
    use blitz_dom::NodeId;
    use ds::{Point, Px, Rect, Size};

    const fn at(x: f32, y: f32) -> Point {
        Point { x: Px(x), y: Px(y) }
    }

    fn node(raw: Option<u64>) -> Option<NodeId> {
        raw.map(NodeId::from_u64)
    }

    fn shift(before: Option<u64>, after: Option<u64>) -> HoverShift {
        HoverShift {
            before: node(before),
            after: node(after),
        }
    }

    fn probe(x: f32, element: Option<u64>) -> Probe {
        Probe {
            at: at(x, 0.0),
            element: node(element),
        }
    }

    /// A case: the shift, the rest, the probes, and the decision.
    type Case = (HoverShift, Rest, Vec<Probe>, HoverSync);

    fn cases() -> Vec<Case> {
        vec![
            // Nothing moved.
            (
                shift(Some(4), Some(4)),
                Rest::Known,
                vec![],
                HoverSync::Unchanged,
            ),
            (shift(None, None), Rest::Known, vec![], HoverSync::Unchanged),
            // Moved, but no pointer to replay.
            (
                shift(Some(4), Some(9)),
                Rest::Unknown,
                vec![probe(1.0, Some(4))],
                HoverSync::Unchanged,
            ),
            // Nothing was hovered: clearing is restoring.
            (shift(None, Some(9)), Rest::Known, vec![], HoverSync::Clear),
            // The first probe that hits the old element wins.
            (
                shift(Some(4), Some(9)),
                Rest::Known,
                vec![
                    probe(1.0, Some(9)),
                    probe(2.0, Some(4)),
                    probe(3.0, Some(4)),
                ],
                HoverSync::Restore(at(2.0, 0.0)),
            ),
            (
                shift(Some(4), None),
                Rest::Known,
                vec![probe(1.0, Some(4))],
                HoverSync::Restore(at(1.0, 0.0)),
            ),
            // Covered everywhere probed, or gone.
            (
                shift(Some(4), Some(9)),
                Rest::Known,
                vec![probe(1.0, Some(9)), probe(2.0, None)],
                HoverSync::Clear,
            ),
            (
                shift(Some(4), Some(9)),
                Rest::Known,
                vec![],
                HoverSync::Clear,
            ),
        ]
    }

    #[test]
    fn decisions_follow_the_table() {
        for (n, (shift, rest, probes, want)) in cases().into_iter().enumerate() {
            assert_eq!(decide(shift, rest, probes), want, "case {n}");
        }
    }

    #[test]
    fn probes_are_the_centre_then_the_inset_corners() {
        let rect = Rect {
            origin: at(10.0, 20.0),
            size: Size {
                width: Px(100.0),
                height: Px(40.0),
            },
        };
        assert_eq!(
            probe_points(rect),
            [
                at(60.0, 40.0),
                at(11.0, 21.0),
                at(109.0, 21.0),
                at(11.0, 59.0),
                at(109.0, 59.0),
            ]
        );
    }
}
