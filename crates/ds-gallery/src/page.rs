//! The gallery's pages.

/// One gallery page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Page {
    /// Every token: colours, radii, shadows, z, durations, easings.
    Tokens,
    /// The faces and the size ramp.
    Type,
    /// Buttons, fields, toggles, sliders, chips, avatars, tabs.
    Controls,
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
}

impl Page {
    /// Every page, in the gallery's order.
    pub const ALL: [Page; 13] = [
        Page::Tokens,
        Page::Type,
        Page::Controls,
        Page::Lists,
        Page::Overlays,
        Page::Materials,
        Page::Motion,
        Page::Space,
        Page::Gaps,
        Page::Matrix,
        Page::MotionLab,
        Page::Polish,
        Page::Edit,
    ];

    /// The `--page` word.
    pub fn slug(self) -> &'static str {
        match self {
            Page::Tokens => "tokens",
            Page::Type => "type",
            Page::Controls => "controls",
            Page::Lists => "lists",
            Page::Overlays => "overlays",
            Page::Materials => "materials",
            Page::Motion => "motion",
            Page::Space => "space",
            Page::Gaps => "gaps",
            Page::Matrix => "matrix",
            Page::MotionLab => "motion-lab",
            Page::Polish => "polish",
            Page::Edit => "edit",
        }
    }
}
