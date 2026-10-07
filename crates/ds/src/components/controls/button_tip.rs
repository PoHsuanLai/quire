//! A control's `title` as a Mac tooltip (`NSView.toolTip`, design/30 section 2.5): the control
//! is wrapped in a [`Hint`] on the Tip profile, so the tip waits by the hover hub (1 s cold, at
//! once while the hub is warm), never takes focus, never blocks a click and goes on a press.
//!
//! A control already inside a hint (a `Tooltip` the caller put round it) draws nothing: never two
//! tips. With no `Ds` to provide a hub the native `title` attribute stays.

use crate::components::overlays::tooltip::{Hint, HintSide, Hinted};
use crate::stack::hover_hub::HoverHub;
use dioxus::prelude::*;
use ds_motion::hover_intent::HoverProfile;

/// How a control's `title` is shown.
pub(crate) enum Tip {
    /// Nothing to draw: no title, or a tip of the caller's already stands over the control.
    Bare,
    /// The plain `title` attribute: no `Ds` here provides a hub to draw a tip with.
    Native(String),
    /// A tooltip through the hover hub.
    Hinted(String),
}

/// How `title` is shown here.
pub(crate) fn use_tip(title: Option<String>) -> Tip {
    let Some(text) = title else {
        return Tip::Bare;
    };
    match (
        try_consume_context::<Hinted>(),
        try_consume_context::<HoverHub>(),
    ) {
        (Some(_), _) => Tip::Bare,
        (None, Some(_)) => Tip::Hinted(text),
        (None, None) => Tip::Native(text),
    }
}

impl Tip {
    /// The value of the control's `title` attribute.
    pub(crate) fn native(&self) -> Option<String> {
        match self {
            Tip::Native(text) => Some(text.clone()),
            Tip::Bare | Tip::Hinted(_) => None,
        }
    }

    /// `face` inside its hint when the title is one.
    pub(crate) fn wrap(self, face: Element) -> Element {
        match self {
            Tip::Hinted(text) => rsx! {
                Hint {
                    text,
                    profile: HoverProfile::Tip,
                    side: HintSide::Below,
                    root: "ds-tooltip",
                    {face}
                }
            },
            Tip::Bare | Tip::Native(_) => face,
        }
    }
}
