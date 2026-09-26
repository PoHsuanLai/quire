//! Giving a field the keyboard again (sill FINDINGS Q44): a menu that took the keyboard closes,
//! and the palette's field should have it back without being remounted (which replays the
//! palette's entrance).

use crate::focus::caret::InitialCaret;
use crate::focus::select::{Landing, Select};
use crate::task::{try_get, try_set};
use dioxus::prelude::*;

/// How many times focus has been asked for: a field serves each new ticket once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct FocusTicket(pub u32);

/// A caller's handle on a field's focus: pass it as `Focus::Controlled(request)` to a
/// `TextInput` (or as `focus` to a `SearchField`'s palette), then call [`FocusRequest::request`]
/// from a handler whenever the field should have the keyboard again.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusRequest {
    asked: Signal<FocusTicket>,
    /// What the field does with its text each time the request lands.
    landing: Landing,
}

impl FocusRequest {
    /// This request, selecting the field's whole value each time it lands (and as the field
    /// mounts), so the first key typed replaces it: a rename field opened on the old name.
    pub fn with_select_all(self) -> Self {
        FocusRequest {
            landing: Landing::Place(InitialCaret::SelectAll),
            ..self
        }
    }

    /// This request, putting the field's caret at `caret` each time it lands (and as the field
    /// mounts): a command palette opened on a query puts it at the end (sill Q341). Needs the
    /// host's [`HostPlaceCaret`](crate::HostPlaceCaret) for `End` and `Start`; without one the
    /// caret stays where the renderer put it.
    pub fn with_caret(self, caret: InitialCaret) -> Self {
        FocusRequest {
            landing: Landing::Place(caret),
            ..self
        }
    }

    /// What the field does with its text when the focus lands: [`Select::All`] for a request
    /// that selects the whole value, [`Select::None`] otherwise (a caret placed at an end too).
    pub fn select(&self) -> Select {
        match self.landing {
            Landing::Place(InitialCaret::SelectAll) => Select::All,
            Landing::Leave | Landing::Place(InitialCaret::End | InitialCaret::Start) => {
                Select::None
            }
        }
    }

    /// Where the field's caret goes when the focus lands, if anywhere.
    pub fn caret(&self) -> Option<InitialCaret> {
        match self.landing {
            Landing::Leave => None,
            Landing::Place(caret) => Some(caret),
        }
    }

    /// What the focus write does with the field's text.
    pub(crate) fn landing(&self) -> Landing {
        self.landing
    }

    /// Ask for the focus: the field takes it on its next render, or as it mounts. Call it from a
    /// handler (a menu's `onclose`), not from render.
    pub fn request(&self) {
        if let Ok(FocusTicket(count)) = try_get(self.asked) {
            let _ = try_set(self.asked, FocusTicket(count.wrapping_add(1)));
        }
    }

    /// The latest ticket, read so the field re-renders when a new one is asked for.
    pub(crate) fn ticket(&self) -> FocusTicket {
        self.asked
            .try_read()
            .map_or(FocusTicket::default(), |ticket| *ticket)
    }

    /// The latest ticket, without subscribing.
    pub(crate) fn peek(&self) -> FocusTicket {
        try_get(self.asked).unwrap_or_default()
    }
}

/// A focus handle owned by the calling component.
pub fn use_focus_request() -> FocusRequest {
    FocusRequest {
        asked: use_signal(FocusTicket::default),
        landing: Landing::Leave,
    }
}
