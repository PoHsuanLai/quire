//! The pure core of a pane stack's motion: given the path the caller holds now, which page is
//! shown, which one is leaving, and which of the switch's two slots each lives in.

use super::path::{PanePath, PaneWay};
use ds_motion::pane_slide::Pane;

/// What the stack remembers between renders: the transition's phase, nothing the caller owns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PaneTrack<K> {
    /// The path last drawn as the shown page.
    pub(crate) shown: PanePath<K>,
    /// The path shown before it, while it may still be leaving.
    pub(crate) leaving: Option<PanePath<K>>,
    /// The slot the shown page lives in; the leaving page lives in the other.
    pub(crate) slot: Pane,
    /// Which way the last change went.
    pub(crate) way: PaneWay,
}

impl<K: Clone + PartialEq> PaneTrack<K> {
    /// A stack that opens on `path`, nothing leaving.
    pub(crate) fn opening(path: &PanePath<K>) -> Self {
        PaneTrack {
            shown: path.clone(),
            leaving: None,
            slot: Pane::Root,
            way: PaneWay::Push,
        }
    }

    /// The track after the caller's path is `path`: unchanged while it is the same, else the
    /// shown page leaves into the other slot and the new one takes this one.
    pub(crate) fn following(self, path: &PanePath<K>) -> Self {
        if self.shown == *path {
            return self;
        }
        let way = if path.depth() < self.shown.depth() {
            PaneWay::Pop
        } else {
            PaneWay::Push
        };
        PaneTrack {
            slot: self.slot.other(),
            leaving: Some(self.shown),
            shown: path.clone(),
            way,
        }
    }

    /// The path drawn in `slot`: the shown page's, or the leaving one's.
    pub(crate) fn in_slot(&self, slot: Pane) -> Option<&PanePath<K>> {
        if slot == self.slot {
            Some(&self.shown)
        } else {
            self.leaving.as_ref()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PaneTrack;
    use crate::components::forms::pane_stack::path::{PanePath, PaneWay};
    use ds_motion::pane_slide::Pane;

    #[test]
    fn a_change_swaps_the_slots_and_names_the_way() {
        let root = PanePath::new("accounts");
        let detail = root.pushed("dana");
        let pushed = PaneTrack::opening(&root).following(&detail);
        assert_eq!((pushed.slot, pushed.way), (Pane::Detail, PaneWay::Push));
        assert_eq!(pushed.in_slot(Pane::Root), Some(&root));
        assert_eq!(pushed.in_slot(Pane::Detail), Some(&detail));
        let popped = pushed.clone().following(&root);
        assert_eq!((popped.slot, popped.way), (Pane::Root, PaneWay::Pop));
        assert_eq!(
            popped.in_slot(Pane::Detail),
            Some(&detail),
            "the popped page leaves"
        );
        let same = popped.clone().following(&root);
        assert_eq!(same, popped, "the same path changes nothing");
        let deeper = pushed.following(&detail.pushed("mail"));
        assert_eq!(deeper.slot, Pane::Root, "a third page takes the first slot");
    }
}
