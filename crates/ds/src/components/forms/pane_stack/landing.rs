//! Where the keyboard goes when a stack's page changes, pure: the selector of the element that
//! takes the focus, or `None` where nothing should.

use super::path::PaneWay;
use super::track::PaneTrack;

/// The id of the back button in `slot` of the stack named `id`.
pub(crate) fn back_id(id: &str, slot: ds_motion::pane_slide::Pane) -> String {
    use ds_core::word::Word;
    format!("{id}-back-{}", slot.slug())
}

/// The selector the focus moves to after `track` changed. A push lands on the new page's back
/// button, the first stop of its header; a pop returns to the element that opened the page that
/// left, which `opener` names by id from that page's key.
pub(crate) fn landing<K: Clone + PartialEq>(
    track: &PaneTrack<K>,
    id: &str,
    opener: impl Fn(&K) -> Option<String>,
) -> Option<String> {
    match track.way {
        PaneWay::Push => track
            .shown
            .parent()
            .map(|_| format!("#{}", back_id(id, track.slot))),
        PaneWay::Pop => track
            .leaving
            .as_ref()
            .and_then(|left| opener(left.current()))
            .map(|opener| format!("#{opener}")),
    }
}

#[cfg(test)]
mod tests {
    use super::landing;
    use crate::components::forms::pane_stack::{path::PanePath, track::PaneTrack};

    #[test]
    fn a_push_lands_on_the_back_button_and_a_pop_on_the_opener() {
        let root = PanePath::new("accounts");
        let detail = root.pushed("dana");
        let opener = |page: &&str| Some(format!("open-{page}"));
        let pushed = PaneTrack::opening(&root).following(&detail);
        assert_eq!(
            landing(&pushed, "ps", opener).as_deref(),
            Some("#ps-back-detail")
        );
        let popped = pushed.following(&root);
        assert_eq!(
            landing(&popped, "ps", opener).as_deref(),
            Some("#open-dana")
        );
        let replaced = PaneTrack::opening(&root).following(&PanePath::new("other"));
        assert_eq!(
            landing(&replaced, "ps", opener),
            None,
            "the root has no back button"
        );
    }
}
