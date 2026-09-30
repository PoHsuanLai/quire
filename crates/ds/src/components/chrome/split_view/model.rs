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

/// The widths a pane keeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaneSpec {
    /// Its width before anyone drags it, and what a double-click on its divider returns to.
    pub preferred: Px,
    /// The least a drag leaves it.
    pub min: Px,
    /// The most a drag gives it.
    pub max: Px,
    /// Whether it can fold away.
    pub collapsing: Collapsing,
}

impl PaneSpec {
    /// A sidebar's widths: 240 preferred, between 180 and 320, and it folds away.
    pub const SIDEBAR: PaneSpec = PaneSpec {
        preferred: Px(240.0),
        min: Px(180.0),
        max: Px(320.0),
        collapsing: Collapsing::Snaps,
    };

    /// What a drag that would make the pane `raw` wide does.
    pub fn dragged(self, raw: Px) -> DividerDrag {
        match self.collapsing {
            Collapsing::Snaps if raw.0 < self.min.0 / 2.0 => DividerDrag::Collapse,
            Collapsing::Snaps | Collapsing::Never => {
                DividerDrag::Resize(Px(raw.0.clamp(self.min.0, self.max.0)))
            }
        }
    }
}

/// What a drag of a divider does to the pane before it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DividerDrag {
    /// The pane takes this width, and opens if it was folded.
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
}

impl SplitPane {
    /// An open pane of `spec` holding `body`.
    pub fn new(spec: PaneSpec, body: Element) -> Self {
        SplitPane {
            spec,
            body,
            shown: Shown::Visible,
        }
    }

    /// The same pane, `shown`.
    pub fn shown(self, shown: Shown) -> Self {
        SplitPane { shown, ..self }
    }
}

#[cfg(test)]
mod tests {
    use super::{Collapsing, DividerDrag, PaneSpec};
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
            assert_eq!(spec.dragged(Px(raw)), want, "{name}");
        }
    }
}
