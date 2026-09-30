//! The shell's metric tokens: the control center's sizes on the ladder, the dock, OSD,
//! notifications and the widget grid. Each family is a `Token` listed in `ds_shell::KIT`, so the
//! stylesheet and the linter read them from there. The type scale, the menus' and bar's sizes and
//! the widgets' paints are `ds-style`'s, because generic components read them.

pub(crate) mod control_center;
pub(crate) mod dock;
pub(crate) mod notifications;
pub(crate) mod osd;
pub(crate) mod widgets;

#[cfg(test)]
mod scale_tests;
