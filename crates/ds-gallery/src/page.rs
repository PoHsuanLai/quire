//! The gallery's pages.

use ds::Word;

/// One gallery page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum Page {
    /// Every token: colours, radii, shadows, z, durations, easings.
    Tokens,
    /// The faces and the size ramp.
    Type,
    /// Buttons, fields, toggles, sliders, chips, avatars, tabs.
    Controls,
    /// Every control and field of the catalogue in every state: label, button, toggle, checkbox,
    /// radio group, segmented control, slider, text field, progress and level indicators, badge,
    /// key equivalent.
    Catalogue,
    /// Rows, strips, sidebar items, the roster.
    Lists,
    /// Menus, popovers, hover cards, tooltips, toast, scrim, sheet, peek, palette.
    Overlays,
    /// The eight materials over black, white and a wallpaper.
    Materials,
    /// Every animation at every level.
    Motion,
    /// The Space editor and the eight presets.
    Space,
    /// What is missing: components and tokens the design names and quire does not draw yet.
    Gaps,
    /// Every component in every state, one grid.
    Matrix,
    /// Fire each animation; the Rust `settle` beside the CSS declaration.
    MotionLab,
    /// The shell chrome beside the macOS numbers it targets.
    Polish,
    /// The edit surface over an app's own text, with the caret the app draws from its rect.
    Edit,
    /// The level control's three looks and the OSD card that carries it.
    Level,
    /// The widgets in their flat, bright look: the battery and the world clock, with the Space
    /// tint on the card.
    WidgetLooks,
    /// The widgets posed as the reference screenshots, at their size, for side-by-side proof.
    WidgetReference,
    /// The shell's own lock screen, polkit prompt and app switcher (M11).
    #[word(slug = "lock")]
    LockSwitcher,
    /// Animated emoji: the set, the reactions, sizes and discs.
    Emoji,
    /// The small-state details: every primitive of design/26 with a replay button.
    Details,
    /// The voice orb: the three demo variants and a size ladder over every metric threshold.
    VoiceOrb,
}
