//! The props vocabulary every component shares (design/04-COMPONENTS.md "Shared vocabulary").
//!
//! No `bool` props: every two-state prop is one of these enums, so a call site reads
//! `Availability::Disabled`, never `true`.

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

/// Where an item stands in a drag (design/04-COMPONENTS.md section 34): the place under the
/// pointer that would take the drop writes `data-drop="target"`, every other place that could
/// take it `data-drop="accepts"`, the thing being dragged `data-drag="source"`, and every
/// other item neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DropState {
    /// Not part of the drag.
    #[default]
    Idle,
    /// Under the pointer and accepting: lit and grown.
    Target,
    /// Able to take what is being dragged, while the pointer is elsewhere (mailo gaps 5): every
    /// place a dragged thread could land is outlined as the drag starts, before the pointer is
    /// over any, so the reader sees where it may go. A quiet dashed accent hairline, weaker
    /// than `Target`'s fill, scale and shadow; written `data-drop="accepts"`.
    Accepts,
    /// Being dragged: dimmed to .35 while its ghost follows the pointer.
    Source,
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

/// Whether the menu, popover or disclosure a control opens is showing: `aria-expanded`.
///
/// Its own type rather than a [`Switch`]: a toggle's pressed state and a trigger's open state
/// are different facts, and a control can carry both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Expanded {
    /// What it controls is showing.
    Open,
    /// What it controls is hidden.
    #[default]
    Closed,
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
        StaggerIndex(u8::try_from(n).map_or(Self::CAP, |n| n.min(Self::CAP)))
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
        let phase = match self.phase {
            PulsePhase::Rest | PulsePhase::B => PulsePhase::A,
            PulsePhase::A => PulsePhase::B,
        };
        PulseKey { phase, ..self }
    }

    /// The class and `data-pulse` value to render, or `None` while at rest.
    pub fn attrs(self) -> Option<(String, &'static str)> {
        let alias = match self.phase {
            PulsePhase::Rest => return None,
            PulsePhase::A => "a",
            PulsePhase::B => "b",
        };
        Some((self.anim.class().to_string(), alias))
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
    /// Home: the line's start in a text surface.
    Home,
    /// End: the line's end.
    End,
    /// Delete (forward delete), drawn `⌦`.
    Delete,
    /// Page Up.
    PageUp,
    /// Page Down.
    PageDown,
    /// Insert: with Shift a paste, with Ctrl a copy, in a text surface.
    Insert,
    /// The context-menu key (the Menu key): opens a text surface's spelling menu at the caret.
    ContextMenu,
}

/// A shape a key's glyph draws as, for `Kbd`'s `data-glyph`: a hook for a face rule that only
/// some glyphs need. `Arrow` is the only member today — a Small cap's `data-glyph="arrow"`
/// draws Up, Down, Left and Right larger than the rest of the small face (sill Q111: at 9.5 px
/// an arrow's stroke reads as a dash).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum GlyphKind {
    /// Up, Down, Left, Right.
    Arrow,
}

impl GlyphKind {
    /// The `data-glyph` word.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            GlyphKind::Arrow => "arrow",
        }
    }
}

/// A key combination, modifiers first, rendered as glyphs with no separator: `⌃T`
/// (design/04-COMPONENTS.md O-2).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Shortcut(pub Vec<Key>);

impl Shortcut {
    /// The glyph text: `⌃T`.
    pub fn glyphs(&self) -> String {
        self.0.iter().map(|key| key.glyph()).collect()
    }
}

impl Key {
    /// The text one key cap shows. Only `⌃ ⇧ ⌥ ⌘`, upper-case characters and `↵` are the
    /// doc's; the rest are not specified in design/04-COMPONENTS.md (O-2 names only the
    /// modifiers). TODO(O-2): Space, Escape, Tab, Backspace, the arrows and Home, End, Delete, PageUp,
    /// PageDown, Insert and ContextMenu need sign-off.
    pub(crate) fn glyph(self) -> String {
        match self {
            Key::Ctrl => "⌃".to_string(),
            Key::Shift => "⇧".to_string(),
            Key::Alt => "⌥".to_string(),
            Key::Super => "⌘".to_string(),
            Key::Char(c) => c.to_uppercase().collect(),
            Key::Space => "Space".to_string(),
            Key::Enter => "↵".to_string(),
            Key::Escape => "Esc".to_string(),
            Key::Tab => "⇥".to_string(),
            Key::Backspace => "⌫".to_string(),
            Key::Up => "↑".to_string(),
            Key::Down => "↓".to_string(),
            Key::Left => "←".to_string(),
            Key::Right => "→".to_string(),
            Key::Home => "↖".to_string(),
            Key::End => "↘".to_string(),
            Key::Delete => "⌦".to_string(),
            Key::PageUp => "⇞".to_string(),
            Key::PageDown => "⇟".to_string(),
            Key::Insert => "Ins".to_string(),
            Key::ContextMenu => "Menu".to_string(),
        }
    }

    /// The `data-glyph` shape `Kbd` writes for this key, or `None` for every key whose glyph
    /// needs no face rule of its own (sill Q111).
    pub(crate) fn glyph_kind(self) -> Option<GlyphKind> {
        match self {
            Key::Up | Key::Down | Key::Left | Key::Right => Some(GlyphKind::Arrow),
            _ => None,
        }
    }
}

impl Availability {
    /// `aria-disabled`: present only when disabled.
    pub(crate) fn aria_disabled(self) -> Option<&'static str> {
        match self {
            Availability::Enabled => None,
            Availability::Disabled => Some("true"),
        }
    }
}

impl DropState {
    /// The `data-drop` value: `target`, `accepts`, or nothing.
    pub fn drop_attr(self) -> Option<&'static str> {
        match self {
            DropState::Target => Some("target"),
            DropState::Accepts => Some("accepts"),
            DropState::Idle | DropState::Source => None,
        }
    }

    /// The `data-drag` value: `source`, or nothing.
    pub fn drag_attr(self) -> Option<&'static str> {
        match self {
            DropState::Source => Some("source"),
            DropState::Idle | DropState::Target | DropState::Accepts => None,
        }
    }
}

impl Switch {
    /// The `aria-pressed` / `aria-checked` word.
    pub(crate) fn aria(self) -> &'static str {
        match self {
            Switch::On => "true",
            Switch::Off => "false",
        }
    }

    /// The other state.
    pub(crate) fn flipped(self) -> Self {
        match self {
            Switch::On => Switch::Off,
            Switch::Off => Switch::On,
        }
    }
}

impl Here {
    /// The `aria-current` word.
    pub(crate) fn aria_current(self) -> &'static str {
        match self {
            Here::Current => "true",
            Here::Elsewhere => "false",
        }
    }
}

impl Expanded {
    /// The `aria-expanded` word.
    pub(crate) fn aria(self) -> &'static str {
        match self {
            Expanded::Open => "true",
            Expanded::Closed => "false",
        }
    }

    /// The `data-expanded` word, for a part styled by whether what it heads is open.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            Expanded::Open => "open",
            Expanded::Closed => "closed",
        }
    }
}

impl Selection {
    /// The `aria-selected` word.
    pub(crate) fn aria(self) -> &'static str {
        match self {
            Selection::Selected => "true",
            Selection::Unselected => "false",
        }
    }

    /// `Selected` when `a == b`.
    pub(crate) fn of<T: PartialEq>(a: &T, b: &T) -> Self {
        if a == b {
            Selection::Selected
        } else {
            Selection::Unselected
        }
    }
}

impl Fraction {
    /// All of it.
    pub(crate) const ONE: Fraction = Fraction(1000);

    /// The value a component draws: clamped to 0..=1000 (FINDINGS F18).
    pub(crate) fn clamped(self) -> Self {
        Fraction(self.0.min(Self::ONE.0))
    }

    /// The clamped value as a CSS number for `--f`: `0.35`, `0`, `1`.
    pub(crate) fn css(self) -> String {
        let permille = self.clamped().0;
        let text = format!("{}.{:03}", permille / 1000, permille % 1000);
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}
