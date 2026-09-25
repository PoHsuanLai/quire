//! The window seam's vocabulary: what a frame asks of its host (`ResizeEdge`, `Zoom`,
//! `WindowTile`) and what the host reports back (`WindowState`, `Support`, `TileError`). Closed
//! sets, so a host that is not ds-native (shell-host's `SurfaceHandle`) maps each one exhaustively.

/// The edge or corner an interactive resize grabs (the xdg-shell `resize_edge` set, winit's
/// `ResizeDirection`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResizeEdge {
    /// The top edge.
    Top,
    /// The bottom edge.
    Bottom,
    /// The left edge.
    Left,
    /// The right edge.
    Right,
    /// The top-left corner.
    TopLeft,
    /// The top-right corner.
    TopRight,
    /// The bottom-left corner.
    BottomLeft,
    /// The bottom-right corner.
    BottomRight,
}

impl ResizeEdge {
    /// Every edge, sides first, in the order the frame draws its grab zones (corners last, so
    /// they sit over the sides where they meet).
    pub const ALL: [ResizeEdge; 8] = [
        ResizeEdge::Top,
        ResizeEdge::Bottom,
        ResizeEdge::Left,
        ResizeEdge::Right,
        ResizeEdge::TopLeft,
        ResizeEdge::TopRight,
        ResizeEdge::BottomLeft,
        ResizeEdge::BottomRight,
    ];

    /// The `data-edge` word.
    pub fn slug(self) -> &'static str {
        match self {
            ResizeEdge::Top => "top",
            ResizeEdge::Bottom => "bottom",
            ResizeEdge::Left => "left",
            ResizeEdge::Right => "right",
            ResizeEdge::TopLeft => "top-left",
            ResizeEdge::TopRight => "top-right",
            ResizeEdge::BottomLeft => "bottom-left",
            ResizeEdge::BottomRight => "bottom-right",
        }
    }
}

/// What the zoom control asks for (design/13-BEHAVIOUR-menus-windows.md section 13.3.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Zoom {
    /// Maximized becomes restored and anything else maximized: the green light and a
    /// double-click on the titlebar.
    Toggle,
    /// Fill the work area (the tiling menu's Fill).
    Maximize,
    /// Back to the size and place it had before.
    Restore,
}

/// A placement the tiling menu offers (macOS Sequoia's Window > Move & Resize). Named
/// `WindowTile` because `ds::Tile` is a menu row's leading tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowTile {
    /// The whole work area: the same as `Zoom::Maximize`.
    Fill,
    /// The left half of the output.
    LeftHalf,
    /// The right half of the output.
    RightHalf,
    /// The current size, centred on the output.
    Centre,
}

impl WindowTile {
    /// Every placement, in the menu's order.
    pub const ALL: [WindowTile; 4] = [
        WindowTile::Fill,
        WindowTile::LeftHalf,
        WindowTile::RightHalf,
        WindowTile::Centre,
    ];

    /// The menu row's title.
    pub fn title(self) -> &'static str {
        match self {
            WindowTile::Fill => "Fill",
            WindowTile::LeftHalf => "Left half",
            WindowTile::RightHalf => "Right half",
            WindowTile::Centre => "Centre",
        }
    }
}

/// Why a host did not place its window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
pub enum TileError {
    /// The platform does not let a client place itself there: on Wayland a toplevel cannot
    /// read or set its own position (FINDINGS "Window frame").
    #[error("the platform does not let this window place itself there")]
    Unsupported,
}

/// Whether a host can do something, asked before offering it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Support {
    /// It can.
    Yes,
    /// It cannot: the control is shown unavailable.
    No,
}

/// Whether the window fills its output's work area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Maximized {
    /// Maximized (zoomed).
    On,
    /// Its own size.
    #[default]
    Off,
}

/// Whether the window covers its whole output with no frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Fullscreen {
    /// Fullscreen.
    On,
    /// Not fullscreen.
    #[default]
    Off,
}

/// Whether the window is the one the keyboard goes to (the xdg `activated` state).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Activation {
    /// The active window: its lights are coloured at rest.
    #[default]
    Active,
    /// Behind another: its lights are grey until the pointer is over them.
    Inactive,
}

/// What the host last reported about its window. The default is what a window with no host
/// looks like: its own size, not fullscreen, active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WindowState {
    /// Zoomed or not.
    pub maximized: Maximized,
    /// Fullscreen or not.
    pub fullscreen: Fullscreen,
    /// Active or not.
    pub activated: Activation,
}

impl WindowState {
    /// Whether the frame may start a move or a resize: not while the window fills its output
    /// (a maximized window's place is the compositor's, and a drag would fight it).
    pub fn movable(self) -> bool {
        self.maximized == Maximized::Off && self.fullscreen == Fullscreen::Off
    }

    /// The `data-window` word the frame is drawn by.
    pub(crate) fn slug(self) -> &'static str {
        match (self.fullscreen, self.maximized) {
            (Fullscreen::On, _) => "fullscreen",
            (Fullscreen::Off, Maximized::On) => "maximized",
            (Fullscreen::Off, Maximized::Off) => "normal",
        }
    }

    /// The `data-activation` word.
    pub(crate) fn activation_slug(self) -> &'static str {
        match self.activated {
            Activation::Active => "active",
            Activation::Inactive => "inactive",
        }
    }
}
