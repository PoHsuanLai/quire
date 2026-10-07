//! The gallery's pages.

use ds::prelude::*;

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
    /// Rows, lists, section headers, disclosures, the roster.
    Lists,
    /// Form and FormSection over a grouped list, field rows and the icon tile (design/34).
    Forms,
    /// Menus, menu items and pop-up buttons.
    Menus,
    /// Stepper, Table, Toolbar, SplitView, Sidebar, TabView, FieldRow and FieldGroup,
    /// the MenuBar model, the drag image and the window titlebar, in every state (design/30
    /// sections 2.1 to 2.7).
    Structure,
    /// A System-Settings-like window built from those parts, with real controls.
    #[word(slug = "settings-window")]
    SettingsWindow,
    /// Popovers, hover cards, tooltips, toast, scrim, sheet, peek, palette.
    Overlays,
    /// Popover, sheet, alert, side panel, tooltip, hover card, toast, empty state and
    /// skeleton, in every state (design/30 sections 2.5 and 2.9).
    Feedback,
    /// App features: pinned tiles, Today tabs, the edge-peek sidebar, the link pill, grouped
    /// launcher commands and Space switching (design/30 section 2.11).
    App,
    /// The eight materials over black, white and a wallpaper.
    Materials,
    /// Every animation at every level.
    Motion,
    /// The Space editor and the eight presets.
    Space,
    /// What Blitz cannot do and what quire draws instead, and what the design names that quire
    /// does not draw yet.
    BlitzLimits,
    /// Every component in every state, one grid.
    Matrix,
    /// Fire each animation; the Rust `settle` beside the CSS declaration.
    MotionLab,
    /// The edit surface over an app's own text, with the caret the app draws from its rect.
    Edit,
    /// The small-state details: every primitive of design/26 with a replay button.
    Details,
    /// The voice orb: the three demo variants and a size ladder over every metric threshold.
    VoiceOrb,
    /// Symbol effects: every effect on a few icons, with a button for each `Once` effect and a
    /// toggle for each `While` one (design/35-SYMBOL-EFFECTS.md).
    Symbols,
    /// The account sheets: the consent alert, the add-account steps and the parts an app shows.
    Accounts,
    /// The missing-helper sheet in every phase.
    Helpers,
}
