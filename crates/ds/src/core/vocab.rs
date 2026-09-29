//! The props vocabulary every component shares (design/04-COMPONENTS.md "Shared vocabulary").
//!
//! No `bool` props: every two-state prop is one of these enums, so a call site reads
//! `Availability::Disabled`, never `true`.

use crate::core::word::Word;
use serde::{Deserialize, Serialize};

/// Whether a control takes input (design/30 section 1.5). `Disabled` adds `aria-disabled="true"`
/// and drops the handler; `Busy` adds `aria-busy="true"`, drops the handler as well, and is
/// where a control's busy accessory shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Availability {
    /// Takes input.
    #[default]
    Enabled,
    /// Shown, but takes no input.
    Disabled,
    /// Working: takes no input until the work ends.
    Busy,
}

/// Whether what a control shows or opens is up (design/30 section 1.5): a surface or a tooltip
/// its caller drives, a menu, popover or disclosure a trigger opens (`aria-expanded`), a
/// notification group's expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Shown {
    /// Up, at once.
    Visible,
    /// Down, even under the pointer.
    #[default]
    Hidden,
}

/// Whether an option is the selected one (`aria-selected`), or the place the person is
/// (`aria-current`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
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
    /// Able to take what is being dragged, while the pointer is elsewhere: every
    /// place a dragged thread could land is outlined as the drag starts, before the pointer is
    /// over any, so the reader sees where it may go. A quiet dashed accent hairline, weaker
    /// than `Target`'s fill, scale and shadow; written `data-drop="accepts"`.
    Accepts,
    /// Being dragged: dimmed to .35 while its ghost follows the pointer.
    Source,
}

/// On, off or mixed (`NSControl.StateValue`, design/30 section 1.5): a toggle's value, a
/// toggle button's pressed state, a menu item's check mark. `aria-pressed` on a button,
/// `aria-checked` on a toggle or a menu item; `Mixed` writes `"mixed"`.
///
/// `Toggle` has no mixed look: it refuses `Mixed` and draws it as `Off`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Check {
    /// Pressed, checked, on.
    On,
    /// Not pressed, unchecked, off.
    #[default]
    Off,
    /// Some of what it covers is on and some is off.
    Mixed,
}

/// What closes a floating layer (`NSPopover.Behavior`, design/30 section 1.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Dismiss {
    /// Escape, or a click outside.
    #[default]
    Transient,
    /// Escape only.
    Semitransient,
    /// Neither: only its owner closes it.
    Manual,
}

/// Where a press on a control stands (design/30 section 1.5): from a pointer or a key going
/// down until it is released. `data-pressed` is written for `Pressed` and `Held`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum PressPhase {
    /// No press under way.
    #[default]
    Idle,
    /// Down, by pointer or key; released inside, it activates.
    Pressed,
    /// Down for the long-press delay: the press has become a long press.
    Held,
}

impl PressPhase {
    /// `data-pressed`: present while a press is under way.
    pub fn attr(self) -> Option<&'static str> {
        match self {
            PressPhase::Idle => None,
            PressPhase::Pressed | PressPhase::Held => Some("true"),
        }
    }
}

/// How the person last drove the surface: `data-modality` on `.ds` (design/30 section 1.5).
///
/// blitz-dom hard-codes `:focus-visible` and `:focus-within` to false and a click does not focus
/// a button (spike S12), so the focus ring is `.ds[*|data-modality=keyboard] :focus` and the host
/// says which modality is current.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum InputModality {
    /// The last input was a pointer: no focus rings.
    #[default]
    Pointer,
    /// The last input was a key: focus rings show.
    Keyboard,
}

/// How focus is drawn on an element (design/30 section 1.4, `data-focus`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum FocusStyle {
    /// A translucent accent ring around the element, for keyboard modality only: controls and
    /// fields.
    #[default]
    Ring,
    /// The element's own fill: accent with accent ink while its list is focused in the active
    /// window, grey otherwise. List rows, menu rows and launcher rows.
    Highlight,
}

/// The state every row shares (design/30 section 1.5): list, menu, palette, sidebar and
/// settings rows are drawn from the same four facts, so a row takes one prop, not four.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RowState {
    /// Whether the row is the selected one, or the place the person is.
    pub selection: Selection,
    /// Whether the row is unread or read weight.
    pub emphasis: Emphasis,
    /// Whether the row takes input, and whether it is working.
    pub availability: Availability,
    /// The row's part in a drag.
    pub drop: DropState,
}

/// Whether a thing is heard, or shown at full strength (design/30 section 1.5): a volume's
/// speaker glyph, an avatar's colour (an account not in view keeps its hue at .55 of its
/// chroma; `data-muting`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Muting {
    /// Heard, or the colour as given.
    #[default]
    Audible,
    /// Muted: a speaker's waves go and a slash comes in; a colour keeps its lightness and hue
    /// at .55 of its chroma.
    Muted,
}

/// Whether a thing is taking part right now (`data-activity`, design/30 section 1.5): an
/// `Active` orb listens and turns; an `Inactive` one is held still, and nothing runs for it. A
/// window is `Active` while it is the one the person works in (`.ds[data-activity]`); a prop of
/// a component that has no window to ask defaults to `Inactive`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Activity {
    /// Taking part.
    Active,
    /// At rest.
    #[default]
    Inactive,
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

/// A percentage, 0 to 100, clamped on construction: `IdleDim`'s `level`
/// (design/22-SETTINGS.md section 3.24 `idle.dim_level_pct`). `ds` may not depend on
/// `ds-settings` (`scripts/check-boundary.sh`), so the two `Percent` types agree by shape, not
/// by sharing a definition; a caller already holding `ds_settings::Percent` passes its `.0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(from = "u8", into = "u8")]
pub struct Percent(pub u8);

impl Percent {
    /// `value`, clamped to 100.
    pub fn new(value: u8) -> Self {
        Percent(value.min(100))
    }
}

impl From<u8> for Percent {
    fn from(value: u8) -> Self {
        Percent::new(value)
    }
}

impl From<Percent> for u8 {
    fn from(value: Percent) -> Self {
        value.0
    }
}

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

/// One key in a shortcut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutKey {
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
/// draws Up, Down, Left and Right larger than the rest of the small face (at 9.5 px
/// an arrow's stroke reads as a dash).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum GlyphKind {
    /// Up, Down, Left, Right.
    Arrow,
}

/// A key combination, modifiers first, rendered as glyphs with no separator: `⌃T`
/// (design/04-COMPONENTS.md O-2). Bind a standard one with [`Shortcut::standard`] and an app's
/// own with [`Shortcut::custom`], which refuses a reserved combination (design/27 section 6.2).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Shortcut(pub Vec<ShortcutKey>);

impl Shortcut {
    /// The glyph text, modifiers in the Mac's order whatever order they were given in: `⌃⌥⇧⌘`,
    /// then the key (`⇧⌘Z`).
    pub fn glyphs(&self) -> String {
        self.keys().into_iter().map(|key| key.glyph()).collect()
    }

    /// The keys in the order they are drawn: modifiers first in the Mac's order (⌃⌥⇧⌘),
    /// deduplicated, then the rest as given.
    pub fn keys(&self) -> Vec<ShortcutKey> {
        crate::core::standard_action::normalized(self.0.iter().copied())
    }
}

impl ShortcutKey {
    /// A modifier's place in the Mac's order (Control, Option, Shift, Command), or `None` for a
    /// key that is not a modifier.
    pub(crate) fn modifier_rank(self) -> Option<u8> {
        match self {
            ShortcutKey::Ctrl => Some(0),
            ShortcutKey::Alt => Some(1),
            ShortcutKey::Shift => Some(2),
            ShortcutKey::Super => Some(3),
            _ => None,
        }
    }

    /// The text one key cap shows. Only `⌃ ⇧ ⌥ ⌘`, upper-case characters and `↵` are the
    /// doc's; the rest are not specified in design/04-COMPONENTS.md (O-2 names only the
    /// modifiers). TODO(O-2): Space, Escape, Tab, Backspace, the arrows and Home, End, Delete, PageUp,
    /// PageDown, Insert and ContextMenu need sign-off.
    pub fn glyph(self) -> String {
        match self {
            ShortcutKey::Ctrl => "⌃".to_string(),
            ShortcutKey::Shift => "⇧".to_string(),
            ShortcutKey::Alt => "⌥".to_string(),
            ShortcutKey::Super => "⌘".to_string(),
            ShortcutKey::Char(c) => c.to_uppercase().collect(),
            ShortcutKey::Space => "Space".to_string(),
            ShortcutKey::Enter => "↵".to_string(),
            ShortcutKey::Escape => "Esc".to_string(),
            ShortcutKey::Tab => "⇥".to_string(),
            ShortcutKey::Backspace => "⌫".to_string(),
            ShortcutKey::Up => "↑".to_string(),
            ShortcutKey::Down => "↓".to_string(),
            ShortcutKey::Left => "←".to_string(),
            ShortcutKey::Right => "→".to_string(),
            ShortcutKey::Home => "↖".to_string(),
            ShortcutKey::End => "↘".to_string(),
            ShortcutKey::Delete => "⌦".to_string(),
            ShortcutKey::PageUp => "⇞".to_string(),
            ShortcutKey::PageDown => "⇟".to_string(),
            ShortcutKey::Insert => "Ins".to_string(),
            ShortcutKey::ContextMenu => "Menu".to_string(),
        }
    }

    /// The `data-glyph` shape `Kbd` writes for this key, or `None` for every key whose glyph
    /// needs no face rule of its own.
    pub fn glyph_kind(self) -> Option<GlyphKind> {
        match self {
            ShortcutKey::Up | ShortcutKey::Down | ShortcutKey::Left | ShortcutKey::Right => {
                Some(GlyphKind::Arrow)
            }
            _ => None,
        }
    }
}

impl Availability {
    /// `aria-disabled`: present only when disabled.
    pub fn aria_disabled(self) -> Option<&'static str> {
        match self {
            Availability::Enabled | Availability::Busy => None,
            Availability::Disabled => Some("true"),
        }
    }

    /// `aria-busy`: present only while busy.
    pub(crate) fn aria_busy(self) -> Option<&'static str> {
        match self {
            Availability::Enabled | Availability::Disabled => None,
            Availability::Busy => Some("true"),
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

impl Check {
    /// The `aria-pressed` / `aria-checked` word.
    pub fn aria(self) -> &'static str {
        match self {
            Check::On => "true",
            Check::Off => "false",
            Check::Mixed => "mixed",
        }
    }

    /// The other state: `Mixed` resolves to `On`, as a mixed checkbox does when clicked.
    pub fn flipped(self) -> Self {
        match self {
            Check::On => Check::Off,
            Check::Off | Check::Mixed => Check::On,
        }
    }
}

impl Shown {
    /// The other state: what a press on a trigger asks for.
    pub fn flipped(self) -> Self {
        match self {
            Shown::Visible => Shown::Hidden,
            Shown::Hidden => Shown::Visible,
        }
    }

    /// The `aria-expanded` word.
    pub fn aria(self) -> &'static str {
        match self {
            Shown::Visible => "true",
            Shown::Hidden => "false",
        }
    }
}

impl Selection {
    /// The `aria-current` word, where the selected one is the place the person is.
    pub fn aria_current(self) -> &'static str {
        self.aria()
    }

    /// The `aria-selected` word.
    pub fn aria(self) -> &'static str {
        match self {
            Selection::Selected => "true",
            Selection::Unselected => "false",
        }
    }

    /// `Selected` when `a == b`.
    pub fn of<T: PartialEq>(a: &T, b: &T) -> Self {
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

    /// The value a component draws: clamped to 0..=1000.
    pub fn clamped(self) -> Self {
        Fraction(self.0.min(Self::ONE.0))
    }

    /// The value in whole percent, rounded: what a battery's figure prints and its
    /// `aria-valuenow` reads.
    pub fn whole_percent(self) -> u16 {
        (self.clamped().0 + 5) / 10
    }

    /// The clamped value as a CSS number for `--f`: `0.35`, `0`, `1`.
    pub fn css(self) -> String {
        let permille = self.clamped().0;
        let text = format!("{}.{:03}", permille / 1000, permille % 1000);
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod percent_tests {
    use super::Percent;

    #[test]
    fn a_percent_is_clamped_on_construction() {
        const CASES: &[(u8, u8)] = &[(0, 0), (80, 80), (100, 100), (101, 100), (255, 100)];
        for &(given, want) in CASES {
            assert_eq!(Percent::new(given), Percent(want), "{given}");
            assert_eq!(Percent::from(given), Percent(want), "from {given}");
        }
    }
}

#[cfg(test)]
mod fraction_tests {
    use super::Fraction;

    #[test]
    fn a_fraction_reads_as_whole_percent_rounded_and_clamped() {
        const CASES: &[(u16, u16)] = &[
            (0, 0),
            (4, 0),
            (5, 1),
            (844, 84),
            (845, 85),
            (1000, 100),
            (1200, 100),
        ];
        for &(given, want) in CASES {
            assert_eq!(Fraction(given).whole_percent(), want, "{given}");
        }
    }
}

#[cfg(test)]
mod availability_tests {
    use super::{Availability, PressPhase};

    #[test]
    fn availability_writes_its_own_aria_attribute() {
        const CASES: &[(Availability, Option<&str>, Option<&str>)] = &[
            (Availability::Enabled, None, None),
            (Availability::Disabled, Some("true"), None),
            (Availability::Busy, None, Some("true")),
        ];
        for &(availability, disabled, busy) in CASES {
            assert_eq!(availability.aria_disabled(), disabled, "{availability:?}");
            assert_eq!(availability.aria_busy(), busy, "{availability:?}");
        }
    }

    #[test]
    fn data_pressed_is_written_from_the_press_going_down_until_release() {
        const CASES: &[(PressPhase, Option<&str>)] = &[
            (PressPhase::Idle, None),
            (PressPhase::Pressed, Some("true")),
            (PressPhase::Held, Some("true")),
        ];
        for &(phase, want) in CASES {
            assert_eq!(phase.attr(), want, "{phase:?}");
        }
    }
}
