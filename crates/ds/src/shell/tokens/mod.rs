//! The shell's metric tokens: the type scale, dock, OSD, notifications, widget grid and paints,
//! and the shell's sizes on the ladder. Each family is a `Token` listed in `ds::shell`'s kit, so
//! the stylesheet and the linter read them from there.

pub(crate) mod control_center;
pub(crate) mod dock;
pub(crate) mod notifications;
pub(crate) mod osd;
pub(crate) mod shell_scale;
pub(crate) mod shell_type;
pub(crate) mod widget_paint;
pub(crate) mod widgets;

#[cfg(test)]
mod scale_tests;
