//! Giving a field the keyboard again (sill FINDINGS Q44): a menu that took the keyboard closes,
//! and the palette's field should have it back without being remounted (which replays the
//! palette's entrance).

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
}

impl FocusRequest {
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
    }
}
