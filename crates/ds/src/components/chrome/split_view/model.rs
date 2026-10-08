//! What a `SplitView` is given, and the pure rule of what a drag of a divider does to the pane
//! before it.

use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::vocab::Shown;
use ds_core::word::Word;

/// Whether dragging a pane below its least folds it away.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Collapsing {
    /// The pane stops at its least.
    #[default]
    Never,
    /// Past half its least the pane folds away, and a drag back past that opens it.
    Snaps,
}

/// What a folded pane does with what lies outside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Folded {
    /// The pane clips its body: nothing of it shows, and nothing reaches the pointer.
    #[default]
    Clips,
    /// The body is an `EdgePeek`: once the pane is folded away its strip at the window's edge
    /// waits for the pointer, and the sidebar it floats out stands over the content beside it.
    Peeks,
}

/// The way the panes of a split view lie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum SplitAxis {
    /// Side by side, dividers upright: a pane's size is its width.
    #[default]
    Row,
    /// One above the next, dividers flat: a pane's size is its height.
    Column,
}

/// What a split view does to the panes that do not hold the focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum UnfocusedPanes {
    /// Every pane is drawn alike.
    #[default]
    Keep,
    /// Once the focus has been put in a pane, the other panes are drawn at `--pane-dim`.
    Dim,
}

/// A place in a split view where the focus can be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaneAt {
    /// The pane before the last, by index.
    Pane(usize),
    /// The last pane, holding the view's children.
    Rest,
}

/// Whether a pane holds the focus, as far as the view knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum PaneFocus {
    /// The focus is in this pane.
    Holds,
    /// The focus is in another pane.
    Lacks,
    /// Nobody has put the focus anywhere in the view yet.
    #[default]
    Unplaced,
}

impl PaneFocus {
    /// The `data-focus` value, written once the view knows where the focus is.
    pub(crate) fn attribute(self) -> Option<&'static str> {
        (self != PaneFocus::Unplaced).then(|| self.slug())
    }

    /// What pane `this` has when the focus is at `held`.
    pub fn of(this: PaneAt, held: Option<PaneAt>) -> Self {
        match held {
            Some(at) if at == this => PaneFocus::Holds,
            Some(_) => PaneFocus::Lacks,
            None => PaneFocus::Unplaced,
        }
    }
}

/// A part of the view's whole extent along its axis, from 0 (none) to 1 (all). A float because
/// the pixels it resolves to are floats, and a drag must land on the pixel the pointer is on.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Share(pub f32);

/// How much room a pane has along the axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaneSize {
    /// So many pixels, whatever the view's size.
    Fixed(Px),
    /// This part of the view, so it grows and shrinks with it.
    Share(Share),
}

impl PaneSize {
    /// The pixels this is in a view `extent` long.
    pub fn resolve(self, extent: Px) -> Px {
        match self {
            PaneSize::Fixed(px) => px,
            PaneSize::Share(Share(part)) => Px(part * extent.0),
        }
    }

    /// `px` in the unit `self` is in: a share stays a share, so a pane the person sized keeps
    /// following the view. In a view with no extent a share has nothing to be a part of.
    pub fn like(self, px: Px, extent: Px) -> PaneSize {
        match self {
            PaneSize::Fixed(_) => PaneSize::Fixed(px),
            PaneSize::Share(_) if extent.0 > 0.0 => PaneSize::Share(Share(px.0 / extent.0)),
            PaneSize::Share(_) => PaneSize::Share(Share(0.0)),
        }
    }
}

/// The sizes a pane keeps along the view's axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaneSpec {
    /// Its size before anyone drags it, and what a double-click on its divider returns to.
    pub preferred: PaneSize,
    /// The least a drag leaves it.
    pub min: PaneSize,
    /// The most a drag gives it.
    pub max: PaneSize,
    /// Whether it can fold away.
    pub collapsing: Collapsing,
}

impl PaneSpec {
    /// A sidebar's widths: 240 preferred, between 180 and 320, and it folds away.
    pub const SIDEBAR: PaneSpec = PaneSpec {
        preferred: PaneSize::Fixed(Px(240.0)),
        min: PaneSize::Fixed(Px(180.0)),
        max: PaneSize::Fixed(Px(320.0)),
        collapsing: Collapsing::Snaps,
    };

    /// A pane that is `preferred` of the view, a drag keeping it between a tenth and nine
    /// tenths of it. It never folds.
    pub const fn sharing(preferred: Share) -> PaneSpec {
        PaneSpec {
            preferred: PaneSize::Share(preferred),
            min: PaneSize::Share(Share(0.1)),
            max: PaneSize::Share(Share(0.9)),
            collapsing: Collapsing::Never,
        }
    }

    /// The preferred size in a view `extent` long.
    pub fn preferred_px(self, extent: Px) -> Px {
        self.preferred.resolve(extent)
    }

    /// What a drag that would make the pane `raw` long, in a view `extent` long, does.
    pub fn dragged(self, raw: Px, extent: Px) -> DividerDrag {
        let (min, max) = (self.min.resolve(extent), self.max.resolve(extent));
        match self.collapsing {
            Collapsing::Snaps if raw.0 < min.0 / 2.0 => DividerDrag::Collapse,
            Collapsing::Snaps | Collapsing::Never => {
                DividerDrag::Resize(Px(raw.0.clamp(min.0, max.0.max(min.0))))
            }
        }
    }
}

/// What a drag of a divider does to the pane before it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DividerDrag {
    /// The pane takes this size, and opens if it was folded.
    Resize(Px),
    /// The pane folds away.
    Collapse,
}

/// One pane before the last: its widths, what it holds, and whether it is open.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitPane {
    /// Its widths.
    pub spec: PaneSpec,
    /// What it holds.
    pub body: Element,
    /// Open, or folded away.
    pub shown: Shown,
    /// What it does with its body while folded away.
    pub folded: Folded,
}

impl SplitPane {
    /// An open pane of `spec` holding `body`.
    pub fn new(spec: PaneSpec, body: Element) -> Self {
        SplitPane {
            spec,
            body,
            shown: Shown::Visible,
            folded: Folded::Clips,
        }
    }

    /// The same pane, `shown`.
    pub fn shown(self, shown: Shown) -> Self {
        SplitPane { shown, ..self }
    }

    /// The same pane, its body an `EdgePeek` that stays reachable while the pane is folded.
    pub fn peeking(self) -> Self {
        SplitPane {
            folded: Folded::Peeks,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Collapsing, DividerDrag, PaneSize, PaneSpec, Share};
    use ds_core::geometry::units::Px;

    #[test]
    fn a_drag_is_held_to_the_widths_and_may_fold_a_collapsible_pane() {
        const CASES: &[(&str, Collapsing, f32, DividerDrag)] = &[
            (
                "inside",
                Collapsing::Snaps,
                250.0,
                DividerDrag::Resize(Px(250.0)),
            ),
            (
                "over the most",
                Collapsing::Snaps,
                500.0,
                DividerDrag::Resize(Px(320.0)),
            ),
            (
                "under the least but past half",
                Collapsing::Snaps,
                100.0,
                DividerDrag::Resize(Px(180.0)),
            ),
            (
                "under half the least",
                Collapsing::Snaps,
                60.0,
                DividerDrag::Collapse,
            ),
            (
                "not collapsible",
                Collapsing::Never,
                60.0,
                DividerDrag::Resize(Px(180.0)),
            ),
        ];
        for &(name, collapsing, raw, want) in CASES {
            let spec = PaneSpec {
                collapsing,
                ..PaneSpec::SIDEBAR
            };
            assert_eq!(spec.dragged(Px(raw), Px(1000.0)), want, "{name}");
        }
    }

    #[test]
    fn a_share_is_a_part_of_the_view_and_a_drag_keeps_it_one() {
        const CASES: &[(&str, f32, f32, f32, f32)] = &[
            ("half of a wide view", 0.5, 1000.0, 500.0, 0.5),
            ("half of a narrow view", 0.5, 400.0, 200.0, 0.5),
            ("dragged to a quarter", 0.5, 800.0, 200.0, 0.25),
        ];
        for &(name, share, extent, px, kept) in CASES {
            let size = PaneSize::Share(Share(share));
            assert_eq!(size.resolve(Px(extent)), Px(share * extent), "{name}");
            assert_eq!(
                size.like(Px(px), Px(extent)),
                PaneSize::Share(Share(kept)),
                "{name}: kept"
            );
        }
        let fixed = PaneSize::Fixed(Px(240.0));
        assert_eq!(fixed.like(Px(300.0), Px(900.0)), PaneSize::Fixed(Px(300.0)));
    }

    #[test]
    fn a_share_pane_is_held_to_shares_of_the_view() {
        let spec = PaneSpec::sharing(Share(0.5));
        assert_eq!(
            spec.dragged(Px(50.0), Px(1000.0)),
            DividerDrag::Resize(Px(100.0)),
            "least a tenth"
        );
        assert_eq!(
            spec.dragged(Px(990.0), Px(1000.0)),
            DividerDrag::Resize(Px(900.0)),
            "most nine tenths"
        );
    }
}
