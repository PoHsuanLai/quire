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
//! Ported from sill's `bar/menu_track`, with its tests.

pub(crate) mod machine;
mod pointer;
pub(crate) mod triangle;
pub mod types;

#[cfg(test)]
mod keyboard_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod wake_tests;
