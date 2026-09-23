//! The props vocabulary every component shares (design/04-COMPONENTS.md "Shared vocabulary").
//!
//! No `bool` props: every two-state prop is one of these enums, so a call site reads
//! `Availability::Disabled`, never `true`.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use crate::motion::anim::Anim;
use serde::{Deserialize, Serialize};

/// Whether a control takes input. `Disabled` adds `aria-disabled="true"` and drops the handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Availability {
    /// Takes input.
    #[default]
    Enabled,
    /// Shown, but takes no input.
    Disabled,
}

/// Whether an option is the selected one: `aria-selected`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Selection {
    /// The selected option.
    Selected,
    /// Any other option.
    #[default]
    Unselected,
}

/// Unread weight versus read weight: `data-emphasis="strong|plain"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Emphasis {
    /// Heavier: unread.
    Strong,
    /// Lighter: read.
    #[default]
    Plain,
}

/// Whether this is the place the person is: `aria-current`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Here {
    /// The current place.
    Current,
    /// Somewhere else.
    #[default]
    Elsewhere,
}

/// A toggle's state: `aria-pressed` on buttons, `aria-checked` on a Toggle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Switch {
    /// Pressed or on.
    On,
    /// Not pressed, off.
    #[default]
    Off,
}

/// A menu item's check mark: `aria-checked`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Check {
    /// Checked.
    Checked,
    /// Not checked.
    #[default]
    Unchecked,
}

/// A proportion in thousandths: 0 is none, 1000 is all. Written inline as `--f`.
///
/// Not clamped by the type: a settings gain may exceed 1000 (design/22-SETTINGS.md section 4
/// uses the same type); a component clamps what it draws.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Fraction(pub u16);

/// A row's place in a staggered entrance, saturating at [`StaggerIndex::CAP`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct StaggerIndex(u8);

impl StaggerIndex {
    /// The highest index a stagger reaches: transforms on more nodes lag in Blitz
    /// (design/05-MOTION.md section 9 rule 3).
    pub const CAP: u8 = 12;

    /// The index for position `n`, saturating at [`Self::CAP`].
    pub fn new(n: usize) -> Self {
        todo!()
    }

    /// The index, 0 to [`Self::CAP`].
    pub fn get(self) -> u8 {
        self.0
    }
}

/// Which of the two identical keyframe aliases a pulse is playing (`X` or `X--b`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PulsePhase {
    /// Not playing.
    #[default]
    Rest,
    /// Playing the `X` alias: `data-pulse="a"`.
    A,
    /// Playing the `X--b` alias: `data-pulse="b"`.
    B,
}

/// The render-side half of a `use_pulse`: which animation, and which alias is playing.
/// Renders as `class="a-<anim>"` plus `data-pulse="a|b"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PulseKey {
    anim: Anim,
    phase: PulsePhase,
}

impl PulseKey {
    /// A pulse of `anim` that is not playing.
    pub fn rest(anim: Anim) -> Self {
        PulseKey {
            anim,
            phase: PulsePhase::Rest,
        }
    }

    /// The animation this key plays.
    pub fn anim(self) -> Anim {
        self.anim
    }

    /// Which alias is playing.
    pub fn phase(self) -> PulsePhase {
        self.phase
    }

    /// The same animation with the other alias playing: what `Pulse::fire` stores.
    pub fn fired(self) -> Self {
        todo!()
    }

    /// The class and `data-pulse` value to render, or `None` while at rest.
    pub fn attrs(self) -> Option<(String, &'static str)> {
        todo!()
    }
}

/// One key in a shortcut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// Control, drawn `⌃`.
    Ctrl,
    /// Shift, drawn `⇧`.
    Shift,
    /// Alt, drawn `⌥`.
    Alt,
    /// Super, drawn `⌘`.
    Super,
    /// A printable key, drawn upper-case.
    Char(char),
    /// Space.
    Space,
    /// Enter, drawn `↵`.
    Enter,
    /// Escape.
    Escape,
    /// Tab.
    Tab,
    /// Backspace.
    Backspace,
    /// Up arrow.
    Up,
    /// Down arrow.
    Down,
    /// Left arrow.
    Left,
    /// Right arrow.
    Right,
}

/// A key combination, modifiers first, rendered as glyphs with no separator: `⌃T`
/// (design/04-COMPONENTS.md O-2).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Shortcut(pub Vec<Key>);

impl Shortcut {
    /// The glyph text: `⌃T`.
    pub fn glyphs(&self) -> String {
        todo!()
    }
}
