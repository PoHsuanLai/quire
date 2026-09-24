//! Menu tracking (design/13-BEHAVIOUR-menus-windows.md sections 13.3.2-13.3.4, 13.4 and 13.5):
//! open on press, click mode, press-drag-release, hover switch between open menus, the submenu
//! delay, and the safe triangle that keeps a submenu open while the pointer heads for it.
//!
//! Pure: `now` is an argument and every effect is returned for the caller to perform. The
//! machine is generic over the caller's menu key `K` (a bar title, a context menu's owner), so
//! the bar and every ds `Menu` drive the same one. Arrow keys inside a menu, wrap, disabled
//! skipping and type-to-filter stay with the `Menu` component (design/06 section 21 decision
//! 3); this machine takes Escape, Left, Right and Enter, the keys that cross menus.
//!
//! Ported from sill's `bar/menu_track` (sill FINDINGS F15, quire gap Q1), with its tests.

mod machine;
mod triangle;
mod types;

pub use triangle::{inside, shielded};
pub use types::{
    Branch, Entered, Held, ItemPath, MenuAnim, MenuDirection, MenuKey, MenuPhase, MenuTarget,
    MenuTiming, MenuTrack, MenuTrackEffect, MenuTrackEvent, Pickable, SafeTriangle, Session,
    Submenu,
};

#[cfg(test)]
mod keyboard_tests;
#[cfg(test)]
mod tests;
