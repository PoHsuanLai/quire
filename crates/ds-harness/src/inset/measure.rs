//! The measuring: for each painted vertical edge of each box, the distance to the nearest
//! content the box owns, held to the [`Policy`].
//!
//! Content belongs to its nearest box, so a nested box is measured against its own edges and
//! the box around it never sees what is inside the nested one (a pill in a row is the pill's
//! business). Content on any line of the box counts: the nearest to an edge is the nearest on
//! some line, so the least gap over all of them is the least gap over every line band. Content
//! whose middle is outside the box's top and bottom (scrolled or clipped away) is not measured.

use crate::inset::bounds::Bounds;
use crate::inset::offence::{Excused, Findings, Offence, Side};
use crate::inset::policy::Policy;
use crate::inset::scene::{Content, ContentKind, Scene, VisibleBox};
use ds::prelude::Px;

/// How far apart the two gaps of a lone glyph may be for it to count as centred.
const CENTRED: f32 = 1.0;

/// Measure every box of `scene` against `policy`.
pub fn measure(scene: &Scene, policy: &Policy) -> Findings {
    let mut findings = Findings::default();
    for (index, visible) in scene.boxes.iter().enumerate() {
        let inside: Vec<&Content> = scene
            .contents
            .iter()
            .filter(|content| {
                content.owner == index && visible.bounds.spans_middle_of(&content.bounds)
            })
            .collect();
        if inside.is_empty() || is_centred_glyph(&visible.bounds, &inside) {
            continue;
        }
        let (min, allowance) = policy.min_for(&visible.tokens);
        for side in sides(visible) {
            let Some((gap, nearest)) = nearest(&visible.bounds, &inside, side) else {
                continue;
            };
            // Content past the edge is clipped or overflowing, not inset.
            if gap + policy.slack.0 >= policy.min.0 || gap < -policy.slack.0 {
                continue;
            }
            if gap + policy.slack.0 >= min.0 {
                findings
                    .excused
                    .extend(allowance.map(|entry| Excused { when: entry.when }));
                continue;
            }
            findings.offences.push(Offence {
                path: visible.path.clone(),
                content: nearest.label.clone(),
                side,
                gap: Px(gap),
                min,
                paints: visible.paints.clone(),
            });
        }
    }
    findings
}

/// The painted vertical edges of `visible`.
fn sides(visible: &VisibleBox) -> impl Iterator<Item = Side> {
    let edges = visible.edges;
    [(edges.left, Side::Left), (edges.right, Side::Right)]
        .into_iter()
        .filter_map(|(painted, side)| painted.then_some(side))
}

/// The gap from `side` of `outer` to the content nearest it, and that content.
fn nearest<'a>(outer: &Bounds, inside: &[&'a Content], side: Side) -> Option<(f32, &'a Content)> {
    let gap = |content: &Content| match side {
        Side::Left => content.bounds.left - outer.left,
        Side::Right => outer.right - content.bounds.right,
    };
    inside
        .iter()
        .copied()
        .min_by(|a, b| gap(a).total_cmp(&gap(b)))
        .map(|content| (gap(content), content))
}

/// Whether the box holds nothing but glyphs, centred across it: an icon button, whose glyph's
/// distance from the edge is the button's size, not an inset.
fn is_centred_glyph(outer: &Bounds, inside: &[&Content]) -> bool {
    if inside
        .iter()
        .any(|content| content.kind != ContentKind::Glyph)
    {
        return false;
    }
    let held = inside
        .iter()
        .map(|content| content.bounds)
        .reduce(|a, b| a.union(&b));
    held.is_some_and(|held| {
        ((held.left - outer.left) - (outer.right - held.right)).abs() <= CENTRED
    })
}
